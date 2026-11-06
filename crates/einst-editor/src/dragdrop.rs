//! Drag and drop between controls ([re:3d6549] → the
//! target's drop handler). Sources: a pad ([re:2d30eb]), a pattern
//! selector key ([re:561273]) and the browser file list
//! ([re:fb6b2b], the selected entries).
//! Modifier flags as [re:77e545]
//! (shift 1, ctrl 2, alt 4, cmd 8).
//!
//! - [re:3d06b7]: nothing on a locked pad.
//!   Pad onto pad: alt copies ([re:3d06b7], 0x56
//!   `e_setPad`), otherwise swaps ([re:3d06b7], 0x54); the target is
//!   selected. v1 has no alt+shift move.
//!   Browser files: one file → [re:4a45a2]
//!   [re:4a45a2] mode 9 (layer 0) / 10 with alt (the first layer without a
//!   sample, "No available layers on selected pad!" when there is none);
//!   several audio files → consecutive pads from the target (alt: layers
//!   0.. of the target, [re:4a0eaf]).
//! - [re:f23396] / [re:f23396]
//!   [re:f23396]: audio files onto the selected pad from that layer on (when
//!   they fit in 8 layers); not when the pad or layer is locked.
//!   [re:3d06b7]: the same from layer 0.
//! - [re:561273]: key onto key — alt copies, otherwise
//!   swaps (v1: no move), then selects the target; one
//!   pattern file loads into that slot.
//! - [re:0fbd59]: one engine file (v1 takes no kits) loads into the
//!   engine under the pointer, which is
//!   selected.

use dreist_formats::profile::{DocKind, doc_kind};
use std::path::{Path, PathBuf};

use dreist_engine::binding::id;
use dreist_gui::app::UiEvent;

use crate::command::{Command, msg};
use crate::model::LAYERS;
use crate::{Editor, padmenu, pads, patmenu};

/// Drop-target highlights other than the pads.
///
/// - [re:0aa494]: another key, or one browser
///   pattern file (a single item), sets the highlight
///   (cleared on drag exit / drop); the timer tick
///   [re:7cd143] steps the highlight alpha by 25 per tick, back to 0
///   past 120; draw layer 3 [re:7cd143] fills the bounds in
///   StandardControlHighlight (set in the constructor) at that alpha.
/// - [re:19c270]: remembers the selector
///   under the pointer (−1 on drag exit); drawing fills its rect.
#[derive(Default)]
pub(crate) struct DropHighlights {
    key: Option<Vec<usize>>,
    engine: Option<Vec<usize>>,
    alpha: u32,
    tick: Option<std::time::Instant>,
}

/// Where a drag dwells.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Dwell {
    Engine(usize),
    Pads,
}

/// What a drag started on.
#[derive(PartialEq)]
enum Source {
    Pad(usize),
    Pattern(usize),
    Files(Vec<PathBuf>),
    /// The sampler's current take ([re:5eb723]).
    Take,
    /// The slicer's selected slice.
    Slice,
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
    /// The browser's shortlist pane.
    Shortlist,
}

fn ext(p: &Path) -> String {
    p.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default()
}

fn all_audio(files: &[PathBuf]) -> bool {
    !files.is_empty() && files.iter().all(|f| !f.is_dir() && crate::search::is_audio(&ext(f)))
}

