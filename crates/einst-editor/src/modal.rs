//! Modal panels over the editor: About ([re:3d40d0], [re:323f99]),
//! Preferences ([re:3a9b5d] / [re:6039df]) and Export ([re:0dfe90]). Dialogs
//! show and close at once, centred over a shaded snapshot ([re:24b9de]);
//! Preferences and Export sit in a titled frame ([re:2a01cd]).
//!
//! Preferences controls ↔ options (on showing): InvertButton
//! `GuiDragDownToZoom`, InvertXButton `GuiDragLeftToScrollRight`,
//! QuantizeREXButton `AllocatorQuantizeREX`, PreFadeButton
//! `AuxesArePreFade`, MultiButton `PatternEditorRightClickMode`,
//! ForceModeButton `ForcedLayersFallThrough`, ResetLayersButton
//! `AllocatorClearLayersOnLoad`, ActiveSensingButton `SendActiveSensing`,
//! RetardButton `SequencerRetardation`, RestoreOutputsButton
//! `AllocatorRestoreOutputs` (default on), AutoAssign `AutoMIDICC` (on);
//! MIDIModeGroup `MIDINumbering` (child ids, default 1); MetroType
//! `MetronomeSound` (metronome folders, "Default"); ShadingMode
//! `PatternEditorShadingMode` (Fixed, Adaptive, Cyclic, Off — [re:c192c5]);
//! ThemeCombo (stylesheets here; v1 lists `Theme_*` files Einst does not
//! ship); UndoStages `MaxUndoSteps` (at least 20, more than 50 asks);
//! DocumentsPath `LocalDocumentsPath` (shown only). The DirectWrite and
//! Launchpad rows are hidden, as v1 does with Launchpad off ([re:3a9b5d]).
//!
//! [re:3fca26]: every change is written at once; Finished closes and, when
//! a toggle changed, tells the engine. Browse asks for a folder ("Please
//! choose a new <product> documents folder"), Reset asks "Are you sure you
//! want to reset the Documents path to %s?"; a new path, MIDI numbering,
//! metronome or undo stages alert "Changes will be applied the next time
//! you start <product>".
//!
//! v2-only, dormant (the v1 skin has no such controls): KeyboardButton
//! `RespondToKeyboard`, ExtendedKeyboardButton `RespondToExtendedKeyboard`,
//! RemMod `RemoveModBehavour`, UIScaling `LastEditorScaleFactor`, UIStyle.

use dreist_gui::app::{App, MouseKind, UiEvent};
use dreist_gui::paint::{Painter, RectF};
use dreist_gui::render::Renderer;
use dreist_gui::{Colour, Ctrl, Rect, Skin};

use crate::options::Options;
use crate::{EditorError, err};

pub const TOGGLES: [(&str, &str, bool); 13] = [
    ("QuantizeREXButton", "AllocatorQuantizeREX", false),
    ("PreFadeButton", "AuxesArePreFade", false),
    ("MultiButton", "PatternEditorRightClickMode", false),
    ("ForceModeButton", "ForcedLayersFallThrough", false),
    ("ResetLayersButton", "AllocatorClearLayersOnLoad", false),
    ("ActiveSensingButton", "SendActiveSensing", false),
    ("RetardButton", "SequencerRetardation", false),
    ("RestoreOutputsButton", "AllocatorRestoreOutputs", true),
    ("KeyboardButton", "RespondToKeyboard", true),
    ("ExtendedKeyboardButton", "RespondToExtendedKeyboard", false),
    ("AutoAssign", "AutoMIDICC", true),
    // [re:079fd4]: wave displays' drag zoom / scroll directions.
    ("InvertButton", "GuiDragDownToZoom", false),
    ("InvertXButton", "GuiDragLeftToScrollRight", false),
];
pub const SHADING: [&str; 4] = ["Fixed", "Adaptive", "Cyclic", "Off"];
pub const REMOVE_BEHAVIOUR: [&str; 4] = ["Prompt", "AlwaysClear", "NeverClear", "ClearOnModifier"];
pub const SCALINGS: [f32; 11] = [1.0, 1.1, 1.2, 1.25, 1.35, 1.5, 1.75, 2.0, 2.25, 2.5, 3.0];
pub const UNDO_DEFAULT: i64 = 20;
/// v1's fewest undo stages.
pub const UNDO_MIN: i64 = 20;

/// The default stylesheet ("v2") as the style menu shows it.
pub const DEFAULT_STYLE_LABEL: &str = "Default";

fn style_label(name: &str) -> &str {
    if name == dreist_gui::skin::default_theme() { DEFAULT_STYLE_LABEL } else { name }
}

/// The About panel's text.
/// The About text (the installed profile's).
pub fn about() -> &'static str {
    crate::app::ABOUT
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    About,
    Preferences,
    Export,
}

/// [re:8e3557] normalisation items (`NormalizeComboBox`).
pub const NORMALIZE: [&str; 3] = ["Prompt if clipping detected", "Normalize if clipping detected", "Always Normalize"];

