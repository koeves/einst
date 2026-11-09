//! The Pad / Layers page (`Pad Editor/LayerEditor.skin`): the pad / layer
//! selector on the left (the pad item `PadSelectionButton.skin` and eight
//! layer rows `SelectionButton.skin`), and the editor on the right — the
//! pad view (`PadControls.skin`: pan, gain, sends, output, Vel-Amp, layer
//! play mode and the velocity split lanes) when the pad item is selected,
//! the layer view (`PadEditorWavePanel.skin`: setup, timestretch and
//! envelope panels, the wave display with both envelopes, the zoom bar)
//! when a layer is (manual §5:1–5:9).
//!
//! - The wave display draws the waveform with its markers ([re:68305f]:
//!   start/end 2 px lines with a round handle), the timeline and the Amp /
//!   Free envelopes over it ([re:18fbfe]).
//! - Layer controls post `e_setLayerParameter` 0x66 [param, value, source];
//!   Vel-Pitch / Vel-Cutoff are the Velocity depths of CoarseTune / Cutoff
//!   and the envelope destination depths the Amp / Free envelope depths of
//!   Cutoff, Resonance, Drive or Pitch (knob 0.5 = none, depth 2v − 1, as
//!   the v1 document conversion maps them).

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;

use dreist_engine::binding::id;
use dreist_formats::{LayerParam, PadParam, TransmodSource};
use dreist_gui::app::{App, MouseKind, UiEvent};
use dreist_gui::paint::{Painter, RectF};
use dreist_gui::render::Renderer;
use dreist_gui::{Colour, Ctrl, Rect, Skin};

use crate::EditorError;
use crate::command::{Command, msg};
use crate::model::{LAYERS, Model};

const WAVE_STYLE: &str = "WaveDisplayStyle";

/// A wave display colour (`WaveDisplayStyle`'s `attr`).
fn wave_colour(s: &dreist_gui::style::StyleSheet, attr: &str, default: Colour) -> Colour {
    s.colour(WAVE_STYLE, attr, default)
}

/// The layer toggles posted as their own messages: Reverse
/// `IsReversed`, Phase `PhasePosition` < 0, envelopes track length
/// `EnvTracksPitch`.
pub fn layer_toggle_state(node: Option<&dreist_formats::Node>, name: &str) -> Option<bool> {
    let truthy = |k: &str| node.is_some_and(|n| n.truthy(k));
    Some(match name {
        "RevButton" | "Reverse" => truthy("IsReversed"),
        "PhaseButton" | "Phase" => node.and_then(|n| n.f32("PhasePosition")).is_some_and(|v| v < 0.0),
        "TrackPitchButton" | "EnvMode" => truthy("EnvTracksPitch"),
        "LayerMode" => node.and_then(|n| n.attr("SynthesisMode")).is_some_and(|v| v.trim() == "VintageSampler"),
        _ => return None,
    })
}

/// The message a layer toggle posts.
fn toggle_message(name: &str) -> Option<u32> {
    Some(match name {
        "RevButton" | "Reverse" => id::SET_LAYER_REVERSE_STATUS,
        "PhaseButton" | "Phase" => id::SET_LAYER_PHASE_POSITION,
        "TrackPitchButton" | "EnvMode" => id::SET_LAYER_ENVELOPE_TRACKING,
        "LayerMode" => id::SET_LAYER_SYNTHESIS_MODE,
        _ => return None,
    })
}

/// The layer parameter a layer view control sets directly.
pub fn param_for_control(name: &str) -> Option<LayerParam> {
    let n = match name {
        "PanKnob" => "Pan",
        "GainKnob" => "Gain",
        "TuneKnob" => "CoarseTune",
        "FineKnob" => "FineTune",
        "PreDelayKnob" | "PreDelayTime" => "PreDelay",
        "CutoffKnob" | "Cutoff" => "FilterCutoff",
        "ResKnob" | "Resonance" => "FilterResonance",
        "DriveKnob" => "FilterDrive",
        "AmountKnob" => "TimestretchAmount",
        "FilterActiveButton" => "FilterActive",
        "TSActiveButton" => "TimestretchActive",
        "TrackBPMButton" => "TimestretchTracksBPM",
        n => n,
    };
    LayerParam::from_name(n)
}

/// Bipolar depth knobs: Vel-Pitch / Vel-Cutoff (the Velocity depth of a
/// parameter; knob 0.5 = none).
fn velocity_control(name: &str) -> Option<LayerParam> {
    match name {
        "V2P" => Some(LayerParam::CoarseTune),
        "V2C" => Some(LayerParam::FilterCutoff),
        _ => None,
    }
}

/// Envelope destinations (manual §5:8): any of Cutoff, Resonance, Drive
/// and Pitch, two per envelope.
const ENV_DESTS: [(&str, LayerParam); 4] = [("Cutoff", LayerParam::FilterCutoff), ("Resonance", LayerParam::FilterResonance), ("Drive", LayerParam::FilterDrive), ("Pitch", LayerParam::CoarseTune)];
const NO_DEST: &str = "No destination";

/// `AmpEnvDest1Combo` → (amp, slot 0); `FreeEnvDepth2Knob` → (free, slot 1).
fn env_slot(name: &str, kind: &str) -> Option<(bool, usize)> {
    let (amp, rest) = if let Some(r) = name.strip_prefix("AmpEnv") { (true, r) } else { (false, name.strip_prefix("FreeEnv")?) };
    let n = rest.strip_prefix(kind)?.strip_suffix(if kind == "Dest" { "Combo" } else { "Knob" })?.parse::<usize>().ok()?;
    (1..=2).contains(&n).then_some((amp, n - 1))
}

/// The original v1's routing slot of an envelope's destination row (the
/// amp envelope's rows are routings 0 and 1, the free envelope's 2 and 3).
fn routing_index(amp: bool, slot: usize) -> usize {
    if amp { slot } else { 2 + slot }
}

/// An [`ENV_DESTS`] choice (1-based, 0 none) as a routing destination (0
/// pitch, 1 cutoff, 2 resonance, 3 drive), and back.
fn routing_dest(choice: usize) -> Option<u8> {
    match choice {
        1..=3 => Some(choice as u8),
        4 => Some(0),
        _ => None,
    }
}

fn dest_choice(dest: u8) -> usize {
    if dest == 0 { 4 } else { dest as usize }
}

/// The layer's v1 routing slots, when it has them (converted v1 documents).
fn layer_slots(model: &Model, e: usize, p: usize, l: usize) -> Option<[dreist_formats::legacy::RoutingSlot; 4]> {
    model.layer_node(e, p, l).and_then(dreist_formats::legacy::routing_slots)
}

/// After a routing slot changed: each envelope's depth on every destination
/// as the mean of its slots there (`2d − 1`, as the conversion writes it,
/// for the v2 voice and the transmod view), then a rebuild so the voices
/// read the slots.
fn sync_slots(model: &Model, e: usize, p: usize, l: usize, out: &mut Vec<Command>) {
    let Some(slots) = layer_slots(model, e, p, l) else { return };
    let idx = [e as i32, p as i32, l as i32];
    for amp in [true, false] {
        for (k, (_, par)) in ENV_DESTS.iter().enumerate() {
            let Some(id) = routing_dest(k + 1) else { continue };
            let ds: Vec<f32> = [0, 1].iter().filter_map(|&s| slots[routing_index(amp, s)]).filter(|(d, v)| *d == id && (v - 0.5).abs() >= 1e-6).map(|(_, v)| v).collect();
            let col = if ds.is_empty() { 0.0 } else { (2.0 * ds.iter().sum::<f32>() / ds.len() as f32 - 1.0).clamp(-1.0, 1.0) };
            out.push(msg(id::SET_LAYER_PARAMETER, &idx, &[*par as u8 as f32, col, env_source(amp) as u8 as f32]));
        }
    }
    out.push(model.rebuild_command());
}

fn env_source(amp: bool) -> TransmodSource {
    if amp { TransmodSource::AmpEnvelope } else { TransmodSource::FreeEnvelope }
}

/// A depth (−1..1) of `src` on `param` as a centred knob value.
fn depth_knob(node: Option<&dreist_formats::Node>, param: LayerParam, src: TransmodSource) -> f32 {
    let d = src.attr(param.name()).and_then(|a| node.and_then(|n| n.f32(&a))).unwrap_or(0.0);
    ((d + 1.0) / 2.0).clamp(0.0, 1.0)
}

/// Layer filter models (`round(FilterAlgorithm · 3)`) and their type counts.
const FILTER_ALGORITHMS: [&str; 4] = ["Ladder", "State variable", "Diode", "Comb"];
const TS_MODES: [&str; 2] = ["Default", "Fast"];

/// The filter type names of a filter model.
fn filter_types(algorithm: usize) -> Vec<String> {
    match algorithm {
        0 => dreist_dsp::sh101::SH101_MODES.iter().map(|s| s.to_string()).collect(),
        // [re:6f3ef7]: low / high / band pass, notch and peak, 2 and 4 pole.
        1 => ["LPF", "HPF", "BPF", "Notch", "Peak"].iter().flat_map(|t| [format!("{t} 2 pole"), format!("{t} 4 pole")]).collect(),
        2 => dreist_dsp::ms20::MS20_MODES.iter().map(|s| s.to_string()).collect(),
        _ => (1..=dreist_dsp::convert::FILTER_TYPE_COUNTS[3] as usize + 1).map(|k| format!("Type {k}")).collect(),
    }
}

use crate::mixer::output_index;

/// v1's Choke menu (its label table): the commons cut mode indices in
/// v1's order (no "Pad", no global groups 3–8).
const V1_CUT_MODES: [usize; 9] = [0, 1, 3, 4, 5, 6, 7, 8, 9];
/// v1's Play mode menu: One shot, Gated, Looping (no "Length bound").
const V1_PLAY_MODES: [usize; 3] = [0, 2, 3];

/// Index of a layer's stored mode (display name, or enum name) in `labels` / `names`.
fn mode_index(node: Option<&dreist_formats::Node>, key: &str, labels: &[&str], names: &[&str], default: usize) -> usize {
    let v = node.and_then(|n| n.attr(key)).map(str::trim);
    v.and_then(|v| labels.iter().position(|x| *x == v).or_else(|| names.iter().position(|x| *x == v))).unwrap_or(default)
}

/// An open popup of the page.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Menu {
    LayerOutput,
    PadOutput,
    Choke,
    PlayMode,
    FilterType,
    FilterAlgorithm,
    StretchMode,
    EnvDest(bool, usize),
}

/// Min/max per bucket of a sample's left channel.
struct Peaks {
    /// (min, max) buckets per channel (left, then right of a stereo file).
    buckets: Vec<Vec<(f32, f32)>>,
    frames: usize,
    /// File sample rate (envelope overlay times).
    rate: f32,
    /// Zero crossings of the left channel.
    zeros: Vec<u32>,
}

/// Waveforms kept (most recently shown).
const PEAKS_KEPT: usize = 16;

/// Peaks of the samples shown: reloaded when the file changes on disk (a
/// layer sample saved over it, a missing file appearing), the least
/// recently shown dropped past [`PEAKS_KEPT`].
#[derive(Default)]
struct PeaksCache {
    /// (file, modification time, peaks); most recent last.
    entries: Vec<(PathBuf, Option<std::time::SystemTime>, Option<Peaks>)>,
}

impl PeaksCache {
    fn get(&mut self, path: &std::path::Path) -> Option<&Peaks> {
        let stamp = std::fs::metadata(path).and_then(|m| m.modified()).ok();
        let entry = match self.entries.iter().position(|e| e.0 == path) {
            Some(i) => {
                let e = self.entries.remove(i);
                if e.1 == stamp { e } else { (e.0, stamp, load_peaks(path)) }
            }
            None => (path.to_path_buf(), stamp, load_peaks(path)),
        };
        self.entries.push(entry);
        if self.entries.len() > PEAKS_KEPT {
            self.entries.remove(0);
        }
        self.entries.last().and_then(|e| e.2.as_ref())
    }
}

/// Marker frames of a layer: (frames, start, end, loop start, loop end,
/// looping).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WaveMarkers {
    pub frames: u32,
    pub start: u32,
    pub end: u32,
    pub loop_start: u32,
    pub loop_end: u32,
    pub looping: bool,
}

/// The layer's [re:0d952f] (whole file when absent).
pub fn wave_markers(model: &Model, e: usize, p: usize, l: usize, frames: u32) -> WaveMarkers {
    let n = model.layer_node(e, p, l);
    let mk = |name: &str| n.and_then(|n| n.child("AudioFileMarkers", name)).map(|m| (m.u32("StartSample").unwrap_or(0), m.u32("EndSample").unwrap_or(frames)));
    let (start, end) = mk("StartEndMarkers").unwrap_or((0, frames));
    let (loop_start, loop_end) = mk("LoopMarkers").unwrap_or((start, end));
    let looping = n.and_then(|n| n.attr("PlaybackMode")).map(dreist_dsp::voice::PlaybackMode::canonical) == Some("Looping");
    WaveMarkers { frames, start: start.min(frames), end: end.min(frames), loop_start: loop_start.min(frames), loop_end: loop_end.min(frames), looping }
}

