//! Slicer ([re:c241a0], v1's `Preview/PreviewPanel.skin` 820×395): the
//! LCD panel for cutting an audio file into slices and extracting them to
//! pads and a pattern. It has no page tab: it covers the LCD (the page tabs
//! are off) while slicing.
//!
//! - Opened by [re:bff97e] (the browser's Slice mode on an audio file, a
//!   layer's "Send to slicer", the sampler's Slice); the wave editor resets
//!   its zoom and the analysis (a `.gsa` sidecar, else [re:c5ff4b]) fills
//!   the slices. With the browser's Auto load on, the slices and the
//!   pattern are extracted at once and again after every edit; otherwise
//!   the Extract buttons (Audio+Ptn / Audio / Pattern) do it.
//! - Slicing [re:77d1da]: Transient mode keeps the slices usable at the
//!   sensitivity, Divide cuts equal divisions; locked and deleted slices
//!   are carried over.
//! - Controls per slicing mode [re:cc20ff]: Transient shows the
//!   Sensitivity knob, Divide the division count.
//! - Meta data changes [re:168d6a]: BPM 20..1000, Bars 0..1000, Beats
//!   0..100; beats ≥ 4 carry into bars and tempo ↔ bars/beats follow the
//!   region length.
//! - Allocation [re:90734a] / job [re:f09ec6]: the pattern is extracted from the analysis,
//!   then the allocation manager assigns slices to pads (and the pattern).
//! - Revert [re:db19a5] (Cancel) restores the engine saved when the
//!   panel opened.
//! - The wave editor: a click on the ruler adds a locked slice (Transient),
//!   markers drag left / right (alt-click deletes), a click in the upper
//!   wave auditions the slice, a drag in the middle zooms (up / down) and
//!   scrolls (left / right); right click: the slice menu (Lock / Unlock,
//!   Delete / Restore). Markers [re:b13346] / [re:be5846]: a line per slice
//!   with a head in the marker strip (a box when locked, else a triangle,
//!   amber when selected).

/// The zoom scrollbar under the wave (added to v1's layout, the
/// manual's "Zoom in/out and scrollbar").
pub(crate) const ZOOM_BAR: &str = "PreviewZoomSlider";
use std::path::{Path, PathBuf};

use dreist_dsp::slice::{self, SlicePoint, SlicedPattern};
use dreist_engine::gsa::SliceFile;
use dreist_formats::Node;
use dreist_gui::app::{App, MouseKind, UiEvent};
use dreist_gui::paint::{Painter, RectF};
use dreist_gui::render::Renderer;
use dreist_gui::style::{HAlign, VAlign};
use dreist_gui::{Colour, Ctrl, Rect, Skin};
use einst_dsp::classify;

use crate::EditorError;
use crate::command::Command;
use crate::model::Model;
use crate::sampler::{RULER, WAVE_TOP, colour};

const BUCKET: usize = 64;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Extract {
    Both,
    Audio,
    Pattern,
}

struct Audio {
    /// The file, shared with the preview voice.
    data: std::sync::Arc<dreist_dsp::voice::SampleData>,
    /// (min, max) per `BUCKET` frames.
    buckets: Vec<(f32, f32)>,
}

/// Slide animation tags.
const SLIDE_IN: u32 = 0x51c1;
const SLIDE_OUT: u32 = 0x51c0;

/// The slice context menu (Transient mode).
const SLICE_MENU: [&str; 7] = ["Lock slice", "Delete slice", "", "Kick", "Snare", "Hihat", "Percussion"];

pub struct Slicer {
    /// Path of the overlay (a root child over the LCD).
    overlay: Option<Vec<usize>>,
    /// The LCD's page panel (`MainTabs`, which the slicer covers).
    main: Option<Vec<usize>>,
    /// On screen (sliding in, shown or sliding out) / sliding out.
    shown: bool,
    closing: bool,
    pub open: bool,
    file: Option<PathBuf>,
    audio: Option<Audio>,
    /// All analysed slices (the `.gsa` / detector output).
    analysis: SliceFile,
    /// Slices in use (after mode / sensitivity, plus user edits).
    pub slices: Vec<SlicePoint>,
    pub mode: usize,
    pub sensitivity: f32,
    pub divisions: u32,
    auto_tempo: bool,
    multi_layer: bool,
    /// Classify (Transient mode): the button's state.
    pub classify: bool,
    /// The browser's Auto load when the panel opened.
    pub auto_load: bool,
    /// The file's tempo when it opened (File BPM).
    file_bpm: f32,
    selected: Option<usize>,
    drag: Option<usize>,
    /// Visible frames.
    pub(crate) view: (u32, u32),
    /// A drag on the wave that zooms / scrolls: (x, y, view at the press).
    view_drag: Option<(f32, f32, (u32, u32))>,
    /// The slices came from Divide mode.
    chopped: bool,
    /// A drag on the zoom scrollbar: (zone, last x) (`zoomslider::hit`).
    bar_drag: Option<(u8, f32)>,
    /// The slice whose context menu is open.
    menu: Option<usize>,
    /// Where the last extraction put each slice: (slice start, engine, pad, layer).
    placed: Vec<(u32, usize, usize, usize)>,
    /// The document when the panel opened (Cancel restores it).
    snapshot: Option<Node>,
    extracted: bool,
    pub pattern: SlicedPattern,
    /// A REX file: its own slices and tempo (Sensitivity and BPM disabled,
    /// [re:4cb63d] / meta data from the file
    /// [re:17c579]).
    rex: bool,
}

/// Preference "Quantize REX" (`AllocatorQuantizeREX`): extracted steps get
/// no timing offset (pattern extraction [re:150941] sets lane 3 to
/// 0.5 instead of the slice's distance from the step) — for every slicing,
/// not only REX files.
fn quantize_rex() -> bool {
    crate::options::Options::load().bool("AllocatorQuantizeREX", false)
}

impl Slicer {
    fn closed(main: Option<Vec<usize>>) -> Slicer {
        Slicer {
            overlay: None,
            main,
            shown: false,
            closing: false,
            open: false,
            file: None,
            audio: None,
            analysis: SliceFile::new(0),
            slices: Vec::new(),
            mode: 0,
            sensitivity: 0.5,
            divisions: 16,
            auto_tempo: true,
            multi_layer: false,
            classify: false,
            auto_load: false,
            file_bpm: 0.0,
            selected: None,
            drag: None,
            view: (0, 0),
            bar_drag: None,
            view_drag: None,
            chopped: false,
            menu: None,
            placed: Vec::new(),
            snapshot: None,
            extracted: false,
            pattern: SlicedPattern { steps: 16, hits: Vec::new() },
            rex: false,
        }
    }

