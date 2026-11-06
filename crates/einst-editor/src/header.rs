//! The front panel's engine selectors, transport and master section
//! ([re:6e0e89], [re:4139f8] in the original's header bar; Einst's v1
//! layout has them under the LCD: `EditorEngineSelectors`, `Transport`,
//! `MasterControls`).
//!
//! Each `EngineSelector<n>` ([re:4fd51c]) shows its number on
//! `EngineSelectionButton`; selecting one ([re:a570ec]) posts
//! `e_selectEngine` (0x41). Mute/Solo/PatternPower post 0x8c/0x8d/0xa8 for
//! that engine ([re:c7d50e]); with pattern power off the button shows its
//! power-off art.
//!
//! Transport and master controls ([re:46c9b4] callbacks):
//! - MasterSlider ([re:3f9657]) posts `e_setMasterChannelGain` 0xa5; a
//!   double click ([re:dd3107]) resets it to 0 dB (0.7937) when the
//!   option `DoubleClickResets` (default on) is set, else opens a text edit
//!   in dB ("Master Level", decibel text). It follows
//!   `e_updateMasterGain` 0x192.
//! - PanicButton ([re:f41aad]) `e_panic` 0x12; the engine answers with the
//!   play and record buttons off (0x1d4 / 0x1d5).
//! - SongButton ([re:e49218]) `e_songSetActive` 0x117 [on, false];
//!   LoopButton ([re:e3d5c1]) `e_songSetInternalLoopActive` 0x120 [on];
//!   RestartButton ([re:7dd0f4]) `e_returnToZero` 0x1d.
//! - `e_songStateChanged` 0x1f6 / learn state 0x132: Song on = song
//!   active, Loop on = internal loop on; Restart is enabled in learn mode or
//!   with the song active, Loop in learn mode or with the song active and
//!   no host loop (the song sequencer's state queries, [re:3a88f8]: active,
//!   internal loop enabled, host loop enabled).
//! - The transport options ([re:d718a1]; the main menu's "Transport
//!   options" submenu here): check items for the options
//!   `PatternEditorFollow` (default off; editor message 0x18c),
//!   `PadSelectionFollowsMIDI` (on) and `EngineSelectionFollowsMIDI` (off)
//!   (both posted as `e_registryFollowOptionsChanged` 0x2 [pad, engine]);
//!   the pre-roll length `PrerollTime` ("PrerollOff", "Preroll1Bar",
//!   "Preroll2Bars", "Preroll4Bars"; 0x27 [0..3]); "Only pre-roll when
//!   recording" (`PrerollOnlyOnRecord`, on; 0x28), "Metronome only when
//!   recording" (stored under the same key, read with default off; 0x2a)
//!   and "Wait for MIDI input when recording" (`WaitForMIDI`, off; 0x29);
//!   the metronome volume slider (0x24) and the "Metronome output..." sub
//!   menu (Master and Sub output 1–14, output locations 1..=15; 0x25), also on
//!   a right click on the Metro button.
//! - MetroButton follows the document's `Metronome Mute`.
//! - Indicators (timer tick [re:8ae44a]): MIDIIndi lights for 10 ticks when a
//!   block carried MIDI, VoiceIndi for 10 ticks
//!   after `e_updateVoiceStolen` 0x18a.

use dreist_engine::binding::id;
use dreist_gui::app::{App, UiEvent};
use dreist_gui::{Ctrl, Rect};

use crate::command::{Command, msg};
use crate::model::{ENGINES, Model};

pub struct Header {
    /// The front-panel sections this panel owns: their button clicks come
    /// from under them (the pattern toolbar and the sampler reuse
    /// RestartButton / QuantizeButton / RecButton).
    scopes: Vec<Vec<usize>>,
    pub(crate) selectors: Option<Vec<usize>>,
    /// The anchor of the open transport options menu and its level (1
    /// options, 2 the metronome output sub menu).
    options: Option<Vec<usize>>,
    menu: u8,
    /// Indicator countdowns in timer ticks ([re:f5d1f8] MIDI,
    /// voices).
    midi_ticks: u32,
    voice_ticks: u32,
    /// The controls `sync` sets.
    named: dreist_gui::app::PathCache,
    /// A tempo box drag ([re:95a5c7]).
    tempo_drag: Option<TempoDrag>,
}

