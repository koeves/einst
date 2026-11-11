//! Einst as a CLAP/VST3 instrument (nih-plug): a copy of Dreist's
//! `dreist-plugin` shell with Einst's names and IDs. The engine is
//! [re:c79b4c]; this crate maps host MIDI, transport, automation
//! parameters and output buses onto it. The editor (`editor.rs`) is
//! `dreist-editor` under Einst's profile (`einst::install`). v1 documents
//! open through `einst_formats` (converted to the v2 tree, see
//! [`load_project`]).

// nih-plug's `nih_export_vst3!` expands to a statement with a trailing
// semicolon in expression position (upstream).
#![allow(semicolon_in_expressions_from_non_local_macros)]

pub mod editor;
pub mod midiclock;
#[cfg(feature = "standalone")]
pub mod standalone;

use std::num::NonZeroU32;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use dreist_engine::Dreist;
use nih_plug::prelude::*;

/// A project file: a the original v1 document (any of `.geist`, `.geistkit`,
/// `.geistpad`, `.geistengine`, `.geistpattern`, `.geistbank`) converted by
/// `einst_formats`, anything else through Dreist's loader.
pub fn load_project(p: &std::path::Path, sr: f32, options: dreist_engine::RenderOptions, cache: &mut dreist_engine::audio::SampleCache) -> Result<Dreist, String> {
    match einst_formats::v1::open_as_project(p) {
        Ok(Some((doc, samples))) => Dreist::from_document(&doc, &samples, sr, options, cache).ok_or_else(|| "the converted v1 document did not build".into()),
        Ok(None) => Dreist::load_cached(p, sr, options, cache).map_err(|e| e.to_string()),
        Err(e) => Err(e.to_string()),
    }
}

/// Stereo sub outputs exposed to the host ("Sub output 1..16").
pub const SUB_PORTS: usize = 16;
/// Host automation parameters routed to [re:0ad0c9] host bindings.
pub const AUTOMATION_PARAMS: usize = 128;

#[derive(Params)]
pub struct AutoParam {
    #[id = "auto"]
    pub value: FloatParam,
}

#[derive(Params)]
pub struct DreistParams {
    /// The loaded `.geist` project (restored with the host session).
    #[persist = "project-path"]
    pub project_path: RwLock<String>,
    /// The whole document as the host saves it with its session
    /// (`effGetChunk`: unsaved edits survive reopening the session).
    #[persist = "session"]
    pub session: RwLock<Option<Session>>,

    #[nested(array, group = "Automation")]
    pub automation: Vec<AutoParam>,
}

/// Host session state: the document (`.geist` XML) and where its relative
/// sample paths resolve.
#[derive(Clone, Debug, PartialEq, dreist_host::serde::Serialize, dreist_host::serde::Deserialize)]
#[serde(crate = "dreist_host::serde")]
pub struct Session {
    pub doc: String,
    pub samples: Option<String>,
}

impl Default for DreistParams {
    fn default() -> Self {
        DreistParams {
            project_path: RwLock::new(String::new()),
            session: RwLock::new(None),
            automation: (0..AUTOMATION_PARAMS).map(|i| AutoParam { value: FloatParam::new(format!("Automation {}", i + 1), 0.0, FloatRange::Linear { min: 0.0, max: 1.0 }) }).collect(),
        }
    }
}

pub struct EinstPlugin {
    params: Arc<DreistParams>,
    engine: Option<Box<Dreist>>,
    /// A swapped-out engine waiting for room in `Bridge::retired` (never
    /// dropped on the audio thread).
    retiring: Option<Box<Dreist>>,
    /// The rebuild request the running engine was built for.
    built: u64,
    replay: Replay,
    /// Steps the engine recorded lately (see `RecentWrites`).
    recent_writes: RecentWrites,
    /// Commands being applied (with their queue tags) / applied and
    /// waiting to go back to the GUI.
    cmds: Vec<(u64, einst_editor::Command)>,
    applied: Vec<einst_editor::Command>,
    last_auto: Vec<f32>,
    sub_scratch: Vec<[Vec<f32>; 2]>,
    input_scratch: [Vec<f32>; 2],
    bridge: Arc<editor::Bridge>,
    live_take: einst_editor::model::LiveTake,
    /// Editor window size shared with the open editor (see `DreistEditor`).
    editor_size: Arc<std::sync::atomic::AtomicU64>,
    /// The standalone's MIDI clock input (sync on).
    #[cfg(feature = "standalone")]
    clock: Option<midiclock::ClockInput>,
}

