//! Sampler page (`Sampler/SamplerPanel.skin`, the LCD's last tab): v1's
//! take recorder ([re:9fbf48]).
//!
//! - Trigger mode (Immediate / MIDI / Host / Threshold, `e_setSamplerStartMode`
//!   0xf2, [re:5fa03c]) with the threshold panel (Start / Stop sliders and the
//!   input meter between them) shown for Threshold; record mode (Free /
//!   Timed / Looped, 0xf3 off / stop / retake) with the Beats / Seconds switch
//!   and the Bars / Beats or Seconds boxes shown for Timed and Looped.
//! - Control events [re:7941cb]: Arm → 0xec, Rec → 0xef, Thru → 0xff,
//!   InputGain → 0xf4 (knob × 4), Preview vol → 0xe8, thresholds → 0xf6 /
//!   0xf7 via [re:c57733], sync group → 0xfa, seconds → 0xf8 (0..3600),
//!   bars / beats → 0xf9, source → 0xfd, Mono → 0xfe (left channel).
//! - Finished takes go to the take list (the engine posts the audio); Take
//!   picks the current one, which the wave area shows with its start / end
//!   markers. Clear (0xfb) / Clear All (0xfc), Preview / Loop, Crop,
//!   Normalize, the Export/Send menu, Slice and → Pad / → Pads work on the
//!   editor's copy of the takes, saved as WAV files where they are used.

use dreist_engine::binding::id;
use dreist_gui::app::{App, MouseKind, UiEvent};
use dreist_gui::paint::{Painter, RectF};
use dreist_gui::render::Renderer;
use dreist_gui::style::{HAlign, VAlign};
use dreist_gui::{Colour, Ctrl, Rect, Skin};

use crate::EditorError;
use crate::command::{Command, msg};
use crate::model::{LAYERS, Model};

/// `e_setPreviewVolume`.
const SET_PREVIEW_VOLUME: u32 = 0xe8;
/// [re:1864e3]: "External" (the host
/// input, 38), then output locations 1–33 by name (Master, Sub output
/// 1–32); routing plugins would follow (out of scope).
pub fn source_items() -> Vec<String> {
    std::iter::once("External".to_string()).chain((1..34).map(|i| dreist_engine::engine::Route::from_index(i).name())).collect()
}

/// List index ↔ `recordsource`: item 0 = 38, item i = location i.
fn source_of_item(i: usize) -> i32 {
    if i == 0 { dreist_engine::sampler::SOURCE_EXTERNAL } else { i.min(33) as i32 }
}

fn item_of_source(s: i32) -> usize {
    if (1..34).contains(&s) { s as usize } else { 0 }
}

/// [re:c57733].
pub fn level_to_value(l: f32) -> f32 {
    if l <= 0.0 {
        return 0.0;
    }
    let db = 20.0 * l.log10();
    if db > -1000.0 { (db / 26.888344).exp() / 1.25 } else { 0.0 }
}

/// [re:c57733].
pub fn value_to_level(v: f32) -> f32 {
    if (v as f64) <= 5.64045724672641e-17 {
        return 0.0;
    }
    10f32.powf((v * 1.25).ln() * 6.0 / 4.4629)
}

/// Frames per wave overview bucket.
const BUCKET: usize = 64;

/// A recorded take (the editor's copy).
#[derive(Clone, Debug)]
pub struct Take {
    pub left: Vec<f32>,
    pub right: Vec<f32>,
    pub sample_rate: f32,
    /// Start / end markers (frames): Crop, Normalize, previews and saves.
    pub start: u32,
    pub end: u32,
    /// "Take n" (numbered in recording order).
    pub number: usize,
    /// (min, max) per [`BUCKET`] frames of both channels.
    buckets: Vec<(f32, f32)>,
}

impl Take {
    fn new(left: &[f32], right: &[f32], sample_rate: f32, number: usize) -> Take {
        let right = if right.len() == left.len() { right.to_vec() } else { left.to_vec() };
        let mut t = Take { left: left.to_vec(), right, sample_rate: sample_rate.max(1.0), start: 0, end: left.len() as u32, number, buckets: Vec::new() };
        t.overview();
        t
    }

    fn overview(&mut self) {
        self.buckets = self.left.chunks(BUCKET).zip(self.right.chunks(BUCKET)).map(|(l, r)| crate::model::bucket(l, r)).collect();
    }

    pub fn frames(&self) -> u32 {
        self.left.len() as u32
    }

    pub fn label(&self) -> String {
        format!("Take {}", self.number)
    }

    /// The marked region (start < end).
    fn region(&self) -> (usize, usize) {
        let n = self.left.len();
        let a = (self.start as usize).min(n);
        let b = (self.end as usize).clamp(a, n);
        if a < b { (a, b) } else { (0, n) }
    }

    /// Crop to the markers.
    fn crop(&mut self) {
        let (a, b) = self.region();
        self.left = self.left[a..b].to_vec();
        self.right = self.right[a..b].to_vec();
        self.start = 0;
        self.end = self.frames();
        self.overview();
    }

    /// The region between the markers to a 0 dB peak.
    fn normalize(&mut self) {
        let (a, b) = self.region();
        let peak = self.left[a..b].iter().chain(&self.right[a..b]).fold(0.0f32, |m, v| m.max(v.abs()));
        if peak <= 1e-9 {
            return;
        }
        let g = 1.0 / peak;
        for v in self.left[a..b].iter_mut().chain(self.right[a..b].iter_mut()) {
            *v *= g;
        }
        self.overview();
    }

    fn data(&self) -> std::sync::Arc<dreist_dsp::voice::SampleData> {
        std::sync::Arc::new(dreist_dsp::voice::SampleData { left: self.left.clone(), right: Some(self.right.clone()), sample_rate: self.sample_rate })
    }

    /// Write the take (cropped to the markers when `crop`) as a WAV file;
    /// peaks above 0 dBFS are scaled to 0.998.
    pub fn save(&self, path: &std::path::Path, crop: bool) -> std::io::Result<()> {
        let (a, b) = if crop { self.region() } else { (0, self.left.len()) };
        let (l, r) = (&self.left[a..b], &self.right[a..b]);
        let peak = l.iter().chain(r).fold(0.0f32, |m, v| m.max(v.abs()));
        let g = if peak > 1.0 { 0.998 / peak } else { 1.0 };
        let (l, r): (Vec<f32>, Vec<f32>) = (l.iter().map(|v| v * g).collect(), r.iter().map(|v| v * g).collect());
        if let Some(d) = path.parent() {
            std::fs::create_dir_all(d)?;
        }
        dreist_engine::write_wav(path, self.sample_rate.round() as u32, &l, &r)
    }
}