/// [re:863166]: a press on the tempo box's whole-number digits drags the
/// whole number (20..1000 over 3000 px, 5000 with shift), elsewhere the
/// decimals (0..0.99 over 200 px, 2000 with shift), keeping the other part.
#[derive(Clone, Copy, Debug, PartialEq)]
struct TempoDrag {
    y: f32,
    whole: bool,
    acc: f32,
    tempo: f32,
}

impl TempoDrag {
    fn start(tempo: f32, y: f32, whole: bool) -> TempoDrag {
        let acc = if whole { (tempo.floor() - 20.0) / 980.0 } else { tempo.fract() / 0.99 };
        TempoDrag { y, whole, acc: acc.clamp(0.0, 1.0), tempo }
    }

    /// The tempo after a move to `y`.
    fn drag(&mut self, y: f32, shift: bool) -> f32 {
        let span = match (self.whole, shift) {
            (true, false) => -3000.0,
            (true, true) => -5000.0,
            (false, false) => -200.0,
            (false, true) => -2000.0,
        };
        self.acc = ((y - self.y).round() / span + self.acc).clamp(0.0, 1.0);
        self.y = y;
        self.tempo = if self.whole { (self.acc * 980.0 + 20.0).floor() + self.tempo.fract() } else { self.tempo.floor() + self.acc * 0.99 };
        self.tempo
    }
}

/// Indicator hold (10 ticks of the 1/30 s timer).
const INDICATOR_TICKS: u32 = 10;
const GAIN_EDIT: &str = "MasterGainEdit";

/// The master gain knob.
pub(crate) const MASTER_GAIN: &str = "MasterSlider";

/// [re:b4c931]: the learn binding of a
/// header control: (path, name, type, message, value slot or none).
pub(crate) fn binding_for(name: &str) -> Option<(&'static str, &'static str, &'static str, u32)> {
    Some(match name {
        MASTER_GAIN => ("Global", "Gain", "Continuous", id::SET_MASTER_CHANNEL_GAIN),
        "PlayButton" => ("Transport", "Play", "Gated", id::TOGGLE_PLAYBACK),
        "RecButton" => ("Transport", "Record", "Gated", id::TOGGLE_RECORD),
        "MetroButton" => ("Transport", "Metronome", "Gated", id::TOGGLE_METRONOME),
        "QuantizeButton" => ("Transport", "Quantize", "Gated", id::TOGGLE_QUANTIZE),
        "PanicButton" => ("Transport", "Panic", "Continuous", id::PANIC),
        "RetroButton" => ("Transport", "Retro", "Trigger", id::GRAB_RETRO_RECORDING),
        "SongButton" => ("Transport", "Song", "Continuous", id::SONG_SET_ACTIVE),
        "LoopButton" => ("Transport", "Loop", "Gated", id::SONG_SET_INTERNAL_LOOP_ACTIVE),
        "RestartButton" => ("Transport", "Restart", "Continuous", id::RETURN_TO_ZERO),
        "MetroVol" => ("Transport", "Metronome Volume", "Continuous", id::SET_METRONOME_LEVEL),
        "BPMTextBox" => ("Transport", "Tap tempo", "Gated", id::TAP_TEMPO),
        _ => return None,
    })
}

fn song_of(model: &Model) -> dreist_engine::song::Song {
    model.doc.children.iter().find(|c| c.class == "SongSequencer").map(dreist_engine::song::Song::from_node).unwrap_or_default()
}

fn metronome(model: &Model) -> Option<&dreist_formats::Node> {
    model.doc.children.iter().find(|c| c.class == "Metronome")
}

fn metronome_mut(model: &mut Model) -> &mut dreist_formats::Node {
    model.doc.child_mut_or_insert("Metronome", "Metronome")
}

/// The metronome's `Gain` (default ∛0.5, [re:7fe011]).
fn metronome_gain(model: &Model) -> f32 {
    metronome(model).and_then(|n| n.f32("Gain")).unwrap_or(0.5f32.cbrt())
}

/// The metronome state is not an undo step (the header changes it
/// directly): keep `from`'s Metronome node in `to`.
pub fn carry_metronome(from: &dreist_formats::Node, to: &mut dreist_formats::Node) {
    let Some(m) = from.children.iter().find(|c| c.class == "Metronome") else { return };
    match to.children.iter_mut().find(|c| c.class == "Metronome") {
        Some(t) => *t = m.clone(),
        None => to.children.push(m.clone()),
    }
}