    pub fn build(skin: &mut Skin, root: &mut Ctrl) -> Result<Slicer, EditorError> {
        let mut out = Slicer::closed(root.path_of("MainTabs"));
        let Some(m) = out.main.clone() else { return Ok(out) };
        // The panel covers the LCD's page area (MainTabs, in root
        // coordinates).
        let mut r = Rect::new(0, 0, 0, 0);
        let mut c = &*root;
        for &i in &m {
            c = &c.children[i];
            r.x += c.rect.x;
            r.y += c.rect.y;
        }
        r.w = c.rect.w;
        r.h = c.rect.h;
        let mut p = crate::front::load(skin, "Preview/PreviewPanel.skin")?;
        p.name = "SlicePanel".into();
        p.overload = "SlicePanel".into();
        p.set_rect(Rect::new(0, -r.h, r.w, r.h));
        if let Some(s) = p.find_mut("ChopDivisionBox") {
            s.overload = "SliceChopStepper".into();
        }
        for n in ["SrcBPM", "SenseLabel", "ExtractLabel"] {
            if let Some(c) = p.find_mut(n) {
                c.passive = true;
            }
        }
        let overlay = Ctrl {
            class: "CtrlPanel".into(),
            name: "SlicerOverlay".into(),
            overload: "SlicerOverlay".into(),
            style: "InvisibleStyle".into(),
            rect: r,
            zorder: 1000,
            visible: false,
            enabled: true,
            children: vec![p],
            ..Ctrl::default()
        };
        root.children.push(overlay);
        out.overlay = Some(vec![root.children.len() - 1]);
        Ok(out)
    }

    /// Slicing started: load `file` and its analysis, remember the
    /// document for Cancel.
    pub fn start(&mut self, model: &Model, file: &Path) -> bool {
        // Loop load [re:660edf]: a REX ".truncated" path does not
        // slice; a REX file loads its slices end to end with the file's
        // own slice points.
        let (_, truncated) = dreist_engine::rex::strip_truncated(file);
        let rex = dreist_engine::rex::is_rex(file);
        if rex && truncated {
            return false;
        }
        let rex_c = if rex { dreist_engine::rex::load(file) } else { None };
        if rex && rex_c.is_none() {
            return false;
        }
        let Ok(data) = dreist_engine::audio::load(file) else { return false };
        let n = data.left.len();
        if n == 0 {
            return false;
        }
        let buckets = (0..n.div_ceil(BUCKET))
            .map(|b| {
                let s = &data.left[b * BUCKET..((b + 1) * BUCKET).min(n)];
                let r = data.right.as_ref().map(|r| &r[b * BUCKET..((b + 1) * BUCKET).min(n)]);
                let mut lo = s.iter().fold(f32::MAX, |a, &x| a.min(x));
                let mut hi = s.iter().fold(f32::MIN, |a, &x| a.max(x));
                if let Some(r) = r {
                    lo = r.iter().fold(lo, |a, &x| a.min(x));
                    hi = r.iter().fold(hi, |a, &x| a.max(x));
                }
                (lo, hi)
            })
            .collect();
        let gsa = file.with_extension("gsa");
        let mut analysis = match &rex_c {
            Some(c) => {
                let mut a = SliceFile::new(n as u32);
                a.slices = c.slices.iter().map(|&p| SlicePoint::manual(p as u32, false)).collect();
                a.bpm = c.tempo;
                a.bars = c.bars;
                a
            }
            None => SliceFile::load(&gsa).unwrap_or_else(|| SliceFile::analyze(&data.left, data.right.as_deref())),
        };
        self.rex = rex;
        if analysis.end == 0 || analysis.end as usize > n {
            analysis.end = n as u32;
        }
        self.audio = Some(Audio { data: std::sync::Arc::new(data), buckets });
        self.analysis = analysis;
        self.file = Some(file.to_path_buf());
        self.view = (0, n as u32);
        self.selected = None;
        self.snapshot = Some(model.doc.clone());
        self.extracted = false;
        self.open = true;
        self.slices.clear();
        self.menu = None;
        if self.auto_tempo {
            self.tempo_from_bars();
        }
        self.file_bpm = self.analysis.bpm;
        self.reslice();
        true
    }

    fn region(&self) -> (u32, u32) {
        (self.analysis.start, self.analysis.end)
    }

    fn frames(&self) -> u32 {
        self.audio.as_ref().map(|a| a.data.left.len() as u32).unwrap_or(0)
    }

    /// Slice the file again (locked and deleted slices are kept in
    /// Transient mode). Divide makes locked slices, as v1's chop does
    /// ([re:7b32bf], [re:d73885]); they are not carried back into Transient mode.
    pub fn reslice(&mut self) {
        let kept: Vec<SlicePoint> = if self.chopped { Vec::new() } else { self.slices.iter().filter(|s| s.locked || s.deleted).copied().collect() };
        let (start, end) = self.region();
        let mut v = if self.mode == 0 { slice::for_sensitivity(&self.analysis.slices, self.sensitivity) } else { slice::chop(start, end - start, self.divisions as f32) };
        self.chopped = self.mode != 0;
        if self.chopped {
            v.iter_mut().for_each(|s| s.locked = true);
        }
        if self.mode == 0 {
            for k in kept {
                if let Some(s) = v.iter_mut().find(|s| s.start == k.start) {
                    s.locked = k.locked;
                    s.deleted = k.deleted;
                } else {
                    v.push(k);
                }
            }
        }
        v.sort_by_key(|s| s.start);
        self.slices = v;
        self.selected = self.selected.filter(|&i| i < self.slices.len());
        self.classify_slices();
        self.update_pattern();
    }

    fn update_pattern(&mut self) {
        let (start, end) = self.region();
        self.pattern = slice::extract_pattern_full(&self.slices, start, end, self.analysis.bars, self.analysis.beats, self.multi_layer, quantize_rex(), self.classify, &|_, _| false);
    }