/// What the page asks the editor to do (file dialogs, pads, the slicer).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Request {
    /// Export take...: a file.
    ExportTake,
    /// Export all takes...: a folder, "<Name> <Take>.wav" each.
    ExportAll,
    /// Send take to current pad (its first layer, the pad's samples cleared).
    SendToPad,
    /// Send all takes to empty pads (first layers).
    SendAllToPads,
    /// Send all takes to free layers on current pad.
    SendToFreeLayers,
    /// Send all takes to free layers on empty pads.
    SendToEmptyPadLayers,
    /// Slice: save the take and open it in the slicer.
    Slice,
}

/// The Export/Send menu as v1 builds it ([re:3da5b8]): items and their
/// requests (`None`: the auto-crop check or a separator).
fn send_menu(auto_crop: bool) -> Vec<(String, Option<Request>)> {
    vec![
        ("Export take...".into(), Some(Request::ExportTake)),
        ("Export all takes...".into(), Some(Request::ExportAll)),
        ("Send take to current pad...".into(), Some(Request::SendToPad)),
        ("Send all takes to empty pads...".into(), Some(Request::SendAllToPads)),
        ("Send all takes to free layers on current pad...".into(), Some(Request::SendToFreeLayers)),
        ("Send all takes to free layers on empty pads...".into(), Some(Request::SendToEmptyPadLayers)),
        (String::new(), None),
        (format!("{}Auto-crop saved samples", if auto_crop { dreist_gui::menu::CHECK } else { "" }), None),
    ]
}

/// The open popup.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Menu {
    Source,
    Take,
    Send,
    Timeline,
}

/// A press in the wave area being dragged.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Drag {
    /// The start (true) or end marker.
    Marker(bool),
    /// Zoom (vertical) and scroll (horizontal) from (x, y, view).
    View(f32, f32, (f32, f32)),
}

#[derive(Default)]
pub struct Sampler {
    /// The page's `SamplerPanel` (the skin's controls).
    host: Option<Vec<usize>>,
    /// The take display (`WaveArea`).
    wave: Option<Vec<usize>>,
    pub takes: Vec<Take>,
    /// The current take (Take list).
    pub current: usize,
    /// Takes recorded this session (numbers).
    taken: usize,
    /// Record mode: 0 Free, 1 Timed, 2 Looped.
    pub record_mode: u8,
    /// Name box: the saved files' name.
    pub name: String,
    previewing: bool,
    loop_preview: bool,
    /// Export/Send › Auto-crop saved samples.
    pub auto_crop: bool,
    /// Visible part of the take (start, length) as fractions.
    view: (f32, f32),
    zoom_mode: i64,
    drag: Option<Drag>,
    menu: Option<(Menu, Vec<Option<Request>>)>,
    /// A message for the status line (until the state changes).
    pub message: Option<String>,
    /// What the editor should run after the event.
    pub request: Option<Request>,
    timeline_beats: bool,
}

/// [re:e12a3e] [re:3fbef6]: takes are
/// saved to `<record path>/<date>/` ([re:9c6245] +
/// "Sampler Captures", unless `SamplerRecordPath` is set) when no file
/// dialog is available.
pub fn record_dir(opts: &crate::options::Options) -> std::path::PathBuf {
    let custom = opts.string("SamplerRecordPath", "");
    let base = if custom.is_empty() { crate::options::documents_dir().join("Sampler Captures") } else { std::path::PathBuf::from(custom) };
    let (y, mo, d, _, _, _) = now_utc();
    base.join(format!("{y:04}-{mo:02}-{d:02}"))
}

/// UTC civil time (year, month, day, hour, minute, second).
fn now_utc() -> (i64, u32, u32, u32, u32, u32) {
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0);
    let (days, rem) = (secs.div_euclid(86400), secs.rem_euclid(86400));
    // Howard Hinnant's civil_from_days.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = yoe + era * 400 + (m <= 2) as i64;
    (y, m, d, (rem / 3600) as u32, (rem / 60 % 60) as u32, (rem % 60) as u32)
}

/// A free sample file name [re:b4ec72]: `name.wav`, then `name (2).wav` …
/// up to 9999.
pub fn free_filename(dir: &std::path::Path, name: &str) -> Option<std::path::PathBuf> {
    let p = dir.join(format!("{name}.wav"));
    if !p.exists() {
        return Some(p);
    }
    (2..=9999).map(|n| dir.join(format!("{name} ({n}).wav"))).find(|p| !p.exists())
}

/// Wave display layout: the time ruler, the marker strip, the wave below.
pub(crate) const RULER: f32 = crate::timeline::STRIP;
pub(crate) const WAVE_TOP: f32 = crate::timeline::WAVE_TOP;

/// A theme colour (the v1 `WaveEditor*` / `PreviewEditor*` names).
/// A wave display colour (`WaveDisplayStyle`'s `attr`).
pub(crate) fn wave(r: &Renderer, attr: &str, default: Colour) -> Colour {
    r.styles.colour("WaveDisplayStyle", attr, default)
}

pub(crate) fn colour(r: &Renderer, name: &str, default: Colour) -> Colour {
    r.styles.colours.get(name).copied().unwrap_or(default)
}

/// The wave displays' ruler and marker strip: time labels (seconds from
/// the file start, or bars.beats at `tempo`) over the visible `start_s` ..
/// `start_s + len_s`.
pub(crate) fn draw_ruler(r: &Renderer, p: &mut Painter, w: f32, start_s: f64, len_s: f64, beats: bool, tempo: f64) {
    let ink = wave(r, "colTimelineText", Colour::rgba(18, 25, 60, 255));
    p.fill_rect(RectF::new(0.0, 0.0, w, WAVE_TOP), colour(r, "LcdLight", Colour::rgba(222, 229, 237, 255)));
    p.line(0.0, RULER + 0.5, w, RULER + 0.5, ink.with_alpha(0.4), 1.0);
    p.line(0.0, WAVE_TOP - 0.5, w, WAVE_TOP - 0.5, ink, 1.0);
    let font = r.styles.font("StandardLabelStyle");
    let x_of = |t: f64| ((t - start_s) / len_s.max(1e-9) * w as f64) as f32;
    let marks: Vec<(f64, String)> = if beats {
        let (unit, labels) = crate::timeline::beat_labels(start_s * 1000.0, len_s * 1000.0, tempo);
        labels.into_iter().map(|(i, t, _)| (i as f64 * unit * 0.001, t)).collect()
    } else {
        crate::timeline::time_labels(start_s * 1000.0, len_s * 1000.0).into_iter().map(|ms| (ms * 0.001, crate::timeline::time_text(ms))).collect()
    };
    for (t, label) in marks {
        let x = x_of(t).floor();
        if x < 0.0 || x > w {
            continue;
        }
        p.line(x + 0.5, 0.0, x + 0.5, RULER, ink, 1.0);
        p.text(RectF::new(x + 2.0, 0.0, 60.0, RULER), &label, ink, &font, HAlign::Left, VAlign::Center);
    }
}

