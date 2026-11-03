//! Drag and drop between controls (`VCtrl::CDragMapper::OnDrop` → the
//! target's `OnDroppedObject`). Sources: a pad (`EditorPad`), a pattern
//! selector key (`PatternSelector`) and the browser file list
//! (`BrowserFileList::GetDragItems` 0x3acdb0, the selected entries).
//! Modifier flags as `Geist2_RootView::keyFlagsForModifiers:` 0x2a0d70
//! (shift 1, ctrl 2, alt 4, cmd 8).
//!
//! - `EditorPads::OnDroppedObject` 0x1d6980: nothing on a locked pad.
//!   Pad onto pad: alt copies (`EditorPad::CopyAndPastePad` 0x39c8e0, 0x56
//!   `e_setPad`), alt+shift moves (the copy then 0xe3 clears the source),
//!   otherwise swaps (`SwapPads` 0x39ce40, 0x54); the target is selected.
//!   Browser files: one file → `BrowserController::HandleDragAndDrop`
//!   0x29cea0 mode 9 (layer 0) / 10 with alt (the first layer without a
//!   sample, "No available layers on selected pad!" when there is none);
//!   several audio files → consecutive pads from the target (alt: layers
//!   0.. of the target, `BrowserFileList::DragEnded` 0x3ac1a0).
//! - `LayerSelectionButton::OnDroppedObject` 0x3a3b60 / `LayerPanel`
//!   0xf60b0: audio files onto the selected pad from that layer on (when
//!   they fit in 8 layers); not when the pad or layer is locked.
//!   `PadSelectionButton` 0x21db30: the same from layer 0.
//! - `PatternSelector::OnDroppedObject` 0x1c94a0: key onto key — alt+shift
//!   moves, alt copies, otherwise swaps, then selects the target; one
//!   pattern file loads into that slot.
//! - `EditorEngineSelectors::OnDroppedObject` 0x3c71a0: one .geistengine /
//!   .geistkit file loads into the engine under the pointer, which is
//!   selected.

use std::path::{Path, PathBuf};

use geist_engine::binding::id;
use geist_gui::app::UiEvent;

use crate::command::{Command, msg};
use crate::model::LAYERS;
use crate::{Editor, padmenu, pads, patmenu};

/// Drop-target highlights other than the pads.
///
/// - `PatternSelector::DragMove` 0x1c9f80: another key, or one browser
///   pattern file (`IsListAllPatternFiles` and a single item), sets +0x274 /
///   +0x276 (cleared by `DragExit` / `OnDroppedObject`); `ATC_Tick`
///   0x1ca2a0 steps the highlight alpha (+0x278) by 25 per tick, back to 0
///   past 120; `DrawLayer` 0x1c81c0 layer 3 fills the bounds in
///   StandardControlHighlight (set in the constructor) at that alpha.
/// - `EditorEngineSelectors::DragMove` 0x3c7680: +0x260 = the selector
///   under the pointer (−1 on `DragExit`); `Draw` fills its rect.
#[derive(Default)]
pub(crate) struct DropHighlights {
    key: Option<Vec<usize>>,
    engine: Option<Vec<usize>>,
    alpha: u32,
    tick: Option<std::time::Instant>,
}

/// What a drag started on.
enum Source {
    Pad(usize),
    Pattern(usize),
    Files(Vec<PathBuf>),
}

/// What it was dropped on.
#[derive(Debug, PartialEq)]
enum Target {
    Pad(usize),
    /// A layer row of the pad inspector, or the layer panel (current layer).
    Layer(usize),
    /// The pad inspector's pad header.
    PadHeader,
    Pattern(usize),
    Engine(usize),
}

fn ext(p: &Path) -> String {
    p.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default()
}

fn all_audio(files: &[PathBuf]) -> bool {
    !files.is_empty() && files.iter().all(|f| !f.is_dir() && crate::search::is_audio(&ext(f)))
}

impl Editor {
    fn drag_source(&self, from: &[usize]) -> Option<Source> {
        // Files from Finder (`Geist2_RootView` → `CDragMapper::OnDragEnter`
        // with the pasteboard's file names): the browser files' targets.
        if from.is_empty() {
            return self.external_drag.clone().filter(|f| !f.is_empty()).map(Source::Files);
        }
        let c = self.app.ctrl(from)?;
        match c.overload.as_str() {
            "EditorPad" => c.attr("padIndex")?.parse().ok().map(Source::Pad),
            "PatternSelector" => c.name.strip_prefix("Key")?.parse().ok().map(Source::Pattern),
            "BrowserFileList" => {
                let b = self.browser.as_ref()?;
                let items = b.drag_items();
                (!items.is_empty()).then_some(Source::Files(items))
            }
            _ => None,
        }
    }

