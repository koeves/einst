//! The v2 editor: the installed skin (via `dreist-gui`) assembled as
//! [re:757608] does, with the the original panels bound to a
//! [`Model`] and emitting [`Command`]s for the engine. See
//! the dreist-re GUI notes.

mod accel;
pub mod app;
pub mod browser;
pub mod browsericons;
pub mod command;
mod conv;
mod convertor;
mod cursors;
pub mod devices;
mod dragdrop;
mod editbind;
mod editor;
pub mod filefind;
mod front;
mod fxchain;
mod fxmove;
mod fxplot;
mod fxtext;
pub mod guru;
mod header;
/// The standalone MIDI clock state the plugin sets (MIDI indicator).
pub use header::{CLOCK_LAST_MS, CLOCK_SYNC, now_ms};
mod inspector;
mod learn;
mod metadata;
mod meters;
mod midiconsole;
mod midimap;
mod mixer;
mod modal;
pub mod model;
mod multipad;
pub mod options;
mod padmenu;
mod pads;
mod patmenu;
mod pattern;
mod popups;
pub mod project;
mod retro;
mod sampler;
mod scenes;
mod search;
mod slicer;
mod song;
mod swing;
mod tabs;
mod timeline;
mod tooltips;
mod transmod;
mod transmodvis;
mod trigger;
mod undo;
mod velsplit;
mod zoomslider;

use dreist_gui::app::{App, UiEvent};
use dreist_gui::{Ctrl, Skin};

pub use command::Command;
use editor::engine_menu::EngineAction;
pub use model::Model;

/// What the file dialogs are asked for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Dialog {
    OpenProject,
    SaveProject,
    ExportAudio,
    /// Missing files: Auto Find start folder.
    ChooseFolder(Option<std::path::PathBuf>),
    /// Source list › Export to folder ([re:e8becb]): the
    /// folder to copy the shortlist into (from the desktop).
    ExportFolder,
    /// Missing files: Manual, "Please locate <file>" from a folder.
    LocateFile {
        title: String,
        start: Option<std::path::PathBuf>,
    },
    /// Extract groove: where to save the `.swingtemplate`.
    SaveSwingTemplate,
    /// Pad menu › Save layer sample...: where to write the copy.
    /// Default file name.
    SaveSample(String),
    /// FX chain menu › Save preset: a `.fxchain`.
    SaveChainPreset,
    /// Effect menu › Save preset: a `.fxpreset` for the effect type.
    SaveEffectPreset(usize),
    /// File menu › Open Engine / Kit / others: a part file.
    OpenPart(project::Part),
    /// File menu › Save … As: a part file.
    SavePart(project::Part),
    /// SAVE › Save current pattern as MIDI: a `.mid`.
    SaveMidi,
    /// LOAD › Load pattern from MIDI: a `.mid`.
    OpenMidi,
    /// Swing menu › Import Guru shuffle: a Guru 1 `.txt` template.
    OpenGuruSwing,
    /// LOAD › Load Guru... ▸: a file of that kind.
    OpenGuru(guru::Kind),
    /// Swing menu › Import MIDI template: a `.mid` ("Please choose the
    /// file(s) you wish to import").
    OpenMidiTemplate,
    /// Preferences › Browse: the new documents folder, starting at the
    /// current one.
    ChooseDocuments(std::path::PathBuf),
}

/// The product name shown in the UI (the installed profile's).
pub fn product() -> &'static str {
    dreist_formats::profile::get().product
}

/// File menu items ([re:4139f8] [re:42ba5f]).
#[derive(Debug, Clone)]
enum FileAction {
    None,
    OpenPreset,
    GuruMenu,
    OpenGuru(guru::Kind),
    ExportAudio,
    Open(project::Part),
    SavePreset { as_new: bool },
    Save { part: project::Part, as_new: bool },
    SaveMidi,
    OpenMidi,
    ToggleEmbedSamples,
}

/// [re:b86a03] [re:2761f5]: five entries,
/// newest first, kept as options `RecentDoc0..4`.
const RECENT: usize = 5;