    /// With Classify on, [re:49a32c]: one classifier for the file runs over
    /// the first 128 slices in order (each up to the next slice, at most
    /// `2·sr` frames), skipping empty and deleted slices and locked slices
    /// that have a type.
    fn classify_slices(&mut self) {
        let Some(a) = self.audio.as_ref().filter(|_| self.classify) else { return };
        let (data, n) = (&a.data, a.data.left.len());
        let end = |i: usize| self.slices.get(i + 1).map_or(n, |s| s.start as usize).min(n);
        let max = (0..self.slices.len()).map(|i| end(i).saturating_sub(self.slices[i].start as usize)).max().unwrap_or(0);
        if max == 0 {
            return;
        }
        let mut c = classify::Classifier::new(max, if data.right.is_some() { 2 } else { 1 }, data.sample_rate);
        let ends: Vec<usize> = (0..self.slices.len()).map(end).collect();
        for (i, s) in self.slices.iter_mut().enumerate().take(128) {
            let a0 = s.start as usize;
            if ends[i] <= a0 || s.deleted || (s.locked && s.kind != 0) {
                continue;
            }
            let b = a0 + classify::slice_frames(ends[i] - a0, data.sample_rate);
            s.kind = classify::slice_type(c.process(&data.left[a0..b], data.right.as_ref().map(|r| &r[a0..b])));
        }
    }

    /// Tempo from bars and beats.
    fn tempo_from_bars(&mut self) {
        let (start, end) = self.region();
        let Some(sr) = self.audio.as_ref().map(|a| a.data.sample_rate) else { return };
        let secs = (end - start) as f32 / sr;
        let beats = self.analysis.bars * 4.0 + self.analysis.beats;
        if secs > 0.0 && beats > 0.0 {
            self.analysis.bpm = (beats * 60.0 / secs).clamp(20.0, 1000.0);
        }
    }

    /// Bars and beats from the tempo.
    fn bars_from_tempo(&mut self) {
        let (start, end) = self.region();
        let Some(sr) = self.audio.as_ref().map(|a| a.data.sample_rate) else { return };
        let beats = ((end - start) as f32 / sr * self.analysis.bpm / 60.0).round();
        self.analysis.bars = (beats / 4.0).floor();
        self.analysis.beats = beats - self.analysis.bars * 4.0;
    }

    pub fn sync(&mut self, app: &mut App) {
        let Some(op) = self.overlay.clone() else { return };
        let editing = app.is_editing();
        // [re:695294] 0x13a / 0x145: slides down over the LCD (350 ms
        // exponential), back up on close (350 ms logarithmic, then
        // [re:5e9662]).
        let panel = [op.clone(), vec![0]].concat();
        if let Some(r) = app.ctrl(&op).map(|c| c.rect) {
            let up = Rect::new(0, -r.h, r.w, r.h);
            let down = Rect::new(0, 0, r.w, r.h);
            if self.open && !self.shown {
                app.animate_rect(&panel, up, down, 350.0, dreist_gui::collapse::exponential, SLIDE_IN);
                self.shown = true;
                self.closing = false;
            } else if !self.open && self.shown && !self.closing {
                let from = app.ctrl(&panel).map(|c| c.rect).unwrap_or(down);
                app.animate_rect(&panel, from, up, 350.0, dreist_gui::collapse::curve, SLIDE_OUT);
                self.closing = true;
            }
        }
        // The other pages are off while slicing.
        let open = self.shown;
        if let Some(g) = app.root.find_mut("MainTabButtons") {
            for b in &mut g.children {
                b.enabled = !self.open;
            }
        }
        let (mode, multi, classify, auto) = (self.mode, self.multi_layer, self.classify, self.auto_load);
        let (sens, bpm, bars, beats, rex, file_bpm, div) = (self.sensitivity, self.analysis.bpm, self.analysis.bars, self.analysis.beats, self.rex, self.file_bpm, self.divisions);
        let Some(o) = app.ctrl_mut(&op) else { return };
        o.visible = open;
        if !open {
            return;
        }
        o.walk_mut(&mut |c: &mut Ctrl| match c.name.as_str() {
            "SliceModeButtonGroup" => {
                for (i, b) in c.children.iter_mut().enumerate() {
                    b.on = i == mode;
                }
            }
            "SensitivitySlider" => {
                c.visible = mode == 0;
                c.enabled = !rex;
                c.value = sens;
            }
            "ChopDivisionBox" => {
                c.visible = mode != 0;
                c.text = div.to_string();
            }
            "SenseLabel" => c.text = if mode == 0 { "Sensitivity" } else { "Divisions" }.into(),
            "AnalyzeButton" => {
                c.on = classify && mode == 0;
                c.enabled = mode == 0;
            }
            "MultiLayersButton" => c.on = multi,
            // The Extract buttons are for slicing without Auto load.
            "BothButton" | "AudioButton" | "PatternButton" | "ExtractLabel" => c.visible = !auto,
            "SrcBPM" => c.text = format!("{file_bpm:.2}"),
            "BPMBox" if rex => {
                c.enabled = false;
                c.text = format!("{bpm:.2}");
            }
            "BPMBox" if !editing => {
                c.enabled = true;
                c.text = format!("{bpm:.2}");
            }
            "BarsBox" if !editing => c.text = format!("{bars}"),
            "BeatsBox" if !editing => c.text = format!("{beats}"),
            _ => {}
        });
    }

    /// Cursor for the pointer over a slice marker.
    pub fn hover_cursor(&self, app: &App, path: &[usize], overload: &str, x: f32, _y: f32) -> Option<dreist_gui::cursor::Cursor> {
        if !self.mine(path) || overload != "EditorPreviewWaveEditor" {
            return None;
        }
        let w = app.ctrl(path).map(|c| c.rect.w as f32)?;
        Some(if self.marker_at(x, w).is_some() { dreist_gui::cursor::Cursor::LeftRight } else { dreist_gui::cursor::Cursor::Arrow })
    }

    fn mine(&self, path: &[usize]) -> bool {
        self.open && self.overlay.as_ref().is_some_and(|o| path.starts_with(o))
    }

    fn rebuild(model: &Model, out: &mut Vec<Command>) {
        out.push(model.rebuild_command());
    }

    /// Preview frames `start..end` ([re:e80b43] with the
    /// region markers).
    fn preview(&self, start: u32, end: u32, looping: bool, out: &mut Vec<Command>) {
        if let Some(a) = &self.audio {
            out.push(Command::PreviewRegion { data: a.data.clone(), start, end, looping });
        }
    }

    /// Slice preview at the click [re:e50ea1]: the last slice starting
    /// at or before `frame`, up to the next slice, clamped to the region;
    /// the whole region when there are no slices.
    pub fn slice_span(&self, frame: u32) -> Option<(u32, u32)> {
        let (start, end) = self.region();
        let live: Vec<u32> = self.slices.iter().filter(|s| !s.deleted).map(|s| s.start).collect();
        if live.is_empty() {
            return Some((start, end));
        }
        let i = live.iter().rposition(|&s| s <= frame)?;
        let next = live.get(i + 1).copied().unwrap_or(end);
        Some((live[i].max(start), next.min(end)))
    }

