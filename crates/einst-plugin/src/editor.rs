//! Plugin editor: `einst-editor` (Einst's copy of Dreist's editor) in the host's
//! parent view. The GUI talks to the audio thread through [`Bridge`]:
//! commands one way, a state snapshot the other, both behind locks the
//! audio thread only ever `try_lock`s.

use std::any::Any;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, OnceLock, PoisonError, Weak};

use einst_editor::{Command, Editor as GeistGui, Model};
use geist_engine::Geist;
use geist_formats::Node;
use nih_plug::prelude::*;

/// Engine state the GUI displays, refreshed by the audio thread.
#[derive(Clone, Debug, Default)]
pub struct UiState {
    pub playing: bool,
    pub tempo: f64,
    pub steps: [Option<u32>; 8],
    /// Mini sequencer playing step per transmod slot (−1: none).
    pub seq_steps: Option<[i32; 16]>,
    /// Retriggered transmod sources (bit mask) for the source flashes.
    pub mods_triggered: u32,
    /// Song position (QN) while playing.
    pub song_pos: Option<f64>,
    /// The host reports a loop this block.
    pub host_loop: bool,
    /// Messages from editor-only bindings.
    pub editor_messages: Vec<geist_engine::binding::Message>,
    /// Bound parameters moved by MIDI CCs / host automation (latest per
    /// target), for the editor's document.
    pub automation: Vec<geist_engine::binding::Message>,
    pub sampler: einst_editor::model::SamplerView,
    /// Last MIDI CC (count, channel, controller) for MIDI learn.
    pub last_cc: (u32, u8, u8),
    /// Peak levels since the editor's last meter tick.
    pub meters: geist_engine::Meters,
    /// Finished sampler takes and their sample rate, for the editor to save.
    pub takes: Vec<(geist_engine::sampler::Take, f32)>,
    /// Incoming MIDI for the MIDI console.
    pub midi_log: Vec<[u8; 3]>,
    /// Live take display: a new take started, and new min/max pairs.
    pub live_reset: bool,
    pub live: Vec<(f32, f32)>,
    /// Song recorder messages.
    pub song_events: Vec<geist_engine::songrec::SongEvent>,
    /// Pattern retro recording input.
    pub retro_events: Vec<geist_engine::songrec::RetroEvent>,
    /// Steps the engine wrote (recording).
    pub step_writes: Vec<geist_engine::StepWrite>,
    /// Scene notifications.
    pub scene_events: Vec<geist_engine::scene::SceneEvent>,
    /// A reserved buffer for the sampler's next take (see
    /// `Sampler::give_spare`), kept ready by the rebuild worker.
    pub spare_take: Option<geist_engine::sampler::Take>,
}

/// The GUI's copy of what the audio thread left in [`UiState`], taken under
/// the lock and handled after releasing it.
#[derive(Default)]
struct Inbox {
    playing: bool,
    tempo: f64,
    steps: [Option<u32>; 8],
    seq_steps: Option<[i32; 16]>,
    mods_triggered: u32,
    song_pos: Option<f64>,
    host_loop: bool,
    editor_messages: Vec<geist_engine::binding::Message>,
    automation: Vec<geist_engine::binding::Message>,
    sampler: Option<einst_editor::model::SamplerView>,
    last_cc: (u32, u8, u8),
    meters: geist_engine::Meters,
    takes: Vec<(geist_engine::sampler::Take, f32)>,
    midi_log: Vec<[u8; 3]>,
    live_reset: bool,
    live: Vec<(f32, f32)>,
    song_events: Vec<geist_engine::songrec::SongEvent>,
    retro_events: Vec<geist_engine::songrec::RetroEvent>,
    step_writes: Vec<geist_engine::StepWrite>,
    scene_events: Vec<geist_engine::scene::SceneEvent>,
}