    fn drop_target(&self, to: &[usize]) -> Option<Target> {
        if let Some(base) = &self.header.selectors {
            if to.len() > base.len() && to.starts_with(base) {
                return Some(Target::Engine(to[base.len()]));
            }
        }
        for n in (1..=to.len()).rev() {
            let c = self.app.ctrl(&to[..n])?;
            match c.overload.as_str() {
                "EditorPad" => return c.attr("padIndex")?.parse().ok().map(Target::Pad),
                "LayerRow" => return c.attr("layer")?.parse().ok().map(Target::Layer),
                "PatternSelector" => return c.name.strip_prefix("Key")?.parse().ok().map(Target::Pattern),
                _ => {}
            }
            match c.name.as_str() {
                "PadSelButton" => return Some(Target::PadHeader),
                "LayerPanel" => return Some(Target::Layer(self.model.layer[self.model.engine])),
                _ => {}
            }
        }
        None
    }

    /// `EditorPads::DragMove` 0x1d7c80 flashes while dragging over the pads
    /// (`DragExit` ends them elsewhere): a pad, or anything with alt, flashes
    /// the pad under the pointer; browser kit files flash every pad; other
    /// files the pads from the one under the pointer on, one per file.
    /// Pattern keys flash nothing.
    pub(crate) fn pad_drag_flash(&mut self, from: &[usize], to: &[usize], x: f32, y: f32) {
        if !self.pads.in_matrix(to) {
            self.pads.end_drag_flash();
            return;
        }
        let alt = self.app.mods.alt;
        let kits = |f: &[PathBuf]| !f.is_empty() && f.iter().all(|p| matches!(ext(p).as_str(), "geistkit" | "gkit"));
        let what = match self.drag_source(from) {
            Some(Source::Pad(_)) => pads::DragFlash::AtPosition,
            Some(Source::Files(f)) if !from.is_empty() && kits(&f) => pads::DragFlash::All,
            Some(Source::Files(_)) if alt => pads::DragFlash::AtPosition,
            Some(Source::Files(f)) => pads::DragFlash::Files(f.len()),
            _ => return,
        };
        self.pads.drag_flash(&self.app, &self.model, what, x, y);
    }

    /// Pattern key / engine selector drop highlights while dragging.
    pub(crate) fn drop_highlights(&mut self, from: &[usize], to: &[usize]) {
        let target = self.drop_target(to);
        let key = match (self.drag_source(from), &target) {
            (Some(Source::Pattern(p)), Some(Target::Pattern(q))) if p != *q => true,
            (Some(Source::Files(f)), Some(Target::Pattern(_))) if !from.is_empty() && f.len() == 1 && crate::search::is_pattern(&ext(&f[0])) => true,
            _ => false,
        };
        let key = key.then(|| (1..=to.len()).rev().map(|n| to[..n].to_vec()).find(|p| self.app.ctrl(p).is_some_and(|c| c.overload == "PatternSelector"))).flatten();
        let engine = match (&target, &self.header.selectors) {
            (Some(Target::Engine(i)), Some(base)) if self.accepts_drop(from, to) => Some([base.clone(), vec![*i]].concat()),
            _ => None,
        };
        self.set_drop_highlights(key, engine);
    }

    pub(crate) fn set_drop_highlights(&mut self, key: Option<Vec<usize>>, engine: Option<Vec<usize>>) {
        if key != self.drop_hl.key {
            if let Some(c) = self.drop_hl.key.take().and_then(|p| self.app.ctrl_mut(&p)) {
                c.remove_attr("dragAlpha");
            }
            if let Some(c) = key.as_ref().and_then(|p| self.app.ctrl_mut(p)) {
                c.set_attr("dragAlpha", self.drop_hl.alpha.to_string());
            }
            self.drop_hl.tick = key.is_some().then(std::time::Instant::now);
            self.drop_hl.key = key;
            self.app.dirty = true;
        }
        if engine != self.drop_hl.engine {
            if let Some(c) = self.drop_hl.engine.take().and_then(|p| self.app.ctrl_mut(&p)) {
                c.remove_attr("dropHighlight");
            }
            if let Some(c) = engine.as_ref().and_then(|p| self.app.ctrl_mut(p)) {
                c.set_attr("dropHighlight", "1".into());
            }
            self.drop_hl.engine = engine;
            self.app.dirty = true;
        }
    }

    /// The pattern key highlight pulse (1/30 s ticks).
    pub(crate) fn animate_drop_highlights(&mut self) {
        let Some(path) = self.drop_hl.key.clone() else {
            self.drop_hl.tick = None;
            return;
        };
        let now = std::time::Instant::now();
        let last = *self.drop_hl.tick.get_or_insert(now);
        let ticks = (now.duration_since(last).as_secs_f32() * 30.0).floor() as u32;
        if ticks == 0 {
            return;
        }
        self.drop_hl.tick = Some(now);
        for _ in 0..ticks.min(30) {
            self.drop_hl.alpha = if self.drop_hl.alpha + 25 < 121 { self.drop_hl.alpha + 25 } else { 0 };
        }
        if let Some(c) = self.app.ctrl_mut(&path) {
            c.set_attr("dragAlpha", self.drop_hl.alpha.to_string());
        }
        self.app.dirty = true;
    }