    /// [re:8402ce]: over the wave, a slice that the last extraction loaded
    /// names its place and type.
    pub fn wave_tooltip(&self, path: &[usize], x: f32, w: f32) -> Option<String> {
        if !self.mine(path) {
            return None;
        }
        let i = self.slice_of(self.frame_at(x, w))?;
        let &(_, e, p, l) = self.placed.iter().rev().find(|t| t.0 == self.slices[i].start)?;
        let kind = slice::SLICE_TYPE_NAMES.get(self.slices[i].kind as usize).unwrap_or(&"No type");
        Some(format!("Slice {} - Engine {} Pad {} Layer {} Type {kind}", i + 1, e + 1, p + 1, l + 1))
    }

    /// The slice a frame lies in (the last one starting at or before it).
    fn slice_of(&self, frame: u32) -> Option<usize> {
        self.slices.iter().rposition(|s| s.start <= frame)
    }

    fn close(&mut self) {
        self.open = false;
        self.snapshot = None;
        self.audio = None;
        self.menu = None;
        self.placed.clear();
    }

    /// Frame under a wave-editor x.
    fn frame_at(&self, x: f32, w: f32) -> u32 {
        let (a, b) = self.view;
        (a as f64 + (x.max(0.0) / w.max(1.0)) as f64 * (b - a) as f64) as u32
    }

    fn x_of(&self, frame: u32, w: f32) -> f32 {
        let (a, b) = self.view;
        (frame as f64 - a as f64) as f32 / (b - a).max(1) as f32 * w
    }

    /// Slice whose marker is within 4 px of x (deleted ones too).
    fn marker_at(&self, x: f32, w: f32) -> Option<usize> {
        self.slices.iter().enumerate().map(|(i, s)| (i, (self.x_of(s.start, w) - x).abs())).filter(|(_, d)| *d <= 4.0).min_by(|a, b| a.1.total_cmp(&b.1)).map(|(i, _)| i)
    }

    /// The view as fractions of the file (start, size), for the scrollbar.
    fn view_frac(&self) -> (f32, f32) {
        let n = self.frames().max(1) as f32;
        let (a, b) = self.view;
        (a as f32 / n, (b - a) as f32 / n)
    }

    fn set_view_frac(&mut self, v: (f32, f32)) {
        let n = self.frames().max(1) as f64;
        let (s, z) = crate::zoomslider::clamp(v);
        let a = (s as f64 * n).round();
        let len = (z as f64 * n).round().clamp(64f64.min(n), n);
        let a = a.min(n - len).max(0.0);
        self.view = (a as u32, (a + len) as u32);
    }

    /// Zoom the view by `factor` about the fraction `centre` of it.
    fn zoom(&mut self, factor: f64, centre: f64) {
        let n = self.frames().max(1) as u64;
        let (a, b) = self.view;
        let len = (((b - a) as f64 / factor).round() as u64).clamp(64.min(n), n);
        let at = a as f64 + (b - a) as f64 * centre.clamp(0.0, 1.0);
        let a2 = (at - len as f64 * centre).max(0.0) as u64;
        let a2 = a2.min(n - len);
        self.view = (a2 as u32, (a2 + len) as u32);
    }

    /// With Auto load on, the edit goes to the pads and the pattern at once.
    fn edited(&mut self, model: &mut Model, out: &mut Vec<Command>) {
        if self.auto_load {
            self.extract(model, Extract::Both, out);
        }
    }