impl Inbox {
    /// Move the queues out of `s` (draining keeps the audio thread's
    /// reserved capacity) and copy the display state.
    fn collect(&mut self, s: &mut UiState) {
        self.playing = s.playing;
        self.tempo = s.tempo;
        self.steps = s.steps;
        self.seq_steps = s.seq_steps.take();
        self.mods_triggered |= std::mem::take(&mut s.mods_triggered);
        self.song_pos = s.song_pos;
        self.host_loop = s.host_loop;
        self.editor_messages.append(&mut s.editor_messages);
        self.automation.append(&mut s.automation);
        self.sampler = Some(s.sampler.clone());
        self.last_cc = s.last_cc;
        self.meters.merge(&s.meters);
        s.meters.clear();
        s.sampler.meters = (0.0, 0.0);
        self.takes.append(&mut s.takes);
        self.midi_log.append(&mut s.midi_log);
        self.live_reset |= std::mem::take(&mut s.live_reset);
        self.live.append(&mut s.live);
        self.song_events.append(&mut s.song_events);
        self.retro_events.append(&mut s.retro_events);
        self.step_writes.append(&mut s.step_writes);
        self.scene_events.append(&mut s.scene_events);
    }

    /// Handled: empty the queues, keeping their capacity.
    fn clear(&mut self) {
        self.editor_messages.clear();
        self.automation.clear();
        self.meters.clear();
        self.midi_log.clear();
        self.live_reset = false;
        self.live.clear();
        self.song_events.clear();
        self.retro_events.clear();
        self.step_writes.clear();
        self.scene_events.clear();
    }
}

#[derive(Default)]
pub struct Bridge {
    /// Commands for the audio thread, each with the number of rebuilds
    /// requested when it was queued: the documents of those rebuilds
    /// predate it (see `EinstPlugin::replay`).
    pub commands: Mutex<Vec<(u64, Command)>>,
    /// Commands the audio thread applied, handed back so their payloads
    /// (documents, sample buffers) are freed on the GUI thread.
    pub spent: Mutex<Vec<Command>>,
    pub state: Mutex<UiState>,
    /// The current document: set by the plugin when it (re)builds the
    /// engine (`doc_gen` counts those), kept up to date by the editor.
    pub doc: Mutex<Option<Node>>,
    pub doc_gen: AtomicU64,
    /// Where the document's relative sample paths resolve.
    pub samples: Mutex<Option<PathBuf>>,
    /// A rebuilt engine and its request number, waiting to be swapped in
    /// by the audio thread.
    pub replace: Mutex<Option<(u64, Box<Geist>)>>,
    /// Engines swapped out, dropped by the rebuild worker.
    pub retired: Mutex<Vec<Box<Geist>>>,
    /// An editor window is open (it applies the engine's output to the
    /// document; otherwise `Bridge::headless_sync` does).
    pub editor_open: std::sync::atomic::AtomicBool,
    /// The plugin's parameters (their persisted session), for the worker.
    params: OnceLock<Weak<crate::GeistParams>>,
    /// Rebuild requests issued (the newest wins).
    pub requested: AtomicU64,
    /// The request the running engine was built for (published by the
    /// audio thread).
    pub built: AtomicU64,
    request: Mutex<Option<(u64, Node, Option<PathBuf>)>>,
    wake: Condvar,
    worker: OnceLock<()>,
    /// Decoded samples shared between rebuilds.
    pub cache: Arc<Mutex<geist_engine::audio::SampleCache>>,
    /// Sample rate (f32 bits) for rebuilds.
    pub sample_rate: AtomicU32,
    /// Project path chosen in the editor (persisted by the plugin).
    pub project_path: Mutex<Option<PathBuf>>,
    /// Rebuild progress for the browser's loading bar (taken by the editor).
    pub load_progress: Arc<Mutex<Option<f32>>>,
}

/// A lock that survives a panic elsewhere (the data is plain state).
pub fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

impl Bridge {
    /// A new document from the plugin side (load, session restore).
    /// Unchanged documents (a host re-activating the plugin) leave the
    /// open editor alone.
    pub fn set_document(&self, doc: Node, samples: Option<PathBuf>) {
        if lock(&self.doc).as_ref() == Some(&doc) && *lock(&self.samples) == samples {
            return;
        }
        *lock(&self.doc) = Some(doc);
        *lock(&self.samples) = samples;
        self.doc_gen.fetch_add(1, Ordering::AcqRel);
    }

