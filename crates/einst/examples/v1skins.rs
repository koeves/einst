//! Render the v1-layout UI set (`assets/ui-v1`, made by
//! `scripts/uigen/uigen.py`) to PNGs: `v1skins [out_dir]` (default
//! `docs/screens/v1`). Installs Einst's profile with the UI folder pointed
//! at `assets/ui-v1` (`EINST_UI`, unless already set), loads `RootSkin.skin`
//! and the page skins, mounts the sub-skins the editor would mount into the
//! code-built panels, and writes `root.png` plus one image per page.

use std::path::{Path, PathBuf};

use dreist_gui::paint::Painter;
use dreist_gui::render::{CustomDraw, Renderer};
use dreist_gui::text::Fonts;
use dreist_gui::{Colour, Ctrl, Rect, Skin};

/// Code-built panels (by `overloadClass`) and the skin mounted into them.
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

/// Effects shown in the chain's first four slots.
const EFFECTS: [&str; 4] = ["Effects/Filter.skin", "Effects/Delay.skin", "Effects/BusComp.skin", "Effects/ParaEQ.skin"];

/// The LCD pages: (image name, page skin).
const PAGES: &[(&str, &str)] = &[
    ("pattern", "Pattern Editor/GlobalControls.skin"),
    ("scene", "Scene/ScenePanel.skin"),
    ("song", "Song/SongPanel.skin"),
    ("pad", "Pad Editor/LayerEditor.skin"),
    ("layer", "Pad Editor/LayerEditor.skin"),
    ("layer-mixer", "Mixer/LayerMixerPanel.skin"),
    ("pad-mixer", "Mixer/MixerPanelPadLayout.skin"),
    ("engine-mixer", "Mixer/EngineMixerPanel.skin"),
    ("global-mixer", "Mixer/MixerPanelGlobalLayout.skin"),
    ("sampler", "Sampler/SamplerPanel.skin"),
    ("slicer", "Preview/PreviewPanel.skin"),
];

/// Panels shown on their own: browser column parts, mapping and dialogs.
const PANELS: &[(&str, &str)] = &[
    ("browser-header", "Browser/Header.skin"),
    ("browser-search", "Browser/SearchPane.skin"),
    ("browser-footer", "Browser/Footer.skin"),
    ("mapping", "Automation/AutoMapPanel.skin"),
    ("host-params", "Automation/HostPanel.skin"),
    ("options", "Global/OptionsPanel.skin"),
    ("export", "Global/ExportPanel.skin"),
    ("file-find", "Global/FileFindPanel.skin"),
    ("import", "Global/ImportPanel.skin"),
    ("pattern-convertor", "Patterns/PatternConvertor.skin"),
    ("about", "About/AboutPanel.skin"),
    ("song-pattern-popup", "Song/SongPatternPopupPanel.skin"),
];

/// The skin to mount into `c`, if the editor builds it from one.
fn skin_for(c: &Ctrl) -> Option<&'static str> {
    if let Some((_, s)) = BY_OVERLOAD.iter().find(|(o, _)| *o == c.overload) {
        return Some(s);
    }
    if c.name.strip_prefix("Scene").is_some_and(|n| n.parse::<u32>().is_ok()) && c.rect.w == 105 {
        return Some("Scene/Scene.skin");
    }
    if c.name == "ScrollView" && c.style == "EmptyScrollViewStyle" {
        return Some("Effects/EffectChainFX.skin");
    }
    None
}

/// The top control of skin `rel`, moved to the origin.
fn load(skin: &mut Skin, rel: &str) -> Ctrl {
    let root = skin.load_skin(rel).unwrap_or_else(|e| panic!("{rel}: {e}"));
    let mut top = root.children.into_iter().next().unwrap_or_else(|| panic!("{rel}: empty"));
    top.rect = Rect::new(0, 0, top.rect.w, top.rect.h);
    top
}

/// Mount `rel`'s top control's children into `host` (sized to it).
fn mount_into(skin: &mut Skin, host: &mut Ctrl, rel: &str) {
    let mut top = load(skin, rel);
    top.set_rect(Rect::new(0, 0, host.rect.w, host.rect.h));
    if host.style.is_empty() {
        host.style = top.style.clone();
    }
    host.children.extend(top.children);
}

/// Mount sub-skins into every code-built panel below `c`.
fn mount_all(skin: &mut Skin, c: &mut Ctrl, depth: usize) {
    assert!(depth < 16, "mount loop at {}", c.name);
    let mut fx = 0;
    for ch in &mut c.children {
        if let Some(rel) = skin_for(ch) {
            mount_into(skin, ch, rel);
        }
        if ch.name == "EffectPanel" && ch.rect.w == 193 {
            // An effect's own skin inside its slot.
            let rel = EFFECTS[fx.min(EFFECTS.len() - 1)];
            fx += 1;
            mount_into(skin, ch, rel);
        }
        mount_all(skin, ch, depth + 1);
    }
}