    /// Whether a drop from `from` on `to` would do something (for the drag
    /// cursor; locked pads take nothing).
    pub(crate) fn accepts_drop(&self, from: &[usize], to: &[usize]) -> bool {
        let e = self.model.engine;
        match (self.drag_source(from), self.drop_target(to)) {
            (Some(Source::Pad(p)), Some(Target::Pad(q))) => p != q && !padmenu::pad_locked(&self.model, e, q),
            (Some(Source::Pattern(p)), Some(Target::Pattern(q))) => p != q,
            (Some(Source::Files(_)), Some(Target::Pad(q))) => !padmenu::pad_locked(&self.model, e, q),
            (Some(Source::Files(f)), Some(Target::Layer(_) | Target::PadHeader)) => all_audio(&f) && !padmenu::pad_locked(&self.model, e, self.model.pad[e]),
            (Some(Source::Files(f)), Some(Target::Pattern(_))) => f.len() == 1 && crate::search::is_pattern(&ext(&f[0])),
            (Some(Source::Files(f)), Some(Target::Engine(_))) => f.len() == 1 && matches!(ext(&f[0]).as_str(), "geistengine" | "geistkit"),
            _ => false,
        }
    }

    fn rebuild(&mut self) {
        self.outbox.push(self.model.rebuild_command());
    }

    fn drop_alert(&mut self, anchor: &[usize], text: &str) {
        self.app.open_menu(anchor, vec![text.to_string()], None);
    }

    /// `UiEvent::Dropped`.
    /// A drag left the window (`CDragMapper::StartSystemDrag` →
    /// `IFileExportMapper::CreateFilesFromObject`): a pattern key becomes
    /// `<temp>/<pattern name>.mid` ('/', '\\', ':' → '_';
    /// `PatternSelector::CreateFilesFromObject` 0x1ca350), browser files go
    /// as they are (`PatternSelectors::CreateFilesFromObject` 0x4210); pads
    /// export nothing.
    pub(crate) fn drag_out(&mut self, ev: &UiEvent) -> bool {
        let UiEvent::DragOut { from } = ev else { return false };
        let files = match self.drag_source(from) {
            Some(Source::Pattern(n)) => {
                let e = self.model.engine;
                let name = self.model.pattern_name(e, n);
                let name = if name.is_empty() { format!("Pattern {}", n + 1) } else { name };
                let name: String = name.chars().map(|c| if matches!(c, '/' | '\\' | ':') { '_' } else { c }).collect();
                let path = std::env::temp_dir().join(format!("{name}.mid"));
                if !self.save_pattern_midi_of(&path, n) {
                    return false;
                }
                vec![path]
            }
            Some(Source::Files(f)) => f,
            _ => return false,
        };
        self.app.drag_out = Some(files);
        true
    }