    fn sample_rate(&self) -> f32 {
        let sr = f32::from_bits(self.sample_rate.load(Ordering::Relaxed));
        if sr > 0.0 { sr } else { 44100.0 }
    }

    /// Rebuild the engine from `doc` off the audio thread and queue the
    /// swap. One worker builds the newest request; older ones still
    /// waiting are dropped.
    pub fn rebuild(self: &Arc<Self>, doc: Node, samples: Option<PathBuf>) {
        let n = self.requested.fetch_add(1, Ordering::AcqRel) + 1;
        *lock(&self.request) = Some((n, doc, samples));
        self.wake.notify_one();
        self.ensure_worker();
    }

    /// Start the worker (rebuilds, freeing retired engines, spare takes,
    /// the headless document sync) for the plugin with `params`.
    pub fn start(self: &Arc<Self>, params: &Arc<crate::GeistParams>) {
        let _ = self.params.set(Arc::downgrade(params));
        self.ensure_worker();
    }

    fn ensure_worker(self: &Arc<Self>) {
        self.worker.get_or_init(|| {
            let me = Arc::downgrade(self);
            let _ = std::thread::Builder::new().name("rebuild".into()).spawn(move || worker(me));
        });
    }

    /// With no editor open, what the engine changed in the document (steps
    /// recorded live, bound parameters moved by MIDI CCs / host
    /// automation, the running tempo) goes into the bridge's document and
    /// the host's session, as the editor does when open.
    pub fn headless_sync(&self) {
        let mut doc = lock(&self.doc);
        // Checked under the document lock: an editor opening reads the
        // document after this.
        if self.editor_open.load(Ordering::Acquire) {
            return;
        }
        let Some(d) = doc.as_mut() else { return };
        let Ok(mut s) = self.state.try_lock() else { return };
        // Drained, not taken: the audio thread keeps the reserved room.
        #[allow(clippy::drain_collect)]
        let writes: Vec<geist_engine::StepWrite> = s.step_writes.drain(..).collect();
        #[allow(clippy::drain_collect)]
        let automation: Vec<geist_engine::binding::Message> = s.automation.drain(..).collect();
        let tempo = s.tempo;
        drop(s);
        let mut changed = !writes.is_empty() || !automation.is_empty();
        for w in &writes {
            geist_engine::engine::doc::write_step(d, w.engine, w.pattern, w.track, w.step, &w.data);
        }
        for m in &automation {
            geist_engine::engine::doc::mirror_message(d, m);
        }
        let doc_tempo = d.child("TimeInfo", "TimeInfo").and_then(|t| t.attr("Tempo")).and_then(|v| v.trim().parse::<f64>().ok());
        if tempo > 0.0 && doc_tempo != Some(tempo.clamp(20.0, 999.0)) {
            d.child_mut_or_insert("TimeInfo", "TimeInfo").set("Tempo", tempo.clamp(20.0, 999.0));
            changed = true;
        }
        if !changed {
            return;
        }
        let xml = d.to_xml();
        drop(doc);
        if let Some(params) = self.params.get().and_then(Weak::upgrade) {
            let samples = lock(&self.samples).clone();
            if let Ok(mut s) = params.session.write() {
                *s = Some(crate::Session { doc: xml, samples: samples.map(|p| p.to_string_lossy().into_owned()) });
            }
        }
    }

    /// Whether `rt` (commands for the audio thread) carry state a rebuild
    /// in flight lacks (its document predates them; only messages are
    /// replayed onto it) and `structural` asks for no rebuild already.
    pub fn needs_newer_rebuild(&self, rt: &[Command], structural: &[Command]) -> bool {
        let in_flight = self.built.load(Ordering::Acquire) < self.requested.load(Ordering::Acquire);
        in_flight && rt.iter().any(Command::outlives_rebuild) && !structural.iter().any(|c| matches!(c, Command::Rebuild { .. }))
    }