/// [re:751e2d]: an output location's name (`Master`, `Sub output n`).
/// Where the export prompt starts ([re:85bfd4]): option `LastSavePath`
/// (the last export's file or folder) when it exists, else the documents
/// folder.
pub fn last_save_location() -> std::path::PathBuf {
    let last = std::path::PathBuf::from(Options::load().string("LastSavePath", ""));
    let dir = if last.is_file() { last.parent().map(std::path::Path::to_path_buf).unwrap_or_default() } else { last };
    if !dir.as_os_str().is_empty() && dir.is_dir() { dir } else { crate::options::documents_dir() }
}

pub fn export_output_name(o: usize) -> String {
    if o == 0 { "Master".into() } else { dreist_engine::export::output_name(o) }
}

/// [re:751e2d]: the stored single output, by name; `Master` unless it is
/// one of `outputs`.
pub fn export_single_output(o: &Options, outputs: &[usize]) -> usize {
    let name = o.string("ExportSingleOutput", "Master");
    outputs.iter().copied().find(|&k| export_output_name(k) == name).unwrap_or(0)
}

/// [re:751e2d]: C `atoi` of the tail text, held to 0..=60 seconds.
pub fn export_tail(o: &Options) -> f64 {
    let t = o.string("ExportTailSecs", "0");
    let t = t.trim_start();
    let n: String = t.char_indices().take_while(|(i, c)| c.is_ascii_digit() || (*i == 0 && (*c == '-' || *c == '+'))).map(|(_, c)| c).collect();
    (n.parse::<i64>().unwrap_or(0) as f64).clamp(0.0, 60.0)
}

/// Export panel settings, kept in the options file ([re:751e2d]).
pub fn export_options(o: &Options, engine: usize) -> dreist_engine::export::Options {
    use dreist_engine::export::{Files, Normalize, Source};
    dreist_engine::export::Options {
        source: match o.int("ExportSource", 0).clamp(0, 2) {
            0 => Source::AllEngines,
            1 => Source::Engine(engine),
            _ => Source::Song,
        },
        files: match o.int("ExportOutputType", 0).clamp(0, 2) {
            1 => Files::Single(export_single_output(o, &(0..=dreist_engine::SUB_OUTPUTS).collect::<Vec<_>>())),
            2 => Files::AllOutputs,
            _ => Files::Mixdown,
        },
        normalize: [Normalize::Prompt, Normalize::IfClipping, Normalize::Always][o.int("ExportNormalizeMode", 0).clamp(0, 2) as usize],
        tail: export_tail(o),
        looped: o.bool("ExportLoopMode", false),
        auto_crop: o.bool("ExportAutoCropMode", false),
        render: crate::options::render_options_from(o),
    }
}

