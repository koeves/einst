//! Scene page (`Scene/ScenePanel.skin`, [re:04e91a]): 64 scene slots
//! (v1's six octaves, C-2 to B3), 16 of them visible (`Scene0..15`, a 4 × 4
//! grid, bottom row first) with the page / row scroll buttons and the
//! scene view control (`ScrollArea` mini-map under the `ViewSlider`).
//!
//! - [re:828540]: the select button shows the scene's MIDI key name
//!   ([re:8ad0c7], C-2 = scene 0) and selects without playing.
//! - Trigger [re:4c8e3a] posts `e_sceneTrigger` 0x10a [scene, on],
//!   snapshot [re:dbb7fb] `e_sceneSnapshot` 0x10d, clear [re:ff0484]
//!   `e_sceneClear` 0x108.
//! - The trigger image shows the slot's state: empty, used, cued (waiting
//!   for its start) or active.
//! - Start mode, name and the scene info table act on the selected scene.

use dreist_engine::binding::id;
use dreist_gui::app::{App, MouseKind, UiEvent};
use dreist_gui::{Colour, Ctrl, Rect, Skin};

use crate::EditorError;
use crate::command::{Command, msg};
use crate::model::Model;

const NOTES: [&str; 12] = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];

/// v1's scene slots: six octaves (and C4 to D#4).
pub const SLOTS: usize = 64;
/// Slot rows (4 slots each) and the rows on view.
const ROWS: usize = SLOTS / 4;
const VISIBLE_ROWS: usize = 4;
/// The highest first visible row.
const MAX_BASE: usize = ROWS - VISIBLE_ROWS;

/// [re:8ad0c7]: note name and octave (C-2 = 0).
pub fn key_name(n: usize) -> String {
    format!("{}{}", NOTES[n % 12], n as i32 / 12 - 2)
}

pub struct Scenes {
    page: Vec<usize>,
    /// The `ScenesPanel` and the `ScrollArea` mini-map.
    matrix: Option<Vec<usize>>,
    /// The first visible slot row (0 = scenes 0..3 on the bottom row).
    base: usize,
    selected: usize,
    held: Option<usize>,
    /// Scene clipboard (the engine keeps its own copy, messages 0x111..0x113).
    clipboard: Option<dreist_formats::Node>,
    /// Scene the context menu was opened on.
    menu: Option<usize>,
    /// The scene waiting for its start and the playing one.
    cued: Option<usize>,
    active: Option<usize>,
    /// View slider drag: the pointer row offset inside the visible area.
    view_drag: Option<f32>,
}

const MENU: [&str; 3] = ["Cut", "Copy", "Paste"];

fn find_path(root: &Ctrl, base: &[usize], name: &str) -> Option<Vec<usize>> {
    let mut c = root;
    for &i in base {
        c = &c.children[i];
    }
    Some([base.to_vec(), c.path_of(name)?].concat())
}

/// The mini-map cell of slot `n` (inside the `ScrollArea`, 4 columns of 16
/// rows, bottom row first).
fn mini_rect(n: usize) -> Rect {
    let (row, col) = ((n / 4) as i32, (n % 4) as i32);
    Rect::new(3 + col * 8, 3 + (ROWS as i32 - 1 - row) * 13, 7, 12)
}

fn manager(model: &Model) -> dreist_engine::scene::SceneManager {
    dreist_engine::scene::SceneManager::from_node(model.doc.children.iter().find(|c| c.class == "SceneManager").unwrap_or(&dreist_formats::Node::new("SceneManager", None)))
}