/// A waveform from (min, max) buckets of `per` frames over frames `v0..v1`
/// into `area`, the pixel columns `cols` only.
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_wave(p: &mut Painter, area: RectF, buckets: &[(f32, f32)], per: usize, v0: f64, v1: f64, col: Colour, cols: (f32, f32)) {
    if buckets.is_empty() || v1 <= v0 {
        return;
    }
    let mid = area.y + area.h / 2.0;
    let fpp = (v1 - v0) / area.w.max(1.0) as f64;
    for px in cols.0.max(0.0) as usize..cols.1.min(area.w).max(0.0) as usize {
        let f0 = v0 + px as f64 * fpp;
        let b0 = (f0 / per as f64).floor().max(0.0) as usize;
        let b1 = (((f0 + fpp) / per as f64).ceil() as usize).max(b0 + 1).min(buckets.len());
        if b0 >= buckets.len() {
            break;
        }
        let (lo, hi) = buckets[b0..b1].iter().fold((f32::MAX, f32::MIN), |(l, h), &(a, b)| (l.min(a), h.max(b)));
        let (y0, y1) = (mid - hi.clamp(-1.0, 1.0) * area.h / 2.0 * 0.95, mid - lo.clamp(-1.0, 1.0) * area.h / 2.0 * 0.95);
        p.fill_rect(RectF::new(area.x + px as f32, y0, 1.0, (y1 - y0).max(1.0)), col);
    }
}

impl Sampler {
    /// A control of the sampler page by name.
    pub fn path_in(&self, app: &App, name: &str) -> Option<Vec<usize>> {
        let h = self.host.clone()?;
        let rel = app.ctrl(&h)?.path_of(name)?;
        Some([h, rel].concat())
    }

    /// `path` is a control of the sampler page.
    pub fn owns(&self, path: &[usize]) -> bool {
        self.host.as_ref().is_some_and(|h| path.starts_with(h))
    }

    /// Mount the page skin into `page` (in a `SamplerPanel` the meters
    /// feed from the sampler input).
    pub fn build(skin: &mut Skin, root: &mut Ctrl, page: &[usize]) -> Result<Sampler, EditorError> {
        let mut host = &mut *root;
        for &i in page {
            host = &mut host.children[i];
        }
        let mut panel = Ctrl {
            class: "CtrlPanel".into(),
            name: "SamplerPanel".into(),
            style: "InvisibleStyle".into(),
            rect: Rect::new(0, 0, host.rect.w, host.rect.h),
            visible: true,
            enabled: true,
            ..Ctrl::default()
        };
        crate::front::mount_into(skin, &mut panel, "Sampler/SamplerPanel.skin")?;
        crate::front::mount_all(skin, &mut panel)?;
        if let Some(w) = panel.find_mut("WaveArea") {
            w.overload = "SamplerWaveArea".into();
        }
        for n in ["RecordStatus", "ThresholdVU", "VUMeterLeft", "VUMeterRight"] {
            if let Some(c) = panel.find_mut(n) {
                c.passive = true;
            }
        }
        host.children.push(panel);
        let hp = [page, &[host.children.len() - 1]].concat();
        let wave = root.path_of("WaveArea").filter(|p| p.starts_with(&hp));
        let o = crate::options::Options::load();
        Ok(Sampler {
            host: Some(hp),
            wave,
            record_mode: o.int("SamplerLastTimedMode", 0).clamp(0, 2) as u8,
            name: "Sample".into(),
            view: (0.0, 1.0),
            timeline_beats: crate::timeline::beats_mode("Sampler"),
            ..Sampler::default()
        })
    }

    /// The current take.
    pub fn take(&self) -> Option<&Take> {
        self.takes.get(self.current)
    }

    /// A take finished recording: it joins the take list and becomes the
    /// current one (the display shows all of it); its index.
    pub fn add_take(&mut self, left: &[f32], right: &[f32], sr: f32) -> usize {
        self.taken += 1;
        self.takes.push(Take::new(left, right, sr, self.taken));
        self.current = self.takes.len() - 1;
        self.view = (0.0, 1.0);
        self.message = None;
        self.current
    }

    /// The take's file name: "<Name> <Take>".
    pub fn file_stem(&self, i: usize) -> String {
        let name = if self.name.trim().is_empty() { "Sample" } else { self.name.trim() };
        format!("{name} {}", self.takes.get(i).map(Take::label).unwrap_or_default())
    }

    fn status(&self, model: &Model) -> String {
        let s = &model.sampler;
        match s.state {
            2 => "Recording".into(),
            1 => match s.start_mode {
                1 => "Armed: waiting for MIDI".into(),
                2 => "Armed: waiting for the host".into(),
                3 => "Armed: waiting for the threshold".into(),
                _ => "Armed".into(),
            },
            _ => self.message.clone().unwrap_or_else(|| match self.takes.len() {
                0 => "Idle".into(),
                1 => "Idle (1 take)".into(),
                n => format!("Idle ({n} takes)"),
            }),
        }
    }