pub struct Modals {
    /// A style chosen in Preferences, for the editor to apply now.
    pub style_changed: Option<String>,
    /// Preferences were confirmed: the editor tells the engine
    /// (`e_registryOptionsChanged`).
    pub options_confirmed: bool,
    about: Option<Vec<usize>>,
    prefs: Option<Vec<usize>>,
    export: Option<Vec<usize>>,
    /// Export panel's Export button was pressed (the editor asks for a path).
    pub export_requested: bool,
    /// The outputs the single-output menu offers (set by the editor on
    /// showing the Export panel).
    pub export_outputs: Vec<usize>,
    pub open: Option<Kind>,
    /// The panel on screen.
    shown: Option<Kind>,
    /// A toggle changed this showing: Confirm tells the engine.
    dirty: bool,
    pub options: Options,
    /// A UI scale tried from Preferences: the old factor and
    /// `LastEditorSize`, when it was applied, whether the prompt is open.
    scale_trial: Option<(i64, String, std::time::Instant, bool)>,
    /// The running export job ([re:ece5dd]).
    pub export_job: Option<crate::command::ExportProgress>,
    /// Its prompt or failure alert is open.
    export_alert: bool,
    /// Preferences › Browse was pressed (the editor runs the folder dialog).
    pub browse_requested: bool,
    /// The reset question or the restart alert is open.
    docs_prompt: Option<DocsPrompt>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum DocsPrompt {
    Reset,
    Alert,
    /// More than 50 undo stages: Yes keeps them, No goes back to 20.
    Undo(Vec<usize>, i64),
    /// The minimum alert: 20 is written once it closes.
    UndoMin(Vec<usize>),
}

pub fn docs_restart() -> String {
    format!("Changes will be applied the next time you start {}", crate::product())
}
/// [re:3fca26]: the Browse dialog's title.
#[cfg_attr(not(feature = "window"), allow(dead_code))]
pub fn docs_browse_title() -> String {
    format!("Please choose a new {} documents folder", crate::product())
}

/// v2 only (no UI scaling in v1): the question asked 2 s after a new UI
/// scale was applied.
pub const KEEP_SCALE: &str = "Do you want to keep this UI scaling ratio?";

/// v1's dialog shade over the window (`SynthEditorBase::AddDialogControl`,
/// [re:24b9de]).
pub(crate) const SHADE: Colour = Colour::rgba(32, 32, 32, 210);

/// Show a dialog as v1 does ([re:24b9de]): at once (no animation), centred in
/// the window on both axes.
pub(crate) fn place(app: &mut App, overlay: &[usize]) {
    let Some((w, h)) = app.ctrl(overlay).map(|c| (c.rect.w, c.rect.h)) else { return };
    let mut p = overlay.to_vec();
    p.push(0);
    let Some(r) = app.ctrl(&p).map(|c| c.rect) else { return };
    let q = Rect::new(w / 2 - r.w / 2, h / 2 - r.h / 2, r.w, r.h);
    if let Some(c) = app.ctrl_mut(&p) {
        c.set_rect(q);
    }
}

/// One dialog's shown state (for dialogs other than the `Modals` trio).
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Slide {
    shown: bool,
}

impl Slide {
    /// Show / hide at once to follow `open`; whether the overlay shows.
    pub fn sync(&mut self, app: &mut App, overlay: &[usize], open: bool) -> bool {
        if open && !self.shown {
            place(app, overlay);
        }
        self.shown = open;
        if let Some(o) = app.ctrl_mut(overlay) {
            o.visible = open;
        }
        open
    }
}

/// A dialog overlay from `rel` ([re:2a01cd]): with a `title`, the panel sits in
/// a `ModalPanelBackgroundStyle` frame 10 px wider and 35 px taller, at
/// (5, 29), under a `ModalTitleLabelStyle` label at (5, 0) as wide as the
/// panel and 30 px high.
pub(crate) fn overlay(skin: &mut Skin, root: &mut Ctrl, rel: &str, name: &str) -> Result<Option<Vec<usize>>, EditorError> {
    overlay_titled(skin, root, rel, name, "")
}

pub(crate) fn overlay_titled(skin: &mut Skin, root: &mut Ctrl, rel: &str, name: &str, title: &str) -> Result<Option<Vec<usize>>, EditorError> {
    let (w, h) = (root.rect.w, root.rect.h);
    // A dialog the UI set does not have stays closed.
    let Some(rel) = skin.find(rel) else { return Ok(None) };
    let Some(mut panel) = skin.load_skin(&rel).map_err(err)?.children.into_iter().next() else { return Ok(None) };
    if !title.is_empty() {
        let (cw, ch) = (panel.rect.w, panel.rect.h);
        panel.set_rect(Rect::new(5, 29, cw, ch));
        let label = Ctrl {
            class: "CtrlLabel".into(),
            name: "DialogTitle".into(),
            style: "ModalTitleLabelStyle".into(),
            text: title.into(),
            rect: Rect::new(5, 0, cw, 30),
            visible: true,
            enabled: true,
            ..Ctrl::default()
        };
        panel = Ctrl {
            class: "CtrlPanel".into(),
            name: "DialogFrame".into(),
            style: "ModalPanelBackgroundStyle".into(),
            rect: Rect::new(0, 0, cw + 10, ch + 35),
            visible: true,
            enabled: true,
            children: vec![panel, label],
            ..Ctrl::default()
        };
    }
    let (pw, ph) = (panel.rect.w.min(w), panel.rect.h.min(h));
    panel.set_rect(Rect::new((w - pw) / 2, (h - ph) / 2, pw, ph));
    let mut o = Ctrl {
        class: "CtrlPanel".into(),
        name: name.into(),
        overload: "ModalOverlay".into(),
        style: "InvisibleStyle".into(),
        rect: Rect::new(0, 0, w, h),
        zorder: 1001,
        visible: false,
        enabled: true,
        ..Ctrl::default()
    };
    o.children.push(panel);
    root.children.push(o);
    Ok(Some(vec![root.children.len() - 1]))
}

fn camel(s: &str) -> String {
    crate::transmod::camel_words(s)
}

impl Modals {
    pub fn build(skin: &mut Skin, root: &mut Ctrl) -> Result<Modals, EditorError> {
        // [re:3d40d0] untitled; [re:6039df] "<product> Preferences";
        // [re:0dfe90] "Export Audio".
        let about = overlay(skin, root, "AboutPanel.skin", "AboutOverlay")?;
        let prefs = overlay_titled(skin, root, "OptionsPanel.skin", "PreferencesOverlay", &format!("{} Preferences", crate::product()))?;
        let export = overlay_titled(skin, root, "ExportPanel.skin", "ExportOverlay", "Export Audio")?;
        // [re:0dfe90]: the progress label and bar show only while a job
        // runs ([re:9106dd]).
        if let Some(c) = export.as_ref().and_then(|p| {
            let mut c = &mut *root;
            for &i in p {
                c = c.children.get_mut(i)?;
            }
            Some(c)
        }) {
            for n in ["ProgressBar", "ProgressLabel"] {
                if let Some(k) = c.find_mut(n) {
                    k.visible = false;
                }
            }
        }
        // v1's experimental Windows text renderer and controller support
        // are not ported.
        if let Some(c) = prefs.as_ref().and_then(|p| {
            let mut c = &mut *root;
            for &i in p {
                c = c.children.get_mut(i)?;
            }
            Some(c)
        }) {
            for n in ["DirectWritePanel", "LaunchpadPanel"] {
                if let Some(k) = c.find_mut(n) {
                    k.visible = false;
                }
            }
        }
        let options = Options::load();
        crate::pattern::set_shading_mode(&options.string("PatternEditorShadingMode", SHADING[0]));
        Ok(Modals {
            style_changed: None,
            options_confirmed: false,
            about,
            prefs,
            export,
            export_requested: false,
            export_outputs: vec![0],
            open: None,
            shown: None,
            dirty: false,
            options,
            scale_trial: None,
            export_job: None,
            export_alert: false,
            browse_requested: false,
            docs_prompt: None,
        })
    }