impl Scenes {
    pub fn build(skin: &mut Skin, root: &mut Ctrl, page: &[usize]) -> Result<Scenes, EditorError> {
        crate::front::mount_page(skin, root, page, crate::front::SCENE_PAGE)?;
        let tpl = crate::front::load(skin, "Scene/Scene.skin")?;
        let mut host = &mut *root;
        for &i in page {
            host = &mut host.children[i];
        }
        // The slots: `Scene<k>` panels get the slot skin.
        if let Some(m) = host.find_mut("ScenesPanel") {
            for c in m.children.iter_mut() {
                let Some(k) = c.name.strip_prefix("Scene").and_then(|n| n.parse::<usize>().ok()) else { continue };
                if c.children.is_empty() {
                    let r = c.rect;
                    c.children = tpl.children.clone();
                    c.style = tpl.style.clone();
                    c.rect = r;
                }
                c.overload = "EditorScene".into();
                c.set_attr("cell", k.to_string());
                for n in ["TriggerImageView", "OverlayImageView", "NameLabel"] {
                    if let Some(k) = c.find_mut(n) {
                        k.passive = true;
                    }
                }
                if let Some(t) = c.find_mut("TriggerImageView") {
                    t.overload = "SceneTrigger".into();
                }
            }
        }
        // The mini-map: one cell per slot (the skin's two cells are the
        // even / odd row templates).
        if let Some(a) = host.find_mut("ScrollArea") {
            let even = a.children.iter().find(|c| c.name == "MiniSceneEven").cloned();
            let odd = a.children.iter().find(|c| c.name == "MiniSceneOdd").cloned();
            if let (Some(even), Some(odd)) = (even, odd) {
                a.children.clear();
                for n in 0..SLOTS {
                    let mut c = if (n / 4) % 2 == 0 { even.clone() } else { odd.clone() };
                    c.name = format!("MiniScene{n}");
                    c.set_rect(mini_rect(n));
                    c.passive = true;
                    c.enabled = false;
                    a.children.push(c);
                }
            }
        }
        // The view control: the visible area over the mini-map, dragged.
        if let Some(v) = host.find_mut("ViewSlider") {
            v.class = "CtrlPanel".into();
            v.overload = "SceneViewSlider".into();
        }
        if let Some(t) = host.find_mut("DetailsTable") {
            t.overload = "SceneDetailsTable".into();
            t.children.clear();
        }
        let matrix = find_path(root, page, "ScenesPanel");
        Ok(Scenes { page: page.to_vec(), matrix, base: 0, selected: 0, held: None, clipboard: None, menu: None, cued: None, active: None, view_drag: None })
    }

    /// Scroll so that `n` is on view.
    fn reveal(&mut self, n: usize) {
        let row = n / 4;
        if row < self.base {
            self.base = row;
        } else if row >= self.base + VISIBLE_ROWS {
            self.base = (row + 1 - VISIBLE_ROWS).min(MAX_BASE);
        }
    }

    pub fn sync(&mut self, app: &mut App, model: &Model) {
        let scenes = manager(model);
        let sel = self.selected;
        let base = self.base;
        let (cued, active) = (self.cued, self.active);
        let Some(mp) = self.matrix.clone() else { return };
        if let Some(m) = app.ctrl_mut(&mp) {
            for c in &mut m.children {
                let Some(cell) = c.attr("cell").and_then(|v| v.parse::<usize>().ok()) else { continue };
                let n = base * 4 + cell;
                let used = scenes.scenes.get(n).is_some_and(|s| s.used);
                c.set_attr("scene", n.to_string());
                c.style = if (cell / 4 + cell % 4) % 2 == 0 { "SceneNoBorderPanelStyleEven" } else { "SceneNoBorderPanelStyleOdd" }.into();
                let state = if active == Some(n) {
                    "active"
                } else if cued == Some(n) {
                    "cued"
                } else if used {
                    "used"
                } else {
                    "empty"
                };
                for k in &mut c.children {
                    match k.name.as_str() {
                        "SelectButton" => {
                            k.text = key_name(n);
                            k.on = n == sel;
                        }
                        "NameLabel" => k.text = model.scene_name(n),
                        "TriggerImageView" => k.set_attr("sceneState", state.into()),
                        "ClearButton" => k.enabled = used,
                        _ => {}
                    }
                }
            }
        }
        // Mini-map: used scenes lit; the view control's area.
        if let Some(a) = find_path(&app.root, &self.page, "ScrollArea").and_then(|p| app.ctrl_mut(&p)) {
            for (n, c) in a.children.iter_mut().enumerate() {
                c.on = scenes.scenes.get(n).is_some_and(|s| s.used);
            }
        }
        if let Some(v) = find_path(&app.root, &self.page, "ViewSlider").and_then(|p| app.ctrl_mut(&p)) {
            v.set_attr("viewBase", base.to_string());
        }
        let pp = |app: &mut App, name: &str, en: bool| {
            if let Some(b) = find_path(&app.root, &self.page, name).and_then(|p| app.ctrl_mut(&p)) {
                b.enabled = en;
            }
        };
        pp(app, "PageUpButton", base < MAX_BASE);
        pp(app, "RowUpButton", base < MAX_BASE);
        pp(app, "PageDownButton", base > 0);
        pp(app, "RowDownButton", base > 0);
        pp(app, "TriggerPrevSceneButton", sel > 0);
        pp(app, "TriggerNextSceneButton", sel + 1 < SLOTS);
        let selected = scenes.scenes.get(sel).copied().unwrap_or_default();
        if let Some(c) = find_path(&app.root, &self.page, "NameTextBox").and_then(|p| app.ctrl_mut(&p)) {
            c.text = model.scene_name(sel);
        }
        if let Some(g) = find_path(&app.root, &self.page, "StartModeButtonGroup").and_then(|p| app.ctrl_mut(&p)) {
            let mode = selected.start as usize;
            for (i, b) in g.children.iter_mut().enumerate() {
                b.on = i == mode;
            }
        }
    }