    /// Build one request; `None` when it failed or a newer one arrived.
    fn build(&self, doc: &Node, samples: Option<PathBuf>) -> Option<Geist> {
        let sr = self.sample_rate();
        let samples = samples.unwrap_or_else(|| PathBuf::from("."));
        // Loader progress (`SetupJobCallbacks` $_6 / $_4): lookups over
        // the document's layers, then 1.5 when done.
        fn layers(n: &Node) -> usize {
            n.children.iter().map(|c| (c.class == "Layer") as usize + layers(c)).sum()
        }
        let total = layers(doc).max(1) as f32;
        *lock(&self.load_progress) = Some(0.0);
        let built = {
            let mut cache = lock(&self.cache);
            // Samples only the cache still holds (an earlier kit) go.
            cache.prune();
            let counter = self.load_progress.clone();
            let mut n = 0.0f32;
            cache.on_lookup = Some(Box::new(move || {
                n += 1.0;
                *lock(&counter) = Some((n / total).min(1.0));
            }));
            let built = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let mut g = Geist::from_document(doc, &samples, sr, einst_editor::options::render_options(), &mut cache)?;
                g.set_metronome_sound(&einst_editor::options::metronome_sound());
                g.set_document_mirroring(false);
                Some(g)
            }));
            cache.on_lookup = None;
            built.unwrap_or_else(|_| {
                nih_log!("rebuild: the document did not build (panic)");
                None
            })
        };
        *lock(&self.load_progress) = Some(1.5);
        // Built for a sample rate the host has since changed: discard.
        built.filter(|_| self.sample_rate() == sr)
    }
}

/// The rebuild worker: builds the newest request, frees retired engines and
/// keeps a reserved buffer ready for the sampler's next take. It exits with
/// the plugin.
fn worker(bridge: Weak<Bridge>) {
    loop {
        let Some(me) = bridge.upgrade() else { return };
        let retired: Vec<Box<Geist>> = lock(&me.retired).drain(..).collect();
        drop(retired);
        if lock(&me.state).spare_take.is_none() {
            let t = geist_engine::sampler::Take::with_capacity((me.sample_rate() * geist_engine::sampler::TAKE_RESERVE_SECONDS) as usize);
            lock(&me.state).spare_take.get_or_insert(t);
        }
        me.headless_sync();
        let req = lock(&me.request).take();
        if let Some((n, doc, samples)) = req {
            let built = me.build(&doc, samples);
            drop(doc);
            // A newer request supersedes this build.
            if let Some(g) = built.filter(|_| lock(&me.request).is_none()) {
                let stale = lock(&me.replace).replace((n, Box::new(g)));
                drop(stale);
            }
            continue;
        }
        // Woken by a request; otherwise a periodic sweep (the plugin's
        // drop ends the loop within one).
        let guard = lock(&me.request);
        if guard.is_none() {
            let _ = me.wake.wait_timeout(guard, std::time::Duration::from_millis(100));
        }
    }
}

pub struct GeistEditor {
    pub bridge: Arc<Bridge>,
    pub theme: String,
    pub params: Arc<crate::GeistParams>,
    /// The window size the editor last asked for (pixels; 0 = from the
    /// options), what `size()` reports to the host.
    pub size: Arc<std::sync::atomic::AtomicU64>,
}

/// `Editor::ComputeStartupSize`: `LastEditorSize` (window pixels) or the
/// default size × the UI scale.
fn startup_size() -> (u32, u32) {
    let o = einst_editor::options::Options::load();
    let ui = o.int("LastEditorScaleFactor", 100).clamp(50, 300) as f64 / 100.0;
    let saved = o.string("LastEditorSize", "0,0");
    let parsed = saved.split_once(',').and_then(|(w, h)| Some((w.trim().parse::<u32>().ok()?, h.trim().parse::<u32>().ok()?))).filter(|(w, h)| *w > 0 && *h > 0);
    parsed.unwrap_or(((SIZE.0 as f64 * ui).round() as u32, (SIZE.1 as f64 * ui).round() as u32))
}