    pub(crate) fn dropped(&mut self, ev: &UiEvent) -> bool {
        let UiEvent::Dropped { from, to, .. } = ev else { return false };
        let (Some(src), Some(dst)) = (self.drag_source(from), self.drop_target(to)) else { return false };
        if let Some(b) = self.browser.as_mut() {
            b.drag_started();
        }
        let m = self.app.mods;
        let e = self.model.engine;
        self.drop_label.clear();
        match (src, dst) {
            (Source::Pad(p), Target::Pad(q)) => {
                if padmenu::pad_locked(&self.model, e, q) || p == q {
                    return false;
                }
                self.drop_label = if m.alt {
                    format!("{} pad {} to pad {} on engine {}", if m.shift { "Moving" } else { "Copying" }, p + 1, q + 1, e + 1)
                } else {
                    format!("Swapping pads {} and {} on engine {}", p + 1, q + 1, e + 1)
                };
                if m.alt {
                    let node = padmenu::pad_or_empty(&self.model, e, p);
                    padmenu::put_pad(&mut self.model, e, q, node);
                    if m.shift {
                        padmenu::clear_samples(self.model.pad_mut(e, p));
                    }
                } else {
                    let (a, b) = (padmenu::pad_or_empty(&self.model, e, p), padmenu::pad_or_empty(&self.model, e, q));
                    padmenu::put_pad(&mut self.model, e, q, a);
                    padmenu::put_pad(&mut self.model, e, p, b);
                }
                let sel = crate::multipad::select(&mut self.model, e, q, false, false);
                self.outbox.push(sel);
                self.rebuild();
                true
            }
            (Source::Pattern(p), Target::Pattern(q)) => {
                if p == q {
                    return false;
                }
                if m.alt {
                    let node = patmenu::pattern_or_empty(&self.model, e, p);
                    patmenu::put_pattern(&mut self.model, e, q, node);
                    if m.shift {
                        let sp = self.model.engine_mut(e).child_mut_or_insert("StepPattern", &format!("StepPattern{p}"));
                        patmenu::clear_pattern(sp);
                    }
                } else {
                    let (a, b) = (patmenu::pattern_or_empty(&self.model, e, p), patmenu::pattern_or_empty(&self.model, e, q));
                    patmenu::put_pattern(&mut self.model, e, q, a);
                    patmenu::put_pattern(&mut self.model, e, p, b);
                }
                self.rebuild();
                self.outbox.push(msg(id::SELECT_PATTERN, &[e as i32, -1, -1, q as i32], &[0.0]));
                crate::pattern::set_current_pattern(&mut self.model, e, q);
                true
            }
            (Source::Files(files), Target::Pad(q)) => {
                if padmenu::pad_locked(&self.model, e, q) {
                    return false;
                }
                if files.len() == 1 {
                    let f = &files[0];
                    if crate::search::is_audio(&ext(f)) {
                        let layer = if m.alt { (0..LAYERS).find(|&l| !self.model.layer(e, q, l).valid) } else { Some(0) };
                        let Some(l) = layer else {
                            let anchor = to.clone();
                            self.drop_alert(&anchor, "No available layers on selected pad!");
                            return true;
                        };
                        crate::browser::load_sample_into_layer(&mut self.model, e, q, l, f);
                        self.rebuild();
                        return true;
                    }
                    return self.load_file_at(f, Some(q), None);
                }
                if !all_audio(&files) {
                    return self.load_file_at(&files[0], Some(q), None);
                }
                for (i, f) in files.iter().enumerate() {
                    let (p, l) = if m.alt { (q, i) } else { (q + i, 0) };
                    if p < 64 && l < LAYERS {
                        crate::browser::load_sample_into_layer(&mut self.model, e, p, l, f);
                    }
                }
                self.rebuild();
                true
            }
            (Source::Files(files), t @ (Target::Layer(_) | Target::PadHeader)) => {
                let p = self.model.pad[e];
                let l0 = if let Target::Layer(l) = t { l } else { 0 };
                let layer_locked = self.model.layer_node(e, p, l0).is_some_and(|n| n.locked());
                if padmenu::pad_locked(&self.model, e, p) || layer_locked || !all_audio(&files) || l0 + files.len() > LAYERS {
                    return false;
                }
                for (i, f) in files.iter().enumerate() {
                    crate::browser::load_sample_into_layer(&mut self.model, e, p, l0 + i, f);
                }
                self.rebuild();
                true
            }
            (Source::Files(files), Target::Pattern(q)) => {
                if files.len() != 1 || !crate::search::is_pattern(&ext(&files[0])) {
                    return false;
                }
                self.load_file_at(&files[0], None, Some(q))
            }
            (Source::Files(files), Target::Engine(t)) => {
                let x = files.first().map(|f| ext(f)).unwrap_or_default();
                if files.len() != 1 || !matches!(x.as_str(), "geistengine" | "geistkit") || t >= crate::model::ENGINES {
                    return false;
                }
                self.model.engine = t;
                self.model.doc.set("CurrentEngine", t);
                self.outbox.push(Command::SelectEngine(t));
                let part = if x == "geistkit" { crate::project::Part::Kit(t) } else { crate::project::Part::Engine(t) };
                self.open_part(part, &files[0])
            }
            _ => false,
        }
    }

    /// The file loader for a dropped non-audio file (`HandleDragAndDrop`
    /// → the document's load call): presets replace the project, kits and
    /// engines the selected engine, pad files the target pad, pattern
    /// files the target (or current) pattern slot.
    fn load_file_at(&mut self, f: &Path, pad: Option<usize>, slot: Option<usize>) -> bool {
        use crate::project::Part;
        let e = self.model.engine;
        match ext(f).as_str() {
            "geist" => {
                self.outbox.push(Command::LoadProject(f.to_path_buf()));
                true
            }
            "geistkit" | "gkit" => self.open_part(Part::Kit(e), f),
            "geistengine" => self.open_part(Part::Engine(e), f),
            "geistpad" => self.open_part(Part::Pad(e, pad.unwrap_or(self.model.pad[e])), f),
            "geistbank" => self.open_part(Part::Bank(e), f),
            "geistpattern" => self.open_part(Part::Pattern(e, slot.unwrap_or(self.model.current_pattern(e))), f),
            "mid" | "midi" => self.open_pattern_midi_at(f, slot.unwrap_or(self.model.current_pattern(e))),
            _ => false,
        }
    }
}