    /// `e_sceneChangeCued` / `Activated` / `ChangesCleared` from the engine.
    pub fn event(&mut self, ev: dreist_engine::scene::SceneEvent) {
        use dreist_engine::scene::SceneEvent;
        match ev {
            SceneEvent::Cued(n) => self.cued = Some(n),
            SceneEvent::Activated(n) => {
                self.active = Some(n);
                if self.cued == Some(n) {
                    self.cued = None;
                }
            }
            SceneEvent::Selected(n) => {
                self.selected = n.min(SLOTS - 1);
                self.reveal(self.selected);
            }
            SceneEvent::Cleared => {
                self.cued = None;
                self.active = None;
            }
        }
    }

    pub fn draw(&self, model: &Model, r: &dreist_gui::render::Renderer, p: &mut dreist_gui::paint::Painter, c: &Ctrl) -> bool {
        match c.overload.as_str() {
            "SceneDetailsTable" => self.draw_details(model, r, p, c),
            "SceneTrigger" => draw_trigger(r, p, c),
            "SceneViewSlider" => draw_view(r, p, c),
            _ => return false,
        }
        true
    }

    /// [re:04e91a] into the `DetailsTable` (columns "Engine" 40 px centred,
    /// "Patt." 200 px, "Power" 50 px centred, [re:0474d8]; rows 1..8): for a
    /// used scene "n: name" and On/Off per engine.
    fn draw_details(&self, model: &Model, r: &dreist_gui::render::Renderer, p: &mut dreist_gui::paint::Painter, c: &Ctrl) {
        use dreist_gui::paint::RectF;
        use dreist_gui::style::{HAlign, VAlign};
        let s = r.styles;
        let (w, h) = (c.rect.w as f32, c.rect.h as f32);
        let st = "GlobalTableStyle";
        // v1's table style: the header band, rows and alternate rows,
        // column dividers and the frame.
        let ink = s.colour(st, "colBorder", Colour::rgba(18, 25, 60, 255));
        p.fill_rect(RectF::new(0.0, 0.0, w, h), s.colour(st, "colFill", Colour::rgba(222, 229, 237, 255)));
        let row_h = (h / 9.0).floor().max(12.0);
        let cols = [(0.0, 40.0, HAlign::Center), (40.0, 200.0, HAlign::Left), (240.0, 50.0, HAlign::Center)];
        p.fill_rect(RectF::new(0.0, 0.0, w, row_h), s.colour(st, "colHeaderFill", Colour::rgba(169, 176, 191, 255)));
        p.line(0.0, row_h - 0.5, w, row_h - 0.5, s.colour(st, "colHeaderDivider", ink), 1.0);
        let font = s.font(st);
        let text = s.colour(st, "colText", ink);
        let htext = s.colour(st, "colHeaderText", text);
        let (row, alt) = (s.colour(st, "colRow", Colour::TRANSPARENT), s.colour(st, "colRowAlt", Colour::rgba(199, 206, 217, 100)));
        for ((x, cw, al), name) in cols.iter().zip(["Engine", "Patt.", "Power"]) {
            p.text(RectF::new(x + 3.0, 0.0, cw - 6.0, row_h), name, htext, &font, *al, VAlign::Center);
        }
        let sc = manager(model).scenes.get(self.selected).cloned().unwrap_or_default();
        for e in 0..8 {
            let y = row_h * (e + 1) as f32;
            p.fill_rect(RectF::new(0.0, y, w, row_h), if e % 2 == 1 { alt } else { row });
            let (pat, pow) = if sc.used {
                let se = sc.engines[e];
                let name = model.pattern_name(e, se.pattern);
                (if name.trim().is_empty() { (se.pattern + 1).to_string() } else { format!("{}: {name}", se.pattern + 1) }, if se.power { "On" } else { "Off" }.to_string())
            } else {
                (String::new(), String::new())
            };
            for ((x, cw, al), t) in cols.iter().zip([(e + 1).to_string(), pat, pow]) {
                p.text(RectF::new(x + 3.0, y, cw - 6.0, row_h), &t, text, &font, *al, VAlign::Center);
            }
        }
        let div = s.colour(st, "colColumnDivider", ink);
        for (x, ..) in cols.iter().skip(1) {
            p.line(x + 0.5, 0.0, x + 0.5, h, div, 1.0);
        }
        p.stroke_rect(RectF::new(0.5, 0.5, w - 1.0, h - 1.0), ink, 1.0);
    }