    pub fn sync(&mut self, app: &mut App, model: &Model) -> bool {
        let Some(hp) = self.host.clone() else { return false };
        if self.current >= self.takes.len() {
            self.current = self.takes.len().saturating_sub(1);
        }
        let editing = app.is_editing();
        let s = model.sampler.clone();
        let recording = s.state == 2;
        let has_take = !self.takes.is_empty() && !recording;
        let status = self.status(model);
        let take = self.take().map(Take::label).unwrap_or_default();
        let (mode, previewing, looping, name, zoom) = (self.record_mode, self.previewing, self.loop_preview, self.name.clone(), self.zoom_mode);
        let (ml, mr) = (level_to_value(s.meters.0), level_to_value(s.meters.1));
        let Some(h) = app.ctrl_mut(&hp) else { return false };
        h.walk_mut(&mut |c: &mut Ctrl| match c.name.as_str() {
            "ArmButton" => {
                c.on = s.state >= 1;
                c.set_attr("flashing", (s.state == 1).to_string());
            }
            "RecButton" => c.on = recording,
            "SourceList" => {
                c.enabled = s.state == 0;
                c.text = source_items()[item_of_source(s.source)].clone();
            }
            "TriggerModeButtonGroup" => {
                for (i, b) in c.children.iter_mut().enumerate() {
                    b.on = i == s.start_mode as usize;
                    b.enabled = !recording;
                }
            }
            "TimedModeButtonGroup" => {
                for (i, b) in c.children.iter_mut().enumerate() {
                    b.on = i == mode as usize;
                    b.enabled = !recording;
                }
            }
            "ThresholdPanel" => c.visible = s.start_mode == 3,
            "SyncPanel" => c.visible = mode != 0,
            "BeatsPanel" => c.visible = s.beats_mode,
            "SecondsPanel" => c.visible = !s.beats_mode,
            "SyncButtonGroup" => {
                for (i, b) in c.children.iter_mut().enumerate() {
                    b.on = (i == 0) == s.beats_mode;
                    b.enabled = !recording;
                }
            }
            "SecsTextBox" | "BarsTextBox" | "BeatsTextBox" => {
                c.enabled = !recording;
                if !editing {
                    c.text = match c.name.as_str() {
                        "SecsTextBox" => format!("{:.3}", s.secs),
                        "BarsTextBox" => s.bars.to_string(),
                        _ => s.beats.to_string(),
                    };
                }
            }
            "InputGain" => c.value = s.input_gain * 0.25,
            "PreviewVolume" => c.value = s.preview_volume,
            "MonoButton" => c.on = s.channels != 0,
            "ThruButton" => c.on = s.thru,
            "StartThresholdSlider" => c.value = level_to_value(s.start_threshold),
            "StopThresholdSlider" => c.value = level_to_value(s.stop_threshold),
            "VUMeterLeft" => c.value = ml.min(1.0),
            "VUMeterRight" => c.value = mr.min(1.0),
            "ThresholdVU" => c.value = ml.max(mr).min(1.0),
            "NameTextBox" if !editing => c.text = name.clone(),
            "TakeList" => {
                c.text = take.clone();
                c.enabled = has_take;
            }
            "ClearAllButton" | "ClearButton" | "CropButton" | "NormalizeButton" | "SaveButton" | "SliceButton" | "SendToPadButton" | "SendAllToPadsButton" => c.enabled = has_take,
            "PreviewButton" => {
                c.enabled = has_take;
                c.on = previewing && has_take;
            }
            "PreviewLoopButton" => c.on = looping,
            "AutoZoomGroup" => {
                for (i, b) in c.children.iter_mut().enumerate() {
                    b.on = i as i64 == zoom;
                }
            }
            "RecordStatus" => c.text = status.clone(),
            _ => {}
        });
        false
    }

    /// Start-up messages ([re:6a1c94], at messaging start): the last
    /// start mode (default Threshold; 0 also maps to Threshold), record
    /// mode, beats sync, seconds, bars / beats, thresholds, channel mode and
    /// thru, re-sent to the sampler.
    pub fn startup_messages(model: &mut Model) -> Vec<Command> {
        let o = crate::options::Options::load();
        let s = &mut model.sampler;
        let start = match o.int("SamplerLastStartMode", 3) {
            0 => 3,
            n => n.clamp(0, 3),
        };
        s.start_mode = start as u8;
        s.beats_mode = o.bool("SamplerLastSyncMode", true);
        s.secs = crate::conv::number(&o.string("SamplerLastSeconds", "1")).unwrap_or(0.0).clamp(0.0, 3600.0);
        s.bars = o.string("SamplerLastBars", "1").trim().parse::<i32>().unwrap_or(0).max(0) as u32;
        s.beats = o.string("SamplerLastBeats", "0").trim().parse::<i32>().unwrap_or(0).max(0) as u32;
        s.start_threshold = o.string("SamplerLastStartThreshold", "0.1").trim().parse().unwrap_or(0.1);
        s.stop_threshold = o.string("SamplerLastStopThreshold", "0.00562").trim().parse().unwrap_or(0.00562);
        s.channels = o.int("SamplerLastMonoMode", 0).clamp(0, 2) as u8;
        s.thru = o.bool("SamplerLastThruMode", false);
        let timed = o.int("SamplerLastTimedMode", 0).clamp(0, 2);
        vec![
            msg(id::SAMPLER_START_MODE, &[], &[start as f32]),
            msg(id::SAMPLER_TIMED_MODE, &[], &[timed as f32]),
            msg(id::SAMPLER_BEATS_MODE, &[], &[s.beats_mode as i32 as f32]),
            msg(id::SAMPLER_RECORD_SECS, &[], &[s.secs]),
            msg(id::SAMPLER_BARS_BEATS, &[], &[s.bars as f32, s.beats as f32]),
            msg(id::SAMPLER_START_THRESHOLD, &[], &[s.start_threshold]),
            msg(id::SAMPLER_STOP_THRESHOLD, &[], &[s.stop_threshold]),
            msg(id::SAMPLER_MONO, &[], &[s.channels as f32]),
            msg(id::SAMPLER_THRU, &[], &[s.thru as i32 as f32]),
        ]
    }

    fn mine(&self, path: &[usize]) -> bool {
        self.host.as_ref().is_some_and(|h| path.starts_with(h))
    }

    fn open(&mut self, app: &mut App, path: &[usize], which: Menu, items: Vec<(String, Option<Request>)>, checked: Option<usize>) {
        let (labels, acts): (Vec<String>, Vec<Option<Request>>) = items.into_iter().unzip();
        app.open_menu(path, labels, checked);
        self.menu = Some((which, acts));
    }

    /// Preview the current take between its markers.
    fn preview(&self, out: &mut Vec<Command>) {
        if let Some(t) = self.take() {
            let (a, b) = t.region();
            out.push(Command::PreviewRegion { data: t.data(), start: a as u32, end: b as u32, looping: self.loop_preview });
        }
    }

    fn stop_preview(&mut self, out: &mut Vec<Command>) {
        if std::mem::take(&mut self.previewing) {
            out.push(msg(id::STOP_PREVIEW, &[], &[]));
        }
    }

