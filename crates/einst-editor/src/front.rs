//! The v1 layout: `RootSkin.skin` (window 1135 × 640, browser column on
//! the left, the LCD with its nine page tabs and the front panel under it)
//! with the sub-skins mounted into its code-built panels (by
//! `overloadClass`), the LCD pages and the front-panel controls no other
//! panel owns: the pad mode buttons, the master pitch knob, tap tempo and
//! the metronome volume knob.

use dreist_engine::binding::id;
use dreist_engine::live::PadMode;
use dreist_gui::app::{App, UiEvent};
use dreist_gui::{Ctrl, Rect, Skin};

use crate::command::{Command, msg};
use crate::model::Model;
use crate::{EditorError, err};

/// The window's size (fixed: v1 has no anchors, splitters or sizer).
pub const WINDOW: (i32, i32) = (1135, 640);

/// Code-built panels (`overloadClass`) and the skin mounted into each.
const BY_OVERLOAD: &[(&str, &str)] = &[
    ("PadModeControlsPanel", "Pads/PadModeControlsPanel.skin"),
    ("EditorEngineSelectors", "Engines/EngineSelectors.skin"),
    ("EditorEngineSelector", "Engines/EngineSelector.skin"),
    ("EditorPads", "Pads/Pads.skin"),
    ("PatternSelectors", "Patterns/PatternSelectors.skin"),
    ("EngineControls", "Engines/EngineControls.skin"),
    ("MasterControlsPanel", "Global/MasterControls.skin"),
    ("UtilityControlsPanel", "Global/UtilityControls.skin"),
    ("Transport", "Transport/Transport.skin"),
    ("PatternToolsPanel", "Tools/PatternToolsPanel.skin"),
    ("EditorEffectChainPanel", "Effects/EffectChain.skin"),
    ("EditorEffectPanel", "Effects/EffectPanel.skin"),
    ("EditorLayerMixerChannel", "Mixer/LayerMixerStrip.skin"),
    ("EditorMixerPadChannel", "Mixer/MixerStripPad.skin"),
    ("EditorEngineMixerChannel", "Mixer/EngineMixerStrip.skin"),
    ("EditorMixerAuxChannel", "Mixer/MixerAux.skin"),
    ("EditorMixerMasterChannel", "Mixer/MixerMaster.skin"),
    ("LayerSelectionButton", "Pad Editor/SelectionButton.skin"),
    ("PadSelectionButton", "Pad Editor/PadSelectionButton.skin"),
    ("EditorPadControlsPanel", "Pad Editor/PadControls.skin"),
    ("EditorPadWaveDisplayPanel", "Pad Editor/PadEditorWavePanel.skin"),
    ("EditorPadSetupPanel", "Pad Editor/SingleSetupPanel.skin"),
    ("EditorPadTimestretchPanel", "Pad Editor/SingleTimestretchPanel.skin"),
    ("EditorPadEnvelopePanel", "Pad Editor/SingleEnvelopePanel.skin"),
];

/// The LCD pages (`MainTabs`, in tab order) and their skins. The pattern
/// page's step grid and the song page's lanes are built by their panels.
pub const PAGES: [(&str, &str); 9] = [
    ("Pattern", "Pattern Editor/GlobalControls.skin"),
    ("Scene", "Scene/ScenePanel.skin"),
    ("Song", "Song/SongPanel.skin"),
    ("Pad / Layers", "Pad Editor/LayerEditor.skin"),
    ("Layer mixer", "Mixer/LayerMixerPanel.skin"),
    ("Pad mixer", "Mixer/MixerPanelPadLayout.skin"),
    ("Engine mixer", "Mixer/EngineMixerPanel.skin"),
    ("Global mixer", "Mixer/MixerPanelGlobalLayout.skin"),
    ("Sampler", "Sampler/SamplerPanel.skin"),
];

/// Page ids (`MainTabs`).
pub const PATTERN_PAGE: usize = 0;
pub const SONG_PAGE: usize = 2;
pub const SCENE_PAGE: usize = 1;
pub const PADS_PAGE: usize = 3;
pub const LAYER_MIXER_PAGE: usize = 4;
pub const PAD_MIXER_PAGE: usize = 5;
pub const ENGINE_MIXER_PAGE: usize = 6;
pub const GLOBAL_MIXER_PAGE: usize = 7;
pub const SAMPLER_PAGE: usize = 8;