/// [re:2bd5e0] snapping: the nearest zero crossing within
/// `range` frames.
pub fn snap_zero(zeros: &[u32], f: u32, range: u32) -> u32 {
    let i = zeros.partition_point(|&z| z < f);
    // v1 takes the later crossing on a tie.
    let best = [Some(i), i.checked_sub(1)].into_iter().flatten().filter_map(|k| zeros.get(k)).min_by_key(|&&z| (z as i64 - f as i64).abs());
    match best {
        Some(&z) if (z as i64 - f as i64).unsigned_abs() < range as u64 => z,
        _ => f,
    }
}
pub struct Inspector {
    /// The page (the zoom bar's controls are searched under it).
    host: Option<Vec<usize>>,
    /// The layer view (`LayerControls`) and the pad view (`PadControls`).
    params: Option<Vec<usize>>,
    pad_controls: Option<Vec<usize>>,
    /// The eight layer rows and the pad item of the selector.
    rows: Vec<Vec<usize>>,
    pad_header: Option<Vec<usize>>,
    /// The connectors from the selected item to the editor.
    pad_glue: Option<Vec<usize>>,
    layer_glue: Option<Vec<usize>>,
    /// The pad item is selected: the pad view shows (else the layer view).
    pub(crate) pad_view: bool,
    peaks: RefCell<PeaksCache>,
    /// Marker drag: object 1 start, 2 end, 3 loop start, 4 loop end
    /// (the dragged object), and the markers when it started.
    drag: Option<(u8, WaveMarkers)>,
    /// Lower-half drag of the wave: zoom / scroll from (x, y, view).
    view_drag: Option<(f32, f32, (f32, f32))>,
    /// Samples read for the wave's audition.
    audition: std::cell::RefCell<dreist_engine::audio::SampleCache>,
    /// Visible part of the file (start, length) as fractions.
    pub(crate) view: (f32, f32),
    /// The pad view's play mode, Vel-Amp and velocity split lanes.
    vsplit: crate::velsplit::VelSplit,
    /// Zoom slider drag (mode, last x).
    zoom_drag: Option<(u8, f32)>,
    /// Auto zoom mode (`AutoZoomGroup`, option `WaveDisplayZoomMode`).
    zoom_mode: i64,
    /// The layer and file the auto zoom last ran for.
    zoom_key: Option<(usize, usize, usize, PathBuf)>,
    /// Envelope overlays (options `AmpEnvVisible` / `FreeEnvVisible`).
    env_visible: (bool, bool),
    /// Envelope handle drag: (Amp?, handle 0..7).
    env_drag: Option<(bool, usize)>,
    /// Decay / hold handles keep the sustain level (options `LockDecay` /
    /// `LockRelease`, [re:269bca]).
    env_lock: (bool, bool),
    /// The envelope handle lock menu is open on this control (handle 1 or 3).
    env_lock_menu: Option<(Vec<usize>, usize)>,
    /// Timeline in Beats mode (option `PadEditWaveDisplayTimelineMode`).
    timeline_beats: bool,
    /// The timeline's Time / Beats menu is open on this control.
    timeline_menu: Option<Vec<usize>>,
    /// The open combo menu.
    menu: Option<Menu>,
    /// Envelope destinations picked per (engine, pad, layer, amp); the
    /// document keeps the depths only.
    env_dests: HashMap<(usize, usize, usize, bool), [usize; 2]>,
    /// A typed destination tempo per layer (else the song tempo).
    dest_bpm: HashMap<(usize, usize, usize), f64>,
}
const PEAK_BUCKETS: usize = 2048;

/// Visit the controls tagged `layerZoom` (the layer wave display's zoom bar).
fn walk_tagged(c: &mut Ctrl, f: &mut dyn FnMut(&mut Ctrl)) {
    for k in &mut c.children {
        if k.attr("layerZoom").is_some() {
            f(k);
        }
        walk_tagged(k, f);
    }
}

fn find_tagged<'a>(c: &'a Ctrl, name: &str) -> Option<&'a Ctrl> {
    c.children.iter().find_map(|k| if k.attr("layerZoom").is_some() && k.name == name { Some(k) } else { find_tagged(k, name) })
}

fn load_peaks(path: &std::path::Path) -> Option<Peaks> {
    let mut cache = dreist_engine::audio::SampleCache::default();
    let data = cache.get(path)?;
    let n = data.left.len();
    if n == 0 {
        return None;
    }
    let per = n.div_ceil(PEAK_BUCKETS).max(1);
    let peaks = |ch: &[f32]| ch.chunks(per).map(|c| c.iter().fold((f32::MAX, f32::MIN), |(lo, hi), &x| (lo.min(x), hi.max(x)))).collect::<Vec<_>>();
    let buckets = std::iter::once(peaks(&data.left)).chain(data.right.as_deref().map(peaks)).collect();
    let zeros = data.left.windows(2).enumerate().filter(|(_, w)| (w[0] <= 0.0) != (w[1] <= 0.0)).map(|(i, _)| i as u32 + 1).collect();
    Some(Peaks { buckets, frames: n, zeros, rate: data.sample_rate.max(1.0) })
}
/// The path of `name` below `base`.
fn path_under(root: &Ctrl, base: &[usize], name: &str) -> Option<Vec<usize>> {
    let mut c = root;
    for &i in base {
        c = c.children.get(i)?;
    }
    Some([base, &c.path_of(name)?].concat())
}

fn ctrl_at<'a>(root: &'a mut Ctrl, path: &[usize]) -> &'a mut Ctrl {
    let mut c = root;
    for &i in path {
        c = &mut c.children[i];
    }
    c
}

impl Inspector {
    /// Mount the Pad / Layers page into `page` (none: an editor without
    /// the page).
    pub fn build(skin: &mut Skin, root: &mut Ctrl, page: Option<&[usize]>) -> Result<Inspector, EditorError> {
        let o = crate::options::Options::load();
        let mut ins = Inspector {
            host: None,
            params: None,
            pad_controls: None,
            rows: Vec::new(),
            pad_header: None,
            pad_glue: None,
            layer_glue: None,
            pad_view: false,
            peaks: RefCell::default(),
            drag: None,
            view: (0.0, 1.0),
            vsplit: Default::default(),
            zoom_drag: None,
            view_drag: None,
            audition: Default::default(),
            zoom_mode: o.int("WaveDisplayZoomMode", 0),
            zoom_key: None,
            env_visible: (o.bool("AmpEnvVisible", true), o.bool("FreeEnvVisible", true)),
            env_drag: None,
            env_lock: (o.bool("LockDecay", false), o.bool("LockRelease", false)),
            env_lock_menu: None,
            timeline_beats: crate::timeline::beats_mode("PadEdit"),
            timeline_menu: None,
            menu: None,
            env_dests: HashMap::new(),
            dest_bpm: HashMap::new(),
        };
        let Some(page) = page else { return Ok(ins) };
        crate::front::mount_page(skin, root, page, crate::front::PADS_PAGE)?;
        let host = ctrl_at(root, page);
        // The layer rows: code-built, tagged with their layer.
        for l in 0..LAYERS {
            if let Some(r) = host.find_mut(&format!("LayerSelectionButton{l}")) {
                r.overload = "LayerRow".into();
                r.set_attr("layer", l.to_string());
            }
        }
        if let Some(pc) = host.find_mut("PadControls") {
            for l in 0..LAYERS {
                if let Some(s) = pc.find_mut(&format!("Split{l}")) {
                    s.overload = crate::velsplit::OVERLOAD.into();
                    s.style = crate::velsplit::STYLE.into();
                    s.set_attr("layer", l.to_string());
                }
            }
        }
        if let Some(lc) = host.find_mut("LayerControls") {
            // The layer wave display's zoom bar and envelope checks (tagged
            // so the slicer's identically named controls are left alone).
            for n in ["WaveZoomSlider", "ZoomInButton", "ZoomOutButton", "AutoZoomGroup", "AmpButton", "FreeButton"] {
                if let Some(c) = lc.find_mut(n) {
                    c.set_attr("layerZoom", "true".into());
                }
            }
            // The envelope panel's learn boxes are learn targets only.
            if let Some(env) = lc.find_mut("EnvPanel") {
                env.walk_mut(&mut |c: &mut Ctrl| {
                    if c.class == "CtrlTextBox" && c.name.ends_with("LL") {
                        c.visible = false;
                        c.enabled = false;
                    }
                });
            }
        }
        for n in ["PadGlue", "LayerGlue"] {
            if let Some(c) = host.find_mut(n) {
                c.passive = true;
            }
        }
        ins.host = Some(page.to_vec());
        ins.params = path_under(root, page, "LayerControls");
        ins.pad_controls = path_under(root, page, "PadControls");
        ins.pad_header = path_under(root, page, "PadSelection");
        ins.pad_glue = path_under(root, page, "PadGlue");
        ins.layer_glue = path_under(root, page, "LayerGlue");
        ins.rows = (0..LAYERS).filter_map(|l| path_under(root, page, &format!("LayerSelectionButton{l}"))).collect();
        ins.vsplit = crate::velsplit::VelSplit::build(ins.pad_controls.clone());
        Ok(ins)
    }

    /// The layer of the selector row at or above `path`.
    fn row_of(&self, path: &[usize]) -> Option<usize> {
        self.rows.iter().position(|r| path.starts_with(r))
    }
    pub fn hover_cursor(&self, app: &App, model: &Model, path: &[usize], overload: &str, x: f32, y: f32) -> Option<dreist_gui::cursor::Cursor> {
        use dreist_gui::cursor::Cursor;
        let (w, h) = app.ctrl(path).map(|c| (c.rect.w as f32, c.rect.h as f32))?;
        match overload {
            "WaveDisplayZoomSlider" if app.ctrl(path).is_some_and(|c| c.attr("layerZoom").is_some()) => {
                Some(if crate::zoomslider::over_edge(self.view, w, h, x, y) { Cursor::LeftRight } else { Cursor::Arrow })
            }
            "LayerWaveDisplay" => {
                let e = model.engine;
                let (p, l) = (model.pad[e], model.layer[e]);
                let (_, frames) = self.layer_frames(model)?;
                let m = wave_markers(model, e, p, l, frames);
                let (v0, vl) = self.view;
                let to_x = |f: u32| ((f as f32 / frames.max(1) as f32) - v0) / vl * w;
                let mut cands = vec![m.start, m.end];
                if m.looping {
                    cands.extend([m.loop_start, m.loop_end]);
                }
                Some(if cands.into_iter().any(|f| (to_x(f) - x).abs() < 4.0) { Cursor::LeftRight } else { Cursor::Arrow })
            }
            _ => None,
        }
    }

    /// The wave display's tooltip at (x, y) ([re:1392a4]
    ///  [re:1392a4]): an envelope handle — "Amp Envelope " /
    /// "Free Envelope " and its stage ("Attack: 0.25 sec", "Decay: … sec,
    /// Sustain: 050%" when the sustain handle is hidden, curves "Attack
    /// Curve: +20%") — else the marker under the pointer.
    pub fn wave_tooltip(&self, app: &App, model: &Model, path: &[usize], x: f32, y: f32) -> Option<String> {
        let e = model.engine;
        let (p, l) = (model.pad[e], model.layer[e]);
        let c = app.ctrl(path)?;
        let w = c.rect.w as f32;
        let file = model.layer_sample(e, p, l)?;
        let mut cache = self.peaks.borrow_mut();
        let pk = cache.get(&file)?;
        let frames = pk.frames as u32;
        let (v0, vl) = self.view;
        let m = wave_markers(model, e, p, l, frames);
        let h = c.rect.h as f32 - crate::timeline::WAVE_TOP;
        let geo = EnvGeo { w, y0: crate::timeline::WAVE_TOP, h, v0, vl, frames: frames as f32, rate: pk.rate, start: m.start, end: m.end };
        if let Some((amp, k)) = env_hit(model, e, p, l, self.env_visible, &geo, x, y) {
            let v = |i: usize| model.layer_param(e, p, l, env_param(amp, i));
            let secs = |i: usize, max: f32| v(i).powi(4) * max;
            let sus = (v(2) * 100.0) as i64;
            let mode = model.layer_node(e, p, l).and_then(|n| n.attr("PlaybackMode")).map_or("OneShot", dreist_dsp::voice::PlaybackMode::canonical).to_string();
            let (ay, ah) = geo.area(amp, self.env_visible)?;
            let sustain_shown = env_handles(model, e, p, l, amp, &mode, &geo, ay, ah).iter().any(|(i, _)| *i == 2);
            let curve = |i: usize| (v(i) * 200.0 - 100.0) as i64;
            let stage = match k {
                0 => format!("Attack: {:.2} sec", secs(0, 10.0)),
                1 if !sustain_shown => format!("Decay: {:.2} sec, Sustain: {sus:03}%", secs(1, 10.0)),
                1 => format!("Decay: {:.2} sec", secs(1, 10.0)),
                2 => format!("Sustain: {sus:03}%"),
                3 if !sustain_shown => format!("Hold: {:.2} sec, Sustain: {sus:03}%", secs(3, 60.0)),
                3 => format!("Hold: {:.2} sec", secs(3, 60.0)),
                4 => format!("Release: {:.2} sec", secs(4, 15.0)),
                5 => format!("Attack Curve: {:+3}%", curve(5)),
                6 => format!("Decay Curve: {:+3}%", curve(6)),
                _ => format!("Release Curve: {:+3}%", curve(7)),
            };
            return Some(format!("{}{stage}", if amp { "Amp Envelope " } else { "Free Envelope " }));
        }
        let to_x = |f: u32| ((f as f32 / frames.max(1) as f32) - v0) / vl * w;
        let mut cands = vec![(m.start, "Start marker"), (m.end, "End marker")];
        if m.looping {
            cands.extend([(m.loop_start, "Loop start marker"), (m.loop_end, "Loop end marker")]);
        }
        cands.into_iter().find(|(f, _)| (to_x(*f) - x).abs() < 4.0).map(|(_, n)| n.to_string())
    }