/// `PrerollTime` option values ([re:105287]).
pub const PREROLL_NAMES: [&str; 4] = ["PrerollOff", "Preroll1Bar", "Preroll2Bars", "Preroll4Bars"];
const PREROLL_ITEMS: [&str; 4] = ["No pre-roll", "1 bar", "2 bars", "4 bars"];

/// [re:0c66d0]: unknown → off.
pub fn preroll_index(name: &str) -> usize {
    PREROLL_NAMES.iter().position(|n| *n == name).unwrap_or(0)
}

/// Options menu items (order as the original adds them; the slider is item 13, the
/// sub menu 14).
fn options_items(model: &Model) -> Vec<String> {
    let o = crate::options::Options::load();
    let check = |on: bool, t: &str| if on { format!("{}{t}", dreist_gui::menu::CHECK) } else { t.to_string() };
    let preroll = preroll_index(&o.string("PrerollTime", PREROLL_NAMES[0]));
    let mut items = vec![
        check(o.bool("PatternEditorFollow", false), "Follow playback cursor"),
        check(o.bool("PadSelectionFollowsMIDI", true), "Pad selection follows MIDI"),
        check(o.bool("EngineSelectionFollowsMIDI", false), "Engine selection follows MIDI"),
        String::new(),
    ];
    items.extend(PREROLL_ITEMS.iter().enumerate().map(|(i, t)| check(i == preroll, t)));
    items.push(String::new());
    items.push(check(o.bool("PrerollOnlyOnRecord", true), "Only pre-roll when recording"));
    items.push(check(o.bool("PrerollOnlyOnRecord", false), "Metronome only when recording"));
    items.push(check(o.bool("WaitForMIDI", false), "Wait for MIDI input when recording"));
    items.push(String::new());
    items.push(dreist_gui::menu::slider_item(metronome_gain(model)));
    items.push("Metronome output... ▸".into());
    items
}

/// The metronome output menu as v1 lists it ([re:8dc275]): output
/// locations 1..=15 (Master, Sub output 1..14) by long name.
fn output_items(model: &Model) -> Vec<String> {
    let cur = metronome(model).and_then(|n| n.attr("OutputLocation")).map(dreist_engine::Route::from_name).unwrap_or(dreist_engine::Route::Master);
    (1..=15)
        .map(|i| {
            let r = dreist_engine::Route::from_index(i);
            let name = r.name();
            if r == cur { format!("{}{name}", dreist_gui::menu::CHECK) } else { name }
        })
        .collect()
}

fn master_gain(model: &Model) -> f32 {
    model.doc.children.iter().find(|c| c.class == "MasterChannel").and_then(|n| n.f32("Gain")).unwrap_or(dreist_formats::params::UNITY_GAIN)
}

/// Find the engine selector index and child name for a click path.
pub(crate) fn selector_hit(app: &App, base: &[usize], path: &[usize]) -> Option<(usize, String)> {
    if path.len() < base.len() + 2 || !path.starts_with(base) {
        return None;
    }
    let e = path[base.len()];
    let name = app.ctrl(path)?.name.clone();
    Some((e, name))
}

impl Header {
    pub fn build(root: &Ctrl) -> Header {
        let scopes = ["EditorEngineSelectors", "Transport", "MasterControls", "UtilityControls"].iter().filter_map(|n| root.path_of(n)).collect();
        Header { scopes, selectors: root.path_of("EditorEngineSelectors"), options: None, menu: 0, midi_ticks: 0, voice_ticks: 0, named: Default::default(), tempo_drag: None }
    }

    /// Open the transport options menu at `anchor` (the main menu's
    /// "Transport options" item).
    pub fn open_options(&mut self, app: &mut App, model: &Model, anchor: &[usize]) {
        self.options = Some(anchor.to_vec());
        self.menu = 1;
        app.open_menu(anchor, options_items(model), None);
    }

