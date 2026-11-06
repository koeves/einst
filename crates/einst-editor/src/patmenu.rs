//! Pattern selector and track right-click menus.
//!
//! - [re:00b273] 0x5c60: Rename pattern; Cut,
//!   Copy, Paste; Clear pattern (0xaf), Clear all patterns (0xb0); Lock /
//!   Unlock pattern (0xac `e_lockPattern`); Reset pattern velocity (0xae),
//!   Quantize pattern timing (0xad), Extract groove from pattern (editor
//!   message 0x136), Convert timing (Pattern Convertor panel); Swap with /
//!   Copy to / Move to ▸ patterns ([re:dd0504] /
//!   copy / move); Export as audio (current pattern).
//! - Track menu, as the original v1 builds it ([re:fdbfcb], commands
//!   [re:ebb1c9]): Pen / Erase / Select / Multi Tool; Dragging alters... ▸;
//!   Delete (the step under the mouse, or the selection it is in); Cut,
//!   Copy, Paste (at that step), Duplicate selected steps / Duplicate
//!   pattern; Extract groove from pattern, Quantize pattern timing, Reset
//!   pattern velocity, Quantize track timing, Reset track velocity; Clear
//!   track, Clear all tracks; Lock / Unlock track, Unlock all tracks, the
//!   track-and-pad lock pair, Select tracks' pad; Insert ▸.

use dreist_engine::binding::id;
use dreist_formats::Node;
use dreist_gui::app::{App, MouseKind, UiEvent};

use crate::command::{Command, msg};
use crate::model::Model;
use crate::padmenu::{Clipboard, pad_locked, set_lock, track_mut, track_node};

const PATTERNS: usize = 24;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Action {
    // Pattern selector.
    CutPattern,
    CopyPattern,
    PastePattern,
    ClearPattern,
    ClearAllPatterns,
    LockPattern,
    ResetVelocity,
    Quantize,
    ConvertTiming,
    SwapMenu,
    CopyMenu,
    MoveMenu,
    SwapWith(usize),
    CopyTo(usize),
    MoveTo(usize),
    // Track.
    Editor(EditorRequest),
    DragMenu,
    Delete(usize),
    PatternGroove,
    PatternQuantize,
    PatternVelocity,
    TrackQuantize,
    TrackVelocity,
    ClearTrack,
    ClearAllTracks,
    SelectPad,
    UnlockAllTracks,
    LockTrack,
    LockTrackAndPad,
    InsertMenu,
    Insert(usize),
    None,
}

/// Track menu items the pattern editor carries out (v1 routes them to the
/// pattern editor panel).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EditorRequest {
    /// Tools panel index (Pen, Erase, Select, Multi).
    Tool(usize),
    /// "Dragging alters..." lane.
    DragLane(usize),
    Cut,
    Copy,
    /// Paste at (track, step).
    Paste(usize, usize),
    Duplicate,
}

/// [re:24173a]: the `.patterntemplate`
/// files of `<Resources>/Pattern Templates` as (name, steps on/off).
pub fn pattern_templates(resources: &std::path::Path) -> Vec<(String, Vec<bool>)> {
    let dir = resources.parent().unwrap_or(resources).join("Pattern Templates");
    let mut files: Vec<std::path::PathBuf> =
        std::fs::read_dir(&dir).map(|rd| rd.flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("patterntemplate"))).collect()).unwrap_or_default();
    files.sort();
    files
        .iter()
        .filter_map(|f| {
            let text = std::fs::read_to_string(f).ok()?;
            let root = Node::parse(&text).ok()?;
            let n = if root.class == "InsertablePattern" { root } else { root.children.into_iter().find(|c| c.class == "InsertablePattern")? };
            let bits: Vec<bool> = (0..).map_while(|i| n.attr(&format!("Element{i}")).map(|v| v.trim() == "true")).collect();
            if bits.is_empty() {
                return None;
            }
            // `<VString name="PatternName">text</VString>` (text content).
            let name = text
                .split_once("name=\"PatternName\">")
                .and_then(|(_, r)| r.split_once('<'))
                .map(|(t, _)| t.trim().to_string())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| f.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default());
            Some((name, bits))
        })
        .collect()
}