    /// Frames of the selected layer's sample (peaks cache).
    fn layer_frames(&self, model: &Model) -> Option<(PathBuf, u32)> {
        let e = model.engine;
        let file = model.layer_sample(e, model.pad[e], model.layer[e])?;
        let mut cache = self.peaks.borrow_mut();
        let frames = cache.get(&file)?.frames as u32;
        Some((file, frames))
    }

    /// Apply the auto zoom mode to the selected layer's markers.
    fn apply_auto_zoom(&mut self, model: &Model) {
        let e = model.engine;
        let (p, l) = (model.pad[e], model.layer[e]);
        let Some((_, frames)) = self.layer_frames(model) else { return };
        let m = wave_markers(model, e, p, l, frames);
        if let Some(v) = crate::zoomslider::auto_zoom(self.zoom_mode, frames, m.start, m.end) {
            self.view = v;
        }
    }

    /// SP / EP / LS / LE (the toolbar names PS / PE / LS / LE):
    /// hidden, shown only in MIDI learn mode ([re:7fd1e4]
    /// [re:cca449]) as learn targets for the markers.
    pub fn learn_buttons(&self, app: &mut App, learning: bool) {
        let Some(c) = self.params.as_ref().and_then(|p| app.ctrl_mut(p)) else { return };
        fn walk(c: &mut Ctrl, on: bool) {
            for k in &mut c.children {
                if matches!(k.name.as_str(), "PS" | "PE" | "LS" | "LE") && k.class == "CtrlButton" {
                    k.visible = on;
                    k.enabled = on;
                }
                walk(k, on);
            }
        }
        walk(c, learning);
    }

    /// The marker message a learn press on SP / EP / LS / LE binds
    /// ([re:ad8531]: 0x7a..0x7d "Play Start", "Play End",
    /// "Loop Start", "Loop End").
    pub fn marker_learn(&self, app: &App, path: &[usize]) -> Option<(u32, &'static str)> {
        if !self.params.as_ref().is_some_and(|p| path.starts_with(p)) {
            return None;
        }
        use dreist_engine::binding::id;
        match app.ctrl(path)?.name.as_str() {
            "PS" => Some((id::SET_LAYER_START_SAMPLE01, "Play Start")),
            "PE" => Some((id::SET_LAYER_END_SAMPLE01, "Play End")),
            "LS" => Some((id::SET_LAYER_LOOP_START_SAMPLE01, "Loop Start")),
            "LE" => Some((id::SET_LAYER_LOOP_END_SAMPLE01, "Loop End")),
            _ => None,
        }
    }
    /// The envelope's two destinations (0 none, 1.. an [`ENV_DESTS`]
    /// entry): the picked ones, else the parameters it modulates, then the
    /// defaults (the Free envelope's Cutoff and Pitch).
    fn env_dests(&self, model: &Model, e: usize, p: usize, l: usize, amp: bool) -> [usize; 2] {
        // A v1 layer's own routings.
        if let Some(slots) = layer_slots(model, e, p, l) {
            return [0, 1].map(|s| slots[routing_index(amp, s)].map(|(d, _)| dest_choice(d)).unwrap_or(0));
        }
        if let Some(d) = self.env_dests.get(&(e, p, l, amp)) {
            return *d;
        }
        let node = model.layer_node(e, p, l);
        let src = env_source(amp);
        let mut used: Vec<usize> =
            ENV_DESTS.iter().enumerate().filter(|(_, (_, par))| src.attr(par.name()).and_then(|a| node.and_then(|n| n.f32(&a))).is_some_and(|d| d.abs() > 1e-6)).map(|(i, _)| i + 1).collect();
        used.truncate(2);
        let defaults: [usize; 2] = if amp { [0, 0] } else { [1, 4] };
        for d in defaults {
            if used.len() < 2 && (d == 0 || !used.contains(&d)) {
                used.push(d);
            }
        }
        [used[0], used[1]]
    }

    /// The source / destination tempo of the layer's timestretch: the
    /// destination typed (else the song tempo), the source from the
    /// stretch amount (length %, `Curve::Stretch`).
    fn stretch_bpm(&self, model: &Model, e: usize, p: usize, l: usize) -> (f64, f64) {
        // The layer's stored destination (v1 `DestinationBPM`), else the song's.
        let stored = model.layer_node(e, p, l).and_then(|n| n.f32("DestinationBPM")).filter(|v| *v > 0.0).map(f64::from);
        let dest = stored.or_else(|| self.dest_bpm.get(&(e, p, l)).copied()).unwrap_or_else(|| model.tempo());
        let pct = LayerParam::TimestretchAmount.curve().to_units(model.layer_param(e, p, l, LayerParam::TimestretchAmount)) as f64;
        (dest * pct / 100.0, dest)
    }

    pub fn sync(&mut self, app: &mut App, model: &Model) {
        self.vsplit.sync(app, model);
        // [re:bca59e]: a new layer / file gets the auto zoom
        // (or the whole file when the mode is off).
        {
            let e = model.engine;
            let key = self.layer_frames(model).map(|(f, _)| (e, model.pad[e], model.layer[e], f));
            if key != self.zoom_key {
                self.zoom_key = key;
                self.view = (0.0, 1.0);
                self.apply_auto_zoom(model);
            }
        }
        // Zoom bar states: the mode group, and Zoom in only while the
        // thumb is wider than 20 px.
        let mode = self.zoom_mode;
        let view = self.view;
        let env = self.env_visible;
        if let Some(host) = self.host.as_ref().and_then(|h| app.ctrl_mut(h)) {
            let w = find_tagged(host, "WaveZoomSlider").map(|c| c.rect.w as f32);
            walk_tagged(host, &mut |c: &mut Ctrl| match c.name.as_str() {
                "AmpButton" => c.on = env.0,
                "FreeButton" => c.on = env.1,
                "AutoZoomGroup" => {
                    for (i, b) in c.children.iter_mut().enumerate() {
                        b.on = i as i64 == mode;
                    }
                }
                "ZoomInButton" => {
                    if let Some(w) = w {
                        c.enabled = view.1 * w > 20.0;
                    }
                }
                _ => {}
            });
        }
        let e = model.engine;
        let p = model.pad[e];
        let l = model.layer[e];
        let pad_view = self.pad_view;
        // The editor shown and the connector to the selected item.
        for (path, show) in [(&self.pad_controls, pad_view), (&self.params, !pad_view), (&self.pad_glue, pad_view), (&self.layer_glue, !pad_view)] {
            if let Some(c) = path.as_ref().and_then(|p| app.ctrl_mut(p)) {
                c.visible = show;
                c.enabled = show;
            }
        }
        if let (Some(g), Some(r)) = (self.layer_glue.as_ref(), self.rows.get(l)) {
            let y = r.last().and_then(|_| app.ctrl(r)).map(|c| c.rect.y + 2);
            if let (Some(y), Some(c)) = (y, app.ctrl_mut(g)) {
                if c.rect.y != y {
                    c.set_rect(Rect::new(c.rect.x, y, c.rect.w, c.rect.h));
                }
            }
        }
        // Layer rows.
        let (_, lmask) = crate::multipad::layer_selection(model, e, p);
        let pad_locked = model.pad_node(e, p).is_some_and(|n| n.locked());
        for (li, rp) in self.rows.iter().enumerate() {
            let f = model.layer(e, p, li);
            // An empty layer reads "Empty" (manual §5:1 figures).
            let name = if f.valid { model.layer_name(e, p, li).unwrap_or_default() } else { "Empty".to_string() };
            let selected = !pad_view && (li == l || lmask & (1u8 << li.min(7)) != 0);
            let locked = model.layer_node(e, p, li).is_some_and(|n| n.locked());
            let Some(r) = app.ctrl_mut(rp) else { continue };
            // The current layer amber, the others of a multiple selection
            // `SelectionButtonMultiStyle`.
            r.style = if selected && li == l {
                "SelectionButtonPrimaryStyle"
            } else if selected {
                "SelectionButtonMultiStyle"
            } else if li % 2 == 0 {
                "SelectionButtonEvenStyle"
            } else {
                "SelectionButtonOddStyle"
            }
            .into();
            r.set_attr("selected", selected.to_string());
            for c in &mut r.children {
                match c.name.as_str() {
                    // The name edits once the row is selected; until then a
                    // click selects it.
                    // [re:73027d]: an empty layer's name and buttons are
                    // disabled; a locked layer (or pad) keeps its name and
                    // Clear disabled, a locked pad its Lock.
                    "SampleName" => {
                        c.text = name.clone();
                        c.passive = !(selected && li == l);
                        c.enabled = f.valid && !(locked || pad_locked);
                    }
                    "Mute" => {
                        c.on = f.mute;
                        c.enabled = f.valid;
                    }
                    "Solo" => {
                        c.on = f.solo;
                        c.enabled = f.valid;
                    }
                    "Lock" => {
                        c.on = locked;
                        c.enabled = f.valid && !pad_locked;
                    }
                    "Clear" => c.enabled = f.valid && !(locked || pad_locked),
                    _ => {}
                }
            }
        }
        if let Some(hp) = &self.pad_header {
            let name = model.pad_name(e, p).unwrap_or_else(|| format!("Pad {}", p + 1));
            let (m, s) = (model.pad_mute(e, p), model.pad_solo(e, p));
            let locked = crate::padmenu::pad_locked(model, e, p);
            if let Some(h) = app.ctrl_mut(hp) {
                h.style = if pad_view { "PadSelectionButtonPrimaryStyle" } else { "PadSelectionButtonNotSelectedStyle" }.into();
                for c in &mut h.children {
                    match c.name.as_str() {
                        "PadLabel" => {
                            c.text = name.clone();
                            c.passive = !pad_view;
                        }
                        "Mute" => c.on = m,
                        "Solo" => c.on = s,
                        "Lock" => c.on = locked,
                        "AllPads" => c.on = model.all_pads,
                        "AllLayers" => c.on = model.all_layers,
                        "AllGroups" => c.on = model.all_rows,
                        _ => {}
                    }
                }
            }
        }
        // The layer view's controls.
        let node = model.layer_node(e, p, l);
        let dests = [self.env_dests(model, e, p, l, true), self.env_dests(model, e, p, l, false)];
        let slots = layer_slots(model, e, p, l);
        let (src_bpm, dest_bpm) = self.stretch_bpm(model, e, p, l);
        let algorithm = ((model.layer_param(e, p, l, LayerParam::FilterAlgorithm) * 3.0).round() as usize).min(3);
        // The timestretch controls are inactive (tempos "N/A") while it is
        // off (manual §5:4 figure).
        let stretch = model.layer_param(e, p, l, LayerParam::TimestretchActive) >= 0.5;
        if let Some(c) = self.params.as_ref().and_then(|pp| app.ctrl_mut(pp)) {
            c.walk_mut(&mut |k: &mut Ctrl| {
                let name = k.name.as_str();
                if matches!(name, "AmountKnob" | "TSModeCombo" | "SourceBPMBox" | "DestBPMBox" | "FindBPMButton" | "TrackBPMButton") {
                    k.enabled = stretch;
                }
                if k.class == "CtrlButton" {
                    if let Some(on) = layer_toggle_state(node, name) {
                        k.on = on;
                        return;
                    }
                }
                if let Some(par) = velocity_control(name) {
                    k.value = depth_knob(node, par, TransmodSource::Velocity);
                    return;
                }
                if let Some(a) = name.strip_prefix("Send").and_then(|n| n.strip_suffix("Knob")).and_then(|n| n.parse::<usize>().ok()) {
                    k.value = node.and_then(|n| n.f32(&dreist_formats::params::aux_send_name(a.saturating_sub(1)))).unwrap_or(0.0).clamp(0.0, 1.0);
                    return;
                }
                if let Some((amp, slot)) = env_slot(name, "Dest") {
                    let d = dests[!amp as usize][slot];
                    k.text = if d == 0 { NO_DEST.to_string() } else { ENV_DESTS[d - 1].0.to_string() };
                    return;
                }
                if let Some((amp, slot)) = env_slot(name, "Depth") {
                    let d = dests[!amp as usize][slot];
                    let own = slots.map(|s| s[routing_index(amp, slot)].map(|(_, v)| v).unwrap_or(0.5));
                    k.value = own.unwrap_or_else(|| if d == 0 { 0.5 } else { depth_knob(node, ENV_DESTS[d - 1].1, env_source(amp)) });
                    k.enabled = d != 0;
                    return;
                }
                match name {
                    "OutputCombo" => k.text = crate::mixer::output_items()[output_index(node.and_then(|n| n.attr("OutputLocation")).unwrap_or("Parent"))].clone(),
                    "CutModeCombo" => {
                        use dreist_dsp::voices::{CUT_MODE_LABELS, CUT_MODE_NAMES};
                        k.text = CUT_MODE_LABELS[mode_index(node, "CutMode", &CUT_MODE_LABELS, &CUT_MODE_NAMES, 1)].to_string();
                    }
                    "PlayModeCombo" => {
                        use dreist_dsp::voice::{PLAYBACK_MODE_LABELS, PLAYBACK_MODE_NAMES};
                        k.text = PLAYBACK_MODE_LABELS[mode_index(node, "PlaybackMode", &PLAYBACK_MODE_LABELS, &PLAYBACK_MODE_NAMES, 0)].to_string();
                    }
                    "FilterAlgoButton" => k.text = FILTER_ALGORITHMS[algorithm].to_string(),
                    "FilterModeKnob" => {
                        let types = filter_types(algorithm);
                        let n = (types.len() - 1).max(1) as f32;
                        let i = ((model.layer_param(e, p, l, LayerParam::FilterType) * n).round() as usize).min(types.len() - 1);
                        k.text = types[i].clone();
                    }
                    "TSModeCombo" => k.text = TS_MODES[(model.layer_param(e, p, l, LayerParam::TimestretchMode) >= 0.5) as usize].to_string(),
                    "SourceBPMBox" | "DestBPMBox" if !stretch => k.text = "N/A".into(),
                    "SourceBPMBox" => k.text = format!("{src_bpm:.2}"),
                    "DestBPMBox" => k.text = format!("{dest_bpm:.2}"),
                    _ => {
                        let Some(par) = param_for_control(name) else { return };
                        let base = model.layer_param(e, p, l, par);
                        match k.class.as_str() {
                            "CtrlKnob" | "CtrlSlider" | "CtrlPanSlider" => k.value = base,
                            "CtrlButton" => k.on = base >= 0.5,
                            _ => {}
                        }
                    }
                }
            });
        }
        // The pad view's mix controls.
        let pad = model.pad_node(e, p);
        let pget = |k: &str, d: f32| pad.and_then(|n| n.f32(k)).unwrap_or(d).clamp(0.0, 1.0);
        let (gain, pan) = (pget(PadParam::Gain.name(), PadParam::Gain.default_value()), pget(PadParam::Pan.name(), 0.5));
        let sends: Vec<f32> = (0..4).map(|k| pget(&dreist_formats::params::aux_send_name(k), 0.0)).collect();
        let output = crate::mixer::output_items()[output_index(pad.and_then(|n| n.attr("OutputLocation")).unwrap_or("Parent"))].clone();
        if let Some(c) = self.pad_controls.as_ref().and_then(|pp| app.ctrl_mut(pp)) {
            c.walk_mut(&mut |k: &mut Ctrl| match k.name.as_str() {
                "Pan" => k.value = pan,
                "Gain" => k.value = gain,
                "Output" => k.text = output.clone(),
                n => {
                    if let Some(a) = n.strip_prefix("Send").and_then(|n| n.parse::<usize>().ok()).filter(|a| (1..=4).contains(a)) {
                        k.value = sends[a - 1];
                    }
                }
            });
        }
    }