    #[cfg(test)]
    pub fn prefs_path(&self) -> Option<Vec<usize>> {
        self.prefs.clone()
    }

    fn path(&self, k: Kind) -> Option<&Vec<usize>> {
        match k {
            Kind::About => self.about.as_ref(),
            Kind::Preferences => self.prefs.as_ref(),
            Kind::Export => self.export.as_ref(),
        }
    }

    fn metronomes(&self) -> Vec<String> {
        let dir = dreist_engine::metronome::default_metronomes_dir();
        let mut v: Vec<String> = std::fs::read_dir(dir).into_iter().flatten().flatten().filter(|e| e.path().is_dir()).map(|e| e.file_name().to_string_lossy().into_owned()).collect();
        v.sort();
        if v.is_empty() {
            v.push("Default".into());
        }
        v
    }

    fn styles(&self) -> Vec<String> {
        dreist_gui::Skin::themes()
    }

    pub fn show(&mut self, app: &mut App, k: Kind) {
        self.open = Some(k);
        self.dirty = false;
        if k == Kind::About {
            if let Some(o) = self.about.clone().and_then(|p| app.ctrl_mut(&p)) {
                // v1's panel leaves the text to code: a label in `AboutView`.
                if o.find("AboutLabel").is_none() {
                    if let Some(v) = o.find_mut("AboutView") {
                        let r = Rect::new(8, 8, v.rect.w - 16, v.rect.h - 16);
                        v.children.push(Ctrl {
                            class: "CtrlMultiLineLabel".into(),
                            name: "AboutLabel".into(),
                            style: "AboutLabelStyle".into(),
                            rect: r,
                            zorder: 1,
                            visible: true,
                            enabled: true,
                            ..Ctrl::default()
                        });
                    }
                }
                // v1's artwork band (the panel's top 262 px): Einst's own
                // wordmark.
                o.walk_mut(&mut |c: &mut Ctrl| {
                    if c.children.iter().any(|k| k.name == "AboutView") && !c.children.iter().any(|k| k.name == "AboutWordmark") {
                        c.children.push(Ctrl {
                            class: "CtrlLabel".into(),
                            name: "AboutWordmark".into(),
                            style: "AboutWordmarkStyle".into(),
                            text: "einst.".into(),
                            rect: Rect::new(0, 0, c.rect.w, 262),
                            zorder: 1,
                            visible: true,
                            enabled: true,
                            passive: true,
                            ..Ctrl::default()
                        });
                    }
                });
                o.walk_mut(&mut |c: &mut Ctrl| match c.name.as_str() {
                    "Version" | "VersionLabel" => c.text = format!("Version {}", env!("CARGO_PKG_VERSION")),
                    "DateLabel" => c.text = String::new(),
                    "AboutLabel" => c.text = about().into(),
                    _ => {}
                });
            }
        }
    }

    pub fn sync(&mut self, app: &mut App) {
        // v1 shows and removes dialogs at once.
        if self.shown != self.open {
            if let Some(p) = self.open.and_then(|k| self.path(k).cloned()) {
                place(app, &p);
            }
            self.shown = self.open;
        }
        let open = self.open;
        let shown = self.shown;
        for k in [Kind::About, Kind::Preferences, Kind::Export] {
            if let Some(o) = self.path(k).cloned().and_then(|p| app.ctrl_mut(&p)) {
                o.visible = shown == Some(k);
            }
        }
        if open == Some(Kind::Export) {
            self.sync_export(app);
            return;
        }
        if open != Some(Kind::Preferences) {
            return;
        }
        let s = self.options.clone();
        let styles = self.styles();
        let Some(p) = self.prefs.clone() else { return };
        let editing = app.is_editing();
        let Some(o) = app.ctrl_mut(&p) else { return };
        o.walk_mut(&mut |c: &mut Ctrl| {
            if let Some((_, key, d)) = TOGGLES.iter().find(|(n, _, _)| *n == c.name) {
                c.on = s.bool(key, *d);
                return;
            }
            match c.name.as_str() {
                "MIDIModeGroup" => {
                    let n = s.int("MIDINumbering", 1);
                    for b in &mut c.children {
                        b.on = b.id as i64 == n;
                    }
                }
                "MetroType" => c.text = s.string("MetronomeSound", "Default"),
                "ShadingMode" => c.text = s.string("PatternEditorShadingMode", SHADING[0]),
                "RemMod" => c.text = camel(&s.string("RemoveModBehavour", REMOVE_BEHAVIOUR[0])),
                "UIScaling" => c.text = format!("{} %", s.int("LastEditorScaleFactor", 100)),
                "UIStyle" | "ThemeCombo" => {
                    c.text = style_label(&s.string("StyleSheet", styles.iter().find(|n| n.as_str() == dreist_gui::skin::default_theme()).map(String::as_str).unwrap_or(""))).to_string()
                }
                "UndoStages" if !editing => c.text = s.int("MaxUndoSteps", UNDO_DEFAULT).max(UNDO_MIN).to_string(),
                "DocumentsPath" if !editing => c.text = s.string("LocalDocumentsPath", &default_documents()),
                _ => {}
            }
        });
    }