    pub fn sync(&self, app: &mut App, model: &Model, learn: bool) {
        if let Some(base) = &self.selectors {
            if let Some(c) = app.ctrl_mut(base) {
                for s in c.children.iter_mut() {
                    let Some(e) = s.attr("engine").and_then(|v| v.parse::<usize>().ok()) else { continue };
                    let sel = e == model.engine;
                    let (m, so) = (model.engine_mute(e), model.engine_solo(e));
                    let power = model.pattern_power(e);
                    for k in s.children.iter_mut() {
                        match k.name.as_str() {
                            "EngineSelectionButton" => {
                                k.on = sel;
                                k.style = if power { format!("EngineButtonStyle{}", e + 1) } else { format!("EngineButtonPowerOffStyle{}", e + 1) };
                            }
                            "EngineMutedIcon" => k.on = m,
                            "EngineSoloedIcon" => k.on = so,
                            "MuteButton" => k.on = m,
                            "SoloButton" => k.on = so,
                            "PatternPower" => k.on = power,
                            _ => {}
                        }
                    }
                }
            }
        }
        let tempo = model.tempo();
        let named = &self.named;
        let scopes = &self.scopes;
        // The first control called `name` in this panel's sections.
        let find = |app: &App, name: &'static str| scopes.iter().find_map(|s| named.path_in(app, s, name));
        let set_text = |app: &mut App, name: &'static str, text: String| {
            if let Some(c) = find(app, name).and_then(|p| app.ctrl_mut(&p)) {
                c.text = text;
            }
        };
        set_text(app, "BPMTextBox", format!("{tempo:.2}"));
        // e_updateMemoryUsage ([re:91925c]): the engine's sample memory.
        set_text(app, "MemoryUsageBox", model.memory.map(dreist_engine::memory::bytes_label).unwrap_or_default());
        let song = song_of(model);
        let gain = master_gain(model);
        let host_loop = model.host_loop;
        let mut set = |name: &'static str, f: &dyn Fn(&mut Ctrl)| {
            if let Some(c) = find(app, name).and_then(|p| app.ctrl_mut(&p)) {
                f(c);
            }
        };
        set("PlayButton", &|c| c.on = model.playing);
        let metro_on = !metronome(model).and_then(|n| n.bool("Mute")).unwrap_or(true);
        set("MetroButton", &|c| c.on = metro_on);
        set(MASTER_GAIN, &|c| c.value = gain);
        set("SongButton", &|c| c.on = song.active);
        set("LoopButton", &|c| {
            c.on = song.loop_enabled;
            c.enabled = learn || (song.active && !host_loop);
        });
        set("RestartButton", &|c| c.enabled = learn || song.active);
        let (midi, voice) = (self.midi_ticks > 0, self.voice_ticks > 0);
        set("MIDIIndi", &|c| c.on = midi);
        set("VoiceIndi", &|c| c.on = voice);
    }

    /// An Options menu pick ([re:4139f8]): write the
    /// option and tell the engine / editor.
    fn option_chosen(&mut self, app: &mut App, model: &mut Model, menu: u8, index: usize, out: &mut Vec<Command>) {
        let mut o = crate::options::Options::load();
        if menu == 2 {
            let r = dreist_engine::Route::from_index(index + 1);
            metronome_mut(model).set("OutputLocation", r.name());
            out.push(msg(id::SET_METRONOME_OUTPUT_LOCATION, &[], &[(index + 1) as f32]));
            return;
        }
        let toggle = |o: &mut crate::options::Options, key: &str, default: bool| {
            let v = !o.bool(key, default);
            o.set(key, v);
            v
        };
        match index {
            0 => {
                let v = toggle(&mut o, "PatternEditorFollow", false);
                model.follow_playback = v;
            }
            1 | 2 => {
                if index == 1 {
                    toggle(&mut o, "PadSelectionFollowsMIDI", true);
                } else {
                    toggle(&mut o, "EngineSelectionFollowsMIDI", false);
                }
                let (pad, engine) = (o.bool("PadSelectionFollowsMIDI", true), o.bool("EngineSelectionFollowsMIDI", false));
                out.push(msg(id::REGISTRY_FOLLOW_OPTIONS_CHANGED, &[], &[pad as i32 as f32, engine as i32 as f32]));
            }
            4..=7 => {
                o.set("PrerollTime", PREROLL_NAMES[index - 4]);
                out.push(msg(id::SET_PREROLL_MODE, &[], &[(index - 4) as f32]));
            }
            9 => {
                let v = toggle(&mut o, "PrerollOnlyOnRecord", true);
                out.push(msg(id::SET_PREROLL_ONLY_ON_REC, &[], &[v as i32 as f32]));
            }
            10 => {
                let v = toggle(&mut o, "PrerollOnlyOnRecord", false);
                out.push(msg(id::SET_METRO_ONLY_DURING_REC, &[], &[v as i32 as f32]));
            }
            11 => {
                let v = toggle(&mut o, "WaitForMIDI", false);
                out.push(msg(id::SET_REC_WAITS_FOR_MIDI, &[], &[v as i32 as f32]));
            }
            14 => {
                if let Some(p) = self.options.clone() {
                    self.menu = 2;
                    app.open_menu(&p, output_items(model), None);
                }
                return;
            }
            _ => return,
        }
        o.save();
    }

    /// One timer tick of the indicators from the engine's block flags;
    /// true when one changed.
    pub fn tick_indicators(&mut self, midi: bool, voice_stolen: bool) -> bool {
        let before = (self.midi_ticks > 0, self.voice_ticks > 0);
        let step = |t: &mut u32, hit: bool| {
            if hit {
                *t = INDICATOR_TICKS;
            } else {
                *t = t.saturating_sub(1);
            }
        };
        step(&mut self.midi_ticks, midi);
        step(&mut self.voice_ticks, voice_stolen);
        before != (self.midi_ticks > 0, self.voice_ticks > 0)
    }

    pub fn handle(&mut self, app: &mut App, model: &mut Model, ev: &UiEvent, out: &mut Vec<Command>) -> bool {
        match ev {
            UiEvent::Mouse { path, overload, kind, x, y } if overload == "TempoBox" => {
                use dreist_gui::app::MouseKind;
                match kind {
                    MouseKind::Down => {
                        // The whole-number digits: the text left of its point.
                        let (w, text) = app.ctrl(path).map(|c| (c.rect.w as f32, c.text.clone())).unwrap_or_default();
                        let dot = text.find('.').unwrap_or(text.len()) as f32 / text.len().max(1) as f32;
                        self.tempo_drag = Some(TempoDrag::start(model.tempo() as f32, *y, *x < w * dot));
                    }
                    MouseKind::Drag => {
                        if let Some(d) = self.tempo_drag.as_mut() {
                            let t = d.drag(*y, app.mods.shift);
                            out.push(msg(id::SET_TEMPO, &[], &[t]));
                        }
                    }
                    MouseKind::Up => self.tempo_drag = None,
                    MouseKind::RightDown => {}
                }
                return true;
            }
            // [re:29c1c3]: a right click on the Metro button opens the
            // metronome output menu.
            UiEvent::RightClicked { path, name } if name == "MetroButton" && self.scopes.iter().any(|s| path.starts_with(s)) => {
                self.options = Some(path.clone());
                self.menu = 2;
                app.open_menu(path, output_items(model), None);
                return true;
            }
            UiEvent::ValueChanged { name, value, .. } if name == MASTER_GAIN => {
                out.push(msg(id::SET_MASTER_CHANNEL_GAIN, &[], &[*value]));
                return true;
            }
            UiEvent::DoubleClicked { path, name } if name == MASTER_GAIN => {
                if crate::options::Options::load().bool("DoubleClickResets", true) {
                    if let Some(c) = app.ctrl_mut(path) {
                        c.value = dreist_formats::params::UNITY_GAIN;
                    }
                    out.push(msg(id::SET_MASTER_CHANNEL_GAIN, &[], &[dreist_formats::params::UNITY_GAIN]));
                } else {
                    // [re:c8b896]: a text box over the
                    // slider holding the level in dB.
                    let Some(c) = app.ctrl(path).cloned() else { return false };
                    let rect = Rect::new(c.rect.x + c.rect.w / 2 - 30, c.rect.y + c.rect.h / 2 - 9, 60, 18);
                    app.open_edit_box(&path[..path.len() - 1], GAIN_EDIT, rect, format!("{:.1} dB", dreist_formats::params::value01_to_db(c.value).max(-100.0)));
                }
                return true;
            }
            UiEvent::MenuSlider { anchor, index: 13, value } if self.menu == 1 && self.options.as_ref() == Some(anchor) => {
                metronome_mut(model).set("Gain", dreist_formats::node::format_g16(*value as f64));
                out.push(msg(id::SET_METRONOME_LEVEL, &[], &[*value]));
                return true;
            }
            UiEvent::MenuSelected { anchor, index, .. } if self.menu != 0 && self.options.as_ref() == Some(anchor) => {
                let menu = std::mem::replace(&mut self.menu, 0);
                self.option_chosen(app, model, menu, *index, out);
                return true;
            }
            UiEvent::TextCommitted { path, name, text } if name == GAIN_EDIT => {
                let parent = path[..path.len() - 1].to_vec();
                if let Some(c) = app.ctrl_mut(&parent) {
                    c.children.retain(|k| k.name != GAIN_EDIT);
                }
                let t = text.trim().trim_end_matches("dB").trim().to_ascii_lowercase();
                let db = if t == "-inf" { Some(-1000.0) } else { crate::conv::number(&t) };
                if let Some(db) = db {
                    out.push(msg(id::SET_MASTER_CHANNEL_GAIN, &[], &[dreist_formats::params::db_to_value01(db)]));
                }
                return true;
            }
            _ => {}
        }
        let UiEvent::Clicked { path, name, on, .. } = ev else { return false };
        if !self.scopes.iter().any(|h| path.starts_with(h)) {
            return false;
        }
        if let Some(base) = &self.selectors {
            if let Some((i, child)) = selector_hit(app, base, path) {
                let e = app.ctrl(&[base.as_slice(), &[i]].concat()).and_then(|s| s.attr("engine")).and_then(|v| v.parse::<usize>().ok()).unwrap_or(i).min(ENGINES - 1);
                match child.as_str() {
                    "MuteButton" => out.push(msg(id::MUTE_ENGINE, &[e as i32], &[*on as i32 as f32])),
                    "SoloButton" => out.push(msg(id::SOLO_ENGINE, &[e as i32], &[*on as i32 as f32])),
                    "PatternPower" => out.push(msg(id::SET_PATTERN_POWER_STATE, &[e as i32], &[*on as i32 as f32])),
                    _ => {
                        model.engine = e;
                        model.engine_sel = 1 << e;
                        model.doc.set("CurrentEngine", e);
                        out.push(Command::SelectEngine(e));
                    }
                }
                return true;
            }
        }
        match name.as_str() {
            "PlayButton" => {
                model.playing = *on;
                out.push(Command::Play(*on));
                true
            }
            "RecButton" => {
                out.push(msg(id::TOGGLE_RECORD, &[], &[*on as i32 as f32]));
                false
            }
            "MetroButton" => {
                metronome_mut(model).set("Mute", !*on);
                out.push(msg(id::TOGGLE_METRONOME, &[], &[*on as i32 as f32]));
                false
            }
            "QuantizeButton" => {
                out.push(msg(id::TOGGLE_QUANTIZE, &[], &[*on as i32 as f32]));
                false
            }
            "PanicButton" => {
                out.push(msg(id::PANIC, &[], &[0.0, 0.0, 0.0]));
                // The engine's 0x1d4 / 0x1d5 answers: play and record off.
                model.playing = false;
                for n in ["PlayButton", "RecButton"] {
                    if let Some(c) = crate::front::path_in(&app.root, "Transport", n).and_then(|p| app.ctrl_mut(&p)) {
                        c.on = false;
                    }
                }
                true
            }
            "SongButton" => {
                out.push(msg(id::SONG_SET_ACTIVE, &[], &[*on as i32 as f32, 0.0]));
                true
            }
            "LoopButton" => {
                out.push(msg(id::SONG_SET_INTERNAL_LOOP_ACTIVE, &[], &[*on as i32 as f32]));
                true
            }
            "RestartButton" => {
                out.push(msg(id::RETURN_TO_ZERO, &[], &[0.0, 0.0, 0.0]));
                true
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TempoDrag;

    #[test]
    fn tempo_drag_moves_whole_or_decimals() {
        // Whole number: 3000 px for 20..1000 (30 px: +9.8, floored), the
        // decimals kept.
        let mut d = TempoDrag::start(120.5, 100.0, true);
        let t = d.drag(70.0, false);
        assert!((t - 129.5).abs() < 1e-3, "{t}");
        // Shift: 5000 px.
        let mut d = TempoDrag::start(120.0, 100.0, true);
        assert!((d.drag(50.0, true) - 129.0).abs() < 1e-3);
        // Decimals: 200 px for 0..0.99, the whole number kept.
        let mut d = TempoDrag::start(120.0, 100.0, false);
        let t = d.drag(0.0, false);
        assert!((t - 120.495).abs() < 1e-3, "{t}");
        assert!((d.drag(-1000.0, false) - 120.99).abs() < 1e-3, "clamped");
    }
}