/// Store the editor's document in the bridge and the persisted session
/// when it changed (`wait`: even if the worker holds the document).
fn sync_session(bridge: &Bridge, params: &crate::GeistParams, ed: &GeistGui, wait: bool) {
    let mut d = if wait {
        lock(&bridge.doc)
    } else {
        let Ok(d) = bridge.doc.try_lock() else { return };
        d
    };
    let samples = ed.model.samples.clone();
    if d.as_ref() == Some(&ed.model.doc) && bridge.samples.lock().ok().is_some_and(|s| *s == samples) {
        return;
    }
    *d = Some(ed.model.doc.clone());
    drop(d);
    if let Ok(mut s) = bridge.samples.lock() {
        *s = samples.clone();
    }
    let session = crate::Session { doc: ed.model.doc.to_xml(), samples: samples.map(|p| p.to_string_lossy().into_owned()) };
    if let Ok(mut s) = params.session.write() {
        *s = Some(session);
    }
}

struct Handle(geist_gui::window::WindowHandle);
// SAFETY: the handle is only used to close the window from the host's GUI
// thread, as nih-plug's own baseview editors do.
unsafe impl Send for Handle {}

impl Drop for Handle {
    fn drop(&mut self) {
        self.0.close();
    }
}

pub const SIZE: (u32, u32) = (1400, 850);