#[derive(Default)]
pub struct PatMenu {
    /// Pattern or track index, actions, anchor.
    open: Option<(usize, Vec<Action>, Vec<usize>)>,
    /// "Extract groove from pattern" asked for this pattern (the editor
    /// runs the save dialog).
    pub groove: Option<usize>,
    /// "Convert timing" asked for this pattern (the editor opens the panel).
    pub convert: Option<usize>,
    /// Export as audio was chosen (the editor renders the pattern).
    pub export: bool,
    /// The pattern editor's step selection (track, step), set by the
    /// editor before events reach the menu.
    pub selection: std::collections::BTreeSet<(usize, usize)>,
    /// A track menu pick for the pattern editor.
    pub editor: Option<EditorRequest>,
    /// The pattern editor's step clipboard holds steps (set by the editor).
    pub steps_copied: bool,
}

fn pattern_locked(model: &Model, e: usize, p: usize) -> bool {
    model.pattern_node(e, p).is_some_and(|n| n.locked())
}

pub(crate) fn pattern_or_empty(model: &Model, e: usize, p: usize) -> Node {
    model.pattern_node(e, p).cloned().unwrap_or_else(|| Node::new("StepPattern", Some(&format!("StepPattern{p}"))))
}

pub(crate) fn put_pattern(model: &mut Model, e: usize, p: usize, mut node: Node) {
    node.name = Some(format!("StepPattern{p}"));
    let eng = model.engine_mut(e);
    match eng.children.iter_mut().find(|c| c.class == "StepPattern" && c.name.as_deref() == Some(&format!("StepPattern{p}"))) {
        Some(slot) => *slot = node,
        None => eng.children.push(node),
    }
}

/// Steps off, lanes and names kept (clearing a pattern keeps its settings).
pub(crate) fn clear_pattern(n: &mut Node) {
    for t in n.children.iter_mut().filter(|c| c.class == "Track") {
        t.children.retain(|c| c.class != "Step");
    }
}

impl PatMenu {
    fn show(&mut self, app: &mut App, anchor: &[usize], index: usize, items: Vec<(String, Action, bool)>) {
        let labels = items.iter().map(|(s, _, on)| if *on || s.is_empty() { s.clone() } else { format!("({s})") }).collect();
        let actions = items.into_iter().map(|(_, a, on)| if on { a } else { Action::None }).collect();
        self.open = Some((index, actions, anchor.to_vec()));
        app.open_menu(anchor, labels, None);
    }

    fn pattern_menu(&mut self, app: &mut App, anchor: &[usize], model: &Model, clip: &Clipboard, p: usize) {
        let e = model.engine;
        let sep = || (String::new(), Action::None, true);
        // v1's list ([re:00b273]).
        let items = vec![
            ("Cut".into(), Action::CutPattern, true),
            ("Copy".into(), Action::CopyPattern, true),
            ("Paste".into(), Action::PastePattern, clip.pattern.is_some()),
            sep(),
            ("Clear pattern".into(), Action::ClearPattern, true),
            ("Clear all patterns".into(), Action::ClearAllPatterns, true),
            sep(),
            (if pattern_locked(model, e, p) { "Unlock pattern" } else { "Lock pattern" }.into(), Action::LockPattern, true),
            sep(),
            ("Reset pattern velocity".into(), Action::ResetVelocity, true),
            ("Quantize pattern timing".into(), Action::Quantize, true),
            ("Convert timing".into(), Action::ConvertTiming, true),
            sep(),
            ("Swap with... ▸".into(), Action::SwapMenu, true),
            ("Copy to... ▸".into(), Action::CopyMenu, true),
            ("Move to... ▸".into(), Action::MoveMenu, true),
        ];
        self.show(app, anchor, p, items);
    }