    /// [re:751e2d].
    fn sync_export(&mut self, app: &mut App) {
        let o = self.options.clone();
        let editing = app.is_editing();
        let Some(p) = self.export.clone() else { return };
        let Some(panel) = app.ctrl_mut(&p) else { return };
        let (src, ty) = (o.int("ExportSource", 0).clamp(0, 2), o.int("ExportOutputType", 0).clamp(0, 2));
        let single = export_output_name(export_single_output(&o, &self.export_outputs));
        panel.walk_mut(&mut |c: &mut Ctrl| match c.name.as_str() {
            "SourceButtonGroup" => c.children.iter_mut().enumerate().for_each(|(i, b)| b.on = i as i64 == src),
            "OutputTypeButtonGroup" => c.children.iter_mut().enumerate().for_each(|(i, b)| b.on = i as i64 == ty),
            "SingleOutputComboBox" => c.text = single.clone(),
            "NormalizeComboBox" => c.text = NORMALIZE[o.int("ExportNormalizeMode", 0).clamp(0, 2) as usize].into(),
            "TailTextBox" if !editing => c.text = format!("{:.3}", export_tail(&o)),
            "LoopButton" => c.on = o.bool("ExportLoopMode", false),
            "AutoCropButton" => c.on = o.bool("ExportAutoCropMode", false),
            _ => {}
        });
    }

    fn handle_export(&mut self, app: &mut App, ev: &UiEvent) -> bool {
        match ev {
            UiEvent::Clicked { path, name, on, .. } if self.mine(path) => {
                let parent = app.ctrl(&path[..path.len() - 1]).map(|c| c.name.clone()).unwrap_or_default();
                let i = *path.last().unwrap_or(&0) as i64;
                match (parent.as_str(), name.as_str()) {
                    ("SourceButtonGroup", _) => self.options.set("ExportSource", i),
                    ("OutputTypeButtonGroup", _) => self.options.set("ExportOutputType", i),
                    // [re:654f3c]: opening the output menu picks Single
                    // Output; it lists the assigned outputs.
                    (_, "SingleOutputComboBox") => {
                        self.options.set("ExportOutputType", 1);
                        app.open_menu(path, self.export_outputs.iter().map(|&k| export_output_name(k)).collect(), None);
                    }
                    (_, "NormalizeComboBox") => app.open_menu(path, NORMALIZE.iter().map(|s| s.to_string()).collect(), None),
                    (_, "LoopButton") => self.options.set("ExportLoopMode", *on),
                    (_, "AutoCropButton") => self.options.set("ExportAutoCropMode", *on),
                    // Cancel aborts a running job, else closes.
                    (_, "CancelButton") => match &self.export_job {
                        Some(j) => {
                            if let Ok(mut g) = j.lock() {
                                g.abort = true;
                            }
                        }
                        None => self.open = None,
                    },
                    (_, "StartButton") if self.export_job.is_none() => self.export_requested = true,
                    _ => return true,
                }
                self.options.save();
                true
            }
            UiEvent::MenuSelected { anchor, name, index } if self.mine(anchor) => {
                match name.as_str() {
                    "SingleOutputComboBox" => {
                        if let Some(&k) = self.export_outputs.get(*index) {
                            self.options.set("ExportSingleOutput", export_output_name(k));
                        }
                    }
                    "NormalizeComboBox" => self.options.set("ExportNormalizeMode", *index),
                    _ => {}
                }
                self.options.save();
                true
            }
            UiEvent::TextCommitted { path, name, text } if self.mine(path) && name == "TailTextBox" => {
                // [re:654f3c]: the text as typed (read back with `atoi`).
                self.options.set("ExportTailSecs", text.as_str());
                self.options.save();
                true
            }
            UiEvent::Mouse { path, .. } | UiEvent::Wheel { path, .. } | UiEvent::Clicked { path, .. } | UiEvent::ValueChanged { path, .. } => self.mine(path),
            _ => false,
        }
    }