    fn scene_of(app: &App, path: &[usize]) -> Option<usize> {
        (0..=path.len()).rev().find_map(|n| app.ctrl(&path[..n])?.attr("scene")?.parse().ok())
    }

    /// The first visible row for a pointer at `y` in the view control.
    fn view_row_at(&self, y: f32, grab: f32) -> usize {
        // Row r's cell top is at 5 + (ROWS − 1 − r)·13 in the control.
        let top_row = ROWS as f32 - 1.0 - ((y - grab - 5.0) / 13.0);
        let base = top_row - (VISIBLE_ROWS as f32 - 1.0);
        base.round().clamp(0.0, MAX_BASE as f32) as usize
    }

    pub fn handle(&mut self, app: &mut App, model: &mut Model, ev: &UiEvent, out: &mut Vec<Command>) -> bool {
        if let UiEvent::MenuSelected { index, .. } = ev {
            if let Some(n) = self.menu.take() {
                // [re:1264d4] → `OnCut/Copy/PasteScene...Clipboard`
                // [re:aba5b6] / [re:9b7404] / [re:b201dd].
                let node = |model: &Model| model.doc.children.iter().find(|c| c.class == "SceneManager").and_then(|m| m.child("Scene", &format!("Scene{n}"))).cloned();
                match index {
                    0 | 1 => {
                        self.clipboard = Some(node(model).unwrap_or_else(|| dreist_formats::Node::new("Scene", Some(&format!("Scene{n}")))));
                        out.push(msg(if *index == 0 { id::SCENE_CUT } else { id::SCENE_COPY }, &[], &[n as f32]));
                    }
                    _ => {
                        if let Some(mut c) = self.clipboard.clone() {
                            c.name = Some(format!("Scene{n}"));
                            if !model.doc.children.iter().any(|c| c.class == "SceneManager") {
                                model.doc.children.push(dreist_formats::Node::new("SceneManager", Some("Scenes")));
                            }
                            if let Some(mgr) = model.doc.children.iter_mut().find(|c| c.class == "SceneManager") {
                                match mgr.children.iter_mut().find(|k| k.class == "Scene" && k.name.as_deref() == Some(&format!("Scene{n}"))) {
                                    Some(k) => *k = c,
                                    None => mgr.children.push(c),
                                }
                            }
                            out.push(msg(id::SCENE_PASTE, &[], &[n as f32]));
                        }
                    }
                }
                return true;
            }
        }
        match ev {
            UiEvent::Wheel { path, dy, .. } if self.matrix.as_ref().is_some_and(|m| path.starts_with(m)) || app.ctrl(path).is_some_and(|c| c.overload == "SceneViewSlider") => {
                self.base = if *dy > 0.0 { (self.base + 1).min(MAX_BASE) } else { self.base.saturating_sub(1) };
                true
            }
            UiEvent::Mouse { path, overload, kind, y, .. } if overload == "SceneViewSlider" => {
                let first_top = 5.0 + (ROWS - 1 - (self.base + VISIBLE_ROWS - 1)) as f32 * 13.0;
                match kind {
                    MouseKind::Down => {
                        let grab = *y - first_top;
                        // A press outside the visible area centres it there.
                        let inside = (0.0..VISIBLE_ROWS as f32 * 13.0).contains(&grab);
                        self.view_drag = Some(if inside { grab } else { VISIBLE_ROWS as f32 * 13.0 / 2.0 });
                    }
                    MouseKind::Up => {
                        self.view_drag = None;
                        return true;
                    }
                    MouseKind::Drag => {}
                    MouseKind::RightDown => return false,
                }
                let Some(grab) = self.view_drag else { return false };
                let _ = path;
                self.base = self.view_row_at(*y, grab);
                true
            }
            UiEvent::Pressed { path, name } if path.starts_with(&self.page) && matches!(name.as_str(), "TriggerNextSceneButton" | "TriggerPrevSceneButton") => {
                let k = if name == "TriggerNextSceneButton" { id::SCENE_TRIGGER_NEXT } else { id::SCENE_TRIGGER_PREV };
                out.push(msg(k, &[], &[1.0]));
                true
            }
            UiEvent::Clicked { path, name, .. } if path.starts_with(&self.page) && matches!(name.as_str(), "TriggerNextSceneButton" | "TriggerPrevSceneButton") => {
                let k = if name == "TriggerNextSceneButton" { id::SCENE_TRIGGER_NEXT } else { id::SCENE_TRIGGER_PREV };
                out.push(msg(k, &[], &[0.0]));
                if let Some(c) = app.ctrl_mut(path) {
                    c.on = false;
                }
                true
            }
            UiEvent::Mouse { path, overload, kind: MouseKind::RightDown, .. } if overload == "EditorScene" => {
                let Some(n) = Self::scene_of(app, path) else { return false };
                let mut items: Vec<String> = MENU.iter().map(|s| s.to_string()).collect();
                if self.clipboard.is_none() {
                    items[2] = dreist_gui::menu::item("Paste", false);
                }
                app.open_menu(path, items, None);
                self.menu = Some(n);
                self.selected = n;
                true
            }
            UiEvent::Mouse { path, overload, kind, .. } if overload == "EditorScene" => {
                let Some(n) = Self::scene_of(app, path) else { return false };
                match kind {
                    MouseKind::Down => {
                        self.selected = n;
                        self.held = Some(n);
                        out.push(msg(id::SCENE_TRIGGER, &[], &[n as f32, 1.0]));
                        true
                    }
                    MouseKind::Up => {
                        if let Some(h) = self.held.take() {
                            out.push(msg(id::SCENE_TRIGGER, &[], &[h as f32, 0.0]));
                        }
                        false
                    }
                    _ => false,
                }
            }
            UiEvent::TextCommitted { path, name, text } if name == "NameTextBox" && path.starts_with(&self.page) => {
                model.set_scene_name(self.selected, text.trim());
                true
            }
            UiEvent::Clicked { path, name, .. } => {
                if !path.starts_with(&self.page) {
                    return false;
                }
                let parent = app.ctrl(&path[..path.len() - 1]).map(|c| c.name.clone()).unwrap_or_default();
                if parent == "StartModeButtonGroup" {
                    out.push(msg(id::SCENE_SET_START_MODE, &[], &[self.selected as f32, *path.last().unwrap() as f32]));
                    return true;
                }
                let scene = Self::scene_of(app, path);
                match name.as_str() {
                    "PageUpButton" => self.base = (self.base + VISIBLE_ROWS).min(MAX_BASE),
                    "RowUpButton" => self.base = (self.base + 1).min(MAX_BASE),
                    "PageDownButton" => self.base = self.base.saturating_sub(VISIBLE_ROWS),
                    "RowDownButton" => self.base = self.base.saturating_sub(1),
                    "SnapshotButton" => {
                        let n = scene.unwrap_or(self.selected);
                        self.selected = n;
                        out.push(msg(id::SCENE_SNAPSHOT, &[], &[n as f32]));
                    }
                    "ClearButton" => {
                        let n = scene.unwrap_or(self.selected);
                        self.selected = n;
                        out.push(msg(id::SCENE_CLEAR, &[], &[n as f32]));
                    }
                    "SelectButton" => {
                        if let Some(n) = scene {
                            self.selected = n;
                        }
                    }
                    "ClearAllButton" => {
                        for n in 0..SLOTS {
                            out.push(msg(id::SCENE_CLEAR, &[], &[n as f32]));
                        }
                    }
                    "SnapshotToNextButton" => {
                        if let Some(n) = manager(model).scenes.iter().take(SLOTS).position(|s| !s.used) {
                            self.selected = n;
                            self.reveal(n);
                            out.push(msg(id::SCENE_SNAPSHOT, &[], &[n as f32]));
                        }
                    }
                    _ => return false,
                }
                true
            }
            _ => false,
        }
    }