impl Editor for GeistEditor {
    fn spawn(&self, parent: ParentWindowHandle, context: Arc<dyn GuiContext>) -> Box<dyn Any + Send> {
        // Before reading the document (see `Bridge::headless_sync`).
        self.bridge.editor_open.store(true, std::sync::atomic::Ordering::Release);
        let mut seen_gen = self.bridge.doc_gen.load(std::sync::atomic::Ordering::Acquire);
        let doc = self.bridge.doc.lock().ok().and_then(|d| d.clone()).unwrap_or_else(|| Node::new("root", None));
        let mut model = Model::new(doc);
        model.samples = self.bridge.samples.lock().ok().and_then(|s| s.clone());
        if let Some(p) = self.bridge.project_path.lock().ok().and_then(|p| p.clone()).or_else(|| {
            let s = self.params.project_path.read().ok()?.clone();
            (!s.is_empty()).then(|| std::path::PathBuf::from(s))
        }) {
            model.name = p.file_stem().map(|s| s.to_string_lossy().into_owned());
        }
        let project_path = self.bridge.project_path.lock().ok().and_then(|p| p.clone());
        let gui = match GeistGui::new(&self.theme, model) {
            Ok(mut g) => {
                g.project_path = project_path;
                g.menu_bar = cfg!(feature = "standalone");
                #[cfg(feature = "standalone")]
                {
                    g.devices.host = Some(Box::new(crate::standalone::Host));
                    let (app_items, menus) = g.menu_bar_items();
                    geist_gui::menubar::install(einst_editor::PRODUCT, &app_items, &menus);
                }
                g
            }
            Err(e) => {
                nih_log!("Geist2 resources not usable: {e}");
                self.bridge.editor_open.store(false, std::sync::atomic::Ordering::Release);
                return Box::new(());
            }
        };
        let bridge = self.bridge.clone();
        let params = self.params.clone();
        let size = self.size.clone();
        let mut frames = 0u32;
        let mut inbox = Inbox::default();
        // The last previews' samples: the audio thread may still be playing
        // one, and must not be the one to free it.
        let mut previews: std::collections::VecDeque<Arc<geist_dsp::voice::SampleData>> = Default::default();
        let link = Box::new(move |ed: &mut GeistGui, cmds: Vec<Command>| {
            // A new UI scale: ask the host for the new size (`size()` reads it).
            if ed.app.resize_request {
                let (w, h) = ed.app.size();
                let ui = ed.app.ui_scale.max(0.25);
                let px = (((w as f32 * ui).round() as u64) << 32) | (h as f32 * ui).round() as u64;
                size.store(px, std::sync::atomic::Ordering::Relaxed);
                context.request_resize();
            }
            let (mut structural, rt): (Vec<Command>, Vec<Command>) = cmds.into_iter().partition(Command::is_structural);
            if bridge.needs_newer_rebuild(&rt, &structural) {
                structural.push(ed.model.rebuild_command());
            }
            if !rt.is_empty() {
                // Queued before this batch's own rebuild request: its
                // document includes them.
                let requested = bridge.requested.load(std::sync::atomic::Ordering::Acquire);
                lock(&bridge.commands).extend(rt.into_iter().map(|c| (requested, c)));
            }
            let spent: Vec<Command> = lock(&bridge.spent).drain(..).collect();
            for c in spent {
                if let Command::Preview(Some(d)) | Command::PreviewRegion { data: d, .. } = c {
                    previews.push_back(d);
                    if previews.len() > 2 {
                        previews.pop_front();
                    }
                }
            }
            for c in structural {
                match c {
                    Command::Rebuild { doc, samples } => bridge.rebuild(*doc, samples),
                    Command::ExportAudio { doc, samples, path, options, reveal, status } => {
                        let sr = f32::from_bits(bridge.sample_rate.load(std::sync::atomic::Ordering::Relaxed));
                        let sr = if sr > 0.0 { sr as u32 } else { 44100 };
                        std::thread::spawn(move || {
                            let samples = samples.unwrap_or_else(|| std::path::PathBuf::from("."));
                            match einst_editor::command::run_export(&doc, &samples, &path, sr, &options, status.as_ref()) {
                                Ok(r) => {
                                    if r.clipped > 0 {
                                        nih_log!("export: {} clipped samples", r.clipped);
                                    }
                                    // Pattern export: show the file (the original offers it for dragging).
                                    if reveal {
                                        if let Some(f) = r.files.first() {
                                            let _ = std::process::Command::new("open").arg("-R").arg(f).spawn();
                                        }
                                    }
                                }
                                Err(e) => nih_log!("export failed: {e}"),
                            }
                        });
                    }
                    _ => {}
                }
            }
            // Persist the project path chosen in the editor (open/save).
            if let Some(path) = &ed.project_path {
                let s = path.to_string_lossy().into_owned();
                if params.project_path.read().map(|p| *p != s).unwrap_or(false) {
                    if let Ok(mut p) = params.project_path.write() {
                        *p = s;
                    }
                    if let Ok(mut p) = bridge.project_path.lock() {
                        *p = Some(path.clone());
                    }
                }
            }
            // The standalone's menu bar.
            for tag in geist_gui::menubar::take() {
                ed.menu_command(tag);
            }
            // A document from the plugin side (session restore, reload).
            let doc_gen = bridge.doc_gen.load(std::sync::atomic::Ordering::Acquire);
            if doc_gen != seen_gen {
                if let Some(doc) = bridge.doc.try_lock().ok().and_then(|d| d.clone()) {
                    seen_gen = doc_gen;
                    ed.model.samples = bridge.samples.lock().ok().and_then(|s| s.clone());
                    ed.set_document(doc);
                }
            }
            // About twice a second: hand the edited document back, for the
            // next editor and the host's session state.
            frames = frames.wrapping_add(1);
            if ed.closing {
                // The last call: the final document, then the headless
                // sync takes over.
                sync_session(&bridge, &params, ed, true);
                bridge.editor_open.store(false, std::sync::atomic::Ordering::Release);
                return;
            }
            if frames.is_multiple_of(30) {
                sync_session(&bridge, &params, ed, false);
            }
            // Take what the audio thread left, then act on it without the
            // lock (the audio thread only ever try_locks it).
            if let Ok(mut s) = bridge.state.try_lock() {
                inbox.collect(&mut s);
            } else {
                return;
            }
            if inbox.mods_triggered != 0 {
                ed.modulators_triggered(std::mem::take(&mut inbox.mods_triggered));
            }
            if let Some(st) = inbox.seq_steps.take().filter(|st| *st != ed.model.seq_steps) {
                ed.model.seq_steps = st;
                ed.app.dirty = true;
            }
            ed.set_host_loop(inbox.host_loop);
            if inbox.tempo > 0.0 {
                ed.engine_tempo(inbox.tempo);
            }
            if !inbox.automation.is_empty() {
                ed.automation(&inbox.automation);
            }
            if !inbox.editor_messages.is_empty() {
                ed.binding_messages(&inbox.editor_messages);
            }
            if let Some(f) = lock(&bridge.load_progress).take() {
                ed.load_progress(f);
            }
            ed.set_playback(inbox.playing, inbox.steps, inbox.song_pos);
            // The preview volume is editor-side.
            if let Some(view) = inbox.sampler.take() {
                let sampler = einst_editor::model::SamplerView { preview_volume: ed.model.sampler.preview_volume, ..view };
                if ed.model.sampler != sampler {
                    ed.model.sampler = sampler;
                    ed.app.dirty = true;
                }
            }
            ed.update_meters(&inbox.meters);
            let (n, ch, cc) = inbox.last_cc;
            ed.midi_cc(n, ch, cc);
            if !inbox.midi_log.is_empty() {
                ed.midi_in(&inbox.midi_log);
            }
            if inbox.live_reset || !inbox.live.is_empty() {
                ed.sampler_live(inbox.live_reset, &inbox.live);
            }
            if !inbox.scene_events.is_empty() {
                ed.scene_events(&inbox.scene_events);
            } else {
                ed.animate();
            }
            if !inbox.retro_events.is_empty() {
                ed.retro_events(&inbox.retro_events);
            }
            if !inbox.step_writes.is_empty() {
                ed.step_writes(&inbox.step_writes);
            }
            if !inbox.song_events.is_empty() {
                ed.song_events(&inbox.song_events);
            }
            for (t, sr) in inbox.takes.drain(..) {
                ed.sampler_take(&t.left, &t.right, sr);
            }
            inbox.clear();
        });
        Box::new(Handle(einst_editor::window::open_parented(&parent, gui, link)))
    }