    pub fn handle(&mut self, app: &mut App, model: &mut Model, ev: &UiEvent, out: &mut Vec<Command>) -> bool {
        if let (UiEvent::Animated { path, tag: SLIDE_OUT }, Some(op)) = (ev, self.overlay.as_ref()) {
            if self.closing && path.starts_with(op) {
                self.closing = false;
                self.shown = false;
                if let Some(o) = app.ctrl_mut(op) {
                    o.visible = false;
                }
                return true;
            }
        }
        match ev {
            UiEvent::Clicked { path, name, on, .. } if self.mine(path) => {
                let parent = app.ctrl(&path[..path.len() - 1]).map(|c| c.name.clone()).unwrap_or_default();
                if parent == "SliceModeButtonGroup" {
                    self.mode = (*path.last().unwrap()).min(1);
                    self.reslice();
                    self.edited(model, out);
                    return true;
                }
                match name.as_str() {
                    // Classify (Transient only): the state is kept; slices are
                    // allocated in order.
                    "AnalyzeButton" => {
                        self.classify = *on;
                        self.classify_slices();
                        self.update_pattern();
                        self.edited(model, out);
                    }
                    "MultiLayersButton" => {
                        self.multi_layer = *on;
                        self.update_pattern();
                        self.edited(model, out);
                    }
                    "BothButton" => self.extract(model, Extract::Both, out),
                    "AudioButton" => self.extract(model, Extract::Audio, out),
                    "PatternButton" => self.extract(model, Extract::Pattern, out),
                    "ZoomInButton" | "ZoomOutButton" => self.zoom(if name == "ZoomInButton" { 2.0 } else { 0.5 }, 0.5),
                    "CancelButton" => {
                        out.push(crate::command::msg(dreist_engine::binding::id::STOP_PREVIEW, &[], &[]));
                        if self.extracted {
                            if let Some(d) = self.snapshot.take() {
                                model.doc = d;
                                Self::rebuild(model, out);
                            }
                        }
                        self.close();
                    }
                    "DoneButton" => {
                        out.push(crate::command::msg(dreist_engine::binding::id::STOP_PREVIEW, &[], &[]));
                        if let Some(f) = &self.file {
                            // The analysis is saved next to the audio file.
                            let mut a = self.analysis.clone();
                            a.slices = self.slices.clone();
                            let _ = a.save(&f.with_extension("gsa"));
                        }
                        self.close();
                    }
                    _ => {}
                }
                true
            }
            UiEvent::MenuSelected { anchor, index, .. } if self.mine(anchor) => {
                let Some(i) = self.menu.take() else { return true };
                let Some(s) = self.slices.get_mut(i) else { return true };
                match index {
                    0 => s.locked = !s.locked,
                    1 => s.deleted = !s.deleted,
                    // Type overrides lock the slice, which keeps the type
                    // through the next classification ([re:49a32c]).
                    3..=6 => {
                        s.kind = (*index - 2) as u8;
                        s.locked = true;
                    }
                    _ => return true,
                }
                self.update_pattern();
                self.edited(model, out);
                true
            }
            UiEvent::ValueChanged { path, name, value, .. } if self.mine(path) && name == "SensitivitySlider" => {
                self.sensitivity = value.clamp(0.0, 1.0);
                self.reslice();
                self.edited(model, out);
                true
            }
            UiEvent::TextCommitted { path, name, text } if self.mine(path) => {
                let v: f32 = text.trim().parse().unwrap_or(f32::NAN);
                if v.is_nan() {
                    return true;
                }
                match name.as_str() {
                    "ChopDivisionBox" => {
                        self.divisions = (v.round() as i64).clamp(1, 256) as u32;
                        self.reslice();
                    }
                    "BPMBox" => {
                        self.analysis.bpm = v.clamp(20.0, 1000.0);
                        if self.auto_tempo {
                            self.bars_from_tempo();
                        }
                    }
                    "BarsBox" | "BeatsBox" => {
                        if name == "BarsBox" {
                            self.analysis.bars = v.clamp(0.0, 1000.0).floor();
                        } else {
                            self.analysis.beats = v.clamp(0.0, 100.0);
                        }
                        if self.auto_tempo {
                            while self.analysis.beats >= 4.0 {
                                self.analysis.bars += 1.0;
                                self.analysis.beats -= 4.0;
                            }
                            self.tempo_from_bars();
                        }
                    }
                    _ => return false,
                }
                self.update_pattern();
                self.edited(model, out);
                true
            }
            UiEvent::Mouse { path, overload, kind, x, .. } if self.mine(path) && overload == "SliceChopStepper" => {
                if *kind == MouseKind::Down {
                    let w = app.ctrl(path).map(|c| c.rect.w as f32).unwrap_or(1.0);
                    self.divisions = if *x < w / 2.0 { self.divisions.saturating_sub(1).max(1) } else { (self.divisions + 1).min(256) };
                    self.reslice();
                    self.edited(model, out);
                }
                true
            }
            UiEvent::Wheel { path, overload, dy } if self.mine(path) && overload == "SliceChopStepper" => {
                self.divisions = (self.divisions as i32 + dy.signum() as i32).clamp(1, 256) as u32;
                self.reslice();
                self.edited(model, out);
                true
            }
            // The zoom scrollbar: drag the thumb to scroll, an edge to zoom
            // (as the layer page's wave zoom bar).
            UiEvent::Mouse { path, overload, kind, x, y } if self.mine(path) && overload == "WaveDisplayZoomSlider" && app.ctrl(path).is_some_and(|c| c.name == ZOOM_BAR) => {
                let (w, h) = app.ctrl(path).map(|c| (c.rect.w as f32, c.rect.h as f32)).unwrap_or((1.0, 1.0));
                match kind {
                    MouseKind::Down => self.bar_drag = crate::zoomslider::hit(self.view_frac(), w, h, *x, *y).map(|m| (m, *x)),
                    MouseKind::Drag => {
                        if let Some((mode, last)) = self.bar_drag {
                            let min = 64.0 / self.frames().max(64) as f32;
                            self.set_view_frac(crate::zoomslider::drag(self.view_frac(), w, mode, last, *x, min));
                            self.bar_drag = Some((mode, *x));
                        }
                    }
                    MouseKind::Up => self.bar_drag = None,
                    _ => {}
                }
                true
            }
            UiEvent::Wheel { path, overload, dy } if self.mine(path) && overload == "EditorPreviewWaveEditor" => {
                let w = app.ctrl(path).map(|c| c.rect.w as f32).unwrap_or(1.0);
                let mx = ((app.mouse_pos().0 - app.origin(path).0 as f32) / w.max(1.0)) as f64;
                self.zoom(if *dy > 0.0 { 2.0 } else { 0.5 }, mx);
                true
            }
            UiEvent::Mouse { path, overload, kind, x, y } if self.mine(path) && overload == "EditorPreviewWaveEditor" => {
                let (w, h) = app.ctrl(path).map(|c| (c.rect.w as f32, c.rect.h as f32)).unwrap_or((1.0, 1.0));
                let third = (h - WAVE_TOP) / 3.0;
                let transient = self.mode == 0;
                match kind {
                    MouseKind::Down => {
                        let hit = self.marker_at(*x, w);
                        match hit {
                            // Alt-click deletes (restores) a slice point.
                            Some(i) if app.mods.alt && transient => {
                                self.slices[i].deleted = !self.slices[i].deleted;
                                self.update_pattern();
                                self.edited(model, out);
                            }
                            Some(i) => {
                                self.selected = Some(i);
                                self.drag = transient.then_some(i);
                            }
                            // The ruler: a new slice, locked.
                            None if *y < WAVE_TOP => {
                                if transient {
                                    let f = self.frame_at(*x, w);
                                    self.slices.push(SlicePoint::manual(f, true));
                                    self.slices.sort_by_key(|s| s.start);
                                    self.selected = self.slices.iter().position(|s| s.start == f);
                                    self.update_pattern();
                                    self.edited(model, out);
                                }
                            }
                            // Upper wave: audition the slice.
                            None if *y < WAVE_TOP + third => {
                                let f = self.frame_at(*x, w);
                                self.selected = self.slice_of(f);
                                if let Some((a, b)) = self.slice_span(f) {
                                    self.preview(a, b, false, out);
                                }
                            }
                            // The middle: zoom / scroll; below: select.
                            None if *y < WAVE_TOP + 2.0 * third => self.view_drag = Some((*x, *y, self.view)),
                            None => self.selected = self.slice_of(self.frame_at(*x, w)),
                        }
                        true
                    }
                    MouseKind::Drag => {
                        if let Some((x0, y0, v)) = self.view_drag {
                            self.view = v;
                            let (zs, ss) = crate::zoomslider::drag_signs();
                            self.zoom((-(y - y0) as f64 * zs as f64 / 100.0).exp2(), (x0 / w.max(1.0)) as f64);
                            let (a, b) = self.view;
                            let shift = ((x - x0) / w.max(1.0)) as f64 * (b - a) as f64 * ss as f64;
                            let n = self.frames() as f64;
                            let a2 = (a as f64 + shift).clamp(0.0, (n - (b - a) as f64).max(0.0));
                            self.view = (a2 as u32, (a2 + (b - a) as f64) as u32);
                            return true;
                        }
                        let Some(i) = self.drag else { return false };
                        let f = self.frame_at(*x, w).min(self.frames().saturating_sub(1));
                        if let Some(s) = self.slices.get_mut(i) {
                            s.start = f;
                            s.pos = f as i32;
                        }
                        true
                    }
                    MouseKind::Up => {
                        self.view_drag = None;
                        if self.drag.take().is_some() {
                            self.slices.sort_by_key(|s| s.start);
                            self.update_pattern();
                            self.edited(model, out);
                        }
                        true
                    }
                    // The slice menu (Transient mode): on the marker, else the
                    // slice the click is in.
                    MouseKind::RightDown => {
                        if !transient {
                            return true;
                        }
                        let Some(i) = self.marker_at(*x, w).or_else(|| self.slice_of(self.frame_at(*x, w))) else { return true };
                        let (s, classify) = (self.slices[i], self.classify);
                        let items = SLICE_MENU
                            .iter()
                            .enumerate()
                            .map(|(k, t)| match k {
                                0 if s.locked => "Unlock slice".to_string(),
                                1 if s.deleted => "Restore slice".to_string(),
                                0..=2 => t.to_string(),
                                // Type overrides, with Classify on.
                                _ => dreist_gui::menu::item(*t, classify),
                            })
                            .collect();
                        self.selected = Some(i);
                        app.open_menu(path, items, None);
                        self.menu = Some(i);
                        true
                    }
                }
            }
            // The panel swallows everything else aimed at it.
            UiEvent::Mouse { path, .. } | UiEvent::Wheel { path, .. } => self.mine(path),
            _ => false,
        }
    }