/// Browser column pages (`BrowserTabs`).
pub const BROWSER_PAGE: usize = 0;
pub const MAPPING_PAGE: usize = 1;

/// The pad mode buttons (`PadMode` group, in order).
const PAD_MODES: [PadMode; 4] = [PadMode::Normal, PadMode::Chromatic, PadMode::Velocity, PadMode::MuteAndSolo];

/// The path of control `name` inside front-panel section `section` (e.g.
/// `Transport`): names like RecButton and FollowButton recur on the
/// pages.
pub(crate) fn path_in(root: &Ctrl, section: &str, name: &str) -> Option<Vec<usize>> {
    let s = root.path_of(section)?;
    let mut c = root;
    for &i in &s {
        c = &c.children[i];
    }
    Some([s, c.path_of(name)?].concat())
}

/// Style `style`'s colour attribute `attr` when the theme defines it, else
/// `default` (the code-drawn content's colours).
pub(crate) fn colour_or(s: &dreist_gui::style::StyleSheet, style: &str, attr: &str, default: dreist_gui::Colour) -> dreist_gui::Colour {
    if s.raw(style, attr).is_some() { s.colour(style, attr, default) } else { default }
}

/// The skin to mount into `c`, if it is a code-built panel.
fn skin_for(c: &Ctrl) -> Option<&'static str> {
    if c.children.is_empty() {
        if let Some((_, s)) = BY_OVERLOAD.iter().find(|(o, _)| *o == c.overload) {
            return Some(s);
        }
        // The scene slots (Scene0..63, 105 wide) on the scene page.
        if c.name.strip_prefix("Scene").is_some_and(|n| n.parse::<u32>().is_ok()) && c.rect.w == 105 {
            return Some("Scene/Scene.skin");
        }
    }
    None
}

/// The top control of skin `rel`, moved to the origin.
pub(crate) fn load(skin: &mut Skin, rel: &str) -> Result<Ctrl, EditorError> {
    let root = skin.load_skin(rel).map_err(err)?;
    let mut top = root.children.into_iter().next().ok_or_else(|| EditorError(format!("{rel}: empty")))?;
    top.rect = Rect::new(0, 0, top.rect.w, top.rect.h);
    Ok(top)
}

/// Mount `rel`'s top control's children into `host` (sized to it).
pub(crate) fn mount_into(skin: &mut Skin, host: &mut Ctrl, rel: &str) -> Result<(), EditorError> {
    let mut top = load(skin, rel)?;
    top.set_rect(Rect::new(0, 0, host.rect.w, host.rect.h));
    if host.style.is_empty() {
        host.style = top.style.clone();
    }
    host.children.extend(top.children);
    Ok(())
}

/// Mount the sub-skins into every code-built panel below `c`.
pub(crate) fn mount_all(skin: &mut Skin, c: &mut Ctrl) -> Result<(), EditorError> {
    fn rec(skin: &mut Skin, c: &mut Ctrl, depth: usize) -> Result<(), EditorError> {
        if depth > 16 {
            return Err(EditorError(format!("mount loop at {}", c.name)));
        }
        for ch in &mut c.children {
            if let Some(rel) = skin_for(ch) {
                mount_into(skin, ch, rel)?;
            }
            rec(skin, ch, depth + 1)?;
        }
        Ok(())
    }
    rec(skin, c, 0)
}