    fn mine(&self, path: &[usize]) -> bool {
        self.open.and_then(|k| self.path(k)).is_some_and(|p| path.starts_with(p))
    }

    /// Menu items for a preferences popup.
    fn items(&self, name: &str) -> Option<Vec<String>> {
        Some(match name {
            "MetroType" => self.metronomes(),
            "ShadingMode" => SHADING.iter().map(|s| s.to_string()).collect(),
            "RemMod" => REMOVE_BEHAVIOUR.iter().map(|s| camel(s)).collect(),
            "UIScaling" => SCALINGS.iter().map(|f| format!("{} %", (f * 100.0).floor() as i32)).collect(),
            "UIStyle" | "ThemeCombo" => self.styles().iter().map(|n| style_label(n).to_string()).collect(),
            _ => return None,
        })
    }

    /// [re:9106dd] while the job runs: the progress
    /// label and bar show; a clipping question opens as an alert (Yes
    /// normalises); at the end a failure shows "Export failed. Error
    /// reason:\n%s" and the panel closes.
    fn tick_export(&mut self, app: &mut App) -> bool {
        let Some(job) = self.export_job.clone() else { return false };
        let (progress, prompt, result) = {
            let Ok(g) = job.lock() else { return false };
            (g.progress, g.prompt.clone().filter(|_| g.answer.is_none()), g.result.clone())
        };
        let panel = self.export.clone();
        let bar = panel.as_ref().and_then(|p| app.ctrl(p).and_then(|c| c.path_of("ProgressBar")).map(|q| [p.clone(), q].concat()));
        if let Some(c) = bar.as_ref().and_then(|b| app.ctrl_mut(b)) {
            c.visible = result.is_none();
            c.value = progress.clamp(0.0, 1.0);
        }
        let label = panel.as_ref().and_then(|p| app.ctrl(p).and_then(|c| c.path_of("ProgressLabel")).map(|q| [p.clone(), q].concat()));
        if let Some(c) = label.as_ref().and_then(|l| app.ctrl_mut(l)) {
            c.visible = result.is_none();
        }
        if let Some(q) = prompt {
            if !self.export_alert {
                self.export_alert = true;
                let anchor = bar.clone().unwrap_or_default();
                app.open_menu(&anchor, vec![q, String::new(), "Yes".into(), "No".into()], None);
            }
            return true;
        }
        if let Some(r) = result {
            self.export_job = None;
            if let Err(e) = r.filter_err() {
                self.export_alert = true;
                let anchor = bar.unwrap_or_default();
                app.open_menu(&anchor, vec![format!("Export failed. Error reason:\n{e}")], None);
            }
            self.open = None;
        }
        true
    }

    /// After 2 s of a tried UI scale, the keep question (an alert menu on
    /// the scaling popup). Returns whether it opened.
    pub fn tick(&mut self, app: &mut App) -> bool {
        if self.tick_export(app) {
            return true;
        }
        let Some((_, _, t, asked)) = &mut self.scale_trial else { return false };
        if *asked || t.elapsed().as_secs_f32() < 2.0 {
            return false;
        }
        *asked = true;
        let anchor = self.prefs.as_ref().and_then(|p| app.ctrl(p).and_then(|c| c.path_of("UIScaling")).map(|q| [p.clone(), q].concat())).unwrap_or_default();
        app.open_menu(&anchor, vec![KEEP_SCALE.to_string(), String::new(), "Yes".into(), "No".into()], None);
        true
    }

    /// The keep answer: "No" restores the old scale and size (options
    /// written back), anything else keeps the new one.
    fn keep_scale(&mut self, keep: bool) {
        let Some((old, old_size, _, _)) = self.scale_trial.take() else { return };
        if keep {
            return;
        }
        self.options.set("LastEditorScaleFactor", old);
        self.options.set("LastEditorSize", &old_size);
        self.options.save();
    }

    /// The documents path control (anchor for the alerts).
    fn docs_anchor(&self, app: &App) -> Vec<usize> {
        self.prefs.as_ref().and_then(|p| app.ctrl(p).and_then(|c| c.path_of("DocumentsPath")).map(|q| [p.clone(), q].concat())).unwrap_or_default()
    }

    /// [re:3fca26]: written straight away, then the restart alert.
    pub fn write_documents_path(&mut self, app: &mut App, path: &str) {
        self.options.set("LocalDocumentsPath", path);
        self.options.save();
        let anchor = self.docs_anchor(app);
        self.restart_alert(app, &anchor);
        self.sync(app);
    }

    /// [re:3fca26]: "Changes will be applied the next time you start …".
    fn restart_alert(&mut self, app: &mut App, anchor: &[usize]) {
        self.docs_prompt = Some(DocsPrompt::Alert);
        app.open_menu(anchor, vec![docs_restart()], None);
    }