    /// Extract slices to the selected engine's pads and/or current pattern
    /// (from the document as it was when the panel opened):
    /// [re:dd2b54] with the engine's locks,
    /// then [re:cb6b75]:
    /// - fewer than two slices: the whole file into the current pad's first
    ///   layer;
    /// - else every (pad, layer) of the engine: a slice → the layer gets the
    ///   file with the slice's region; none → the layer's sample is removed
    ///   (layer clear [re:7d1e9f], 0xe1, unless pad or layer is locked);
    /// - each loaded layer: `e_resetLayerEnvelopes` (0x86,
    ///   [re:4220aa]) then AmpEnvAttack 0.1;
    /// - pattern (unless the current pattern is locked): a new pattern
    ///   (0xa7) with the hits, the old `MonoModeActive` and the old locked
    ///   tracks.
    pub fn extract(&mut self, model: &mut Model, what: Extract, out: &mut Vec<Command>) {
        let (Some(file), Some(snap)) = (self.file.clone(), self.snapshot.clone()) else { return };
        model.doc = snap;
        let e = model.engine;
        let frames = self.frames();
        let sr = self.audio.as_ref().map(|a| a.data.sample_rate).unwrap_or(44100.0);
        let (start, end) = self.region();
        let stem = file.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        let pat_index = model.current_pattern(e);
        let blocked = |p: usize, l: usize| {
            model.pad_node(e, p).is_some_and(|n| n.locked())
                || model.layer_node(e, p, l).is_some_and(|n| n.locked())
                || model.pattern_node(e, pat_index).and_then(|sp| sp.children.iter().find(|t| t.class == "Track" && t.name.as_deref() == Some(&format!("Track{p}")))).is_some_and(|n| n.locked())
        };
        self.pattern = slice::extract_pattern_full(&self.slices, start, end, self.analysis.bars, self.analysis.beats, self.multi_layer, quantize_rex(), self.classify, &blocked);
        let usable: Vec<SlicePoint> = self.slices.iter().filter(|s| !s.deleted).copied().collect();
        let load = |model: &mut Model, p: usize, l: usize, s0: u32, s1: u32, name: &str| {
            load_region(model, e, p, l, &file, (s0, s1, frames, sr), name);
        };
        if what != Extract::Pattern {
            self.placed.clear();
            if usable.len() < 2 {
                let p = model.pad[e];
                load(model, p, 0, 0, frames, "");
            } else {
                let mut layout = [[None::<(usize, usize)>; crate::model::LAYERS]; 64];
                for h in &self.pattern.hits {
                    if let Some(k) = usable.iter().position(|s| s.start == self.slices[h.slice].start) {
                        layout[h.pad][h.layer] = Some((k, h.slice));
                    }
                }
                for (p, row) in layout.iter().enumerate() {
                    for (l, slot) in row.iter().enumerate() {
                        match slot {
                            Some((k, si)) => {
                                self.placed.push((self.slices[*si].start, e, p, l));
                                let (s0, s1) = slice::slice_markers(&usable, *k, end);
                                load(model, p, l, s0, s1, &format!("{stem} {}", si + 1));
                            }
                            None => clear_layer(model, e, p, l),
                        }
                    }
                }
            }
        }
        let pattern_locked = model.pattern_node(e, pat_index).is_some_and(|n| n.locked());
        if what != Extract::Audio && !pattern_locked {
            let eng = model.engine_mut(e);
            let sp = eng.child_mut_or_insert("StepPattern", &format!("StepPattern{pat_index}"));
            let mono = sp.attr("MonoModeActive").map(str::to_string);
            let locked: Vec<Node> = sp.children.iter().filter(|t| t.class == "Track" && t.locked()).cloned().collect();
            let name = sp.name.clone();
            let version = sp.version;
            *sp = Node::new("StepPattern", name.as_deref());
            sp.version = version;
            sp.set("PatternSteps", self.pattern.steps);
            if let Some(m) = mono {
                sp.set("MonoModeActive", m);
            }
            for h in &self.pattern.hits {
                let tr = sp.child_mut_or_insert("Track", &format!("Track{}", h.pad));
                let st = tr.child_mut_or_insert("Step", &format!("S{}", h.step));
                st.set("State", "true").set("Velocity", 1);
                if (h.shift - 0.5).abs() > 1e-6 {
                    st.set("Shift", h.shift);
                }
                if self.multi_layer {
                    st.set("ForceLayer", h.force_layer);
                }
            }
            for t in locked {
                sp.children.retain(|c| !(c.class == "Track" && c.name == t.name));
                sp.children.push(t);
            }
        }
        self.extracted = true;
        Self::rebuild(model, out);
    }

    /// [re:a1dc9f] / [re:8fe850]: the selected slice (else the whole
    /// region) dragged onto pad `p`'s layer `l`: the file with that region.
    pub fn drop_on_pad(&self, model: &mut Model, e: usize, p: usize, l: usize) -> bool {
        let Some(file) = self.file.clone() else { return false };
        let frames = self.frames();
        let sr = self.audio.as_ref().map(|a| a.data.sample_rate).unwrap_or(44100.0);
        let (start, end) = self.region();
        let usable: Vec<SlicePoint> = self.slices.iter().filter(|s| !s.deleted).copied().collect();
        let (s0, s1) = match self.selected.and_then(|i| self.slices.get(i)).and_then(|sel| usable.iter().position(|u| u.start == sel.start)) {
            Some(k) => slice::slice_markers(&usable, k, end),
            None => (start, end),
        };
        load_region(model, e, p, l, &file, (s0, s1, frames, sr), "")
    }