impl Default for EinstPlugin {
    fn default() -> Self {
        einst::install();
        EinstPlugin {
            params: Arc::new(DreistParams::default()),
            engine: None,
            retiring: None,
            built: 0,
            replay: Replay::default(),
            recent_writes: RecentWrites::default(),
            cmds: Vec::new(),
            applied: Vec::new(),
            last_auto: vec![f32::NAN; AUTOMATION_PARAMS],
            sub_scratch: (0..SUB_PORTS).map(|_| [Vec::new(), Vec::new()]).collect(),
            input_scratch: [Vec::new(), Vec::new()],
            bridge: Arc::new(editor::Bridge::default()),
            live_take: Default::default(),
            editor_size: Default::default(),
            #[cfg(feature = "standalone")]
            clock: None,
        }
    }
}

/// Messages applied while a rebuild is pending, with their queue tags
/// (the number of rebuilds requested when each was queued): rebuild `n`'s
/// document predates the ones tagged `n` or later, so those are replayed
/// onto it; earlier ones are in it already.
#[derive(Default)]
struct Replay(Vec<(u64, dreist_engine::binding::Message)>);

impl Replay {
    /// Keep `m` (tagged `tag`) if a rebuild newer than the running one
    /// (`built`) was requested before it was queued. Bounded by the room
    /// reserved at initialize.
    fn record(&mut self, tag: u64, built: u64, m: &dreist_engine::binding::Message) {
        if tag > built && self.0.len() < self.0.capacity() {
            self.0.push((tag, m.clone()));
        }
    }

    /// Apply to rebuilt engine `n` what its document lacks.
    fn onto(&mut self, n: u64, g: &mut Dreist) {
        self.0.retain(|(t, _)| *t >= n);
        for (_, m) in &self.0 {
            g.apply_message(m);
        }
    }
}

/// Steps the engine recorded (`StepWrite`s, final step data) that a
/// rebuilt engine's document may lack: those written while a rebuild is
/// pending, and the last moments' (the editor may not have taken them in
/// before it asked for the rebuild). Restored into the rebuilt engine.
#[derive(Default)]
struct RecentWrites {
    /// (sample clock when handed over, write).
    writes: Vec<(u64, dreist_engine::StepWrite)>,
    clock: u64,
}

impl RecentWrites {
    /// One block's writes; older than `grace` samples are forgotten while
    /// no rebuild is pending. Bounded by the room reserved at initialize.
    fn record(&mut self, writes: &[dreist_engine::StepWrite], pending: bool, grace: u64) {
        for w in writes {
            if self.writes.len() < self.writes.capacity() {
                self.writes.push((self.clock, w.clone()));
            }
        }
        if !pending {
            let horizon = self.clock.saturating_sub(grace);
            self.writes.retain(|(t, _)| *t >= horizon);
        }
    }

    fn advance(&mut self, frames: usize) {
        self.clock += frames as u64;
    }
}

/// Room reserved in the editor-bound queues (see `initialize`).
const QUEUE: usize = 256;
const EVENT_QUEUE: usize = 4096;
const LIVE_QUEUE: usize = 4096;
const COMMAND_QUEUE: usize = 1024;
const REPLAY_QUEUE: usize = 4096;
const RECENT_WRITES: usize = 1024;

/// Move `src` into `dst` when it fits the room reserved there (no
/// allocation on the audio thread); otherwise the editor is behind (or
/// closed) and `src` is dropped.
fn hand_over<T>(dst: &mut Vec<T>, src: &mut Vec<T>) {
    if dst.capacity() - dst.len() >= src.len() {
        dst.append(src);
    } else {
        src.clear();
    }
}