    /// [re:3fca26]: write the undo stages; alert when they changed.
    fn write_undo(&mut self, app: &mut App, path: &[usize], v: i64) {
        let old = self.options.int("MaxUndoSteps", UNDO_DEFAULT).max(UNDO_MIN);
        self.options.set("MaxUndoSteps", v);
        self.options.save();
        if let Some(c) = app.ctrl_mut(path) {
            c.text = v.to_string();
        }
        if old != v {
            self.restart_alert(app, path);
        }
    }

    /// The folder the Browse dialog starts in.
    pub fn documents_path(&self) -> String {
        self.options.string("LocalDocumentsPath", &default_documents())
    }

    pub fn handle(&mut self, app: &mut App, ev: &UiEvent) -> bool {
        if let Some(q) = self.docs_prompt.clone() {
            if let UiEvent::MenuSelected { index, .. } = ev {
                self.docs_prompt = None;
                // Yes / No prompt: 0 = Yes (item 2).
                match q {
                    DocsPrompt::Reset if *index == 2 => self.write_documents_path(app, &default_documents()),
                    DocsPrompt::Undo(path, v) => self.write_undo(app, &path, if *index == 2 { v } else { UNDO_MIN }),
                    DocsPrompt::UndoMin(path) => self.write_undo(app, &path, UNDO_MIN),
                    _ => {}
                }
                return true;
            }
            if app.menu.is_none() {
                self.docs_prompt = None;
                // A dismissed undo question or minimum alert takes 20.
                if let DocsPrompt::Undo(path, _) | DocsPrompt::UndoMin(path) = q {
                    self.write_undo(app, &path, UNDO_MIN);
                }
            }
        }
        // The export alerts: the clipping question's answer (Yes = item 2).
        if self.export_alert {
            if let UiEvent::MenuSelected { index, .. } = ev {
                self.export_alert = false;
                if let Some(Ok(mut g)) = self.export_job.as_ref().map(|j| j.lock()) {
                    if g.prompt.is_some() && g.answer.is_none() {
                        g.answer = Some(*index == 2);
                    }
                }
                return true;
            }
            if app.menu.is_none() {
                self.export_alert = false;
                if let Some(Ok(mut g)) = self.export_job.as_ref().map(|j| j.lock()) {
                    if g.prompt.is_some() && g.answer.is_none() {
                        g.answer = Some(false);
                    }
                }
            }
        }
        if let Some((.., true)) = self.scale_trial {
            if let UiEvent::MenuSelected { index, .. } = ev {
                self.keep_scale(*index != 3);
                return true;
            }
            if app.menu.is_none() {
                self.keep_scale(true);
            }
        }
        let Some(k) = self.open else { return false };
        if k == Kind::Export {
            return self.handle_export(app, ev);
        }
        match ev {
            UiEvent::Mouse { path, kind: MouseKind::Down, .. } if k == Kind::About && self.mine(path) => {
                self.open = None;
                true
            }
            UiEvent::Clicked { path, name, on, .. } if self.mine(path) => {
                // [re:3fca26]: every change is written at once.
                if let Some((_, key, _)) = TOGGLES.iter().find(|(n, _, _)| n == name) {
                    self.options.set(key, *on);
                    self.options.save();
                    self.dirty = true;
                    return true;
                }
                let parent = app.ctrl(&path[..path.len() - 1]).map(|c| c.name.clone()).unwrap_or_default();
                if parent == "MIDIModeGroup" {
                    let id = app.ctrl(path).map(|c| c.id).unwrap_or(0);
                    self.options.set("MIDINumbering", id);
                    self.options.save();
                    self.restart_alert(app, path);
                    return true;
                }
                if let Some(items) = self.items(name) {
                    app.open_menu(path, items, None);
                    return true;
                }
                match name.as_str() {
                    // [re:3fca26]: Finished tells the engine when a toggle
                    // changed, and closes.
                    "ConfirmButton" => {
                        self.options_confirmed = std::mem::take(&mut self.dirty);
                        self.open = None;
                    }
                    "CancelButton" => self.open = None,
                    "ResetPathButton" => {
                        let q = format!("Are you sure you want to reset the Documents path to {}?", default_documents());
                        self.docs_prompt = Some(DocsPrompt::Reset);
                        app.open_menu(path, vec![q, String::new(), "Yes".into(), "No".into()], None);
                    }
                    "BrowseButton" => self.browse_requested = true,
                    _ => {}
                }
                true
            }
            UiEvent::MenuSelected { anchor, name, index } if self.mine(anchor) => {
                let Some(items) = self.items(name) else { return true };
                let Some(v) = items.get(*index) else { return true };
                match name.as_str() {
                    "MetroType" => {
                        self.options.set("MetronomeSound", v);
                        self.options.save();
                        self.restart_alert(app, anchor);
                    }
                    "ShadingMode" => {
                        self.options.set("PatternEditorShadingMode", v);
                        self.options.save();
                        crate::pattern::set_shading_mode(v);
                    }
                    "RemMod" => {
                        self.options.set("RemoveModBehavour", REMOVE_BEHAVIOUR[(*index).min(3)]);
                        self.options.save();
                    }
                    // v2 only: a new scale applies at once (scale, host
                    // resize, options `LastEditorScaleFactor` /
                    // `LastEditorSize`), and 2 s later asks whether to keep it.
                    "UIScaling" => {
                        let new = (SCALINGS[(*index).min(10)] * 100.0).floor() as i64;
                        let old = self.options.int("LastEditorScaleFactor", 100);
                        if new != old {
                            let (w, h) = app.size();
                            let f = new as f32 / 100.0;
                            let old_size = self.options.string("LastEditorSize", "0,0");
                            self.scale_trial = Some((old, old_size, std::time::Instant::now(), false));
                            self.options.set("LastEditorScaleFactor", new);
                            self.options.set("LastEditorSize", format!("{},{}", (w as f32 * f).floor() as i32, (h as f32 * f).floor() as i32));
                            self.options.save();
                        }
                    }
                    // [re:3fca26]: written at once and applied (repaint).
                    "UIStyle" | "ThemeCombo" => {
                        let name = if v == DEFAULT_STYLE_LABEL { dreist_gui::skin::default_theme().to_string() } else { v.to_string() };
                        self.options.set("StyleSheet", &name);
                        self.options.save();
                        self.style_changed = Some(name);
                    }
                    _ => {}
                }
                true
            }
            // [re:3fca26]: the typed number (leading digits, else 0);
            // under 20 alerts and takes 20, over 50 asks (No takes 20).
            UiEvent::TextCommitted { path, name, text } if self.mine(path) && name == "UndoStages" => {
                let t = text.trim_start();
                let v: i64 = t.char_indices().take_while(|(i, ch)| ch.is_ascii_digit() || (*i == 0 && (*ch == '-' || *ch == '+'))).map(|(_, ch)| ch).collect::<String>().parse().unwrap_or(0);
                let cur = self.options.int("MaxUndoSteps", UNDO_DEFAULT).max(UNDO_MIN);
                if v == cur {
                    if let Some(c) = app.ctrl_mut(path) {
                        c.text = v.to_string();
                    }
                } else if v < UNDO_MIN {
                    self.docs_prompt = Some(DocsPrompt::UndoMin(path.clone()));
                    app.open_menu(path, vec!["You must have a minimum of 20 undo stages".into()], None);
                } else if v > 50 {
                    self.docs_prompt = Some(DocsPrompt::Undo(path.clone(), v));
                    app.open_menu(path, vec!["Are you sure you want this many undo stages? This can dramatically increase memory usage!".into(), String::new(), "Yes".into(), "No".into()], None);
                } else {
                    self.write_undo(app, path, v);
                }
                true
            }
            // Modal: nothing else reaches the editor while open.
            UiEvent::Mouse { path, .. } | UiEvent::Wheel { path, .. } | UiEvent::Clicked { path, .. } | UiEvent::ValueChanged { path, .. } => self.mine(path),
            _ => false,
        }
    }