    /// The page's own controls: selector rows and pad item, the pad
    /// view's mix controls and the layer view's.
    pub fn handle(&mut self, app: &mut App, model: &mut Model, ev: &UiEvent, out: &mut Vec<Command>) -> bool {
        if self.vsplit.handle(app, model, ev, out) {
            return true;
        }
        if self.handle_controls(app, model, ev, out) {
            return true;
        }
        self.handle_wave(app, model, ev, out)
    }

    fn in_params(&self, path: &[usize]) -> bool {
        self.params.as_ref().is_some_and(|pp| path.starts_with(pp))
    }

    fn in_pad_controls(&self, path: &[usize]) -> bool {
        self.pad_controls.as_ref().is_some_and(|pp| path.starts_with(pp))
    }

    fn open(&mut self, app: &mut App, path: &[usize], items: Vec<String>, checked: Option<usize>, menu: Menu) {
        app.open_menu(path, items, checked);
        self.menu = Some(menu);
    }

    fn handle_controls(&mut self, app: &mut App, model: &mut Model, ev: &UiEvent, out: &mut Vec<Command>) -> bool {
        let e = model.engine;
        let p = model.pad[e];
        let l = model.layer[e];
        let idx = [e as i32, p as i32, l as i32];
        let ep = [e as i32, p as i32];
        match ev {
            // The selector: a press on a layer row selects the layer
            // (cmd toggles, shift extends) and shows the layer view; on the
            // pad item the pad view.
            UiEvent::Mouse { path, overload, kind: MouseKind::Down, .. } if overload == "LayerRow" => {
                let Some(li) = self.row_of(path) else { return false };
                let (cmd, shift) = (app.mods.cmd, app.mods.shift);
                out.push(crate::multipad::select_layer(model, e, li, cmd, shift));
                self.pad_view = false;
                true
            }
            UiEvent::Mouse { path, overload, kind: MouseKind::Down, .. } if overload == "PadSelectionButton" && self.pad_header.as_ref().is_some_and(|h| path.starts_with(h)) => {
                self.pad_view = true;
                true
            }
            // Trigger strips: play the layer (a layer row) or the pad (the
            // pad item) while held.
            UiEvent::Pressed { path, name } if name == "Trigger" && (self.row_of(path).is_some() || self.pad_header.as_ref().is_some_and(|h| path.starts_with(h))) => {
                out.push(match self.row_of(path) {
                    Some(li) => Command::LayerHit { engine: e, pad: p, layer: li, velocity: 127 },
                    None => Command::PadHit { engine: e, pad: p, velocity: 127 },
                });
                true
            }
            UiEvent::Clicked { path, name, .. } if name == "Trigger" && (self.row_of(path).is_some() || self.pad_header.as_ref().is_some_and(|h| path.starts_with(h))) => {
                out.push(Command::PadRelease { engine: e, pad: p });
                true
            }
            UiEvent::Clicked { path, name, on, .. } if self.row_of(path).is_some() => {
                let li = self.row_of(path).unwrap_or(0);
                let v = *on as i32 as f32;
                match name.as_str() {
                    "Mute" => out.push(msg(id::SET_LAYER_MUTE, &[e as i32, p as i32, li as i32], &[v])),
                    "Solo" => out.push(msg(id::SET_LAYER_SOLO, &[e as i32, p as i32, li as i32], &[v])),
                    // Clear: the sample and every setting (unless locked).
                    "Clear" => {
                        if crate::padmenu::layer_locked(model, e, p, li) {
                            return true;
                        }
                        *model.layer_mut(e, p, li) = dreist_formats::Node::new("Layer", Some(&format!("Layer{li}")));
                        out.push(model.rebuild_command());
                    }
                    "Lock" => crate::padmenu::set_lock(model.layer_mut(e, p, li), *on),
                    _ => return false,
                }
                true
            }
            UiEvent::Clicked { path, name, on, .. } if self.pad_header.as_ref().is_some_and(|h| path.starts_with(h)) => {
                let v = *on as i32 as f32;
                match name.as_str() {
                    // [re:281f54]: `e_setModifiers` 0x26 [all layers,
                    // all pads, row].
                    "AllLayers" | "AllPads" | "AllGroups" => {
                        match name.as_str() {
                            "AllLayers" => model.all_layers = *on,
                            // v1: All pads and Row exclude each other.
                            "AllPads" => {
                                model.all_pads = *on;
                                model.all_rows &= !*on;
                            }
                            _ => {
                                model.all_rows = *on;
                                model.all_pads &= !*on;
                            }
                        }
                        out.push(msg(id::SET_MODIFIERS, &[], &[model.all_layers as i32 as f32, model.all_pads as i32 as f32, model.all_rows as i32 as f32]));
                    }
                    "Mute" => out.push(msg(id::MUTE_PAD, &ep, &[v])),
                    "Solo" => out.push(msg(id::SOLO_PAD, &ep, &[v])),
                    "Clear" => {
                        if crate::padmenu::pad_locked(model, e, p) {
                            return true;
                        }
                        *model.pad_mut(e, p) = dreist_formats::Node::new("Pad", Some(&format!("Pad{p}")));
                        out.push(model.rebuild_command());
                    }
                    "Lock" => crate::padmenu::set_lock(model.pad_mut(e, p), *on),
                    _ => return false,
                }
                true
            }
            // Timestretch tempos: the source over the destination sets the
            // stretch (length %); a typed destination is kept for the layer.
            UiEvent::TextCommitted { path, name, text } if self.in_params(path) && matches!(name.as_str(), "SourceBPMBox" | "DestBPMBox") => {
                let Some(v) = crate::conv::number(text.trim()).filter(|v| *v > 0.0).map(|v| (v as f64).clamp(20.0, 999.0)) else { return true };
                let (src, mut dest) = self.stretch_bpm(model, e, p, l);
                let src = if name == "SourceBPMBox" {
                    v
                } else {
                    dest = v;
                    self.dest_bpm.insert((e, p, l), v);
                    src
                };
                let amount = LayerParam::TimestretchAmount.curve().from_units((100.0 * src / dest) as f32).clamp(0.0, 1.0);
                out.push(msg(id::SET_LAYER_PARAMETER, &idx, &[LayerParam::TimestretchAmount as u8 as f32, amount, 0.0]));
                // Both tempos are the layer's ([re:c0683b]).
                model.layer_mut(e, p, l).set("SourceBPM", src as f32).set("DestinationBPM", dest as f32);
                true
            }
            // [re:e62449]: Find source tempo is the file's own analysis
            // tempo, else [re:155b84]'s guess from the whole file's length.
            UiEvent::Clicked { path, name, .. } if self.in_params(path) && name == "FindBPMButton" => {
                let Some(file) = model.layer_sample(e, p, l) else { return true };
                let gsa = dreist_engine::gsa::SliceFile::load(&file.with_extension("gsa")).map(|a| a.bpm).filter(|b| *b > 0.0);
                let bpm = gsa.or_else(|| dreist_engine::audio::info(&file).map(|(frames, sr)| guess_bpm(frames as f32, sr))).unwrap_or(0.0);
                let bpm = if bpm <= 0.0 { 120.0 } else { bpm as f64 };
                let (_, dest) = self.stretch_bpm(model, e, p, l);
                let amount = LayerParam::TimestretchAmount.curve().from_units((100.0 * bpm / dest) as f32).clamp(0.0, 1.0);
                out.push(msg(id::SET_LAYER_PARAMETER, &idx, &[LayerParam::TimestretchAmount as u8 as f32, amount, 0.0]));
                model.layer_mut(e, p, l).set("SourceBPM", bpm as f32).set("DestinationBPM", dest as f32);
                true
            }
            // Combo menus.
            UiEvent::Clicked { path, name, .. } if self.in_params(path) && app.ctrl(path).is_some_and(|c| c.class == "CtrlPopupButton") => {
                let node = model.layer_node(e, p, l);
                let algorithm = ((model.layer_param(e, p, l, LayerParam::FilterAlgorithm) * 3.0).round() as usize).min(3);
                let owned = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
                match name.as_str() {
                    "OutputCombo" => {
                        let i = output_index(node.and_then(|n| n.attr("OutputLocation")).unwrap_or("Parent"));
                        self.open(app, path, crate::mixer::output_items(), Some(i), Menu::LayerOutput);
                    }
                    "CutModeCombo" => {
                        use dreist_dsp::voices::{CUT_MODE_LABELS, CUT_MODE_NAMES};
                        let i = mode_index(node, "CutMode", &CUT_MODE_LABELS, &CUT_MODE_NAMES, 1);
                        let labels: Vec<String> = V1_CUT_MODES.iter().map(|&k| CUT_MODE_LABELS[k].to_string()).collect();
                        self.open(app, path, labels, V1_CUT_MODES.iter().position(|&k| k == i), Menu::Choke);
                    }
                    "PlayModeCombo" => {
                        use dreist_dsp::voice::{PLAYBACK_MODE_LABELS, PLAYBACK_MODE_NAMES};
                        let i = mode_index(node, "PlaybackMode", &PLAYBACK_MODE_LABELS, &PLAYBACK_MODE_NAMES, 0);
                        let labels: Vec<String> = V1_PLAY_MODES.iter().map(|&k| PLAYBACK_MODE_LABELS[k].to_string()).collect();
                        self.open(app, path, labels, V1_PLAY_MODES.iter().position(|&k| k == i), Menu::PlayMode);
                    }
                    "FilterAlgoButton" => self.open(app, path, owned(&FILTER_ALGORITHMS), Some(algorithm), Menu::FilterAlgorithm),
                    "FilterModeKnob" => {
                        let types = filter_types(algorithm);
                        let n = (types.len() - 1).max(1) as f32;
                        let i = ((model.layer_param(e, p, l, LayerParam::FilterType) * n).round() as usize).min(types.len() - 1);
                        self.open(app, path, types, Some(i), Menu::FilterType);
                    }
                    "TSModeCombo" => {
                        let i = (model.layer_param(e, p, l, LayerParam::TimestretchMode) >= 0.5) as usize;
                        self.open(app, path, owned(&TS_MODES), Some(i), Menu::StretchMode);
                    }
                    n => {
                        let Some((amp, slot)) = env_slot(n, "Dest") else { return false };
                        let cur = self.env_dests(model, e, p, l, amp)[slot];
                        let items = std::iter::once(NO_DEST).chain(ENV_DESTS.iter().map(|(t, _)| *t)).map(str::to_string).collect();
                        self.open(app, path, items, Some(cur), Menu::EnvDest(amp, slot));
                    }
                }
                true
            }
            UiEvent::Clicked { path, name, .. } if self.in_pad_controls(path) && name == "Output" => {
                let i = output_index(model.pad_node(e, p).and_then(|n| n.attr("OutputLocation")).unwrap_or("Parent"));
                self.open(app, path, crate::mixer::output_items(), Some(i), Menu::PadOutput);
                true
            }
            UiEvent::MenuSelected { index, .. } if self.menu.is_some() => {
                let Some(menu) = self.menu.take() else { return false };
                let i = *index;
                let param = |par: LayerParam, v: f32| msg(id::SET_LAYER_PARAMETER, &idx, &[par as u8 as f32, v, 0.0]);
                match menu {
                    Menu::LayerOutput => out.push(msg(id::SET_LAYER_OUTPUT, &idx, &[crate::mixer::output_location(i)])),
                    Menu::PadOutput => out.push(msg(id::SET_PAD_OUTPUT, &ep, &[crate::mixer::output_location(i)])),
                    Menu::Choke => out.push(msg(id::SET_LAYER_CUT_MODE, &idx, &[V1_CUT_MODES.get(i).copied().unwrap_or(1) as f32])),
                    Menu::PlayMode => out.push(msg(id::SET_LAYER_PLAY_MODE, &idx, &[V1_PLAY_MODES.get(i).copied().unwrap_or(0) as f32])),
                    Menu::FilterAlgorithm => out.push(param(LayerParam::FilterAlgorithm, i.min(3) as f32 / 3.0)),
                    Menu::FilterType => {
                        let algorithm = ((model.layer_param(e, p, l, LayerParam::FilterAlgorithm) * 3.0).round() as usize).min(3);
                        let n = (filter_types(algorithm).len() - 1).max(1) as f32;
                        out.push(param(LayerParam::FilterType, (i as f32 / n).min(1.0)));
                    }
                    Menu::StretchMode => out.push(param(LayerParam::TimestretchMode, (i == 1) as i32 as f32)),
                    // A new destination takes the slot's depth with it.
                    // A v1 layer: the routing slot takes the destination
                    // (its depth stays).
                    Menu::EnvDest(amp, slot) if layer_slots(model, e, p, l).is_some() => {
                        let r = routing_index(amp, slot);
                        let depth = layer_slots(model, e, p, l).and_then(|s| s[r]).map(|(_, v)| v).unwrap_or(0.5);
                        dreist_formats::legacy::set_routing_slot(model.layer_mut(e, p, l), r, routing_dest(i).map(|d| (d, depth)));
                        sync_slots(model, e, p, l, out);
                    }
                    Menu::EnvDest(amp, slot) => {
                        let mut dests = self.env_dests(model, e, p, l, amp);
                        let old = dests[slot];
                        let new = i.min(ENV_DESTS.len());
                        if old == new {
                            return true;
                        }
                        let src = env_source(amp) as u8 as f32;
                        if old != 0 {
                            let node = model.layer_node(e, p, l);
                            let d = depth_knob(node, ENV_DESTS[old - 1].1, env_source(amp)) * 2.0 - 1.0;
                            out.push(msg(id::SET_LAYER_PARAMETER, &idx, &[ENV_DESTS[old - 1].1 as u8 as f32, 0.0, src]));
                            if new != 0 && d.abs() > 1e-6 {
                                out.push(msg(id::SET_LAYER_PARAMETER, &idx, &[ENV_DESTS[new - 1].1 as u8 as f32, d, src]));
                            }
                        }
                        dests[slot] = new;
                        self.env_dests.insert((e, p, l, amp), dests);
                    }
                }
                true
            }
            UiEvent::ValueChanged { path, name, value, .. } if self.in_pad_controls(path) => {
                let v = *value;
                let cmd = match name.as_str() {
                    "Pan" => msg(id::SET_PAD_PARAMETER, &ep, &[v, PadParam::Pan as u8 as f32, 0.0]),
                    "Gain" => msg(id::SET_PAD_PARAMETER, &ep, &[v, PadParam::Gain as u8 as f32, 0.0]),
                    n => {
                        let Some(a) = n.strip_prefix("Send").and_then(|n| n.parse::<usize>().ok()).filter(|a| (1..=4).contains(a)) else { return false };
                        msg(id::SET_PAD_SEND_LEVEL, &ep, &[(a - 1) as f32, v, 0.0])
                    }
                };
                out.push(cmd);
                true
            }
            UiEvent::ValueChanged { path, name, value, .. } if self.in_params(path) => {
                let v = *value;
                if let Some(par) = velocity_control(name) {
                    out.push(msg(id::SET_LAYER_PARAMETER, &idx, &[par as u8 as f32, v * 2.0 - 1.0, TransmodSource::Velocity as u8 as f32]));
                    return true;
                }
                if let Some(a) = name.strip_prefix("Send").and_then(|n| n.strip_suffix("Knob")).and_then(|n| n.parse::<usize>().ok()).filter(|a| (1..=4).contains(a)) {
                    out.push(msg(id::SET_LAYER_SEND_DEPTH, &idx, &[(a - 1) as f32, v, 0.0]));
                    return true;
                }
                if let Some((amp, slot)) = env_slot(name, "Depth") {
                    let r = routing_index(amp, slot);
                    if let Some(dest) = layer_slots(model, e, p, l).map(|s| s[r]) {
                        if let Some((d, _)) = dest {
                            dreist_formats::legacy::set_routing_slot(model.layer_mut(e, p, l), r, Some((d, v)));
                            sync_slots(model, e, p, l, out);
                        }
                        return true;
                    }
                    let d = self.env_dests(model, e, p, l, amp)[slot];
                    if d != 0 {
                        out.push(msg(id::SET_LAYER_PARAMETER, &idx, &[ENV_DESTS[d - 1].1 as u8 as f32, v * 2.0 - 1.0, env_source(amp) as u8 as f32]));
                    }
                    return true;
                }
                let Some(par) = param_for_control(name) else { return false };
                out.push(msg(id::SET_LAYER_PARAMETER, &idx, &[par as u8 as f32, v, 0.0]));
                true
            }
            UiEvent::Clicked { path, name, on, .. } if self.in_params(path) => {
                if let Some(kind) = toggle_message(name) {
                    out.push(msg(kind, &idx, &[*on as i32 as f32]));
                    return true;
                }
                let Some(par) = param_for_control(name) else { return false };
                out.push(msg(id::SET_LAYER_PARAMETER, &idx, &[par as u8 as f32, if *on { 1.0 } else { 0.0 }, 0.0]));
                true
            }
            _ => false,
        }
    }