    /// The first visible slot row (tests).
    #[cfg(test)]
    pub(crate) fn base(&self) -> usize {
        self.base
    }
}

/// The trigger area of a slot by its state (`SceneTrigger{Empty,Used,
/// Cued,Active}Style`): the field (colFill) in its edge (colBorder), the
/// inner block (colAltFill: ink when the slot holds a scene, amber while
/// it plays) and the cue ring (colAltBorder).
fn draw_trigger(r: &dreist_gui::render::Renderer, p: &mut dreist_gui::paint::Painter, c: &Ctrl) {
    use dreist_gui::paint::RectF;
    let s = r.styles;
    let (w, h) = (c.rect.w as f32, c.rect.h as f32);
    let st = match c.attr("sceneState").unwrap_or("empty") {
        "used" => "SceneTriggerUsedStyle",
        "cued" => "SceneTriggerCuedStyle",
        "active" => "SceneTriggerActiveStyle",
        _ => "SceneTriggerEmptyStyle",
    };
    let rr = RectF::new(0.5, 0.5, w - 1.0, h - 1.0);
    p.fill_rect(rr, s.colour(st, "colFill", Colour::rgba(222, 229, 237, 255)));
    p.fill_rect(RectF::new(3.0, 3.0, w - 6.0, h - 6.0), s.colour(st, "colAltFill", Colour::TRANSPARENT));
    p.stroke_rect(RectF::new(2.0, 2.0, w - 4.0, h - 4.0), s.colour(st, "colAltBorder", Colour::TRANSPARENT), 2.0);
    p.stroke_rect(rr, s.colour(st, "colBorder", Colour::rgba(18, 25, 60, 140)), 1.0);
}