    pub fn draw(&self, model: &Model, r: &Renderer, p: &mut Painter, c: &Ctrl) -> bool {
        let s = r.styles;
        let (w, h) = (c.rect.w as f32, c.rect.h as f32);
        let ink = crate::sampler::wave(r, "colLine", Colour::rgba(18, 25, 60, 255));
        let amber = colour(r, "SelectedObjectColour", Colour::rgba(236, 189, 47, 255));
        match c.overload.as_str() {
            // The overlay only holds the panel (no shade); the panel has an
            // LCD background.
            "SlicerOverlay" => true,
            "SlicePanel" => {
                p.fill_rect(RectF::new(0.0, 0.0, w, h), s.colour("PreviewPanelBackgroundStyle", "colFill", Colour::rgba(210, 217, 227, 255)));
                true
            }
            "EditorPreviewWaveEditor" => {
                p.fill_rect(RectF::new(0.0, 0.0, w, h), crate::sampler::wave(r, "colFill", Colour::rgba(178, 170, 158, 255)));
                let Some(a) = &self.audio else { return true };
                let (v0, v1) = self.view;
                let sr = a.data.sample_rate.max(1.0) as f64;
                crate::sampler::draw_ruler(r, p, w, v0 as f64 / sr, (v1 - v0) as f64 / sr, false, model.tempo());
                let area = RectF::new(0.0, WAVE_TOP, w, h - WAVE_TOP);
                // Selected slice region.
                if let Some(i) = self.selected.filter(|&i| i < self.slices.len()) {
                    let (s0, s1) = slice::slice_markers(&self.slices, i, self.analysis.end);
                    let (x0, x1) = (self.x_of(s0, w), self.x_of(s1, w));
                    p.fill_rect(RectF::new(x0, WAVE_TOP, (x1 - x0).max(0.0), h - WAVE_TOP), colour(r, "WaveEditorSelectionColour", amber.with_alpha(0.3)));
                }
                crate::sampler::draw_wave(p, area, &a.buckets, BUCKET, v0 as f64, v1 as f64, ink, (0.0, w));
                let line = colour(r, "PreviewEditorSliceMarkerColour", ink);
                let dim = crate::sampler::wave(r, "colLineDisabled", ink.with_alpha(0.4));
                for (i, sl) in self.slices.iter().enumerate() {
                    if sl.start < v0 || sl.start >= v1 {
                        continue;
                    }
                    let x = self.x_of(sl.start, w).floor() + 0.5;
                    let col = if sl.deleted { dim } else { line };
                    p.line(x, RULER, x, h, col, 1.0);
                    let head = if self.selected == Some(i) { amber } else { col };
                    let (hx, hy) = (x - 6.0, RULER + 2.0);
                    if sl.locked {
                        p.fill_rect(RectF::new(hx + 1.5, hy + 4.0, 9.0, 6.0), head);
                        p.stroke_rect(RectF::new(hx + 3.5, hy, 5.0, 5.0), head, 1.0);
                    } else if sl.deleted {
                        p.polyline(&[(hx, hy), (hx + 12.0, hy), (hx + 6.0, hy + 10.0), (hx, hy)], head, 1.0);
                    } else {
                        p.fill_polygon(&[(hx, hy), (hx + 12.0, hy), (hx + 6.0, hy + 10.0)], head);
                    }
                }
                p.stroke_rect(RectF::new(0.5, 0.5, w - 1.0, h - 1.0), crate::sampler::wave(r, "colTimelineMarkers", ink), 1.0);
                true
            }
            "WaveDisplayZoomSlider" if c.name == ZOOM_BAR => {
                crate::zoomslider::draw(p, s, &c.style, w, h, self.view_frac());
                true
            }
            "SliceChopStepper" => {
                let st = r.style(c);
                p.fill_rect(RectF::new(0.0, 0.0, w, h), s.colour(st, "colFill", Colour::WHITE));
                p.stroke_rect(RectF::new(0.5, 0.5, w - 1.0, h - 1.0), s.colour(st, "colBorder", Colour::BLACK), 1.0);
                let font = s.font(st);
                let text = s.colour(st, "colText", Colour::BLACK);
                p.text(RectF::new(0.0, 0.0, w, h), &format!("◂ {} ▸", self.divisions), text, &font, HAlign::Center, VAlign::Center);
                true
            }
            // The pattern the slicing makes (with Auto load or once
            // extracted), else the current pattern; pad 16 on top.
            "EditorPreviewPanelPatternDisplay" => {
                p.fill_rect(RectF::new(0.0, 0.0, w, h), s.colour("PreviewPanelBackgroundStyle", "colFill", Colour::rgba(210, 217, 227, 255)));
                let e = model.engine;
                let (steps, hits): (u32, Vec<(usize, f32, usize)>) = if self.auto_load || self.extracted {
                    (self.pattern.steps, self.pattern.hits.iter().map(|h| (h.step, h.shift, h.pad)).collect())
                } else {
                    let pat = model.pattern(e, model.current_pattern(e));
                    let hits = pat.tracks.iter().enumerate().flat_map(|(t, tr)| tr.steps.iter().enumerate().filter(|(_, st)| st.on).map(move |(k, _)| (k, 0.5, t))).collect();
                    (pat.steps, hits)
                };
                let steps = steps.max(1) as f32;
                let rows = 16.0;
                let (cw, rh) = (w / steps, h / rows);
                let grid = colour(r, "PatternGridColourLight", ink.with_alpha(0.5));
                for k in 0..steps as usize {
                    let x = (k as f32 * cw).floor() + 0.5;
                    p.line(x, 0.0, x, h, if k % 4 == 0 { ink } else { grid.with_alpha(0.4) }, 1.0);
                }
                for (step, shift, pad) in hits {
                    if pad >= 16 {
                        continue;
                    }
                    let x = step as f32 * cw + (shift - 0.5) * cw;
                    let y = (15 - pad) as f32 * rh;
                    p.fill_rect(RectF::new(x + 1.0, y + 1.0, (cw - 2.0).max(1.0), (rh - 1.0).max(1.0)), ink);
                }
                p.stroke_rect(RectF::new(0.5, 0.5, w - 1.0, h - 1.0), ink, 1.0);
                true
            }
            _ => false,
        }
    }
}