    /// The wave display, its zoom bar and the envelope checks.
    fn handle_wave(&mut self, app: &mut App, model: &mut Model, ev: &UiEvent, out: &mut Vec<Command>) -> bool {
        let e = model.engine;
        let p = model.pad[e];
        let l = model.layer[e];
        let idx = [e as i32, p as i32, l as i32];
        match ev {
            UiEvent::Mouse { path, overload, kind, x, .. } if overload == "LayerWaveDisplay" => {
                let Some(w) = app.ctrl(path).map(|c| c.rect.w as f32) else { return false };
                let Some(file) = model.layer_sample(e, p, l) else { return false };
                let mut cache = self.peaks.borrow_mut();
                let Some(pk) = cache.get(&file) else { return false };
                let frames = pk.frames as u32;
                let (v0, vl) = self.view;
                let to_frame = |x: f32| (((x / w.max(1.0)) * vl + v0).clamp(0.0, 1.0) * frames as f32) as u32;
                let to_x = |f: u32| ((f as f32 / frames.max(1) as f32) - v0) / vl * w;
                // Envelope handles ([re:b2fe1b],
                // [re:874f6e]) come first.
                let h = app.ctrl(path).map(|c| c.rect.h as f32).unwrap_or(0.0) - crate::timeline::WAVE_TOP;
                let geo = EnvGeo {
                    w,
                    y0: crate::timeline::WAVE_TOP,
                    h,
                    v0,
                    vl,
                    frames: frames as f32,
                    rate: pk.rate,
                    start: wave_markers(model, e, p, l, frames).start,
                    end: wave_markers(model, e, p, l, frames).end,
                };
                let y = match ev {
                    UiEvent::Mouse { y, .. } => *y,
                    _ => 0.0,
                };
                // [re:a4b4e4]: on the timeline
                // strip, the Time / Beats check items.
                if *kind == dreist_gui::app::MouseKind::RightDown && y < crate::timeline::STRIP {
                    let items = ["Time", "Beats"].iter().enumerate().map(|(i, t)| if (i == 1) == self.timeline_beats { format!("✓ {t}") } else { t.to_string() }).collect();
                    let path = path.clone();
                    drop(cache);
                    app.open_menu(&path, items, None);
                    self.timeline_menu = Some(path);
                    return true;
                }
                // [re:09f556]: a right click on a decay or hold handle
                // offers its lock as one check item.
                if *kind == dreist_gui::app::MouseKind::RightDown {
                    if let Some((_, h @ (1 | 3))) = env_hit(model, e, p, l, self.env_visible, &geo, *x, y) {
                        let (locked, what) = if h == 1 { (self.env_lock.0, "decay") } else { (self.env_lock.1, "hold") };
                        let label = format!("{} {what} vertical positioning", if locked { "Unlock" } else { "Lock" });
                        let item = if locked { format!("{}{label}", dreist_gui::menu::CHECK) } else { label };
                        let path = path.clone();
                        drop(cache);
                        app.open_menu(&path, vec![item], None);
                        self.env_lock_menu = Some((path, h));
                        return true;
                    }
                }
                match kind {
                    dreist_gui::app::MouseKind::Down => {
                        if let Some(hit) = env_hit(model, e, p, l, self.env_visible, &geo, *x, y) {
                            self.env_drag = Some(hit);
                            return true;
                        }
                    }
                    dreist_gui::app::MouseKind::Drag if self.env_drag.is_some() => {
                        let (amp, handle) = self.env_drag.unwrap();
                        for (param, v) in env_drag_values(model, e, p, l, amp, handle, self.env_visible, &geo, *x, y, self.env_lock) {
                            out.push(msg(id::SET_LAYER_PARAMETER, &idx, &[param as f32, v, 0.0]));
                        }
                        return true;
                    }
                    dreist_gui::app::MouseKind::Up if self.env_drag.is_some() => {
                        self.env_drag = None;
                        return true;
                    }
                    _ => {}
                }
                match kind {
                    dreist_gui::app::MouseKind::Down => {
                        // The dragged object: a marker within 4 px.
                        let m = wave_markers(model, e, p, l, frames);
                        let mut cands = vec![(1u8, m.start), (2, m.end)];
                        if m.looping {
                            cands.push((3, m.loop_start));
                            cands.push((4, m.loop_end));
                        }
                        let hit = cands.into_iter().map(|(o, f)| (o, (to_x(f) - x).abs())).filter(|(_, d)| *d < 4.0).min_by(|a, b| a.1.total_cmp(&b.1));
                        self.drag = hit.map(|(o, _)| (o, m));
                        if self.drag.is_some() {
                            return true;
                        }
                        // v1's zones ([re:dc59ce]): the upper half auditions the
                        // layer's region ([re:75e50d]), the lower half drags the
                        // view ([re:079fd4]).
                        if y >= crate::timeline::WAVE_TOP && y < crate::timeline::WAVE_TOP + h * 0.5 {
                            drop(cache);
                            if let Some(d) = self.audition.borrow_mut().get(&file) {
                                out.push(Command::PreviewRegion { data: d, start: m.start, end: m.end.max(m.start + 1), looping: false });
                            }
                            return true;
                        }
                        if y >= crate::timeline::WAVE_TOP {
                            self.view_drag = Some((*x, y, self.view));
                            return true;
                        }
                        false
                    }
                    dreist_gui::app::MouseKind::Drag if self.view_drag.is_some() => {
                        let (x0, y0, v) = self.view_drag.unwrap();
                        let (zs, ss) = crate::zoomslider::drag_signs();
                        let factor = (-(y - y0) * zs / 100.0).exp2();
                        let tw = w;
                        let zoomed = crate::zoomslider::zoom_in(v, factor, (x0 / w.max(1.0)).clamp(0.0, 1.0), tw, frames);
                        self.view = crate::zoomslider::clamp((zoomed.0 + ss * (x - x0) / w.max(1.0) * zoomed.1, zoomed.1));
                        true
                    }
                    dreist_gui::app::MouseKind::Up if self.view_drag.is_some() => {
                        self.view_drag = None;
                        true
                    }
                    dreist_gui::app::MouseKind::Drag => {
                        let Some((obj, m0)) = self.drag else { return false };
                        let mut f = to_frame(*x);
                        // Snap to a zero crossing within 20 px unless shift.
                        if !app.mods.shift {
                            let range = to_frame(20.0).saturating_sub(to_frame(0.0)).max(1);
                            f = snap_zero(&pk.zeros, f, range);
                        }
                        let kinds = [id::SET_LAYER_START_SAMPLE, id::SET_LAYER_END_SAMPLE, id::SET_LAYER_LOOP_START_SAMPLE, id::SET_LAYER_LOOP_END_SAMPLE];
                        let orig = [m0.start, m0.end, m0.loop_start, m0.loop_end];
                        let k = (obj - 1) as usize;
                        out.push(msg(kinds[k], &idx, &[f as f32]));
                        // Alt moves the pair together (start/end, loop start/end).
                        if app.mods.alt {
                            let other = k ^ 1;
                            let moved = (orig[other] as i64 + f as i64 - orig[k] as i64).clamp(0, frames as i64 - 1) as u32;
                            out.push(msg(kinds[other], &idx, &[moved as f32]));
                        }
                        true
                    }
                    // Release [re:29efd4]: the auto zoom follows the markers.
                    dreist_gui::app::MouseKind::Up => {
                        let was = self.drag.take().is_some();
                        if was {
                            let m = wave_markers(model, e, p, l, frames);
                            if let Some(v) = crate::zoomslider::auto_zoom(self.zoom_mode, frames, m.start, m.end) {
                                self.view = v;
                            }
                        }
                        was
                    }
                    dreist_gui::app::MouseKind::RightDown => false,
                }
            }
            // [re:269bca]: toggle and store the lock.
            UiEvent::MenuSelected { anchor, .. } if self.env_lock_menu.as_ref().is_some_and(|(a, _)| a == anchor) => {
                let Some((_, h)) = self.env_lock_menu.take() else { return false };
                let (flag, key) = if h == 1 { (&mut self.env_lock.0, "LockDecay") } else { (&mut self.env_lock.1, "LockRelease") };
                *flag = !*flag;
                let mut o = crate::options::Options::load();
                o.set(key, *flag);
                o.save();
                true
            }
            UiEvent::MenuSelected { anchor, index, .. } if self.timeline_menu.as_ref() == Some(anchor) => {
                self.timeline_menu = None;
                self.timeline_beats = *index == 1;
                crate::timeline::save_mode("PadEdit", self.timeline_beats);
                true
            }
            // [re:1a34bd]: the wheel goes to the zoom slider, whose thumb
            // moves 5 px.
            UiEvent::Wheel { path, overload, dy } if overload == "LayerWaveDisplay" => {
                let Some(w) = app.ctrl(path).map(|c| c.rect.w as f32) else { return false };
                let tw = find_tagged(&app.root, "WaveZoomSlider").map(|c| c.rect.w as f32).unwrap_or(w);
                self.view = wheel_thumb(self.view, *dy, tw);
                true
            }
            // Zoom bar (zoom in by 2 / 0.5 around 0.5).
            UiEvent::Clicked { path, name, .. } if (name == "ZoomInButton" || name == "ZoomOutButton") && app.ctrl(path).is_some_and(|c| c.attr("layerZoom").is_some()) => {
                let Some((_, frames)) = self.layer_frames(model) else { return true };
                let tw = find_tagged(&app.root, "WaveZoomSlider").map(|c| c.rect.w as f32).unwrap_or(1.0);
                self.view = crate::zoomslider::zoom_in(self.view, if name == "ZoomInButton" { 2.0 } else { 0.5 }, 0.5, tw, frames);
                true
            }
            // A / F: [re:7a2a22] reads `AmpEnvVisible` /
            // `FreeEnvVisible` (default on).
            UiEvent::Clicked { path, name, on, .. } if (name == "AmpButton" || name == "FreeButton") && app.ctrl(path).is_some_and(|c| c.attr("layerZoom").is_some()) => {
                let key = if name == "AmpButton" { "AmpEnvVisible" } else { "FreeEnvVisible" };
                if name == "AmpButton" {
                    self.env_visible.0 = *on
                } else {
                    self.env_visible.1 = *on
                }
                let mut o = crate::options::Options::load();
                o.set(key, *on);
                o.save();
                true
            }
            // Zoom mode buttons: store the mode and apply it.
            UiEvent::Clicked { path, .. } if path.len() > 1 && app.ctrl(&path[..path.len() - 1]).is_some_and(|c| c.name == "AutoZoomGroup" && c.attr("layerZoom").is_some()) => {
                self.zoom_mode = path[path.len() - 1] as i64;
                let mut o = crate::options::Options::load();
                o.set("WaveDisplayZoomMode", self.zoom_mode);
                o.save();
                self.apply_auto_zoom(model);
                true
            }
            UiEvent::Mouse { path, overload, kind, x, y } if overload == "WaveDisplayZoomSlider" && app.ctrl(path).is_some_and(|c| c.attr("layerZoom").is_some()) => {
                let Some((tw, h)) = app.ctrl(path).map(|c| (c.rect.w as f32, c.rect.h as f32)) else { return false };
                let min = self.layer_frames(model).map(|(_, n)| (64.0 / n.max(1) as f32).min(1.0)).unwrap_or(1.0);
                match kind {
                    dreist_gui::app::MouseKind::Down => {
                        self.zoom_drag = crate::zoomslider::hit(self.view, tw, h, *x, *y).map(|m| (m, *x));
                        self.zoom_drag.is_some()
                    }
                    dreist_gui::app::MouseKind::Drag => {
                        let Some((mode, last)) = self.zoom_drag else { return false };
                        self.view = crate::zoomslider::drag(self.view, tw, mode, last, *x, min);
                        self.zoom_drag = Some((mode, *x));
                        true
                    }
                    dreist_gui::app::MouseKind::Up => self.zoom_drag.take().is_some(),
                    dreist_gui::app::MouseKind::RightDown => false,
                }
            }
            // Double click [re:e3eb0c]: the whole file.
            UiEvent::DoubleClicked { path, .. } if app.ctrl(path).is_some_and(|c| c.overload == "WaveDisplayZoomSlider" && c.attr("layerZoom").is_some()) => {
                self.view = (0.0, 1.0);
                true
            }
            // Wheel [re:78e62a]: the thumb moves 5 px.
            UiEvent::Wheel { path, overload, dy } if overload == "WaveDisplayZoomSlider" && app.ctrl(path).is_some_and(|c| c.attr("layerZoom").is_some()) => {
                let Some(tw) = app.ctrl(path).map(|c| c.rect.w as f32) else { return false };
                self.view = wheel_thumb(self.view, *dy, tw);
                true
            }
            _ => false,
        }
    }