    /// The wave area's frame at x (`w` wide) in the current view.
    fn frame_at(&self, x: f32, w: f32) -> u32 {
        let n = self.take().map(Take::frames).unwrap_or(0) as f32;
        (((x / w.max(1.0)) * self.view.1 + self.view.0).clamp(0.0, 1.0) * n) as u32
    }

    fn x_of(&self, f: u32, w: f32) -> f32 {
        let n = self.take().map(Take::frames).unwrap_or(0).max(1) as f32;
        ((f as f32 / n) - self.view.0) / self.view.1.max(1e-9) * w
    }

    fn apply_auto_zoom(&mut self) {
        let Some(t) = self.take() else { return };
        if let Some(v) = crate::zoomslider::auto_zoom(self.zoom_mode, t.frames(), t.start, t.end) {
            self.view = v;
        }
    }

    pub fn handle(&mut self, app: &mut App, model: &mut Model, ev: &UiEvent, out: &mut Vec<Command>) -> bool {
        match ev {
            UiEvent::Mouse { path, overload, kind, x, y } if overload == "SamplerWaveArea" && self.mine(path) => return self.wave_mouse(app, path, *kind, *x, *y, out),
            // v1: the wave forwards the wheel to its zoom slider
            // ([re:1a34bd]), and the sampler's has none, so it does nothing.
            UiEvent::Wheel { path, overload, .. } if overload == "SamplerWaveArea" && self.mine(path) => return true,
            UiEvent::MenuSelected { anchor, index, .. } if self.mine(anchor) => {
                let Some((which, acts)) = self.menu.take() else { return false };
                let s = &mut model.sampler;
                match which {
                    Menu::Source => {
                        s.source = source_of_item(*index);
                        out.push(msg(id::SAMPLER_RECORD_SOURCE, &[], &[s.source as f32]));
                    }
                    Menu::Take => {
                        self.stop_preview(out);
                        self.current = (*index).min(self.takes.len().saturating_sub(1));
                        self.view = (0.0, 1.0);
                        self.apply_auto_zoom();
                    }
                    Menu::Send => match acts.get(*index).copied().flatten() {
                        Some(r) => self.request = Some(r),
                        None if *index + 1 == acts.len() => self.auto_crop = !self.auto_crop,
                        None => {}
                    },
                    Menu::Timeline => {
                        self.timeline_beats = *index == 1;
                        crate::timeline::save_mode("Sampler", self.timeline_beats);
                    }
                }
                return true;
            }
            _ => {}
        }
        let s = &mut model.sampler;
        match ev {
            UiEvent::Clicked { path, name, on, .. } if self.mine(path) => {
                let parent = app.ctrl(&path[..path.len() - 1]).map(|c| c.name.clone()).unwrap_or_default();
                let i = *path.last().unwrap();
                match parent.as_str() {
                    "SyncButtonGroup" => {
                        let beats = i == 0;
                        s.beats_mode = beats;
                        out.push(msg(id::SAMPLER_BEATS_MODE, &[], &[beats as i32 as f32]));
                        save_opt("SamplerLastSyncMode", beats);
                        return true;
                    }
                    // The start mode resets the engine's timing: the record
                    // mode follows it.
                    "TriggerModeButtonGroup" => {
                        s.start_mode = i.min(3) as u8;
                        out.push(msg(id::SAMPLER_START_MODE, &[], &[i.min(3) as f32]));
                        out.push(msg(id::SAMPLER_TIMED_MODE, &[], &[self.record_mode as f32]));
                        save_opt("SamplerLastStartMode", i.min(3));
                        return true;
                    }
                    "TimedModeButtonGroup" => {
                        self.record_mode = i.min(2) as u8;
                        out.push(msg(id::SAMPLER_TIMED_MODE, &[], &[self.record_mode as f32]));
                        save_opt("SamplerLastTimedMode", self.record_mode);
                        return true;
                    }
                    "AutoZoomGroup" => {
                        self.zoom_mode = i as i64;
                        self.apply_auto_zoom();
                        return true;
                    }
                    _ => {}
                }
                match name.as_str() {
                    "ArmButton" => {
                        self.message = None;
                        out.push(msg(id::SAMPLER_TOGGLE_ARMED, &[], &[0.0]));
                    }
                    // Rec / Stop: starts an armed Free recording, stops a
                    // running one.
                    "RecButton" => {
                        self.message = None;
                        out.push(msg(id::SAMPLER_TOGGLE_RECORD, &[], &[0.0]));
                    }
                    "ThruButton" => {
                        s.thru = *on;
                        out.push(msg(id::SAMPLER_THRU, &[], &[*on as i32 as f32]));
                        save_opt("SamplerLastThruMode", *on);
                    }
                    // Mono: the left channel only.
                    "MonoButton" => {
                        s.channels = *on as u8;
                        s.state = 0;
                        out.push(msg(id::SAMPLER_MONO, &[], &[s.channels as f32]));
                        save_opt("SamplerLastMonoMode", s.channels);
                    }
                    "SourceList" => {
                        let cur = item_of_source(s.source);
                        self.open(app, path, Menu::Source, source_items().into_iter().map(|x| (x, None)).collect(), Some(cur));
                    }
                    "TakeList" if !self.takes.is_empty() => {
                        let items = self.takes.iter().map(|t| (t.label(), None)).collect();
                        self.open(app, path, Menu::Take, items, Some(self.current));
                    }
                    "SaveButton" if !self.takes.is_empty() => self.open(app, path, Menu::Send, send_menu(self.auto_crop), None),
                    "ClearButton" if !self.takes.is_empty() => {
                        self.stop_preview(out);
                        out.push(msg(id::SAMPLER_CLEAR_TAKE, &[], &[self.current as f32]));
                        self.takes.remove(self.current);
                        self.current = self.current.min(self.takes.len().saturating_sub(1));
                        self.view = (0.0, 1.0);
                    }
                    "ClearAllButton" => {
                        self.stop_preview(out);
                        out.push(msg(id::SAMPLER_CLEAR_TAKES, &[], &[]));
                        self.takes.clear();
                        self.current = 0;
                    }
                    "PreviewButton" => {
                        if *on && self.take().is_some() {
                            self.previewing = true;
                            self.preview(out);
                        } else {
                            self.previewing = true;
                            self.stop_preview(out);
                        }
                    }
                    "PreviewLoopButton" => {
                        self.loop_preview = *on;
                        if self.previewing {
                            out.push(msg(id::SET_PREVIEW_LOOP_MODE, &[], &[*on as i32 as f32]));
                        }
                    }
                    "CropButton" => {
                        if let Some(t) = self.takes.get_mut(self.current) {
                            t.crop();
                            self.view = (0.0, 1.0);
                        }
                    }
                    "NormalizeButton" => {
                        if let Some(t) = self.takes.get_mut(self.current) {
                            t.normalize();
                        }
                    }
                    "SliceButton" if !self.takes.is_empty() => self.request = Some(Request::Slice),
                    "SendToPadButton" if !self.takes.is_empty() => self.request = Some(Request::SendToPad),
                    "SendAllToPadsButton" if !self.takes.is_empty() => self.request = Some(Request::SendAllToPads),
                    "ZoomInButton" | "ZoomOutButton" => {
                        let (Some(n), Some(w)) = (self.take().map(Take::frames), self.wave.as_ref().and_then(|p| app.ctrl(p)).map(|c| c.rect.w as f32)) else { return true };
                        self.view = crate::zoomslider::zoom_in(self.view, if name == "ZoomInButton" { 2.0 } else { 0.5 }, 0.5, w, n);
                    }
                    _ => return false,
                }
                true
            }
            // Control event [re:7941cb] 8 (double click): In
            // 1.0, Thru 0.7937, Start 0.1, Stop 0.00562.
            UiEvent::DoubleClicked { path, name } if self.mine(path) => {
                match name.as_str() {
                    "InputGain" => {
                        s.input_gain = 1.0;
                        out.push(msg(id::SAMPLER_INPUT_GAIN, &[], &[1.0]));
                    }
                    "PreviewVolume" => {
                        s.preview_volume = 0.793_700_5;
                        out.push(msg(SET_PREVIEW_VOLUME, &[], &[0.793_700_5]));
                    }
                    "StartThresholdSlider" => set_thresholds(s, Some(0.1), None, out),
                    "StopThresholdSlider" => set_thresholds(s, None, Some(0.00562), out),
                    _ => return false,
                }
                true
            }
            UiEvent::ValueChanged { path, name, value, .. } if self.mine(path) => {
                match name.as_str() {
                    "InputGain" => {
                        s.input_gain = value * 4.0;
                        out.push(msg(id::SAMPLER_INPUT_GAIN, &[], &[s.input_gain]));
                    }
                    "PreviewVolume" => {
                        s.preview_volume = *value;
                        out.push(msg(SET_PREVIEW_VOLUME, &[], &[*value]));
                    }
                    "StartThresholdSlider" => set_thresholds(s, Some(value_to_level(*value)), None, out),
                    "StopThresholdSlider" => set_thresholds(s, None, Some(value_to_level(*value)), out),
                    _ => return false,
                }
                true
            }
            UiEvent::TextCommitted { path, name, text } if self.mine(path) => {
                match name.as_str() {
                    "SecsTextBox" => {
                        let v = crate::conv::number(text).unwrap_or(s.secs).clamp(0.0, 3600.0);
                        s.secs = v;
                        out.push(msg(id::SAMPLER_RECORD_SECS, &[], &[v]));
                        save_opt("SamplerLastSeconds", format!("{v:.3}"));
                    }
                    "BarsTextBox" | "BeatsTextBox" => {
                        let n = text.trim().parse::<i32>().unwrap_or(0).max(0) as u32;
                        if name == "BarsTextBox" {
                            s.bars = n.min(16)
                        } else {
                            s.beats = n
                        }
                        out.push(msg(id::SAMPLER_BARS_BEATS, &[], &[s.bars as f32, s.beats as f32]));
                        save_opt("SamplerLastBars", s.bars);
                        save_opt("SamplerLastBeats", s.beats);
                    }
                    "NameTextBox" => self.name = text.trim().to_string(),
                    _ => return false,
                }
                true
            }
            _ => false,
        }
    }