    /// Track `t`'s menu at `anchor` (the track name area, or a step `step`
    /// of the grid under the pen, erase or select tool).
    pub(crate) fn track_menu(&mut self, app: &mut App, anchor: &[usize], model: &Model, _clip: &Clipboard, t: usize, step: Option<usize>) {
        let e = model.engine;
        let cur = model.current_pattern(e);
        let pat = model.pattern(e, cur);
        let tl = track_node(model, e, t).is_some_and(|n| n.locked());
        let pl = pad_locked(model, e, t);
        let sel = !self.selection.is_empty();
        let empty = pat.tracks.iter().all(|tr| tr.steps.iter().all(|s| !s.on));
        let step_on = step.and_then(|s| pat.tracks.get(t).and_then(|tr| tr.steps.get(s))).is_some_and(|s| s.on);
        let pair = match (tl, pl) {
            (true, true) => "Unlock track and pad",
            (true, false) => "Unlock track and lock pad",
            (false, true) => "Lock track and unlock pad",
            (false, false) => "Lock track and pad",
        };
        let sep = || (String::new(), Action::None, true);
        let r = |q: EditorRequest| Action::Editor(q);
        let items = vec![
            ("Pen Tool".into(), r(EditorRequest::Tool(0)), true),
            ("Erase Tool".into(), r(EditorRequest::Tool(1)), true),
            ("Select Tool".into(), r(EditorRequest::Tool(2)), true),
            ("Multi Tool".into(), r(EditorRequest::Tool(3)), true),
            ("Dragging alters... ▸".into(), Action::DragMenu, true),
            sep(),
            ("Delete".into(), step.map_or(Action::None, Action::Delete), step.is_some() && !tl && (sel || step_on)),
            sep(),
            ("Cut".into(), r(EditorRequest::Cut), !tl && sel),
            ("Copy".into(), r(EditorRequest::Copy), sel),
            ("Paste".into(), r(EditorRequest::Paste(t, step.unwrap_or(0))), self.steps_copied && !tl),
            ((if sel { "Duplicate selected steps" } else { "Duplicate pattern" }).into(), r(EditorRequest::Duplicate), !tl && !empty),
            sep(),
            ("Extract groove from pattern".into(), Action::PatternGroove, true),
            ("Quantize pattern timing".into(), Action::PatternQuantize, true),
            ("Reset pattern velocity".into(), Action::PatternVelocity, true),
            ("Quantize track timing".into(), Action::TrackQuantize, !tl),
            ("Reset track velocity".into(), Action::TrackVelocity, !tl),
            sep(),
            ("Clear track".into(), Action::ClearTrack, !tl),
            ("Clear all tracks".into(), Action::ClearAllTracks, !tl),
            sep(),
            (if tl { "Unlock track" } else { "Lock track" }.into(), Action::LockTrack, true),
            ("Unlock all tracks".into(), Action::UnlockAllTracks, true),
            (pair.into(), Action::LockTrackAndPad, true),
            ("Select tracks' pad".into(), Action::SelectPad, true),
            sep(),
            ("Insert ▸".into(), Action::InsertMenu, !tl),
        ];
        self.show(app, anchor, t, items);
    }