    pub fn modulation_mask(&self, app: &App, model: &Model, path: &[usize]) -> Option<u32> {
        if !self.params.as_ref().is_some_and(|pp| path.starts_with(pp)) {
            // [re:fe83c5]: v2a ([re:fadd5a]).
            let e = model.engine;
            return self
                .vsplit
                .reset_value(path, app.ctrl(path).map(|c| c.name.as_str()).unwrap_or(""))
                .map(|_| crate::model::modulation_mask(model.pad_node(e, model.pad[e]), dreist_formats::PadParam::V2A.name()));
        }
        let c = app.ctrl(path)?;
        if c.class != "CtrlTransmodKnob" {
            return None;
        }
        let par = param_for_control(&c.name)?;
        let e = model.engine;
        Some(crate::model::modulation_mask(model.layer_node(e, model.pad[e], model.layer[e]), par.name()))
    }

    /// The layer parameter of the layer panel control at `path`.
    pub fn layer_param_at(&self, path: &[usize], name: &str) -> Option<LayerParam> {
        self.params.as_ref().filter(|pp| path.starts_with(pp)).and_then(|_| param_for_control(name))
    }

    /// Whether `path` is in the layer controls or the pad controls (the
    /// v1 providers' tooltips).
    pub fn in_layer_or_pad_controls(&self, path: &[usize]) -> bool {
        self.in_params(path) || self.in_pad_controls(path)
    }

    /// The velocity split editor's tooltip.
    pub fn vsplit_tooltip(&self, model: &Model, c: &Ctrl) -> Option<String> {
        self.vsplit.tooltip(model, c)
    }

    /// Whether `path` is the velocity split's v2a knob.
    pub fn is_v2a(&self, path: &[usize], name: &str) -> bool {
        self.vsplit.reset_value(path, name).is_some()
    }

    /// The conversion of a layer / pad view knob's typed edit.
    pub fn edit_curve(&self, path: &[usize], name: &str) -> Option<dreist_formats::params::Curve> {
        use dreist_formats::params::Curve;
        if self.in_params(path) {
            if velocity_control(name).is_some() || env_slot(name, "Depth").is_some() {
                return Some(Curve::Linear(-100.0, 100.0));
            }
            if name.starts_with("Send") {
                return Some(Curve::Decibels);
            }
            return param_for_control(name).map(|p| p.curve());
        }
        if self.in_pad_controls(path) {
            match name {
                "Pan" => return Some(PadParam::Pan.curve()),
                "Gain" => return Some(PadParam::Gain.curve()),
                n if n.starts_with("Send") => return Some(Curve::Decibels),
                _ => {}
            }
        }
        self.vsplit.reset_value(path, name).map(|_| PadParam::V2A.curve())
    }

    /// A knob double click's reset value ([re:ed3c05]
    /// [re:f0ac00]: [re:7529a0], the parameter's default; depths to
    /// none, sends to 0; the pad view's gain / pan / sends and Vel-Amp).
    pub fn reset_value(&self, path: &[usize], name: &str) -> Option<f32> {
        if self.in_params(path) {
            if velocity_control(name).is_some() || env_slot(name, "Depth").is_some() {
                return Some(0.5);
            }
            if name.starts_with("Send") && name.ends_with("Knob") {
                return Some(0.0);
            }
            // Hold's −1 ("off") is 0 on the knob.
            return param_for_control(name).filter(|_| !name.ends_with("Button")).map(|p| p.default_value().max(0.0));
        }
        if self.in_pad_controls(path) {
            match name {
                "Pan" => return Some(0.5),
                "Gain" => return Some(PadParam::Gain.default_value()),
                n if n.starts_with("Send") && n[4..].parse::<usize>().is_ok() => return Some(0.0),
                _ => {}
            }
        }
        self.vsplit.reset_value(path, name)
    }

    pub fn draw(&self, model: &Model, r: &Renderer, p: &mut Painter, c: &Ctrl) -> bool {
        if self.vsplit.draw(model, r, p, c) {
            return true;
        }
        match c.overload.as_str() {
            "LayerWaveDisplay" => {
                self.draw_wave(model, r, p, c);
                true
            }
            "WaveDisplayZoomSlider" if c.attr("layerZoom").is_some() => {
                crate::zoomslider::draw(p, r.styles, "WaveDisplayZoomSliderStyle", c.rect.w as f32, c.rect.h as f32, self.view);
                true
            }
            _ => false,
        }
    }