pub fn recent_presets() -> Vec<std::path::PathBuf> {
    let o = options::Options::load();
    (0..RECENT).map(|i| o.string(&format!("RecentDoc{i}"), "")).filter(|v| !v.is_empty()).map(std::path::PathBuf::from).collect()
}

/// [re:bc555c]: move `path` to the front, keep five.
pub fn add_recent_preset(path: &std::path::Path) {
    let mut list = recent_presets();
    list.retain(|p| p != path);
    list.insert(0, path.to_path_buf());
    list.truncate(RECENT);
    let mut o = options::Options::load();
    for i in 0..RECENT {
        o.set(&format!("RecentDoc{i}"), list.get(i).map(|p| p.to_string_lossy().into_owned()).unwrap_or_default());
    }
    o.save();
}

pub type DialogFn = Box<dyn FnMut(Dialog) -> Option<std::path::PathBuf> + Send>;

pub struct Editor {
    /// The standalone app: its menu bar carries the keyboard shortcuts, so
    /// they work whatever the keyboard options say (`accel`).
    pub menu_bar: bool,
    /// The window is closing: the host link's last call (hand the
    /// document back now).
    pub closing: bool,
    /// Files being dragged in from outside the window.
    pub(crate) external_drag: Option<Vec<std::path::PathBuf>>,
    /// Pattern key / engine selector drop highlights.
    pub(crate) drop_hl: dragdrop::DropHighlights,
    /// A drag dwelling over an engine selector or the pads, since when.
    pub(crate) drag_dwell: Option<(dragdrop::Dwell, std::time::Instant)>,
    pub app: App,
    /// Native file dialogs (set by the window host; `None` in tests).
    pub dialogs: Option<DialogFn>,
    /// Where the project was last opened/saved.
    pub project_path: Option<std::path::PathBuf>,
    /// Undo / redo steps (document snapshots).
    history: undo::History,
    /// Undo (false) / redo (true) history menu open.
    history_menu: Option<bool>,
    /// A knob's right-click menu: the control and the items' actions.
    /// An open engine selector menu (anchor, engine, item actions); the
    /// "Reset all engines" confirmation; the engine being renamed.
    engine_menu: Option<(Vec<usize>, usize, Vec<EngineAction>)>,
    /// A typed knob edit in progress: the knob, its id and the conversion.
    knob_edit: Option<(Vec<usize>, String, i32, dreist_formats::params::Curve)>,
    /// The open File menu level (main menu or a submenu) and its actions.
    file_menu: Option<Vec<FileAction>>,
    /// The main menu (`GridButton`) is open.
    main_menu: bool,
    /// Last file per part type and engine ([re:4196af] dialog paths): what
    /// "Save Engine" / "Save Kit" / … write to without asking.
    part_paths: std::collections::HashMap<(&'static str, usize), std::path::PathBuf>,
    /// Undo name of the last drop (the original's undo texts).
    drop_label: String,
    pub model: Model,
    /// Commands produced since the last `take_commands`.
    pub outbox: Vec<Command>,
    pads: pads::Pads,
    front: front::Front,
    sampler: sampler::Sampler,
    header: header::Header,
    editor_tabs: Option<tabs::Tabs>,
    utility_tabs: Option<tabs::Tabs>,
    pattern: Option<pattern::PatternPanel>,
    inspector: inspector::Inspector,
    mixers: Vec<mixer::Mixer>,
    scenes: Option<scenes::Scenes>,
    song: Option<song::SongPanel>,
    browser: Option<browser::Browser>,
    transmod: transmod::Transmod,
    midimap: Option<midimap::MidiMap>,
    slicer: slicer::Slicer,
    padmenu: padmenu::PadMenu,
    /// The open layer row menu: its anchor and (engine, pad, layer).
    layer_menu: Option<(Vec<usize>, (usize, usize, usize))>,
    pad_row_menu: Option<(Vec<usize>, (usize, usize))>,
    /// The sampler's preview output menu is open on this control.
    preview_out_menu: Option<Vec<usize>>,
    /// The Grid menu's reset question is open.
    reset_confirm: bool,
    /// Samples changed on disk.
    watch: editor::changed::Watch,
    /// The sampler's overwrite question is open: anchor, request, file.
    sampler_overwrite: Option<(Vec<usize>, sampler::Request, std::path::PathBuf)>,
    filefind: filefind::FileFind,
    /// Standalone audio / MIDI settings panels.
    pub devices: devices::Devices,
    midiconsole: midiconsole::MidiConsole,
    trigger: trigger::TriggerPanel,
    metadata: metadata::MetaData,
    retro: retro::PatternRetro,
    convertor: convertor::Convertor,
    patmenu: patmenu::PatMenu,
    clipboard: padmenu::Clipboard,
    modals: modal::Modals,
    learn: learn::Learn,
    /// The popup buttons `popups::sync` labels.
    popup_paths: dreist_gui::app::PathCache,
}

#[derive(Debug)]
pub struct EditorError(pub String);

impl std::fmt::Display for EditorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for EditorError {}

fn err(e: impl std::fmt::Display) -> EditorError {
    EditorError(e.to_string())
}

impl Editor {
    /// Show or hide the browser column ([`front::set_browser_visible`]).
    pub fn set_browser_hidden(&mut self, hidden: bool) {
        self.front.browser_hidden = hidden;
        front::set_browser_visible(&mut self.app, !hidden);
    }