    /// The wave area: a click on the ruler previews between the markers
    /// (a right click offers Time / Beats), the markers drag left / right,
    /// and a drag on the wave zooms (up / down) and scrolls (left / right).
    fn wave_mouse(&mut self, app: &mut App, path: &[usize], kind: MouseKind, x: f32, y: f32, out: &mut Vec<Command>) -> bool {
        let Some(w) = app.ctrl(path).map(|c| c.rect.w as f32) else { return false };
        if kind == MouseKind::RightDown && y < RULER {
            let items = ["Time", "Beats"].iter().enumerate().map(|(i, t)| (format!("{}{t}", if (i == 1) == self.timeline_beats { dreist_gui::menu::CHECK } else { "" }), None)).collect();
            self.open(app, path, Menu::Timeline, items, None);
            return true;
        }
        let Some(t) = self.take() else { return false };
        let (start, end) = (t.start, t.end);
        match kind {
            MouseKind::Down if y < RULER => {
                self.previewing = true;
                self.preview(out);
                true
            }
            MouseKind::Down => {
                let near = |f: u32| (self.x_of(f, w) - x).abs();
                let (ds, de) = (near(start), near(end));
                self.drag = if y < WAVE_TOP || ds.min(de) < 4.0 { Some(Drag::Marker(ds <= de)) } else { Some(Drag::View(x, y, self.view)) };
                true
            }
            MouseKind::Drag => match self.drag {
                Some(Drag::Marker(is_start)) => {
                    let f = self.frame_at(x, w);
                    if let Some(t) = self.takes.get_mut(self.current) {
                        if is_start {
                            t.start = f.min(t.end.saturating_sub(1));
                        } else {
                            t.end = f.max(t.start + 1).min(t.frames());
                        }
                    }
                    true
                }
                Some(Drag::View(x0, y0, v)) => {
                    let n = self.take().map(Take::frames).unwrap_or(1);
                    let (zs, ss) = crate::zoomslider::drag_signs();
                    let factor = (-(y - y0) * zs / 100.0).exp2();
                    let centre = (x0 / w.max(1.0)).clamp(0.0, 1.0);
                    let zoomed = crate::zoomslider::zoom_in(v, factor, centre, w, n);
                    self.view = crate::zoomslider::clamp((zoomed.0 + ss * (x - x0) / w.max(1.0) * zoomed.1, zoomed.1));
                    true
                }
                None => false,
            },
            MouseKind::Up => {
                if matches!(self.drag.take(), Some(Drag::Marker(_))) {
                    self.apply_auto_zoom();
                }
                true
            }
            _ => false,
        }
    }