/// Some state so the images show the looks' on states: engine numbers and
/// the selected engine, first tabs, a pattern key, selected strips.
fn demo_state(c: &mut Ctrl) {
    for i in 0..8 {
        if let Some(sel) = c.find_mut(&format!("EngineSelector{i}")) {
            if let Some(b) = sel.find_mut("EngineSelectionButton") {
                b.text = (i + 1).to_string();
                b.style = format!("EngineButtonStyle{}", i + 1);
                b.on = i == 0;
            }
            if i == 3 {
                if let Some(b) = sel.find_mut("EngineMutedIcon") {
                    b.on = true;
                }
            }
        }
    }
    for group in ["MainTabButtons", "BrowserTabButtons"] {
        if let Some(g) = c.find_mut(group) {
            if let Some(b) = g.children.first_mut() {
                b.on = true;
            }
        }
    }
    for (key, style) in [("Key0", "SelectorStyleSelected"), ("Key2", "SelectorStyleAllocated"), ("Key4", "SelectorStyleEditing"), ("Key1", "SelectorStyleAllocated")] {
        if let Some(k) = c.find_mut(key) {
            k.style = style.into();
        }
    }
    for name in ["PlayButton", "MetroButton", "Pad0"] {
        if let Some(b) = c.find_mut(name) {
            b.on = true;
        }
    }
    if let Some(b) = c.find_mut("BPMTextBox") {
        b.text = "120.00".into();
    }
    if let Some(b) = c.find_mut("MemoryUsageBox") {
        b.text = "12 MB".into();
    }
    c.walk_mut(&mut |k| {
        if k.class == "CtrlKnob" {
            k.value = 0.62;
        }
        if k.class == "CtrlSlider" {
            k.value = 0.7;
        }
    });
}

/// The editor draws the logo mark itself; this preview fills Einst's "E"
/// (`einst::LOGO_MARK`, a 50 x 50 design) in the logo style's colFill,
/// scaled to the control's height.
struct Preview;

impl CustomDraw for Preview {
    fn draw(&self, r: &Renderer, p: &mut Painter, c: &Ctrl) -> bool {
        if c.overload != "Logo" {
            return false;
        }
        let col = r.styles.colour(&c.style, "colFill", Colour::WHITE);
        let k = c.rect.h as f32 / 50.0;
        for poly in einst::LOGO_MARK {
            let pts: Vec<(f32, f32)> = poly.iter().map(|(x, y)| (x * k, y * k)).collect();
            p.fill_polygon(&pts, col);
        }
        true
    }
}

fn save(skin: &Skin, fonts: &mut Fonts, c: &Ctrl, out: &Path, name: &str) {
    let pm = dreist_gui::render_ctrl_with(skin, fonts, c, 1.0, Some(&Preview));
    let p = out.join(format!("{name}.png"));
    pm.save_png(&p).expect("png");
    println!("{}", p.display());
}

fn main() {
    let ui = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/ui-v1"));
    if std::env::var_os("EINST_UI").is_none() {
        // SAFETY: set before any other thread starts.
        unsafe { std::env::set_var("EINST_UI", &ui) };
    }
    einst::install();
    let out = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| "docs/screens/v1".into()));
    std::fs::create_dir_all(&out).expect("out dir");
    let mut skin = Skin::load(&Skin::resources_dir(), dreist_gui::skin::default_theme()).expect("skin");
    let mut fonts = Fonts::new(&skin.font_dir());

    // The window, with the pattern page's toolbar on the screen.
    let mut root = skin.load_skin("RootSkin.skin").expect("RootSkin.skin");
    root.rect = Rect::new(0, 0, 1135, 640);
    mount_all(&mut skin, &mut root, 0);
    if let Some(tabs) = root.find_mut("MainTabs") {
        let mut page = load(&mut skin, "Pattern Editor/GlobalControls.skin");
        mount_all(&mut skin, &mut page, 0);
        tabs.children.push(page);
    }
    if let Some(tabs) = root.find_mut("BrowserTabs") {
        let mut header = load(&mut skin, "Browser/Header.skin");
        header.rect.y = 0;
        let mut footer = load(&mut skin, "Browser/Footer.skin");
        footer.rect.y = tabs.rect.h - footer.rect.h;
        tabs.children.push(header);
        tabs.children.push(footer);
    }
    demo_state(&mut root);
    save(&skin, &mut fonts, &root, &out, "root");

    for (name, rel) in PAGES {
        let mut page = load(&mut skin, rel);
        mount_all(&mut skin, &mut page, 0);
        demo_state(&mut page);
        // The pad/layer page shows one editor at a time.
        let hide = match *name {
            "pad" => Some("LayerControls"),
            "layer" => Some("PadControls"),
            _ => None,
        };
        if let Some(h) = hide.and_then(|h| page.find_mut(h)) {
            h.visible = false;
        }
        if *name == "pad" || *name == "layer" {
            if let Some(b) = page.find_mut(if *name == "pad" { "PadSelection" } else { "LayerSelectionButton0" }) {
                b.style = if *name == "pad" { "PadSelectionButtonPrimaryStyle" } else { "SelectionButtonPrimaryStyle" }.into();
            }
        }
        // On the LCD: draw the page over the screen colour.
        let mut lcd = Ctrl { class: "CtrlPanel".into(), style: "ScreenLcdStyle".into(), rect: Rect::new(0, 0, page.rect.w + 10, page.rect.h + 10), visible: true, enabled: true, ..Ctrl::default() };
        page.rect = Rect::new(5, 5, page.rect.w, page.rect.h);
        lcd.children.push(page);
        save(&skin, &mut fonts, &lcd, &out, name);
    }
    for (name, rel) in PANELS {
        let mut panel = load(&mut skin, rel);
        mount_all(&mut skin, &mut panel, 0);
        let mut back = Ctrl { class: "CtrlPanel".into(), style: "ScreenLcdStyle".into(), rect: Rect::new(0, 0, panel.rect.w + 10, panel.rect.h + 10), visible: true, enabled: true, ..Ctrl::default() };
        panel.rect = Rect::new(5, 5, panel.rect.w, panel.rect.h);
        back.children.push(panel);
        save(&skin, &mut fonts, &back, &out, name);
    }
}