    /// Build the editor from the installed resources (`theme`, e.g.
    /// "Einst"): the v1 layout (`front`) with its pages.
    pub fn new(theme: &str, model: Model) -> Result<Editor, EditorError> {
        let mut skin = Skin::load(&Skin::resources_dir(), theme).map_err(err)?;
        let mut root = front::root(&mut skin)?;
        let header = header::Header::build(&root);
        let pads = pads::Pads::build(&mut root, &model)?;
        let editor_tabs = tabs::Tabs::new(&root, "MainTabs", "MainTabButtons");
        let utility_tabs = tabs::Tabs::new(&root, "BrowserTabs", "BrowserTabButtons");
        let mut pattern = None;
        let mut song = None;
        let mut mixers = Vec::new();
        let mut inspector = None;
        let mut scenes = None;
        let mut sampler = sampler::Sampler::default();
        if let Some(t) = &editor_tabs {
            for id in 0..front::PAGES.len() {
                let page = t.add_page(&mut root, id);
                if let Some(k) = [mixer::Kind::Layer, mixer::Kind::Pad, mixer::Kind::Engine, mixer::Kind::Global].into_iter().find(|k| k.page() == id) {
                    mixers.push(mixer::Mixer::build(&mut skin, &mut root, &page, k)?);
                    continue;
                }
                match id {
                    front::PATTERN_PAGE => pattern = Some(pattern::PatternPanel::build(&mut skin, &mut root, &page)?),
                    front::SONG_PAGE => song = Some(song::SongPanel::build(&mut skin, &mut root, &page)?),
                    front::SCENE_PAGE => scenes = Some(scenes::Scenes::build(&mut skin, &mut root, &page)?),
                    front::PADS_PAGE => inspector = Some(inspector::Inspector::build(&mut skin, &mut root, Some(&page))?),
                    front::SAMPLER_PAGE => sampler = sampler::Sampler::build(&mut skin, &mut root, &page)?,
                    _ => front::mount_page(&mut skin, &mut root, &page, id)?,
                }
            }
        }
        let inspector = match inspector {
            Some(i) => i,
            None => inspector::Inspector::build(&mut skin, &mut root, None)?,
        };
        let mut browser = None;
        let mut midimap = None;
        if let Some(t) = &utility_tabs {
            let page = t.add_page(&mut root, front::BROWSER_PAGE);
            browser = Some(browser::Browser::build(&mut skin, &mut root, &page)?);
            let page = t.add_page(&mut root, front::MAPPING_PAGE);
            midimap = Some(midimap::MidiMap::build(&mut skin, &mut root, &page)?);
        }
        let transmod = transmod::Transmod::build(&mut skin, &mut root, None)?;
        // MIDI learn overlay (drawn over everything, never hit).
        root.children.push(Ctrl {
            class: "CtrlPanel".into(),
            name: "LearnOverlay".into(),
            overload: "LearnOverlay".into(),
            style: "InvisibleStyle".into(),
            rect: dreist_gui::Rect::new(0, 0, root.rect.w, root.rect.h),
            zorder: 3000,
            visible: false,
            enabled: true,
            passive: true,
            ..Ctrl::default()
        });
        let slicer = slicer::Slicer::build(&mut skin, &mut root)?;
        let modals = modal::Modals::build(&mut skin, &mut root)?;
        let filefind = filefind::FileFind::build(&mut skin, &mut root)?;
        let devices = devices::Devices::build(&mut skin, &mut root)?;
        let midiconsole = midiconsole::MidiConsole::build(&mut skin, &mut root)?;
        let trigger = trigger::TriggerPanel::build(&mut skin, &mut root)?;
        let metadata = metadata::MetaData::build(&mut skin, &mut root)?;
        let convertor = convertor::Convertor::build(&mut skin, &mut root)?;
        let mut ed = Editor {
            menu_bar: false,
            closing: false,
            app: App::new(skin, root),
            dialogs: None,
            project_path: None,
            history: undo::History::default(),
            history_menu: None,
            engine_menu: None,
            knob_edit: None,
            file_menu: None,
            main_menu: false,
            part_paths: Default::default(),
            drop_label: String::new(),
            external_drag: None,
            drop_hl: Default::default(),
            drag_dwell: None,
            model,
            outbox: Vec::new(),
            pads,
            front: front::Front::default(),
            sampler,
            header,
            editor_tabs,
            utility_tabs,
            pattern,
            inspector,
            mixers,
            scenes,
            song,
            browser,
            transmod,
            midimap,
            slicer,
            modals,
            learn: learn::Learn::default(),
            popup_paths: Default::default(),
            padmenu: padmenu::PadMenu::default(),
            layer_menu: None,
            pad_row_menu: None,
            preview_out_menu: None,
            reset_confirm: false,
            watch: Default::default(),
            sampler_overwrite: None,
            filefind,
            devices,
            midiconsole,
            trigger,
            metadata,
            retro: retro::PatternRetro::default(),
            convertor,
            patmenu: patmenu::PatMenu::default(),
            clipboard: padmenu::Clipboard::default(),
        };
        // [re:427435]: shift alone drags knobs ten times finer.
        ed.app.fine_drag = true;
        ed.app.ctrl_click_right = true;
        // `LastEditorScaleFactor` (Preferences › UI scaling), applied when
        // the window opens. The window keeps v1's fixed size.
        ed.app.ui_scale = (ed.modals.options.int("LastEditorScaleFactor", 100).clamp(50, 300) as f32) / 100.0;
        // [re:6a1c94] (messaging initialized).
        let startup = sampler::Sampler::startup_messages(&mut ed.model);
        // The document follows, as the engine's does (`mirror_message`).
        for c in &startup {
            ed.model.apply(c);
        }
        ed.outbox.extend(startup);
        // `e_registryOptionsChanged` [ForcedLayersFallThrough,
        // SendActiveSensing] ([re:3fca26] posts it on
        // Confirm; the engine also reads it when built).
        let o = &ed.modals.options;
        let reg = command::msg(dreist_engine::binding::id::REGISTRY_OPTIONS_CHANGED, &[], &[o.bool("ForcedLayersFallThrough", false) as i32 as f32, o.bool("SendActiveSensing", false) as i32 as f32]);
        ed.outbox.push(reg);
        // MIDI out options (`e_registryMIDIOptionsChanged` from the options).
        if let Some(m) = &ed.midimap {
            ed.outbox.push(m.midi_options_message());
        }
        ed.sync();
        Ok(ed)
    }