impl EinstPlugin {
    /// Swap in an engine the editor's rebuild worker built (structural
    /// edits), continuing the running one's state; the old engine goes to
    /// the worker to be freed.
    fn swap_in_rebuilt(&mut self) {
        if let Some(old) = self.retiring.take() {
            match self.bridge.retired.try_lock() {
                Ok(mut r) if r.len() < r.capacity() => r.push(old),
                _ => {
                    // Retry next block; no new swap until then.
                    self.retiring = Some(old);
                    return;
                }
            }
        }
        let Some((n, mut new)) = self.bridge.replace.try_lock().ok().and_then(|mut slot| slot.take()) else { return };
        if let Some(old) = self.engine.as_mut() {
            // The sampler (armed state, takes) survives rebuilds.
            std::mem::swap(&mut new.sampler, &mut old.sampler);
            new.adopt(old);
        }
        new.restore_steps(self.recent_writes.writes.iter().map(|(_, w)| w));
        self.replay.onto(n, &mut new);
        self.built = n;
        self.bridge.built.store(n, std::sync::atomic::Ordering::Release);
        self.retiring = self.engine.replace(new);
    }

    fn project_path(&self) -> Option<PathBuf> {
        let p = self.params.project_path.read().ok()?.clone();
        let p = if p.is_empty() { std::env::var("EINST_PROJECT").unwrap_or_default() } else { p };
        (!p.is_empty()).then(|| PathBuf::from(p))
    }
}

const SUB_NAMES: [&str; SUB_PORTS] = [
    "Sub output 1", "Sub output 2", "Sub output 3", "Sub output 4", "Sub output 5", "Sub output 6", "Sub output 7", "Sub output 8", "Sub output 9", "Sub output 10", "Sub output 11", "Sub output 12",
    "Sub output 13", "Sub output 14", "Sub output 15", "Sub output 16",
];