    fn draw_wave(&self, model: &Model, r: &Renderer, p: &mut Painter, c: &Ctrl) {
        let s = r.styles;
        let (w, h) = (c.rect.w as f32, c.rect.h as f32);
        let purple = Colour::rgba(128, 0, 128, 255);
        let top = wave_colour(s, "colFill", purple);
        let mid = wave_colour(s, "colAltFill", purple);
        // Vertical gradient top → middle → bottom, drawn as bands.
        let bands = (h as usize).max(1);
        for i in 0..bands {
            let t = i as f32 / (bands - 1).max(1) as f32;
            let k = 1.0 - (t - 0.5).abs() * 2.0;
            let mix = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * k).round() as u8;
            p.fill_rect(RectF::new(0.0, i as f32, w, 1.0), Colour::rgba(mix(top.r, mid.r), mix(top.g, mid.g), mix(top.b, mid.b), 255));
        }
        let e = model.engine;
        let (pad, l) = (model.pad[e], model.layer[e]);
        if let Some(sm) = &model.sampling {
            draw_sampling(sm, r, p, w, h);
            return;
        }
        // Areas [re:5a3754]: timeline 0..15, marker strip
        // 15..30, the waveform below.
        let (wy, full_h) = (crate::timeline::WAVE_TOP, h);
        let h = (full_h - wy).max(1.0);
        let centre = wave_colour(s, "colCenterLine", purple);
        p.line(0.0, wy + (h / 2.0).floor() + 0.5, w, wy + (h / 2.0).floor() + 0.5, centre.with_alpha(0.5), 1.0);
        let Some(path) = model.layer_sample(e, pad, l) else {
            p.fill_rect(RectF::new(0.0, 0.0, w, wy), wave_colour(s, "colTimelineBackground", purple));
            crate::timeline::draw_no_audio(p, s, WAVE_STYLE, RectF::new(0.0, wy, w, h));
            return;
        };
        let mut cache = self.peaks.borrow_mut();
        let Some(pk) = cache.get(&path) else { return };
        let line = wave_colour(s, "colLine", purple);
        let disabled = wave_colour(s, "colLineDisabled", line);
        let looping_col = wave_colour(s, "colLineLooping", line);
        let m = wave_markers(model, e, pad, l, pk.frames as u32);
        let n = pk.buckets[0].len();
        let cols = w as usize;
        // [re:bca59e]: one lane per channel, stacked.
        let lanes = pk.buckets.len().max(1);
        let lh = h / lanes as f32;
        if lanes > 1 {
            for k in 0..lanes {
                let y = wy + (lh * k as f32 + lh / 2.0).floor() + 0.5;
                p.line(0.0, y, w, y, centre.with_alpha(0.5), 1.0);
            }
        }
        let (v0, vl) = self.view;
        let frac_of = |x: f32| v0 + x / w.max(1.0) * vl;
        for x in 0..cols {
            let fa = frac_of(x as f32);
            let fb = frac_of(x as f32 + 1.0);
            let a = ((fa * n as f32) as usize).min(n - 1);
            let b = ((fb * n as f32).ceil() as usize).max(a + 1).min(n);
            let frame = (fa * pk.frames as f32) as u32;
            let line = if frame < m.start || frame >= m.end {
                disabled
            } else if m.looping && frame >= m.loop_start && frame < m.loop_end {
                looping_col
            } else {
                line
            };
            for (k, ch) in pk.buckets.iter().enumerate() {
                let (lo, hi) = ch[a.min(ch.len() - 1)..b.min(ch.len())].iter().fold((f32::MAX, f32::MIN), |(l0, h0), (l1, h1)| (l0.min(*l1), h0.max(*h1)));
                let mid = wy + lh * k as f32 + lh / 2.0;
                let y0 = mid - hi.clamp(-1.0, 1.0) * lh / 2.0;
                let y1 = mid - lo.clamp(-1.0, 1.0) * lh / 2.0;
                p.fill_rect(RectF::new(x as f32, y0, 1.0, (y1 - y0).max(1.0)), line);
            }
        }
        // Timeline ([re:02b73a]) and the marker strip.
        let frames = pk.frames.max(1) as f32;
        {
            let rate = pk.rate.max(1.0) as f64;
            let x_of = |t: f64| ((t * rate) as f32 / frames - v0) / vl * w;
            let (start_s, len_s) = ((v0 * frames) as f64 / rate, (vl * frames) as f64 / rate);
            crate::timeline::draw(p, s, WAVE_STYLE, w, h, start_s, len_s, m.start as f64 / rate, self.timeline_beats, model.tempo(), &x_of);
        }
        // Markers (start / end and loop markers →
        // [re:68305f]): a 2 px line from the marker strip down and
        // a 12 px disc (15·0.75 rounded up to even) with a black inset at
        // the top of the strip.
        let hole = wave_colour(s, "colMarkerHole", Colour::BLACK);
        let marker = |p: &mut Painter, f: u32, col: Colour| {
            let x = ((f as f32 / frames - v0) / vl * w).round();
            if x < -6.0 || x > w + 6.0 {
                return;
            }
            let top = crate::timeline::STRIP;
            p.fill_rect(RectF::new(x - 1.0, top, 2.0, full_h - 1.0 - top), col);
            let d = 12.0;
            p.fill_ellipse(RectF::new(x - d / 2.0, top, d, d), col);
            p.fill_ellipse(RectF::new(x - d / 2.0 + 2.0, top + 2.0, d - 4.0, d - 4.0), hole);
        };
        marker(p, m.start, wave_colour(s, "colStartMarker", purple));
        marker(p, m.end.min(pk.frames as u32 - 1), wave_colour(s, "colEndMarker", purple));
        if m.looping {
            marker(p, m.loop_start, wave_colour(s, "colLoopStartMarker", purple));
            marker(p, m.loop_end.min(pk.frames as u32 - 1), wave_colour(s, "colLoopEndMarker", purple));
        }
        // [re:18fbfe]: Amp (top half when both
        // show) and Free envelopes over the waveform.
        let (amp, free) = self.env_visible;
        let mode = model.layer_node(e, pad, l).and_then(|n| n.attr("PlaybackMode")).map_or("OneShot", dreist_dsp::voice::PlaybackMode::canonical).to_string();
        {
            let x_of = |t: f32| ((t * pk.rate) / frames - v0) / vl * w;
            let start_t = m.start as f32 / pk.rate;
            let end_t = (m.end.max(1) - 1) as f32 / pk.rate;
            let half = h / 2.0;
            let areas: Vec<(bool, f32, f32)> = match (amp, free) {
                (true, true) => vec![(true, wy, half), (false, wy + half, half)],
                (true, false) => vec![(true, wy, h)],
                (false, true) => vec![(false, wy, h)],
                _ => Vec::new(),
            };
            for (is_amp, y, ah) in areas {
                let pts = envelope_points(model, e, pad, l, is_amp, &mode, start_t, end_t);
                let mut poly: Vec<(f32, f32)> = pts.iter().map(|&(t, v)| (x_of(t), y + ah * (1.0 - v.clamp(0.0, 1.0)))).collect();
                if poly.len() < 2 {
                    continue;
                }
                let line_pts = poly.clone();
                poly.push((poly.last().unwrap().0, y + ah));
                poly.insert(0, (poly[0].0, y + ah));
                let pink = Colour::rgba(255, 192, 203, 255);
                let back = wave_colour(s, if is_amp { "colAmpEnvelopeBack" } else { "colFreeEnvelopeBack" }, pink.with_alpha(0.2));
                p.fill_polygon(&poly, back);
                p.polyline(&line_pts, wave_colour(s, "colEnvelopeLine", pink), 1.0);
                let geo = EnvGeo { w, y0: wy, h, v0, vl, frames, rate: pk.rate, start: m.start, end: m.end };
                let grabbed = self.env_drag.filter(|d| d.0 == is_amp).map(|d| d.1);
                let handles = if mode == "OneShotLengthBound" { Vec::new() } else { env_handles(model, e, pad, l, is_amp, &mode, &geo, y, ah) };
                let (handle, hole) = (wave_colour(s, "colEnvelopeHandle", Colour::WHITE), wave_colour(s, "colMarkerHole", Colour::BLACK));
                for (k, r) in handles {
                    p.fill_rect(r, handle);
                    let inner = RectF::new(r.x + 2.0, r.y + 2.0, r.w - 4.0, r.h - 4.0);
                    let col = if grabbed == Some(k) { wave_colour(s, "colEnvelopeGrab", Colour::BLACK) } else { hole };
                    p.fill_rect(inner, col);
                }
                // Title box: "AMP ENVELOPE" / "FREE ENVELOPE", centred 10 px down.
                let label = if is_amp { "AMP ENVELOPE" } else { "FREE ENVELOPE" };
                let font = s.font(WAVE_STYLE);
                let (mw, mh) = p.fonts.measure(&font, label);
                let (tw, th) = (mw + 10.0, mh + 10.0);
                let bx = ((w - tw) / 2.0).floor();
                p.fill_rounded(RectF::new(bx, y + 10.0, tw, th), 3.0, [true; 4], wave_colour(s, "colTitleFill", Colour::rgba(64, 64, 64, 128)));
                p.text(RectF::new(bx, y + 10.0, tw, th), label, s.colour(WAVE_STYLE, "colText", Colour::WHITE), &font, dreist_gui::style::HAlign::Center, dreist_gui::style::VAlign::Center);
            }
        }
    }
}

/// The wave display's mapping for the envelope handles.
struct EnvGeo {
    w: f32,
    /// Top and height of the waveform area.
    y0: f32,
    h: f32,
    v0: f32,
    vl: f32,
    frames: f32,
    rate: f32,
    start: u32,
    end: u32,
}

impl EnvGeo {
    fn x_of(&self, t: f32) -> f32 {
        ((t * self.rate) / self.frames - self.v0) / self.vl * self.w
    }
    fn t_of(&self, x: f32) -> f32 {
        (self.v0 + x / self.w.max(1.0) * self.vl) * self.frames / self.rate
    }
    fn start_t(&self) -> f32 {
        self.start as f32 / self.rate
    }
    fn end_t(&self) -> f32 {
        (self.end.max(1) - 1) as f32 / self.rate
    }
    /// (y, height) of the Amp / Free area.
    fn area(&self, amp: bool, vis: (bool, bool)) -> Option<(f32, f32)> {
        match (vis, amp) {
            ((true, true), true) => Some((self.y0, self.h / 2.0)),
            ((true, true), false) => Some((self.y0 + self.h / 2.0, self.h / 2.0)),
            ((true, false), true) | ((false, true), false) => Some((self.y0, self.h)),
            _ => None,
        }
    }
}

fn env_param(amp: bool, k: usize) -> dreist_formats::LayerParam {
    dreist_formats::LayerParam::ALL[14 + if amp { 0 } else { 8 } + k]
}

/// Envelope positions [re:f00247]: x of start, A, D, H / note-off, R end
/// and the sustain level.
fn env_xs(model: &Model, e: usize, p: usize, l: usize, amp: bool, gated: bool, g: &EnvGeo) -> ([f32; 5], f32) {
    let v = |k: usize| model.layer_param(e, p, l, env_param(amp, k));
    let p4 = |x: f32| x * x * x * x;
    let st = g.start_t();
    let ta = st + p4(v(0)) * 10.0;
    let td = ta + p4(v(1)) * 10.0;
    let th = if gated { td.max(g.end_t()) } else { td + p4(v(3)) * 60.0 };
    let tr = th + p4(v(4)) * 15.0;
    ([g.x_of(st), g.x_of(ta), g.x_of(td), g.x_of(th), g.x_of(tr)], v(2))
}

/// Envelope handles [re:30873e] + curve visibility
/// [re:466986]: the visible 10 × 10 handles — 0 attack peak, 1 decay end, 2
/// sustain (mid of the sustain line), 3 hold end (one-shot modes), 4
/// release end, 5 / 6 / 7 the attack / decay / release curves.
fn env_handles(model: &Model, e: usize, p: usize, l: usize, amp: bool, mode: &str, g: &EnvGeo, ay: f32, ah: f32) -> Vec<(usize, RectF)> {
    let gated = matches!(mode, "Gated" | "Looping");
    let (x, s) = env_xs(model, e, p, l, amp, gated, g);
    let cv = |k: usize| model.layer_param(e, p, l, env_param(amp, k));
    let sy = ay + ah * (1.0 - s);
    let clampy = |y: f32| y.min(ay + ah - 10.0).max(ay);
    let r = |x: f32, y: f32| RectF::new(x, y, 10.0, 10.0);
    let hs = [
        r(x[1] - 5.0, ay),
        r(x[2] - 5.0, (sy - 5.0).max(ay)),
        r(x[2] + (x[3] - x[2] - 10.0) / 2.0, (sy - 5.0).max(ay)),
        r(x[3] - 5.0, (sy - 5.0).max(ay)),
        r(x[4] - 5.0, ay + ah - 10.0),
        r(x[0] - 5.0 + (x[1] - x[0]) / 2.0, clampy(ay - 5.0 + ah * (1.0 - cv(5)))),
        r(x[1] - 5.0 + (x[2] - x[1]) / 2.0, clampy(ay - 5.0 + (sy - ay) * (1.0 - cv(6)))),
        r(x[3] - 5.0 + (x[4] - x[3]).abs() / 2.0, clampy(sy - 5.0 + (ay + ah - sy) * (1.0 - cv(7)))),
    ];
    let mut vis = [true; 8];
    vis[3] = !gated;
    if hs[0].x < hs[5].x + 5.0 {
        vis[5] = false;
    }
    if hs[7].x <= hs[3].x + 5.0 {
        vis[7] = false;
    }
    if hs[3].x - hs[1].x - 10.0 < 10.0 {
        vis[2] = false;
    }
    if hs[1].x - hs[0].x - 10.0 < 10.0 {
        vis[6] = false;
    }
    (0..8).filter(|&k| vis[k]).map(|k| (k, hs[k])).collect()
}