    /// Turn MIDI learn mode on or off (header LearnButton).
    pub fn set_learn(&mut self, on: bool) {
        self.learn.active = on;
        self.learn.target = None;
        self.inspector.learn_buttons(&mut self.app, on);
        self.sync();
    }

    /// Show the About or Preferences panel.
    pub fn show_modal(&mut self, which: &str) {
        // A sample missing-files list (screenshots).
        if which.eq_ignore_ascii_case("filefind") {
            let items = (0..3).map(|i| filefind::Missing { path: std::path::PathBuf::from(format!("/Missing/Kick {i:02}.wav")), layers: vec![(0, i, 0)] }).collect();
            self.filefind.show(items);
            self.sync();
            return;
        }
        // The layer meta data editor beside layer row 1 (screenshots).
        if which.eq_ignore_ascii_case("metadata") {
            fn row(c: &Ctrl, path: &mut Vec<usize>) -> bool {
                for (i, k) in c.children.iter().enumerate() {
                    path.push(i);
                    if k.overload == "LayerRow" && k.visible || row(k, path) {
                        return true;
                    }
                    path.pop();
                }
                false
            }
            let mut p = Vec::new();
            if row(&self.app.root, &mut p) {
                self.metadata.show(&mut self.app, &self.model, &p);
            }
            self.sync();
            return;
        }
        if which.eq_ignore_ascii_case("console") {
            self.midiconsole.show();
            self.sync();
            return;
        }
        if which.eq_ignore_ascii_case("audio") || which.eq_ignore_ascii_case("midi") {
            self.devices.show(if which.eq_ignore_ascii_case("audio") { devices::Panel::Audio } else { devices::Panel::Midi });
            self.sync();
            return;
        }
        let k = if which.eq_ignore_ascii_case("about") {
            modal::Kind::About
        } else if which.eq_ignore_ascii_case("export") {
            modal::Kind::Export
        } else {
            modal::Kind::Preferences
        };
        self.modals.export_outputs = dreist_engine::export::assigned_outputs(&self.model.doc);
        self.modals.show(&mut self.app, k);
        self.sync();
    }