/// The scene view control: the four visible rows' area over the mini-map.
fn draw_view(r: &dreist_gui::render::Renderer, p: &mut dreist_gui::paint::Painter, c: &Ctrl) {
    use dreist_gui::paint::RectF;
    let s = r.styles;
    let st = "SceneViewSliderStyle";
    let base = c.attr("viewBase").and_then(|v| v.parse::<usize>().ok()).unwrap_or(0);
    let top_row = base + VISIBLE_ROWS - 1;
    // The mini-map sits 2 px in (ScrollArea at 449,112; the control at 447,110).
    let y = 2.0 + 3.0 + (ROWS - 1 - top_row) as f32 * 13.0 - 2.0;
    let rr = RectF::new(1.5, y, c.rect.w as f32 - 3.0, VISIBLE_ROWS as f32 * 13.0 + 2.0);
    p.fill_rect(rr, s.colour(st, "colAltFill", Colour::rgba(232, 239, 247, 120)));
    p.stroke_rect(rr, s.colour(st, "colBorder", Colour::rgba(18, 25, 60, 255)), 1.0);
}

#[cfg(test)]
mod tests {
    #[test]
    fn key_names() {
        crate::app::test_setup();
        assert_eq!(super::key_name(0), "C-2");
        assert_eq!(super::key_name(8), "G#-2");
        assert_eq!(super::key_name(60), "C3");
        assert_eq!(super::key_name(super::SLOTS - 1), "D#3");
    }
}