    pub fn handle(&mut self, app: &mut App, model: &mut Model, clip: &mut Clipboard, ev: &UiEvent, out: &mut Vec<Command>) -> bool {
        match ev {
            UiEvent::RightClicked { path, name } => {
                let Some(n) = name.strip_prefix("Key").and_then(|n| n.parse::<usize>().ok()).filter(|n| *n < PATTERNS) else { return false };
                if !app.root.path_of("PatternSelectors").is_some_and(|s| path.starts_with(&s)) {
                    return false;
                }
                let path = path.clone();
                self.pattern_menu(app, &path, model, clip, n);
                true
            }
            UiEvent::Mouse { path, overload, kind, .. } if overload == "TrackSelectArea" => {
                let Some(t) = app.ctrl(&path[..path.len() - 1]).and_then(|r| r.attr("track")).and_then(|v| v.parse::<usize>().ok()) else { return false };
                let e = model.engine;
                match kind {
                    MouseKind::Down => {
                        model.pad[e] = t;
                        true
                    }
                    MouseKind::RightDown => {
                        let path = path.clone();
                        self.track_menu(app, &path, model, clip, t, None);
                        true
                    }
                    _ => false,
                }
            }
            // The pattern toolbar's Insert menu (manual §9:3): the
            // selected pad's track from the pattern's start.
            UiEvent::Clicked { path, name, .. } if name == "InsertButton" && app.root.path_of("ToolBar").is_some_and(|t| path.starts_with(&t)) => {
                let t = model.pad[model.engine];
                let path = path.clone();
                self.apply(app, &path, model, clip, t, Action::InsertMenu, out)
            }
            UiEvent::MenuSelected { anchor, index, .. } if self.open.as_ref().is_some_and(|o| o.2 == *anchor) => {
                let (n, actions, anchor) = self.open.take().unwrap();
                let a = actions.get(*index).copied().unwrap_or(Action::None);
                self.apply(app, &anchor, model, clip, n, a, out)
            }
            _ => false,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn apply(&mut self, app: &mut App, anchor: &[usize], model: &mut Model, clip: &mut Clipboard, n: usize, a: Action, out: &mut Vec<Command>) -> bool {
        let e = model.engine;
        let rebuild = |model: &Model, out: &mut Vec<Command>| out.push(model.rebuild_command());
        let pattern_items = |model: &Model, f: &dyn Fn(usize) -> Action| -> Vec<(String, Action, bool)> {
            (0..PATTERNS)
                .map(|q| {
                    let name = model.pattern_name(e, q);
                    (if name.is_empty() { format!("Pattern {}", q + 1) } else { name }, f(q), q != n)
                })
                .collect()
        };
        match a {
            Action::None => false,
            Action::CutPattern | Action::CopyPattern => {
                clip.pattern = Some(pattern_or_empty(model, e, n));
                if a == Action::CutPattern {
                    out.push(msg(id::CLEAR_PATTERN, &[e as i32, -1, -1, n as i32], &[0.0]));
                    return true;
                }
                false
            }
            Action::PastePattern => {
                let Some(p) = clip.pattern.clone() else { return false };
                put_pattern(model, e, n, p);
                rebuild(model, out);
                true
            }
            Action::ClearPattern => {
                out.push(msg(id::CLEAR_PATTERN, &[e as i32, -1, -1, n as i32], &[0.0]));
                true
            }
            Action::ClearAllPatterns => {
                out.push(msg(id::CLEAR_ALL_PATTERNS, &[e as i32], &[0.0]));
                true
            }
            Action::LockPattern => {
                let on = !pattern_locked(model, e, n);
                let sp = model.engine_mut(e).child_mut_or_insert("StepPattern", &format!("StepPattern{n}"));
                set_lock(sp, on);
                true
            }
            Action::ResetVelocity => {
                out.push(msg(id::QUANTIZE_PATTERN_VELOCITY, &[e as i32, -1, -1, n as i32, -1, -1], &[0.0]));
                true
            }
            Action::Quantize => {
                out.push(msg(id::QUANTIZE_PATTERN_TIMING, &[e as i32, -1, -1, n as i32, -1, -1], &[0.0]));
                true
            }
            Action::ConvertTiming => {
                self.convert = Some(n);
                true
            }
            Action::SwapMenu | Action::CopyMenu | Action::MoveMenu => {
                let f: &dyn Fn(usize) -> Action = match a {
                    Action::SwapMenu => &Action::SwapWith,
                    Action::CopyMenu => &Action::CopyTo,
                    _ => &Action::MoveTo,
                };
                let items = pattern_items(model, f);
                self.show(app, anchor, n, items);
                false
            }
            Action::SwapWith(q) => {
                let (a_node, b_node) = (pattern_or_empty(model, e, n), pattern_or_empty(model, e, q));
                put_pattern(model, e, q, a_node);
                put_pattern(model, e, n, b_node);
                rebuild(model, out);
                true
            }
            Action::CopyTo(q) | Action::MoveTo(q) => {
                let node = pattern_or_empty(model, e, n);
                put_pattern(model, e, q, node);
                if matches!(a, Action::MoveTo(_)) {
                    let sp = model.engine_mut(e).child_mut_or_insert("StepPattern", &format!("StepPattern{n}"));
                    clear_pattern(sp);
                }
                rebuild(model, out);
                true
            }
            Action::Editor(q) => {
                self.editor = Some(q);
                true
            }
            // [re:fdbfcb]: the step graph lanes as check items.
            Action::DragMenu => {
                let lanes = crate::pattern::drag_lanes();
                let cur = model.pattern_view[e].drag_lane;
                let items = lanes
                    .iter()
                    .map(|&l| {
                        (
                            if l == cur { format!("{}{}", dreist_gui::menu::CHECK, crate::pattern::lane_label(l)) } else { crate::pattern::lane_label(l) },
                            Action::Editor(EditorRequest::DragLane(l)),
                            true,
                        )
                    })
                    .collect();
                self.show(app, anchor, n, items);
                false
            }
            // 0xd3 step off: the selection when the step is in it, else
            // the step.
            Action::Delete(st) => {
                let cur = model.current_pattern(e) as i32;
                let steps: Vec<(usize, usize)> = if self.selection.contains(&(n, st)) { self.selection.iter().copied().collect() } else { vec![(n, st)] };
                for (t, k) in steps {
                    out.push(msg(id::SET_STEP_OFF, &[e as i32, t as i32, -1, cur, t as i32, k as i32], &[0.0]));
                }
                true
            }
            Action::PatternGroove => {
                self.groove = Some(model.current_pattern(e));
                false
            }
            Action::PatternQuantize | Action::PatternVelocity => {
                let cur = model.current_pattern(e) as i32;
                let k = if a == Action::PatternQuantize { id::QUANTIZE_PATTERN_TIMING } else { id::QUANTIZE_PATTERN_VELOCITY };
                out.push(msg(k, &[e as i32, -1, -1, cur, -1, -1], &[0.0]));
                true
            }
            // [re:db9a5f]: the track's timing / velocity back to the
            // defaults (its Shift / Velocity lane).
            Action::TrackQuantize | Action::TrackVelocity => {
                let cur = model.current_pattern(e) as i32;
                let l = if a == Action::TrackQuantize { dreist_engine::pattern::lane::SHIFT } else { dreist_engine::pattern::lane::VELOCITY };
                out.push(msg(id::CLEAR_TRACK_AUTOMATION, &[e as i32, n as i32, -1, cur, n as i32, -1], &[l as f32]));
                true
            }
            Action::ClearTrack => {
                let cur = model.current_pattern(e) as i32;
                out.push(msg(id::CLEAR_TRACK, &[e as i32, -1, -1, cur, n as i32, -1], &[0.0]));
                true
            }
            Action::ClearAllTracks => {
                let cur = model.current_pattern(e) as i32;
                out.push(msg(id::CLEAR_PATTERN, &[e as i32, -1, -1, cur], &[0.0]));
                true
            }
            Action::SelectPad => {
                model.pad[e] = n;
                true
            }
            // 0xb9 `e_unlockAllTracks`.
            Action::UnlockAllTracks => {
                for t in 0..crate::padmenu::PADS {
                    if track_node(model, e, t).is_some() {
                        set_lock(track_mut(model, e, t), false);
                    }
                }
                true
            }
            Action::LockTrack => {
                let on = !track_node(model, e, n).is_some_and(|n| n.locked());
                set_lock(track_mut(model, e, n), on);
                true
            }
            // Each flips: the label names the resulting states.
            Action::LockTrackAndPad => {
                let tl = track_node(model, e, n).is_some_and(|n| n.locked());
                let pl = pad_locked(model, e, n);
                set_lock(track_mut(model, e, n), !tl);
                set_lock(model.pad_mut(e, n), !pl);
                true
            }
            Action::InsertMenu => {
                let items = pattern_templates(&dreist_formats::paths::skin_dir()).into_iter().enumerate().map(|(i, (name, _))| (name, Action::Insert(i), true)).collect();
                self.show(app, anchor, n, items);
                false
            }
            // Lambda #15 [re:182e3a]: the template repeats over the track
            // length; 0xd2 step on (1, 1) / 0xd3 step off per step.
            Action::Insert(k) => {
                let Some((_, bits)) = pattern_templates(&dreist_formats::paths::skin_dir()).into_iter().nth(k) else { return false };
                let cur = model.current_pattern(e);
                let pat = model.pattern(e, cur);
                let len = pat.tracks.get(n).map(|t| t.length).unwrap_or(pat.steps).max(1) as usize;
                for st in 0..len {
                    let index = [e as i32, n as i32, -1, cur as i32, n as i32, st as i32];
                    if bits[st % bits.len()] {
                        out.push(msg(id::SET_STEP_ON, &index, &[1.0, 1.0, 0.0]));
                    } else {
                        out.push(msg(id::SET_STEP_OFF, &index, &[0.0]));
                    }
                }
                true
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_clears_the_source_pattern() {
        crate::app::test_setup();
        let mut doc = Node::new(dreist_formats::legacy::ROOT_ELEMENT, None);
        let mut eng = Node::new("Engine", Some("Engine0"));
        let mut sp = Node::new("StepPattern", Some("StepPattern0"));
        let mut t = Node::new("Track", Some("Track0"));
        let mut st = Node::new("Step", Some("S0"));
        st.set("State", "true");
        t.children.push(st);
        sp.children.push(t);
        eng.children.push(sp);
        doc.children.push(eng);
        let mut m = Model::new(doc);
        let node = pattern_or_empty(&m, 0, 0);
        put_pattern(&mut m, 0, 3, node);
        clear_pattern(m.engine_mut(0).child_mut_or_insert("StepPattern", "StepPattern0"));
        assert!(m.pattern_used(0, 3));
        assert!(!m.pattern_used(0, 0));
    }

    #[test]
    fn factory_pattern_templates() {
        crate::app::test_setup();
        let res = dreist_formats::paths::skin_dir();
        let t = pattern_templates(&res);
        let Some((name, bits)) = t.iter().find(|(n, _)| n.starts_with("E(3,10)")) else { return };
        assert!(name.contains('X'));
        assert_eq!(bits.len(), 10);
        assert_eq!(bits.iter().filter(|b| **b).count(), 3);
    }
}