    fn size(&self) -> (u32, u32) {
        // A live resize (window sizer, UI scale) reports the requested
        // size; otherwise the startup size from the options.
        let px = self.size.load(std::sync::atomic::Ordering::Relaxed);
        if px != 0 {
            return ((px >> 32) as u32, px as u32);
        }
        startup_size()
    }

    fn set_scale_factor(&self, _factor: f32) -> bool {
        // baseview follows the system scale factor itself.
        false
    }

    fn param_value_changed(&self, _id: &str, _normalized_value: f32) {}

    fn param_modulation_changed(&self, _id: &str, _modulation_offset: f32) {}

    fn param_values_changed(&self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc(tempo: u32) -> Node {
        Node::parse(&format!("<root version=\"3000\"><TimeInfo Tempo=\"{tempo}\" /></root>")).unwrap()
    }

    /// Quick successive structural edits: only the newest document's
    /// engine is offered to the audio thread, tagged with its request.
    #[test]
    fn rebuild_worker_offers_the_newest_request() {
        let b = Arc::new(Bridge::default());
        b.sample_rate.store(44100f32.to_bits(), Ordering::Relaxed);
        for t in [90, 100, 110, 120] {
            b.rebuild(doc(t), None);
        }
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
        loop {
            if let Some((n, g)) = lock(&b.replace).as_ref() {
                if *n == 4 {
                    assert_eq!(g.tempo(), 120.0);
                    break;
                }
                assert!(*n < 4);
            }
            assert!(std::time::Instant::now() < deadline, "no build offered");
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    /// Song, swing, binding and pad-mode commands queued while a rebuild is
    /// in flight ask for a newer one; messages (replayed) and idle times
    /// do not.
    #[test]
    fn state_commands_during_a_rebuild_ask_for_another() {
        let b = Bridge::default();
        let song = Command::SongTracks(Box::default());
        let msg = Command::Message(geist_engine::binding::Message::new(geist_engine::binding::id::SET_TEMPO, &[], &[100.0]));
        assert!(!b.needs_newer_rebuild(std::slice::from_ref(&song), &[]), "no rebuild in flight");
        b.requested.store(3, Ordering::Release);
        b.built.store(2, Ordering::Release);
        assert!(b.needs_newer_rebuild(std::slice::from_ref(&song), &[]));
        assert!(!b.needs_newer_rebuild(std::slice::from_ref(&msg), &[]), "messages are replayed");
        let rebuild = Command::Rebuild { doc: Box::new(Node::new("root", None)), samples: None };
        assert!(!b.needs_newer_rebuild(std::slice::from_ref(&song), std::slice::from_ref(&rebuild)), "one is coming");
        b.built.store(3, Ordering::Release);
        assert!(!b.needs_newer_rebuild(&[song], &[]));
    }

    /// With no editor open, recorded steps, automated parameters and the
    /// running tempo reach the document and the host session; with one
    /// open, they are left for it.
    #[test]
    fn headless_sync_keeps_the_session_current() {
        use geist_engine::binding::{Message, id};
        let b = Bridge::default();
        let params = Arc::new(crate::GeistParams::default());
        let _ = b.params.set(Arc::downgrade(&params));
        *lock(&b.doc) = Some(doc(120));
        lock(&b.state).step_writes.reserve(16);
        lock(&b.state).automation.reserve(16);
        let data = geist_engine::pattern::StepData { on: true, ..geist_engine::pattern::StepData::off() };
        let push = |b: &Bridge| {
            let mut s = lock(&b.state);
            s.step_writes.push(geist_engine::StepWrite { engine: 0, pattern: 1, track: 2, step: 3, data: data.clone() });
            s.automation.push(Message::new(id::SET_MASTER_CHANNEL_GAIN, &[], &[0.25]));
            s.tempo = 133.0;
        };
        b.editor_open.store(true, Ordering::Release);
        push(&b);
        b.headless_sync();
        assert_eq!(lock(&b.state).step_writes.len(), 1, "left for the open editor");
        assert!(params.session.read().unwrap().is_none());
        b.editor_open.store(false, Ordering::Release);
        b.headless_sync();
        assert!(lock(&b.state).step_writes.is_empty() && lock(&b.state).automation.is_empty());
        assert!(lock(&b.state).step_writes.capacity() >= 16 && lock(&b.state).automation.capacity() >= 16, "the audio thread's room stays");
        let session = params.session.read().unwrap().clone().expect("session written");
        let saved = Node::parse(&session.doc).unwrap();
        assert_eq!(saved.child("TimeInfo", "TimeInfo").and_then(|t| t.f32("Tempo")), Some(133.0));
        let step = saved
            .child("Engine", "Engine0")
            .and_then(|e| e.child("StepPattern", "StepPattern1"))
            .and_then(|p| p.child("Track", "Track2"))
            .and_then(|t| t.child("Step", "S3"))
            .map(|s| s.attr("State").map(str::to_string));
        assert_eq!(step, Some(Some("true".into())));
        assert!(saved.children.iter().any(|c| c.class == "MasterChannel"), "the automated gain is in the document");
        // Nothing new: the session is left alone.
        *params.session.write().unwrap() = None;
        b.headless_sync();
        assert!(params.session.read().unwrap().is_none());
    }

    /// Engines the audio thread retires are freed by the worker.
    #[test]
    fn retired_engines_are_freed_off_the_audio_thread() {
        let b = Arc::new(Bridge::default());
        b.sample_rate.store(44100f32.to_bits(), Ordering::Relaxed);
        b.rebuild(doc(100), None);
        lock(&b.retired).push(Box::new(Geist::empty(44100.0, Default::default())));
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
        while !lock(&b.retired).is_empty() {
            assert!(std::time::Instant::now() < deadline, "retired engine not freed");
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
}