    /// The take display: ruler and markers, then the current take (or the
    /// take being recorded) between the start / end markers.
    pub fn draw(&self, model: &Model, r: &Renderer, p: &mut Painter, c: &Ctrl) -> bool {
        if c.overload != "SamplerWaveArea" {
            return false;
        }
        let (w, h) = (c.rect.w as f32, c.rect.h as f32);
        let ink = wave(r, "colLine", Colour::rgba(18, 25, 60, 255));
        let area = RectF::new(0.0, WAVE_TOP, w, h - WAVE_TOP);
        p.fill_rect(RectF::new(0.0, 0.0, w, h), wave(r, "colFill", Colour::rgba(178, 170, 158, 255)));
        let live = model.sampler.state == 2 || self.takes.is_empty();
        if let (true, Some(sm)) = (live, model.sampling.as_ref()) {
            // Recording: the whole take so far fills the display.
            let sr = 44100.0;
            draw_ruler(r, p, w, 0.0, (sm.frames as f64 / sr).max(1.0), self.timeline_beats, model.tempo());
            draw_wave(p, area, &sm.buckets, 1, 0.0, sm.buckets.len() as f64, ink, (0.0, w));
        } else if let Some(t) = self.take() {
            let n = t.frames().max(1) as f64;
            let (v0, v1) = (self.view.0 as f64 * n, (self.view.0 + self.view.1) as f64 * n);
            let sr = t.sample_rate as f64;
            draw_ruler(r, p, w, v0 / sr, (v1 - v0) / sr, self.timeline_beats, model.tempo());
            // Outside the markers the wave is dimmed.
            let (xs, xe) = (self.x_of(t.start, w), self.x_of(t.end, w));
            let (a, b) = (xs.clamp(0.0, w).floor(), xe.clamp(0.0, w).ceil());
            let dim = wave(r, "colLineDisabled", ink.with_alpha(0.4));
            draw_wave(p, area, &t.buckets, BUCKET, v0, v1, dim, (0.0, a));
            draw_wave(p, area, &t.buckets, BUCKET, v0, v1, ink, (a, b));
            draw_wave(p, area, &t.buckets, BUCKET, v0, v1, dim, (b, w));
            for (x, name, start) in [(xs, "WaveEditorStartMarkerColour", true), (xe, "WaveEditorEndMarkerColour", false)] {
                if !(-6.0..=w + 6.0).contains(&x) {
                    continue;
                }
                let col = colour(r, name, Colour::rgba(250, 121, 25, 255));
                let x = x.floor() + 0.5;
                p.line(x, RULER, x, h, col, 1.0);
                let tri = if start { [(x, RULER + 1.0), (x + 9.0, RULER + 1.0), (x, WAVE_TOP - 2.0)] } else { [(x - 9.0, RULER + 1.0), (x, RULER + 1.0), (x, WAVE_TOP - 2.0)] };
                p.fill_polygon(&tri, col);
            }
        } else {
            draw_ruler(r, p, w, 0.0, 1.0, self.timeline_beats, model.tempo());
        }
        p.stroke_rect(RectF::new(0.5, 0.5, w - 1.0, h - 1.0), wave(r, "colTimelineMarkers", ink), 1.0);
        true
    }
}

/// `SetStartThreshold02` [re:19df34]: a start below the stop threshold
/// drags the stop threshold down with it, a stop above the start drags the
/// start up.
fn set_thresholds(s: &mut crate::model::SamplerView, start: Option<f32>, stop: Option<f32>, out: &mut Vec<Command>) {
    if let Some(l) = start {
        s.start_threshold = l;
        out.push(msg(id::SAMPLER_START_THRESHOLD, &[], &[l]));
        if l < s.stop_threshold {
            s.stop_threshold = l;
            out.push(msg(id::SAMPLER_STOP_THRESHOLD, &[], &[l]));
        }
    }
    if let Some(l) = stop {
        s.stop_threshold = l;
        out.push(msg(id::SAMPLER_STOP_THRESHOLD, &[], &[l]));
        if l > s.start_threshold {
            s.start_threshold = l;
            out.push(msg(id::SAMPLER_START_THRESHOLD, &[], &[l]));
        }
    }
    save_opt("SamplerLastStartThreshold", s.start_threshold);
    save_opt("SamplerLastStopThreshold", s.stop_threshold);
}

/// Saves the sampler's `SamplerLast*` keys.
fn save_opt(key: &str, v: impl ToString) {
    let mut o = crate::options::Options::load();
    o.set(key, v);
    if !cfg!(test) {
        o.save();
    }
}

/// Whether a pad holds no sample (and is not locked).
fn empty_pad(model: &Model, e: usize, p: usize) -> bool {
    !model.pad_node(e, p).is_some_and(|n| n.locked()) && (0..LAYERS).all(|l| !model.layer(e, p, l).valid)
}

/// A layer that can take a sample: none loaded, not locked.
/// Some layer plays file `f`.
pub(crate) fn layers_use(model: &Model, f: &std::path::Path) -> bool {
    (0..crate::model::ENGINES).any(|e| (0..64).any(|p| (0..LAYERS).any(|l| model.layer_sample(e, p, l).is_some_and(|x| x == f))))
}

fn free_layer(model: &Model, e: usize, p: usize, l: usize) -> bool {
    !model.layer(e, p, l).valid && !model.layer_node(e, p, l).is_some_and(|n| n.locked())
}

impl crate::Editor {
    /// A file for take `i`: the save dialog, or (without one) a free name
    /// in the record folder.
    /// A free file for take `i` in the record folder (no dialog: a dragged
    /// take).
    pub(crate) fn sampler_record_file(&self, i: usize) -> Option<std::path::PathBuf> {
        free_filename(&record_dir(&self.modals.options), &self.sampler.file_stem(i))
    }

    fn sampler_file(&mut self, i: usize) -> Option<std::path::PathBuf> {
        let stem = self.sampler.file_stem(i);
        if self.dialogs.is_some() {
            return self.dialog(crate::Dialog::SaveSample(format!("{stem}.wav"))).map(|p| if p.extension().is_none() { p.with_extension("wav") } else { p });
        }
        let dir = record_dir(&self.modals.options);
        free_filename(&dir, &stem)
    }

    /// A folder for all the takes: the folder dialog, or the record folder.
    fn sampler_folder(&mut self) -> Option<std::path::PathBuf> {
        if self.dialogs.is_some() { self.dialog(crate::Dialog::ExportFolder) } else { Some(record_dir(&self.modals.options)) }
    }