impl Editor {
    fn drag_source(&self, from: &[usize]) -> Option<Source> {
        // Files from Finder ([re:3d6549] → [re:3d6549]
        // with the pasteboard's file names): the browser files' targets.
        if from.is_empty() {
            return self.external_drag.clone().filter(|f| !f.is_empty()).map(Source::Files);
        }
        let c = self.app.ctrl(from)?;
        match c.overload.as_str() {
            "EditorPad" => c.attr("padIndex")?.parse().ok().map(Source::Pad),
            "SamplerWaveArea" if !self.sampler.takes.is_empty() => Some(Source::Take),
            "EditorPreviewWaveEditor" if self.slicer.open => Some(Source::Slice),
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
                "BrowserShortList" => return Some(Target::Shortlist),
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

    /// [re:2a11ae] flashes while dragging over the pads
    /// (drag exit ends them elsewhere): a pad, or anything with alt, flashes
    /// the pad under the pointer; browser kit files flash every pad; other
    /// files the pads from the one under the pointer on, one per file.
    /// Pattern keys flash nothing.
    pub(crate) fn pad_drag_flash(&mut self, from: &[usize], to: &[usize], x: f32, y: f32) {
        if !self.pads.in_matrix(to) {
            self.pads.end_drag_flash();
            return;
        }
        let alt = self.app.mods.alt;
        let kits = |f: &[PathBuf]| !f.is_empty() && f.iter().all(|p| doc_kind(&ext(p)) == Some(DocKind::Kit));
        let what = match self.drag_source(from) {
            Some(Source::Pad(_)) => pads::DragFlash::AtPosition,
            Some(Source::Files(f)) if !from.is_empty() && kits(&f) => pads::DragFlash::All,
            Some(Source::Files(_)) if alt => pads::DragFlash::AtPosition,
            Some(Source::Files(f)) => pads::DragFlash::Files(f.len()),
            _ => return,
        };
        self.pads.drag_flash(&self.app, &self.model, what, x, y);
    }

    /// [re:19c270] / [re:166e34]: dwelling 25 ticks (1/30 s) during a drag
    /// over an engine selector selects that engine ([re:39c385]); entering
    /// the pads starts the same wait, then the Pad / Layers page shows
    /// ([re:8135df], `e_switchToPadLayersPanel`).
    pub(crate) fn drag_dwell_over(&mut self, from: &[usize], to: &[usize]) {
        let over = match self.drop_target(to) {
            Some(Target::Engine(i)) => Some(Dwell::Engine(i)),
            // A pad drag starts inside the pads: no entering.
            _ if self.pads.in_matrix(to) && !self.pads.in_matrix(from) => Some(Dwell::Pads),
            _ => None,
        };
        if over.as_ref() != self.drag_dwell.as_ref().map(|d| &d.0) {
            self.drag_dwell = over.map(|o| (o, std::time::Instant::now()));
        }
    }

    /// The dwell's action once its 25 ticks are up.
    pub(crate) fn animate_drag_dwell(&mut self) -> bool {
        let Some((d, at)) = self.drag_dwell.clone() else { return false };
        if at.elapsed().as_secs_f32() < 25.0 / 30.0 {
            return false;
        }
        // Fired: the dwell stays until the pointer moves to another target.
        self.drag_dwell = Some((d.clone(), at + std::time::Duration::from_secs(3600)));
        match d {
            Dwell::Engine(i) => {
                let e = i.min(crate::model::ENGINES - 1);
                if self.model.engine == e {
                    return false;
                }
                self.model.engine = e;
                self.model.engine_sel = 1 << e;
                self.model.doc.set("CurrentEngine", e);
                self.outbox.push(Command::SelectEngine(e));
            }
            Dwell::Pads => {
                if self.model.editor_tab == crate::front::PADS_PAGE {
                    return false;
                }
                self.model.editor_tab = crate::front::PADS_PAGE;
            }
        }
        self.sync();
        true
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
            (Some(Source::Files(f)), Some(Target::Engine(_))) => f.len() == 1 && matches!(doc_kind(&ext(&f[0])), Some(DocKind::Engine | DocKind::Kit)),
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
    /// A drag left the window ([re:be4a8a] →
    /// [re:be7228]): a pattern key becomes
    /// `<temp>/<pattern name>.mid` ('/', '\\', ':' → '_';
    /// [re:c995b5]), browser files go
    /// as they are ([re:92d0d8] 0x4210); pads
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
        let UiEvent::Dropped { from, to, y, .. } = ev else { return false };
        // The browser's source list: the row under the pointer.
        if let Some(n) = (1..=to.len()).rev().find(|&n| self.app.ctrl(&to[..n]).is_some_and(|c| c.overload == "BrowserSourceList")) {
            let Some(Source::Files(files)) = self.drag_source(from) else { return false };
            let row = ((*y - self.app.origin(&to[..n]).1 as f32) / crate::browser::SOURCE_ROW).max(0.0) as usize;
            return self.browser.as_mut().is_some_and(|b| b.drop_on_source(row, &files));
        }
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
                // v1 ([re:3d06b7]): alt copies (`e_copyPadTo`), else swap.
                self.drop_label = if m.alt { format!("Copying pad {} to pad {} on engine {}", p + 1, q + 1, e + 1) } else { format!("Swapping pads {} and {} on engine {}", p + 1, q + 1, e + 1) };
                if m.alt {
                    let node = padmenu::pad_or_empty(&self.model, e, p);
                    padmenu::put_pad(&mut self.model, e, q, node);
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
                // v1 ([re:561273]): alt copies, else swap.
                if m.alt {
                    let node = patmenu::pattern_or_empty(&self.model, e, p);
                    patmenu::put_pattern(&mut self.model, e, q, node);
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
            // [re:9580b2] / [re:8fe850]: a take or slice onto a pad: layer 0,
            // with alt the first layer without a sample.
            (src @ (Source::Take | Source::Slice), Target::Pad(q)) => {
                if padmenu::pad_locked(&self.model, e, q) {
                    return false;
                }
                let layer = if m.alt { (0..LAYERS).find(|&l| !self.model.layer(e, q, l).valid) } else { Some(0) };
                let Some(l) = layer else {
                    let anchor = to.clone();
                    self.drop_alert(&anchor, "No available layers on selected pad!");
                    return true;
                };
                let loaded = if src == Source::Take {
                    let cur = self.sampler.current;
                    let crop = self.sampler.auto_crop;
                    let Some(f) = self.sampler_record_file(cur) else { return false };
                    self.sampler.take().is_some_and(|t| t.save(&f, crop).is_ok()) && crate::browser::load_sample_into_layer(&mut self.model, e, q, l, &f)
                } else {
                    self.slicer.drop_on_pad(&mut self.model, e, q, l)
                };
                if loaded {
                    self.rebuild();
                }
                loaded
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
                    if p < crate::model::V1_PADS && l < LAYERS {
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
                // v1 ([re:0fbd59]): one engine file.
                let x = files.first().and_then(|f| doc_kind(&ext(f)));
                if files.len() != 1 || x != Some(DocKind::Engine) || t >= crate::model::ENGINES {
                    return false;
                }
                self.model.engine = t;
                self.model.doc.set("CurrentEngine", t);
                self.outbox.push(Command::SelectEngine(t));
                let part = crate::project::Part::Engine(t);
                self.open_part(part, &files[0])
            }
            // [re:d5abda] / [re:13bdb0]: files only, from elsewhere than the
            // shortlist itself, join the current shortlist.
            (Source::Files(files), Target::Shortlist) => {
                let own = self.app.ctrl(from).is_some_and(|c| c.overload == "BrowserShortList");
                if own || files.iter().any(|f| f.is_dir()) {
                    return false;
                }
                self.browser.as_mut().is_some_and(|b| b.add_to_current_shortlist(files))
            }
            _ => false,
        }
    }

    /// The file loader for a dropped non-audio file (the drop handler
    /// → the document's load call): presets replace the project, kits and
    /// engines the selected engine, pad files the target pad, pattern
    /// files the target (or current) pattern slot.
    pub(crate) fn load_file_at(&mut self, f: &Path, pad: Option<usize>, slot: Option<usize>) -> bool {
        use crate::project::Part;
        let e = self.model.engine;
        let x = ext(f);
        match doc_kind(&x) {
            Some(DocKind::Project) => {
                self.outbox.push(Command::LoadProject(f.to_path_buf()));
                true
            }
            Some(DocKind::Kit) => self.open_part(Part::Kit(e), f),
            Some(DocKind::Engine) => self.open_part(Part::Engine(e), f),
            Some(DocKind::Pad) => self.open_part(Part::Pad(e, pad.unwrap_or(self.model.pad[e])), f),
            Some(DocKind::Bank) => self.open_part(Part::Bank(e), f),
            Some(DocKind::Pattern) => self.open_part(Part::Pattern(e, slot.unwrap_or(self.model.current_pattern(e))), f),
            _ if x == "mid" || x == "midi" => self.open_pattern_midi_at(f, slot.unwrap_or(self.model.current_pattern(e))),
            _ => false,
        }
    }
}