/// [re:4220aa]: amp and free envelopes to attack 0,
/// decay 0.02, sustain 1, release 0.001, curves 0.5, then the hold reset
/// (no pitch compensation) for both holds.
fn reset_envelopes(layer: &mut Node, region: f64, total: f64, sr: f32) {
    use dreist_formats::params::LayerParam as P;
    for (a, d, su, r, ac, dc, rc) in [
        (P::AmpEnvAttack, P::AmpEnvDecay, P::AmpEnvSustain, P::AmpEnvRelease, P::AmpEnvAttackCurve, P::AmpEnvDecayCurve, P::AmpEnvReleaseCurve),
        (P::FreeEnvAttack, P::FreeEnvDecay, P::FreeEnvSustain, P::FreeEnvRelease, P::FreeEnvAttackCurve, P::FreeEnvDecayCurve, P::FreeEnvReleaseCurve),
    ] {
        layer.set(a.name(), 0.0).set(d.name(), 0.02).set(su.name(), 1.0).set(r.name(), 0.001);
        layer.set(ac.name(), 0.5).set(dc.name(), 0.5).set(rc.name(), 0.5);
    }
    let n = layer.clone();
    let get = |q: P| n.f32(q.name()).unwrap_or(q.default_value()).clamp(0.0, 1.0);
    let bound = n.attr("PlaybackMode").map(dreist_dsp::voice::PlaybackMode::canonical) == Some("OneShotLengthBound");
    let h = dreist_engine::hold::reset_hold_time(&get, bound, region, total, sr, false);
    layer.set(P::AmpEnvHold.name(), h).set(P::FreeEnvHold.name(), h);
}

/// [re:b30133] → 0xe1 `e_deallocateSample`:
/// the layer loses its sample unless the pad or the layer is locked.
fn clear_layer(model: &mut Model, e: usize, p: usize, l: usize) {
    if model.layer_node(e, p, l).is_none() || model.pad_node(e, p).is_some_and(|n| n.locked()) || model.layer_node(e, p, l).is_some_and(|n| n.locked()) {
        return;
    }
    let layer = model.layer_mut(e, p, l);
    layer.attrs.retain(|(k, _)| k != "AudioFileName");
    layer.children.retain(|c| c.class != "AudioFileMarkers");
}

/// Load `file` with region (start, end) of (frames at sr) into (e, p, l)
/// and reset its envelopes ([re:4220aa], AmpEnvAttack 0.1).
fn load_region(model: &mut Model, e: usize, p: usize, l: usize, file: &Path, (s0, s1, frames, sr): (u32, u32, u32, f32), name: &str) -> bool {
    if !crate::browser::load_sample_into_layer(model, e, p, l, file) {
        return false;
    }
    let layer = model.layer_mut(e, p, l);
    layer.children.retain(|c| c.class != "AudioFileMarkers");
    for nm in ["StartEndMarkers", "LoopMarkers"] {
        let mut m = Node::new("AudioFileMarkers", Some(nm));
        m.set("StartSample", s0).set("EndSample", s1).set("SampleLength", frames);
        layer.children.push(m);
    }
    reset_envelopes(layer, (s1 - s0) as f64, frames as f64, sr);
    layer.set(dreist_formats::params::LayerParam::AmpEnvAttack.name(), 0.1);
    if !name.is_empty() {
        model.set_layer_name(e, p, l, name);
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reslice_modes_keep_locked() {
        crate::app::test_setup();
        let mut s = Slicer::closed(None);
        s.open = true;
        s.analysis = SliceFile::new(44100);
        s.mode = 1;
        s.divisions = 4;
        s.auto_tempo = false;
        s.view = (0, 44100);
        assert_eq!(s.slice_span(500), Some((0, 44100)), "no slices: whole region");
        s.reslice();
        assert_eq!(s.slices.len(), 4);
        s.slices.push(SlicePoint::manual(1000, true));
        s.divisions = 2;
        s.reslice();
        // Divide replaces every slice.
        assert_eq!(s.slices.len(), 2, "{:?}", s.slices);
        assert_eq!(s.pattern.hits.len(), 2);
        assert_eq!(s.slice_span(30000), Some((22050, 44100)));
        assert!(s.slices.iter().all(|p| p.locked), "Divide makes locked slices (v1)");
        // Back in Transient the divided slices go.
        s.mode = 0;
        s.reslice();
        assert!(!s.slices.iter().any(|p| p.locked));
        // Transient keeps the locked and the deleted ones.
        s.slices = vec![SlicePoint::manual(1000, true), SlicePoint { deleted: true, ..SlicePoint::manual(3000, false) }];
        s.reslice();
        assert!(s.slices.iter().any(|p| p.start == 1000 && p.locked));
        assert!(s.slices.iter().any(|p| p.start == 3000 && p.deleted));
        assert_eq!(s.slice_span(3500), Some((1000, 44100)), "deleted slices do not cut");
    }

    #[test]
    fn classify_types_slices_and_allocates_by_type() {
        // Two decaying 55 Hz hits: kicks.
        let sr = 44100usize;
        let mut left = vec![0.0f32; sr * 2];
        for p in [0, sr] {
            for i in 0..sr {
                left[p + i] = (i as f32 * 55.0 * std::f32::consts::TAU / sr as f32).sin() * (-(i as f32) / 4000.0).exp();
            }
        }
        let mut s = Slicer::closed(None);
        s.open = true;
        s.analysis = SliceFile::new(sr as u32 * 2);
        s.audio = Some(Audio { data: std::sync::Arc::new(dreist_dsp::voice::SampleData { left, right: None, sample_rate: sr as f32 }), buckets: Vec::new() });
        s.view = (0, sr as u32 * 2);
        s.slices = vec![SlicePoint::manual(0, false), SlicePoint::manual(sr as u32, false)];
        s.classify = true;
        s.classify_slices();
        s.update_pattern();
        assert!(s.slices.iter().all(|p| p.kind == 1), "{:?}", s.slices);
        assert_eq!(s.pattern.hits.iter().map(|h| h.pad).collect::<Vec<_>>(), vec![0, 1]);
        // A locked slice with a type keeps it.
        s.slices[1] = SlicePoint { kind: 3, ..SlicePoint::manual(sr as u32, true) };
        s.classify_slices();
        assert_eq!(s.slices[1].kind, 3);
        // The tooltip names an extracted slice's place and type.
        s.placed = vec![(sr as u32, 0, 9, 0)];
        s.overlay = Some(vec![7]);
        let path = [7, 0];
        assert_eq!(s.wave_tooltip(&path, 300.0, 400.0).as_deref(), Some("Slice 2 - Engine 1 Pad 10 Layer 1 Type Hihat"));
        assert_eq!(s.wave_tooltip(&path, 10.0, 400.0), None);
    }
}