    /// Save takes `which` into `dir` ("<Name> <Take>.wav"); the files.
    fn save_takes(&mut self, dir: &std::path::Path, which: &[usize]) -> Vec<std::path::PathBuf> {
        let crop = self.sampler.auto_crop;
        which
            .iter()
            .filter_map(|&i| {
                let f = free_filename(dir, &self.sampler.file_stem(i))?;
                self.sampler.takes.get(i)?.save(&f, crop).ok()?;
                Some(f)
            })
            .collect()
    }

    /// Run the sampler page's request; whether the editor changed.
    pub(crate) fn run_sampler_request(&mut self, r: Request) -> bool {
        self.run_sampler_request_with(r, None)
    }

    /// [re:96dd0c]: a chosen file some layer already plays asks first; a
    /// yes runs the request with it.
    pub(crate) fn sampler_overwrite_menu(&mut self, ev: &dreist_gui::app::UiEvent) -> bool {
        let dreist_gui::app::UiEvent::MenuSelected { anchor, index, .. } = ev else { return false };
        if self.sampler_overwrite.as_ref().is_none_or(|(a, _, _)| a != anchor) {
            return false;
        }
        let Some((_, r, f)) = self.sampler_overwrite.take() else { return false };
        *index == 2 && self.run_sampler_request_with(r, Some(f))
    }

    fn run_sampler_request_with(&mut self, r: Request, chosen: Option<std::path::PathBuf>) -> bool {
        let e = self.model.engine;
        let (first, count) = self.model.visible_pads();
        let all: Vec<usize> = (0..self.sampler.takes.len()).collect();
        let cur = self.sampler.current;
        let crop = self.sampler.auto_crop;
        let mut loaded = false;
        match r {
            Request::ExportTake | Request::Slice | Request::SendToPad => {
                let confirmed = chosen.is_some();
                let Some(f) = chosen.or_else(|| self.sampler_file(cur)) else { return false };
                if !confirmed && layers_use(&self.model, &f) {
                    let anchor = self.sampler.path_in(&self.app, "SaveButton").unwrap_or_default();
                    let text = "The file you're about to overwrite is already assigned to layers - if you proceed, layers referencing the old sample will reference this new version.  Do you wish to proceed?";
                    self.app.open_menu(&anchor, vec![text.into(), String::new(), "Yes".into(), "No".into()], None);
                    self.sampler_overwrite = Some((anchor, r, f));
                    return true;
                }
                if self.sampler.take().is_none_or(|t| t.save(&f, crop).is_err()) {
                    self.sampler.message = Some(format!("Could not save {}", f.display()));
                    return true;
                }
                // A file layers play, overwritten after the question: they
                // read it again.
                if confirmed {
                    self.sample_written(&f);
                    self.outbox.push(crate::command::Command::ReloadSamples(vec![f.clone()]));
                    self.outbox.push(self.model.rebuild_command());
                }
                match r {
                    Request::Slice => {
                        self.open_slicer(&f);
                    }
                    // [re:3fbef6] → [re:3c4465]: the take goes to the
                    // current pad's first layer (reset first with "Reset
                    // layer settings on load"); its other layers stay.
                    Request::SendToPad => {
                        let p = self.model.pad[e];
                        loaded = crate::browser::load_sample_into_layer(&mut self.model, e, p, 0, &f);
                    }
                    _ => self.sampler.message = Some(format!("Saved {}", f.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default())),
                }
            }
            Request::ExportAll => {
                let Some(dir) = self.sampler_folder() else { return false };
                let n = self.save_takes(&dir, &all).len();
                self.sampler.message = Some(format!("Saved {n} takes"));
            }
            Request::SendAllToPads => {
                let empty: Vec<usize> = (first..first + count).filter(|&p| empty_pad(&self.model, e, p)).collect();
                if empty.len() < all.len() {
                    self.sampler.message = Some("Not enough empty pads: free up more pads or clear takes".into());
                    return true;
                }
                let Some(dir) = self.sampler_folder() else { return false };
                for (f, p) in self.save_takes(&dir, &all).into_iter().zip(empty) {
                    loaded |= crate::browser::load_sample_into_layer(&mut self.model, e, p, 0, &f);
                }
            }
            Request::SendToFreeLayers | Request::SendToEmptyPadLayers => {
                let slots: Vec<(usize, usize)> = if r == Request::SendToFreeLayers {
                    let p = self.model.pad[e];
                    (0..LAYERS).filter(|&l| free_layer(&self.model, e, p, l)).map(|l| (p, l)).collect()
                } else {
                    (first..first + count).filter(|&p| empty_pad(&self.model, e, p)).flat_map(|p| (0..LAYERS).map(move |l| (p, l))).collect()
                };
                let Some(dir) = self.sampler_folder() else { return false };
                for (f, (p, l)) in self.save_takes(&dir, &all).into_iter().zip(slots) {
                    loaded |= crate::browser::load_sample_into_layer(&mut self.model, e, p, l, &f);
                }
            }
        }
        if loaded {
            self.outbox.push(self.model.rebuild_command());
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn free_filenames_count_up() {
        crate::app::test_setup();
        let dir = std::env::temp_dir().join(format!("dreist-free-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let a = free_filename(&dir, "x").unwrap();
        assert!(a.ends_with("x.wav"));
        std::fs::write(&a, b"").unwrap();
        assert!(free_filename(&dir, "x").unwrap().ends_with("x (2).wav"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn meter_mapping_round_trips() {
        crate::app::test_setup();
        for l in [0.001f32, 0.0056, 0.1, 0.5, 1.0] {
            assert!((value_to_level(level_to_value(l)) - l).abs() < 1e-4 * l.max(1e-3), "{l}");
        }
        assert_eq!(level_to_value(0.0), 0.0);
        // 0 dBFS sits at 0.8 of the travel.
        assert!((level_to_value(1.0) - 0.8).abs() < 1e-6);
    }

    #[test]
    fn takes_crop_and_normalize_between_markers() {
        crate::app::test_setup();
        let l: Vec<f32> = (0..1000).map(|i| if i < 500 { 0.1 } else { 0.5 }).collect();
        let mut t = Take::new(&l, &l, 44100.0, 1);
        t.start = 100;
        t.end = 400;
        t.normalize();
        assert!((t.left[200] - 1.0).abs() < 1e-6 && (t.left[600] - 0.5).abs() < 1e-6, "only the marked region");
        t.crop();
        assert_eq!((t.frames(), t.start, t.end), (300, 0, 300));
    }
}