/// Load the root skin at the fixed window size with the front panel's
/// sub-skins mounted.
pub(crate) fn root(skin: &mut Skin) -> Result<Ctrl, EditorError> {
    let mut root = skin.load_skin("RootSkin.skin").map_err(err)?;
    root.rect = Rect::new(0, 0, WINDOW.0, WINDOW.1);
    mount_all(skin, &mut root)?;
    // The debug FPS label is not ported.
    if let Some(c) = root.find_mut("FPSLabel") {
        c.visible = false;
        c.enabled = false;
    }
    // The engine buttons carry their engine's art and number.
    if let Some(sels) = root.find_mut("EditorEngineSelectors") {
        for s in sels.children.iter_mut() {
            let Some(e) = s.name.strip_prefix("EngineSelector").and_then(|n| n.parse::<usize>().ok()) else { continue };
            s.set_attr("engine", e.to_string());
            if let Some(b) = s.find_mut("EngineSelectionButton") {
                b.text = (e + 1).to_string();
                b.style = format!("EngineButtonStyle{}", e + 1);
            }
            for n in ["EngineMutedIcon", "EngineSoloedIcon", "VULeft"] {
                if let Some(k) = s.find_mut(n) {
                    k.passive = true;
                }
            }
            if let Some(k) = s.find_mut("VULeft") {
                k.set_attr("volEngine", e.to_string());
            }
        }
    }
    // [re:d2f251]: the tempo box edits on a double click; a press drags it.
    if let Some(c) = root.find_mut("BPMTextBox").filter(|c| c.overload == "TempoBox") {
        c.set_attr("optSingleClickEdit", "false".into());
    }
    for (i, n) in ["MasterVULeft", "MasterVURight"].into_iter().enumerate() {
        if let Some(c) = root.find_mut(n) {
            c.set_attr("masterSide", i.to_string());
        }
    }
    // Section labels and readouts take no clicks.
    for n in ["MemoryUsageBox", "MIDIIndi", "VoiceIndi", "MasterVULeft", "MasterVURight", "SwingIcon"] {
        if let Some(c) = root.find_mut(n) {
            c.passive = true;
        }
    }
    Ok(root)
}

/// Mount LCD page `id`'s skin with its sub-skins into `page`.
pub(crate) fn mount_page(skin: &mut Skin, root: &mut Ctrl, page: &[usize], id: usize) -> Result<(), EditorError> {
    let mut host = &mut *root;
    for &i in page {
        host = &mut host.children[i];
    }
    mount_into(skin, host, PAGES[id].1)?;
    mount_all(skin, host)
}

/// The front-panel controls no other panel owns.
#[derive(Default)]
pub struct Front {
    /// Tap tempo: the last taps.
    taps: Vec<std::time::Instant>,
    named: dreist_gui::app::PathCache,
    /// The browser is hidden (the small window).
    pub browser_hidden: bool,
}

/// [re:8157fe]: show or hide the browser column. Hidden, the main panel
/// moves to x 1 and the window shrinks to its width + 1 (`RootPanelSmallStyle`);
/// shown, the window is back at full width (`RootPanelStyle`).
pub fn set_browser_visible(app: &mut App, visible: bool) {
    let Some(back) = app.root.path_of("BackPanel") else { return };
    let (Some(bp), Some(mp)) = (app.root.path_of("BrowsePanel"), app.root.path_of("MovePanel")) else { return };
    let browse_w = app.ctrl(&bp).map_or(0, |c| c.rect.w);
    let main_w = app.ctrl(&mp).map_or(0, |c| c.rect.w);
    let w = if visible { WINDOW.0 } else { main_w + 1 };
    if let Some(c) = app.ctrl_mut(&bp) {
        c.visible = visible;
    }
    if let Some(c) = app.ctrl_mut(&mp) {
        c.rect.x = if visible { browse_w - 2 } else { 1 };
    }
    if let Some(c) = app.ctrl_mut(&back) {
        c.rect.w = w;
        c.style = if visible { "RootPanelStyle" } else { "RootPanelSmallStyle" }.into();
    }
    if app.size().0 != w {
        app.resize(w, WINDOW.1);
        app.resize_request = true;
    }
    app.dirty = true;
}

/// The master channel's `Pitch` (0..1, centre 0.5 = no transposition).
fn master_pitch(model: &Model) -> f32 {
    model.doc.children.iter().find(|c| c.class == "MasterChannel").and_then(|n| n.f32("Pitch")).unwrap_or(0.5)
}

impl Front {
    /// A control of the pad mode column (`PadModePanel`).
    fn mode_path(&self, app: &App, name: &'static str) -> Option<Vec<usize>> {
        let scope = self.named.path_in(app, &[], "PadModePanel")?;
        self.named.path_in(app, &scope, name)
    }