impl Plugin for EinstPlugin {
    const NAME: &'static str = "Einst";
    const VENDOR: &'static str = "Dreist";
    const URL: &'static str = "";
    const EMAIL: &'static str = "";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),
        aux_input_ports: &[],
        aux_output_ports: &[new_nonzero_u32(2); SUB_PORTS],
        names: PortNames { layout: Some("Stereo + 16 subs"), main_input: Some("Sampler input"), main_output: Some("Main"), aux_inputs: &[], aux_outputs: &SUB_NAMES },
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::MidiCCs;
    /// Send Note / Send CC ([re:5a8c24]).
    const MIDI_OUTPUT: MidiConfig = MidiConfig::MidiCCs;
    const SAMPLE_ACCURATE_AUTOMATION: bool = false;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        // Option `StyleSheet` (Preferences › UI style; `EINST_THEME` overrides).
        let theme = std::env::var("EINST_THEME").unwrap_or_else(|_| einst_editor::options::Options::load().string("StyleSheet", dreist_gui::skin::default_theme()));
        Some(Box::new(editor::DreistEditor { bridge: self.bridge.clone(), theme, params: self.params.clone(), size: self.editor_size.clone(), host_scale: 1.0f32.to_bits().into() }))
    }

    fn initialize(&mut self, _layout: &AudioIOLayout, buffer_config: &BufferConfig, _context: &mut impl InitContext<Self>) -> bool {
        let sr = buffer_config.sample_rate;
        self.bridge.sample_rate.store(sr.to_bits(), std::sync::atomic::Ordering::Relaxed);
        let options = einst_editor::options::render_options();
        if let Ok(mut bp) = self.bridge.project_path.lock() {
            *bp = self.project_path();
        }
        #[cfg(feature = "standalone")]
        {
            self.clock = midiclock::ClockInput::open();
        }
        // A relaunch after an audio settings change hands its document over.
        #[cfg(feature = "standalone")]
        if let Some(s) = standalone::take_session() {
            if let Ok(mut cur) = self.params.session.write() {
                *cur = Some(s);
            }
        }
        // Samples decode into the cache later rebuilds share.
        let cache_handle = self.bridge.cache.clone();
        let mut cache = editor::lock(&cache_handle);
        let session = self.params.session.read().ok().and_then(|s| s.clone());
        let from_session = session.as_ref().and_then(|s| {
            // v1's plugin state converts ([re:53d0e3]).
            let doc = einst::v1io::session_document(&s.doc)?;
            let samples = s.samples.as_ref().map(PathBuf::from);
            let g = Dreist::from_document(&doc, samples.as_deref().unwrap_or(std::path::Path::new(".")), sr, options, &mut cache)?;
            Some((g, samples))
        });
        let mut samples = self.project_path().map(|p| dreist_formats::model::samples_dir(&p));
        let engine = match (from_session, self.project_path()) {
            (Some((g, s)), _) => {
                samples = s;
                g
            }
            (None, Some(p)) => load_project(&p, sr, options, &mut cache).unwrap_or_else(|e| {
                nih_log!("failed to load {}: {e}", p.display());
                Dreist::empty(sr, options)
            }),
            // [re:5cbd54] (from the constructor):
            // Tools › Save as default preset, when one was saved.
            (None, None) => einst_editor::options::settings_dir()
                .map(|d| d.join("DefaultPreset.geist"))
                .filter(|p| p.is_file())
                .and_then(|p| Dreist::load_cached(&p, sr, options, &mut cache).ok())
                .unwrap_or_else(|| Dreist::empty(sr, options)),
        };
        drop(cache);
        let mut engine = engine;
        engine.set_metronome_sound(&einst_editor::options::metronome_sound());
        let spare = engine.sampler.new_spare();
        engine.sampler.give_spare(spare);
        self.bridge.set_document(engine.document().clone(), samples);
        // The editor owns the document from here.
        engine.set_document_mirroring(false);
        // The audio thread hands these over only within the room reserved
        // here (the editor drains them, keeping the capacity).
        {
            let mut s = editor::lock(&self.bridge.state);
            s.automation.reserve(QUEUE);
            s.editor_messages.reserve(QUEUE);
            s.midi_log.reserve(QUEUE);
            s.live.reserve(LIVE_QUEUE);
            s.scene_events.reserve(QUEUE);
            s.retro_events.reserve(EVENT_QUEUE);
            s.step_writes.reserve(EVENT_QUEUE);
            s.song_events.reserve(EVENT_QUEUE);
            s.takes.reserve(dreist_engine::sampler::FINISHED_MAX);
        }
        editor::lock(&self.bridge.commands).reserve(COMMAND_QUEUE);
        self.bridge.start(&self.params);
        editor::lock(&self.bridge.spent).reserve(COMMAND_QUEUE);
        editor::lock(&self.bridge.retired).reserve(8);
        self.cmds.reserve(COMMAND_QUEUE);
        self.applied.reserve(COMMAND_QUEUE);
        self.replay.0.reserve(REPLAY_QUEUE);
        self.recent_writes.writes.reserve(RECENT_WRITES);
        // A rebuild in flight was for the old sample rate (its worker
        // discards it too); the new engine stands.
        *editor::lock(&self.bridge.replace) = None;
        self.built = self.bridge.requested.load(std::sync::atomic::Ordering::Acquire);
        self.bridge.built.store(self.built, std::sync::atomic::Ordering::Release);
        self.replay.0.clear();
        self.engine = Some(Box::new(engine));
        let max = buffer_config.max_buffer_size as usize;
        for s in self.sub_scratch.iter_mut().chain(std::iter::once(&mut self.input_scratch)) {
            s[0].resize(max, 0.0);
            s[1].resize(max, 0.0);
        }
        true
    }

    fn reset(&mut self) {
        self.last_auto.fill(f32::NAN);
    }

    fn process(&mut self, buffer: &mut Buffer, aux: &mut AuxiliaryBuffers, context: &mut impl ProcessContext<Self>) -> ProcessStatus {
        self.swap_in_rebuilt();
        let Some(engine) = self.engine.as_mut() else { return ProcessStatus::Normal };
        // The standalone has no host transport: nih-plug's backend reports
        // "playing at 120 BPM" every block, so the original runs its own (Play /
        // Stop, the project tempo), as the original standalone shell does.
        // With "Sync to MIDI clock" the standalone follows the clock
        // (`midiclock`).
        #[cfg(feature = "standalone")]
        if let Some(ci) = self.clock.as_mut() {
            ci.drain();
            if ci.clock.active() {
                engine.set_transport(ci.clock.playing(), ci.clock.tempo, Some(ci.clock.ppq));
            }
            ci.clock.advance(buffer.samples(), f32::from_bits(self.bridge.sample_rate.load(std::sync::atomic::Ordering::Relaxed)));
        }
        if !cfg!(feature = "standalone") {
            let t = context.transport();
            engine.set_transport(t.playing, t.tempo.unwrap_or(120.0), t.pos_beats());
            // `GetEffectivePpqPos` honours the host loop only while slaved
            // to a playing host ([re:b7aa68]); read after
            // this block's transport update (a host start loops at once).
            engine.set_host_loop(t.loop_range_beats().filter(|_| t.playing && engine.is_playing()));
            // [re:2b44b1]: a host time signature
            // (non-zero) replaces the document's.
            if let (Some(n), Some(d)) = (t.time_sig_numerator, t.time_sig_denominator) {
                if n > 0 && d > 0 {
                    engine.set_time_signature(n as u32, d as u32);
                }
            }
        }
        // Editor commands: applied here, handed back to be freed by the GUI.
        if let Ok(mut q) = self.bridge.commands.try_lock() {
            std::mem::swap(&mut *q, &mut self.cmds);
        }
        for (tag, mut c) in self.cmds.drain(..) {
            c.apply(engine);
            if let einst_editor::Command::Message(m) = &c {
                self.replay.record(tag, self.built, m);
            }
            if self.applied.len() < self.applied.capacity() {
                self.applied.push(c);
            }
        }
        if !self.applied.is_empty() {
            if let Ok(mut spent) = self.bridge.spent.try_lock() {
                if spent.capacity() - spent.len() >= self.applied.len() {
                    spent.append(&mut self.applied);
                }
            }
        }
        if let Ok(mut s) = self.bridge.state.try_lock() {
            s.playing = engine.is_playing();
            s.song_pos = s.playing.then(|| engine.song_position());
            s.host_loop = engine.host_loop_active();
            hand_over(&mut s.editor_messages, &mut engine.editor_messages);
            for m in engine.automation_out.drain(..) {
                let out = &mut s.automation;
                if let Some(o) = out.iter_mut().find(|o| o.same_target(&m)) {
                    o.value = m.value;
                } else if out.len() < out.capacity() {
                    out.push(m);
                }
            }
            s.tempo = engine.tempo();
            for (e, st) in s.steps.iter_mut().enumerate() {
                *st = engine.current_step(e);
            }
            s.seq_steps = Some(engine.seq_steps());
            s.mods_triggered |= std::mem::take(&mut engine.mods_triggered);
            let held = s.sampler.meters;
            s.sampler = einst_editor::model::SamplerView::of(&engine.sampler);
            s.sampler.meters = (held.0.max(engine.sampler.meters.0), held.1.max(engine.sampler.meters.1));
            engine.sampler.meters = (0.0, 0.0);
            s.last_cc = engine.last_cc;
            hand_over(&mut s.midi_log, &mut engine.midi_log);
            // Finished takes wait in the engine while the editor has not
            // collected the previous ones.
            let sr = engine.sampler.sample_rate;
            while s.takes.len() < s.takes.capacity() && !engine.sampler.finished.is_empty() {
                let t = engine.sampler.finished.remove(0);
                s.takes.push((t, sr));
            }
            if engine.sampler.needs_spare() {
                if let Some(t) = s.spare_take.take() {
                    engine.sampler.give_spare(t);
                }
            }
            hand_over(&mut s.scene_events, &mut engine.scenes_mut().events);
            hand_over(&mut s.retro_events, &mut engine.retro_events);
            let pending = self.built < self.bridge.requested.load(std::sync::atomic::Ordering::Acquire);
            self.recent_writes.record(&engine.step_writes, pending, (engine.sample_rate() * 0.15) as u64);
            hand_over(&mut s.step_writes, &mut engine.step_writes);
            hand_over(&mut s.song_events, &mut engine.songrec.events);
            let take = engine.sampler.current_take().map(|t| (t.left.as_slice(), t.right.as_slice()));
            if self.live_take.poll(take, &mut s.live) {
                s.live.clear();
                s.live_reset = true;
            }
            s.meters.merge(&engine.meters);
            engine.meters.clear();
        }

        while let Some(ev) = context.next_event() {
            let to7 = |v: f32| (v * 127.0).round().clamp(0.0, 127.0) as u8;
            match ev {
                NoteEvent::NoteOn { timing, channel, note, velocity, .. } => engine.push_midi(timing, [0x90 | channel, note, to7(velocity).max(1)]),
                NoteEvent::NoteOff { timing, channel, note, .. } => engine.push_midi(timing, [0x80 | channel, note, 0]),
                NoteEvent::MidiCC { timing, channel, cc, value } => engine.push_midi(timing, [0xb0 | channel, cc, to7(value)]),
                NoteEvent::MidiChannelPressure { timing, channel, pressure } => engine.push_midi(timing, [0xd0 | channel, to7(pressure), 0]),
                NoteEvent::MidiPitchBend { timing, channel, value } => {
                    let v = (value * 16383.0).round().clamp(0.0, 16383.0) as u16;
                    engine.push_midi(timing, [0xe0 | channel, (v & 0x7f) as u8, (v >> 7) as u8]);
                }
                _ => {}
            }
        }

        // effSetParameter → [re:e5b4e6].
        for (i, p) in self.params.automation.iter().enumerate() {
            let v = p.value.value();
            if v != self.last_auto[i] {
                if !self.last_auto[i].is_nan() {
                    engine.set_host_parameter(i, v);
                }
                self.last_auto[i] = v;
            }
        }

        let n = buffer.samples();
        self.recent_writes.advance(n);
        let main = buffer.as_slice();
        // nih-plug processes in place: the main buffer holds the input first
        // (the scratch is sized for the host's maximum block at initialize).
        if self.input_scratch[0].len() < n {
            self.input_scratch[0].resize(n, 0.0);
            self.input_scratch[1].resize(n, 0.0);
        }
        if main.len() >= 2 {
            self.input_scratch[0][..n].copy_from_slice(&main[0][..n]);
            self.input_scratch[1][..n].copy_from_slice(&main[1][..n]);
        }
        let (left, rest) = main.split_at_mut(1);
        let left = &mut left[0][..n];
        let right = &mut rest[0][..n];
        for [a, b] in self.sub_scratch.iter_mut() {
            if a.len() < n {
                a.resize(n, 0.0);
                b.resize(n, 0.0);
            }
        }
        // A fixed array of the sub output buffers (no allocation per block).
        let mut it = self.sub_scratch.iter_mut();
        let mut subs: [[&mut [f32]; 2]; SUB_PORTS] = std::array::from_fn(|_| {
            let [a, b] = it.next().expect("sub buffer");
            [&mut a[..n], &mut b[..n]]
        });
        let input = [&self.input_scratch[0][..n], &self.input_scratch[1][..n]];
        engine.process_with_input(left, right, &mut subs, Some(input));
        for &(t, [st, d1, d2]) in &engine.midi_out {
            let (timing, channel) = (t.min(n.saturating_sub(1) as u32), st & 15);
            match st & 0xf0 {
                0x90 => context.send_event(NoteEvent::NoteOn { timing, voice_id: None, channel, note: d1, velocity: d2 as f32 / 127.0 }),
                0x80 => context.send_event(NoteEvent::NoteOff { timing, voice_id: None, channel, note: d1, velocity: 0.0 }),
                0xb0 => context.send_event(NoteEvent::MidiCC { timing, channel, cc: d1, value: d2 as f32 / 127.0 }),
                _ => {}
            }
        }
        for (k, out) in aux.outputs.iter_mut().enumerate().take(SUB_PORTS) {
            let ch = out.as_slice();
            if ch.len() >= 2 {
                ch[0][..n].copy_from_slice(subs[k][0]);
                ch[1][..n].copy_from_slice(subs[k][1]);
            }
        }
        ProcessStatus::Normal
    }
}