/// The handle under the mouse: (Amp?, handle), Amp first.
fn env_hit(model: &Model, e: usize, p: usize, l: usize, vis: (bool, bool), g: &EnvGeo, x: f32, y: f32) -> Option<(bool, usize)> {
    let mode = model.layer_node(e, p, l).and_then(|n| n.attr("PlaybackMode")).map_or("OneShot", dreist_dsp::voice::PlaybackMode::canonical).to_string();
    if mode == "OneShotLengthBound" {
        return None;
    }
    for amp in [true, false] {
        let Some((ay, ah)) = g.area(amp, vis) else { continue };
        if let Some((k, _)) = env_handles(model, e, p, l, amp, &mode, g, ay, ah).into_iter().find(|(_, r)| x >= r.x && x < r.x + r.w && y >= r.y && y < r.y + r.h) {
            return Some((amp, k));
        }
    }
    None
}

/// Envelope drag [re:874f6e]: the layer parameter values for a handle
/// dragged to (x, y) — times from x (`((t − base)/range)^¼`, A / D 10 s, H
/// 60 s, R 15 s), sustain / curves from y; dragging the decay or hold
/// handle also sets the sustain from y unless that handle is locked
/// (`lock` = (decay, hold)).
fn env_drag_values(model: &Model, e: usize, p: usize, l: usize, amp: bool, handle: usize, vis: (bool, bool), g: &EnvGeo, x: f32, y: f32, lock: (bool, bool)) -> Vec<(u8, f32)> {
    let Some((ay, ah)) = g.area(amp, vis) else { return Vec::new() };
    let mode = model.layer_node(e, p, l).and_then(|n| n.attr("PlaybackMode")).map_or("OneShot", dreist_dsp::voice::PlaybackMode::canonical).to_string();
    let gated = matches!(mode.as_str(), "Gated" | "Looping");
    let v = |k: usize| model.layer_param(e, p, l, env_param(amp, k));
    let p4 = |x: f32| x * x * x * x;
    let t = g.t_of(x);
    let st = g.start_t();
    let time = |base: f32, range: f32| ((t.clamp(base, base + range) - base) / range).powf(0.25);
    let sustain = |y: f32| (1.0 - (y - ay) / ah.max(1.0)).clamp(0.0, 1.0);
    let mut out = Vec::new();
    let value = match handle {
        0 => time(st, 10.0),
        1 => {
            if !lock.0 {
                out.push((env_param(amp, 2) as u8, sustain(y)));
            }
            time(st + p4(v(0)) * 10.0, 10.0)
        }
        2 | 5 => sustain(y),
        3 => {
            if !lock.1 {
                out.push((env_param(amp, 2) as u8, sustain(y)));
            }
            time(st + p4(v(0)) * 10.0 + p4(v(1)) * 10.0, 60.0)
        }
        4 => {
            let td = st + p4(v(0)) * 10.0 + p4(v(1)) * 10.0;
            let base = if gated { td.max(g.end_t()) } else { td + p4(v(3)) * 60.0 };
            time(base, 15.0)
        }
        6 => {
            let sy = ah * (1.0 - v(2));
            1.0 - (y - ay) / sy.max(1.0)
        }
        _ => {
            let sy = ay + ah * (1.0 - v(2));
            1.0 - (y - sy) / (ay + ah - sy).max(1.0)
        }
    };
    out.push((env_param(amp, handle) as u8, value.clamp(0.0, 1.0)));
    out
}

/// Envelope curve points (time in seconds, level) for the layer's Amp or
/// Free envelope, run through the engine's `Envelope` ([re:50f14d] /
/// [re:f00247]: A, D to sustain, then hold
/// in one-shot modes or sustain until the end marker in gated / looping
/// modes, then the release), starting at the start marker.
fn envelope_points(model: &Model, e: usize, p: usize, l: usize, amp: bool, mode: &str, start_t: f32, end_t: f32) -> Vec<(f32, f32)> {
    use dreist_formats::LayerParam as LP;
    let lp = |a: LP, f: LP| model.layer_param(e, p, l, if amp { a } else { f });
    let params = dreist_dsp::envelope::EnvelopeParams {
        attack: lp(LP::AmpEnvAttack, LP::FreeEnvAttack),
        decay: lp(LP::AmpEnvDecay, LP::FreeEnvDecay),
        sustain: lp(LP::AmpEnvSustain, LP::FreeEnvSustain),
        hold: lp(LP::AmpEnvHold, LP::FreeEnvHold),
        release: lp(LP::AmpEnvRelease, LP::FreeEnvRelease),
        attack_curve: lp(LP::AmpEnvAttackCurve, LP::FreeEnvAttackCurve),
        decay_curve: lp(LP::AmpEnvDecayCurve, LP::FreeEnvDecayCurve),
        release_curve: lp(LP::AmpEnvReleaseCurve, LP::FreeEnvReleaseCurve),
    };
    // Length-bound envelope: the AHR envelope laid over the played
    // part (start..end marker).
    if mode == "OneShotLengthBound" {
        let n = 600u32;
        let mut env = dreist_dsp::envelope::AhrEnvelope::default();
        env.initialize(&params, n, 1, 1.0);
        env.note_on();
        let mut buf = vec![0.0f32; n as usize + 1];
        env.process(&mut buf);
        let span = (end_t - start_t).max(1e-6);
        return buf.iter().enumerate().map(|(i, v)| (start_t + span * i as f32 / n as f32, *v)).collect();
    }
    let p4 = |x: f32| x * x * x * x;
    let (a, d, hold, r) = (p4(params.attack) * 10.0, p4(params.decay) * 10.0, p4(params.hold) * 60.0, p4(params.release) * 15.0);
    let gated = matches!(mode, "Gated" | "Looping");
    let sustain_end = if gated { (start_t + a + d).max(end_t) - start_t } else { a + d + hold };
    let total = (sustain_end + r).max(1e-3);
    // Enough steps for a smooth curve.
    let steps = 600usize;
    let sr = steps as f32 / total;
    let mut env = dreist_dsp::envelope::Envelope::new(sr);
    env.initialize(&params, !gated, 1.0);
    env.note_on();
    let mut out = Vec::with_capacity(steps + 2);
    let mut buf = [0.0f32; 1];
    for i in 0..=steps {
        let t = i as f32 / sr;
        if gated && t >= sustain_end && i > 0 && (t - 1.0 / sr) < sustain_end {
            env.note_off();
        }
        env.process(&mut buf);
        out.push((start_t + t, buf[0]));
    }
    out
}

/// Sampling mode: the whole take fills the display (the take's start and
/// end markers sit at its edges, [re:6b3a28]).
fn draw_sampling(sm: &crate::model::Sampling, r: &Renderer, p: &mut Painter, w: f32, h: f32) {
    let s = r.styles;
    let purple = Colour::rgba(128, 0, 128, 255);
    let line = wave_colour(s, "colLine", purple);
    let n = sm.buckets.len();
    if n == 0 {
        return;
    }
    let cols = w.max(1.0) as usize;
    for x in 0..cols {
        let a = (x * n / cols).min(n - 1);
        let b = ((x + 1) * n / cols).max(a + 1).min(n);
        let (lo, hi) = sm.buckets[a..b].iter().fold((f32::MAX, f32::MIN), |(l0, h0), (l1, h1)| (l0.min(*l1), h0.max(*h1)));
        let y0 = h / 2.0 - hi.clamp(-1.0, 1.0) * h / 2.0;
        let y1 = h / 2.0 - lo.clamp(-1.0, 1.0) * h / 2.0;
        p.fill_rect(RectF::new(x as f32, y0, 1.0, (y1 - y0).max(1.0)), line);
    }
    for (x, col) in [(0.0, "colStartMarker"), (w - 1.0, "colEndMarker")] {
        p.fill_rect(RectF::new((x - 1.0f32).max(0.0), 0.0, 2.0, h), wave_colour(s, col, purple));
    }
}

/// [re:155b84]: the file as 1, 2, 4 or 8 bars of 4/4 within 90..180 BPM (v1
/// adds 0.5 twice before truncating), else 0.
pub(crate) fn guess_bpm(frames: f32, sample_rate: f32) -> f32 {
    let secs = if sample_rate > 0.0 { frames / sample_rate } else { 0.0 };
    let mut bars = 1.0f32;
    loop {
        let mut v = 60.0 / (secs / (bars * 4.0)) + 0.5;
        v = if v >= 0.0 { v + 0.5 } else { v - 0.5 };
        let v = v as i32 as f32;
        if (90.0..=180.0).contains(&v) {
            return v;
        }
        bars += bars;
        if bars > 8.0 {
            return 0.0;
        }
    }
}

/// [re:78e62a]: a wheel step moves the zoom slider's thumb 5 px (of a
/// `track_w` px track).
fn wheel_thumb(view: (f32, f32), dy: f32, track_w: f32) -> (f32, f32) {
    let px = if dy > 0.0 { 5.0 } else { -5.0 };
    crate::zoomslider::clamp((view.0 + px / track_w.max(1.0), view.1))
}

#[cfg(test)]
mod tests {
    #[test]
    fn guess_bpm_as_v1() {
        // 2 s: 1 bar → 120, plus v1's extra 0.5 rounding → 121.
        assert_eq!(super::guess_bpm(88200.0, 44100.0), 121.0);
        // 8 s: 4 bars → 120 → 121; 0.5 s: nothing in 90..180 up to 8 bars.
        assert_eq!(super::guess_bpm(352800.0, 44100.0), 121.0);
        assert_eq!(super::guess_bpm(100.0, 44100.0), 0.0);
    }

    #[test]
    fn envelope_handles_and_drag() {
        crate::app::test_setup();
        use dreist_formats::Node;
        let mut doc = Node::new("root", None);
        let mut eng = Node::new("Engine", Some("Engine0"));
        let mut pad = Node::new("Pad", Some("Pad0"));
        let mut l = Node::new("Layer", Some("Layer0"));
        l.set("AmpEnvAttack", 0.5).set("AmpEnvDecay", 0.5).set("AmpEnvSustain", 0.5).set("AmpEnvHold", 0.3).set("AmpEnvRelease", 0.5);
        pad.children.push(l);
        eng.children.push(pad);
        doc.children.push(eng);
        let model = Model::new(doc);
        let g = EnvGeo { w: 1000.0, y0: 0.0, h: 200.0, v0: 0.0, vl: 1.0, frames: 44100.0 * 20.0, rate: 44100.0, start: 0, end: 44100 * 20 };
        let hs = env_handles(&model, 0, 0, 0, true, "OneShot", &g, 0.0, 100.0);
        let (_, a) = hs.iter().find(|(k, _)| *k == 0).copied().unwrap();
        // Attack 0.5 → 0.625 s → x = 31.25 px.
        assert!((a.x + 5.0 - 31.25).abs() < 0.5, "{a:?}");
        assert_eq!(env_hit(&model, 0, 0, 0, (true, true), &g, a.x + 5.0, a.y + 5.0), Some((true, 0)));
        // Dragging it to 2.5 s gives (2.5/10)^¼.
        let vals = env_drag_values(&model, 0, 0, 0, true, 0, (true, true), &g, 125.0, 0.0, (false, false));
        let (pid, v) = *vals.last().unwrap();
        assert_eq!(pid, dreist_formats::LayerParam::AmpEnvAttack as u8);
        assert!((v - 0.25f32.powf(0.25)).abs() < 1e-3, "{v}");
        // The decay handle also sets the sustain from y (not with shift).
        assert_eq!(env_drag_values(&model, 0, 0, 0, true, 1, (true, true), &g, 125.0, 25.0, (false, false)).len(), 2);
        assert_eq!(env_drag_values(&model, 0, 0, 0, true, 1, (true, true), &g, 125.0, 25.0, (true, false)).len(), 1);
    }

    use super::*;

    #[test]
    fn zero_crossing_snap() {
        crate::app::test_setup();
        let z = [10, 50, 200];
        assert_eq!(snap_zero(&z, 45, 20), 50);
        assert_eq!(snap_zero(&z, 120, 20), 120);
        assert_eq!(snap_zero(&z, 14, 20), 10);
    }
}