    pub fn sync(&self, app: &mut App, model: &Model) {
        let e = model.engine;
        let hidden = self.browser_hidden;
        if let Some(c) = self.named.path_in(app, &[], "ShowHideButton").and_then(|p| app.ctrl_mut(&p)) {
            c.on = hidden;
        }
        let mode = model.pad_mode(e);
        if let Some(g) = self.mode_path(app, "PadMode").and_then(|p| app.ctrl_mut(&p)) {
            for (i, b) in g.children.iter_mut().enumerate() {
                b.on = PAD_MODES.get(i) == Some(&mode);
            }
        }
        let follow = crate::options::Options::load().bool("PadSelectionFollowsMIDI", true);
        if let Some(b) = self.mode_path(app, "FollowButton").and_then(|p| app.ctrl_mut(&p)) {
            b.on = follow;
        }
        let pitch = master_pitch(model);
        if let Some(k) = self.named.ctrl_mut(app, "MasterPitch") {
            k.value = pitch;
        }
        let metro = model.doc.children.iter().find(|c| c.class == "Metronome").and_then(|n| n.f32("Gain")).unwrap_or(0.5f32.cbrt());
        if let Some(k) = self.named.ctrl_mut(app, "MetroVol") {
            k.value = metro;
        }
    }

    /// True when the model changed.
    pub fn handle(&mut self, app: &mut App, model: &mut Model, ev: &UiEvent, out: &mut Vec<Command>) -> bool {
        let e = model.engine;
        match ev {
            UiEvent::Clicked { name, .. } if name == "ShowHideButton" => {
                self.browser_hidden = !self.browser_hidden;
                set_browser_visible(app, !self.browser_hidden);
                true
            }
            UiEvent::Clicked { path, .. } if self.mode_path(app, "PadMode").is_some_and(|g| path.len() == g.len() + 1 && path.starts_with(&g)) => {
                let Some(&mode) = PAD_MODES.get(*path.last().unwrap()) else { return false };
                out.push(Command::SetPadMode { engine: e, mode });
                let label = crate::popups::PAD_MODE_LABELS[mode.index() as usize];
                model.engine_mut(e).set("EnginePadMode", label);
                true
            }
            // Pad selection follows MIDI notes (`e_registryFollowOptionsChanged`).
            UiEvent::Clicked { path, name, on, .. } if name == "FollowButton" && self.mode_path(app, "FollowButton").as_ref() == Some(path) => {
                let mut o = crate::options::Options::load();
                o.set("PadSelectionFollowsMIDI", *on);
                if !cfg!(test) {
                    o.save();
                }
                let engine = o.bool("EngineSelectionFollowsMIDI", false);
                out.push(msg(id::REGISTRY_FOLLOW_OPTIONS_CHANGED, &[], &[*on as i32 as f32, engine as i32 as f32]));
                true
            }
            // Master pitch (`e_setMasterChannelTune`), kept as the master
            // channel's `Pitch`; a double click centres it.
            UiEvent::ValueChanged { name, value, .. } if name == "MasterPitch" => {
                let v = value.clamp(0.0, 1.0);
                model.doc.child_mut_or_insert("MasterChannel", "MasterChannel").set("Pitch", dreist_formats::node::format_g16(v as f64));
                out.push(msg(id::SET_MASTER_CHANNEL_TUNE, &[], &[v]));
                true
            }
            UiEvent::DoubleClicked { name, .. } if name == "MasterPitch" => {
                model.doc.child_mut_or_insert("MasterChannel", "MasterChannel").set("Pitch", "0.5");
                out.push(msg(id::SET_MASTER_CHANNEL_TUNE, &[], &[0.5]));
                true
            }
            UiEvent::ValueChanged { name, value, .. } if name == "MetroVol" => {
                model.doc.child_mut_or_insert("Metronome", "Metronome").set("Gain", dreist_formats::node::format_g16(*value as f64));
                out.push(msg(id::SET_METRONOME_LEVEL, &[], &[*value]));
                true
            }
            // Tap tempo: the mean of the last taps' intervals (taps more
            // than 2 s apart start over).
            UiEvent::Clicked { name, .. } if name == "TapBox" => {
                let now = std::time::Instant::now();
                if self.taps.last().is_some_and(|t| now.duration_since(*t).as_secs_f32() > 2.0) {
                    self.taps.clear();
                }
                self.taps.push(now);
                if self.taps.len() > 5 {
                    self.taps.remove(0);
                }
                if self.taps.len() >= 2 {
                    let span = now.duration_since(self.taps[0]).as_secs_f32() / (self.taps.len() - 1) as f32;
                    if span > 0.0 {
                        out.push(msg(id::SET_TEMPO, &[], &[(60.0 / span).clamp(20.0, 999.0)]));
                    }
                }
                true
            }
            _ => false,
        }
    }
}