impl ClapPlugin for EinstPlugin {
    const CLAP_ID: &'static str = "rs.einst.einst";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Drum sampler and sequencer");
    const CLAP_MANUAL_URL: Option<&'static str> = None;
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::Instrument, ClapFeature::Sampler, ClapFeature::DrumMachine, ClapFeature::Stereo];
}

impl Vst3Plugin for EinstPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"EinstRsDrumSmplr";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[Vst3SubCategory::Instrument, Vst3SubCategory::Drum, Vst3SubCategory::Sampler];
}

nih_export_clap!(EinstPlugin);
nih_export_vst3!(EinstPlugin);

#[cfg(test)]
mod tests {
    use super::*;

    /// A the original v1 kit (no `version` on its root) opens through
    /// `einst_formats` into engine 0 and plays its pads.
    #[test]
    fn opens_v1_files() {
        let dir = std::env::temp_dir().join(format!("einst-plugin-v1-{}", std::process::id()));
        let samples = dir.join("Kit.samples");
        std::fs::create_dir_all(&samples).unwrap();
        let click = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/ui-v1/Metronomes/Default/Beat.wav");
        std::fs::copy(&click, samples.join("Beat.wav")).unwrap();
        let kit = dir.join("Kit.geistkit");
        std::fs::write(&kit, r#"<root><Pad name="Pad0" Ver="2" PadMode="All layers"><Layer name="Layer0" Ver="3.2" AudioFileName="Beat.wav" Envelope0Param3="-1" /></Pad></root>"#).unwrap();
        let mut g = load_project(&kit, 44_100.0, Default::default(), &mut Default::default()).unwrap();
        assert_eq!(g.document().children_of("Engine").next().and_then(|e| e.children_of("Pad").next()).and_then(|p| p.name.clone()).as_deref(), Some("Pad0"));
        g.push_midi(0, [0x90, 36, 100]);
        let (mut l, mut r) = (vec![0.0; 4096], vec![0.0; 4096]);
        g.process(&mut l, &mut r);
        let peak = l.iter().chain(&r).fold(0.0f32, |m, v| m.max(v.abs()));
        let _ = std::fs::remove_dir_all(&dir);
        assert!(peak > 1e-3, "pad 0 silent");
    }

    #[test]
    fn session_round_trips_through_host_state() {
        let p = DreistParams::default();
        let s = Session { doc: "<root version=\"3000\"><TimeInfo Tempo=\"97\" /></root>".into(), samples: Some("/tmp/x.samples".into()) };
        *p.session.write().unwrap() = Some(s.clone());
        let saved = p.serialize_fields();
        let q = DreistParams::default();
        q.deserialize_fields(&saved);
        assert_eq!(q.session.read().unwrap().as_ref(), Some(&s));
    }

    /// A message queued in the same batch as a rebuild request is in the
    /// rebuild's document and must not be replayed onto it (a nudge would
    /// apply twice); one queued after the request is replayed.
    #[test]
    fn replay_skips_what_the_rebuild_document_has() {
        use dreist_engine::binding::{Message, id};
        let tempo = |bpm: f32| Message::new(id::SET_TEMPO, &[], &[bpm]);
        let mut r = Replay(Vec::with_capacity(16));
        let built = 0;
        // Batch queued with no rebuild requested yet (tag 0); the batch's
        // own rebuild request then makes `requested` 1.
        r.record(0, built, &tempo(99.0));
        assert!(r.0.is_empty(), "the rebuild's document has it");
        // Queued while rebuild 1 is pending: tag 1.
        r.record(1, built, &tempo(133.0));
        let mut g = Dreist::empty(44_100.0, Default::default());
        r.onto(1, &mut g);
        assert_eq!(g.tempo(), 133.0, "replayed onto rebuild 1");
        // Rebuild 2 (requested after it was queued) has it in its document.
        r.onto(2, &mut Dreist::empty(44_100.0, Default::default()));
        assert!(r.0.is_empty());
    }

    /// Recorded steps are kept while a rebuild is pending and for a grace
    /// period after, then forgotten.
    #[test]
    fn recent_writes_cover_pending_rebuilds() {
        let w = dreist_engine::StepWrite { engine: 0, pattern: 1, track: 2, step: 3, data: dreist_engine::pattern::StepData::off() };
        let mut r = RecentWrites { writes: Vec::with_capacity(4), clock: 0 };
        r.record(std::slice::from_ref(&w), false, 100);
        r.advance(64);
        r.record(&[], true, 100);
        r.advance(512);
        r.record(&[], true, 100);
        assert_eq!(r.writes.len(), 1, "kept while pending");
        r.record(&[], false, 100);
        assert!(r.writes.is_empty(), "forgotten after the grace period");
        for _ in 0..6 {
            r.record(std::slice::from_ref(&w), true, 100);
        }
        assert_eq!(r.writes.len(), 4, "bounded by the reserved room");
    }
}