    pub fn draw(&self, _r: &Renderer, p: &mut Painter, c: &Ctrl) -> bool {
        if c.overload != "ModalOverlay" {
            return false;
        }
        p.fill_rect(RectF::new(0.0, 0.0, c.rect.w as f32, c.rect.h as f32), SHADE);
        true
    }
}

/// [re:a5b222].
pub fn default_documents() -> String {
    crate::options::default_documents()
}

/// Errors worth an alert (an abort is not).
trait FilterErr {
    fn filter_err(self) -> Result<(), String>;
}

impl FilterErr for Result<Vec<std::path::PathBuf>, String> {
    fn filter_err(self) -> Result<(), String> {
        match self {
            Err(e) if e != "aborted" => Err(e),
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod export_tests {
    use super::*;
    use dreist_engine::export::{Files, Normalize, Source};

    #[test]
    fn export_options_from_settings() {
        crate::app::test_setup();
        let mut o = Options::default();
        let d = export_options(&o, 3);
        // [re:751e2d] defaults: all engines, mixdown, prompt, no tail.
        assert_eq!((d.source, d.files, d.normalize, d.tail, d.looped), (Source::AllEngines, Files::Mixdown, Normalize::Prompt, 0.0, false));
        o.set("ExportSource", 1);
        o.set("ExportOutputType", 1);
        o.set("ExportSingleOutput", "Sub output 5");
        o.set("ExportNormalizeMode", 2);
        o.set("ExportTailSecs", "7.9s");
        o.set("ExportLoopMode", true);
        let x = export_options(&o, 3);
        assert_eq!((x.source, x.files, x.normalize, x.tail, x.looped), (Source::Engine(3), Files::Single(5), Normalize::Always, 7.0, true));
        o.set("ExportTailSecs", "90");
        assert_eq!(export_tail(&o), 60.0);
        // Not an assigned output: Master.
        assert_eq!(export_single_output(&o, &[0, 2]), 0);
        assert_eq!(export_output_name(0), "Master");
    }
}