    #[cfg(test)]
    fn pads_matrix(&self) -> Option<Vec<usize>> {
        Some(self.pads.matrix().to_vec())
    }

    /// Per-frame animations.
    pub fn animate(&mut self) {
        for ev in self.app.animate() {
            self.handle(&ev);
        }
        if self.transmod.animate_flashes(&mut self.app) {
            self.app.dirty = true;
        }
        if self.pads.animate(&mut self.app, &self.model) {
            self.app.dirty = true;
        }
        self.animate_drop_highlights();
        if self.animate_drag_dwell() {
            self.app.dirty = true;
        }
        if self.filefind.scanning() && self.filefind.tick(&mut self.app, &mut self.model) {
            self.outbox.push(self.model.rebuild_command());
            self.sync();
        }
        if self.modals.tick(&mut self.app) {
            self.app.dirty = true;
        }
        self.watch_samples();
    }

    /// Open the slicer on an audio file ([re:401f49]).
    pub fn open_slicer(&mut self, file: &std::path::Path) -> bool {
        let ok = self.open_slicer_from_browser(file);
        self.sync();
        ok
    }

    /// Start slicing `file`; with the browser's Auto load on, the slices
    /// and the pattern go to the engine at once (and follow every edit).
    pub(crate) fn open_slicer_from_browser(&mut self, file: &std::path::Path) -> bool {
        let auto = self.browser.as_ref().is_some_and(|b| b.auto_load());
        if !self.slicer.start(&self.model, file) {
            return false;
        }
        self.slicer.auto_load = auto;
        if auto {
            self.slicer.extract(&mut self.model, slicer::Extract::Both, &mut self.outbox);
        }
        true
    }

    /// Replace the document (project loaded or reset).
    pub fn set_document(&mut self, doc: dreist_formats::Node) {
        self.retro.clear();
        let keep = (self.model.pad, self.model.name.take(), self.model.samples.take(), self.model.editor_tab, self.model.utility_tab);
        self.model = Model::new(doc);
        (self.model.pad, self.model.name, self.model.samples, self.model.editor_tab, self.model.utility_tab) = keep;
        self.sync();
    }
}

#[cfg(feature = "window")]
pub mod window;

pub mod screens;
#[cfg(test)]
mod screens_test;

#[cfg(test)]
mod tests;
