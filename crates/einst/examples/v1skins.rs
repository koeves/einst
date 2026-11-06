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
const EFFECTS: [&str; 4] = ["Effects/Delay.skin", "Effects/ParaEQ.skin", "Effects/Gain.skin", "Effects/ChanComp.skin"];

/// Their names (the slots' type menus and the chain's slot buttons).
const EFFECT_NAMES: [&str; 6] = ["Delay", "Parametric EQ", "Gain", "Channel Compressor", "No Effect", "No Effect"];

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
    mount_rec(skin, c, depth, &mut 0);
}

/// [`mount_all`], counting the effect slots filled so far (`fx`).
fn mount_rec(skin: &mut Skin, c: &mut Ctrl, depth: usize, fx: &mut usize) {
    assert!(depth < 16, "mount loop at {}", c.name);
    for ch in &mut c.children {
        if let Some(rel) = skin_for(ch) {
            mount_into(skin, ch, rel);
        }
        if ch.name == "EffectPanel" && ch.rect.w == 193 {
            // An effect's own skin inside its slot.
            let rel = EFFECTS[(*fx).min(EFFECTS.len() - 1)];
            *fx += 1;
            mount_into(skin, ch, rel);
        }
        mount_rec(skin, ch, depth + 1, fx);
    }
}

/// Some state so the images show the looks' on states, set up like the
/// reference screens: engine 3 selected, pattern power on, a pattern key,
/// named pads with their colour LEDs, a selected pad, the mixers' mix view.
fn demo_state(c: &mut Ctrl) {
    for i in 0..8 {
        if let Some(sel) = c.find_mut(&format!("EngineSelector{i}")) {
            if let Some(b) = sel.find_mut("EngineSelectionButton") {
                b.text = (i + 1).to_string();
                b.style = format!("EngineButtonStyle{}", i + 1);
                b.on = i == 2;
            }
            if let Some(b) = sel.find_mut("PatternPower") {
                b.on = true;
            }
            if let Some(v) = sel.find_mut("VULeft") {
                v.value = [0.0, 0.0, 0.15, 0.0, 0.0, 0.0, 0.55, 0.0][i];
            }
        }
    }
    for group in ["MainTabButtons", "BrowserTabButtons", "ModeGroup", "PadMode"] {
        if let Some(g) = c.find_mut(group) {
            if let Some(b) = g.children.first_mut() {
                b.on = true;
            }
        }
    }
    if let Some(k) = c.find_mut("Key20") {
        k.style = "SelectorStyleSelected".into();
    }
    for name in ["PlayButton", "MetroButton"] {
        if let Some(b) = c.find_mut(name) {
            b.on = true;
        }
    }
    if let Some(b) = c.find_mut("RedoButton") {
        b.enabled = false;
    }
    if let Some(b) = c.find_mut("RedoHistory") {
        b.enabled = false;
    }
    if let Some(b) = c.find_mut("BPMTextBox") {
        b.text = "132.00".into();
    }
    if let Some(b) = c.find_mut("MemoryUsageBox") {
        b.text = "19.7MB".into();
    }
    // Pads: names, the first status LED in the pad's colour, pad 15 selected.
    const PADS: [&str; 16] = ["Kick", "Perc 1", "Tick", "Beep", "Warble", "EpicClap", "Noise", "ElecTom", "ElecTom 2", "ElecTom 3", "11", "12", "Tap", "Glitch", "Zaap", "Noise Bst"];

    for (i, name) in PADS.iter().enumerate() {
        if let Some(pad) = c.find_mut(&format!("Pad{i}")) {
            if pad.overload != "EditorPad" {
                continue;
            }
            if i == 15 {
                pad.children[0].style = "PadSelectedStyle".into();
            }
            if let Some(led) = pad.children.get_mut(1) {
                led.style = format!("PadLed{}Style", i / 4 + 1);
            }
            pad.children.push(label(Rect::new(2, 10, 52, 18), "PadNameStyle", name));
        }
    }
    // Mixer strips in the mix view: number, mute / solo, pan, fader, meters
    // and the output.
    let mut n = 0;
    c.walk_mut(&mut |k| {
        if k.overload == "EditorEngineMixerChannel" || k.overload == "EditorMixerPadChannel" {
            n += 1;
            let no = n;
            k.walk_mut(&mut |s| match s.name.as_str() {
                "NameLabel" => s.text = no.to_string(),
                "OutputCombo" => s.text = "Master".into(),
                // As on v1's mixer screen: centred pans, faders a little
                // under unity, a third of the meters lit.
                "PanKnob" => s.set_attr("demoValue", "0.5".into()),
                "GainSlider" => s.set_attr("demoValue", "0.62".into()),
                "VUMeterLeft" | "VUMeterRight" => s.set_attr("demoValue", "0.37".into()),
                n if n.starts_with("Swing")
                    || n.starts_with("EngineSwing")
                    || n.starts_with("EnginePitch")
                    || n.starts_with("Tempo")
                    || n.starts_with("Send")
                    || n.ends_with("NumBox")
                    || n.ends_with("DenomBox") =>
                {
                    s.visible = false
                }
                _ => {}
            });
        }
    });
    // Mixer strips alternate (v1's code picks the even / odd backgrounds);
    // strip 6 is the selected one, as on v1's mixer screen.
    c.walk_mut(&mut |k| {
        if let Some(n) = k.name.strip_prefix("Channel").and_then(|n| n.parse::<u32>().ok()) {
            if k.style == "EngineChannelBackgroundOdd" && n == 5 {
                k.style = "EngineChannelBackgroundSelected".into();
            } else if k.style == "EngineChannelBackgroundOdd" && n % 2 == 1 {
                k.style = "EngineChannelBackgroundEven".into();
            }
        }
        if k.class == "CtrlRangeSlider" {
            k.set_attr("valueLow", "0.2".into());
            k.set_attr("valueHigh", "0.75".into());
        }
    });
    // Effect slots: type and preset menus, the chain's slot buttons.
    let mut fx = 0;
    c.walk_mut(&mut |k| match k.name.as_str() {
        "EffectTypeCombo" => {
            k.text = EFFECT_NAMES[fx.min(5)].into();
            fx += 1;
        }
        "PresetCombo" => k.text = "Default".into(),
        _ if k.overload == "EditorEffectPanelScrollbar" => {
            for (i, n) in EFFECT_NAMES.iter().enumerate() {
                k.children.push(ctrl("CtrlButton", Rect::new(i as i32 * 75, 4, 74, 27), "LcdButtonStyle", n));
            }
        }
        _ => {}
    });
    for name in ["BlackoutPanel", "PatternRenameTextBox"] {
        if let Some(b) = c.find_mut(name) {
            b.visible = false;
        }
    }
    if let Some(b) = c.find_mut("HitLoopModeButton") {
        b.on = true;
    }
    c.walk_mut(&mut |k| {
        if k.class == "CtrlKnob" {
            k.value = 0.62;
        }
        if k.class == "CtrlSlider" {
            k.value = 0.7;
        }
        // The natively drawn v1 controls: levels, pans, a gain-reduction
        // needle, an EQ curve and table rows.
        match k.class.as_str() {
            "CtrlLevelStrip" if !k.name.starts_with("VULeft") => k.value = if k.name.contains("Right") { 0.55 } else { 0.7 },
            "CtrlPanSlider" => k.value = 0.35,
            "CtrlMeter" => k.value = 0.7,
            "CtrlPlot" => k.set_attr("points", "0,0.5 0.15,0.5 0.3,0.62 0.45,0.5 0.6,0.42 0.75,0.36 1,0.34".into()),
            "CtrlTableView" if k.style != "BrowserListStyle" => {
                k.set_attr("columns", "Parameter|Control".into());
                k.set_attr("rows", "Engine 1 Volume|CC 7\nEngine 1 Pan|CC 10\nMaster Volume|Host 1".into());
                k.set_attr("intSelectedRow", "1".into());
            }
            "CtrlListBox" => k.set_attr("rows", "Kick 01.wav\nSnare 03.wav\nHat Closed.wav".into()),
            _ => {}
        }
        if let Some(v) = k.attr("demoValue").and_then(|v| v.parse::<f32>().ok()) {
            k.value = v;
        }
        match k.name.as_str() {
            "MasterPitch" => k.value = 0.5,
            "MasterSlider" => k.value = 0.72,
            "EngineSwingDepthKnob" => k.value = 0.3,
            "FullLevelKnob" | "MetroVol" | "InstoSlider" => k.value = 0.8,
            "Spinner" => k.value = 0.0,
            _ => {}
        }
    });
}

// ---------------------------------------------------------------- code-drawn content

// The editor draws the step grid, the song's parts and the browser's tree
// and lists in code; these stand-ins (with the theme's content styles) make
// the previews comparable with v1's screens.

fn ctrl(class: &str, r: Rect, style: &str, text: &str) -> Ctrl {
    // zorder 1: the skin's default, so these draw after the skin's own children.
    Ctrl { class: class.into(), style: style.into(), text: text.into(), rect: r, zorder: 1, visible: true, enabled: true, ..Ctrl::default() }
}

fn panel(r: Rect, style: &str) -> Ctrl {
    ctrl("CtrlPanel", r, style, "")
}

fn label(r: Rect, style: &str, text: &str) -> Ctrl {
    ctrl("CtrlLabel", r, style, text)
}

/// A bar ruler `w` wide: a number every `every` cells of `cell` px.
fn timeline(r: Rect, cell: i32, every: i32, first: i32, step: i32) -> Ctrl {
    let mut t = panel(r, "TimelineStyle");
    let mut x = 0;
    let mut n = first;
    while x < r.w {
        t.children.push(panel(Rect::new(x, r.h - 4, 1, 4), "TimelineTickStyle"));
        t.children.push(label(Rect::new(x + 2, 0, 30, r.h - 2), "TimelineLabelStyle", &n.to_string()));
        x += cell * every;
        n += step;
    }
    t
}

/// The pattern page's track rows and step grid (16 pads, 32 steps).
fn pattern_grid(skin: &mut Skin) -> Vec<Ctrl> {
    const NAMES: [&str; 16] =
        ["TickClap", "HattySnare", "Snare 5", "SnareDirt", "Snare 4", "Snare 3", "Snare 2", "Snare 1", "Clap 6", "Clap 5", "Clap 4", "Clap 3", "Clap 2", "ClickClap", "NoiseClap", "Clap 1"];
    let (x0, y0, cw, rh) = (152, 52, 20, 20);
    let mut out = Vec::new();
    out.push(timeline(Rect::new(x0, 37, 32 * cw, 15), cw, 4, 1, 4));
    let mut grid = panel(Rect::new(x0, y0, 32 * cw, 16 * rh), "GridBackStyle");
    for beat in (4..32).step_by(8) {
        grid.children.push(panel(Rect::new(beat * cw, 0, 4 * cw, 16 * rh), "GridShadeStyle"));
    }
    grid.children.push(panel(Rect::new(10, 10 * rh, 32 * cw - 10, rh), "GridRowSelectedStyle"));
    grid.children.push(panel(Rect::new(22 * cw, 0, cw, 16 * rh), "GridPositionStyle"));
    for i in 1..32 {
        let style = if i % 4 == 0 { "GridBarLineStyle" } else { "GridDashStyle" };
        grid.children.push(panel(Rect::new(i * cw, 0, 1, 16 * rh), style));
    }
    for i in 1..16 {
        grid.children.push(panel(Rect::new(0, i * rh, 32 * cw, 1), "GridDashStyle"));
    }
    for row in 0..16 {
        for step in 0..32 {
            let on = match row {
                0 => step % 8 == 2 || step == 27,
                1 | 2 => step % 16 == 12,
                3 => step == 31,
                _ => (step + row) % 8 == 0 || (row % 3 == 0 && step % 16 == 9),
            };
            if on {
                grid.children.push(panel(Rect::new(step * cw + 4, row * rh + 4, cw - 7, rh - 7), "StepOnStyle"));
            }
        }
    }
    out.push(grid);
    for (i, name) in NAMES.iter().enumerate() {
        let mut row = load(skin, "Pattern Editor/TrackControls.skin");
        row.rect.x = 2;
        row.rect.y = y0 + i as i32 * rh;
        if let Some(l) = row.find_mut("SelectArea") {
            l.text = (*name).into();
            l.style = "TrackNameStyle".into();
        }
        if let Some(b) = row.find_mut("AutomateButton") {
            b.text = "\u{25BA}".into();
            b.style = "TrackArrowStyle".into();
        }
        out.push(row);
    }
    out
}

/// The song page's track numbers, lanes and parts.
fn song_lanes(skin: &mut Skin, page: &mut Ctrl) {
    // (track, first bar, bars, pattern, name)
    const PARTS: &[(i32, i32, i32, i32, &str)] = &[
        (0, 0, 16, 1, "Intro Beats"),
        (1, 4, 24, 16, "Bass Loop"),
        (1, 29, 4, 16, "Bass Loop"),
        (2, 0, 4, 1, ""),
        (2, 4, 40, 21, "Percussion & Noise"),
        (3, 12, 8, 1, "Trashy Vox"),
        (3, 20, 8, 11, "Chorus Vox"),
        (3, 28, 8, 17, "Multi Vox"),
        (4, 0, 4, 1, "Riff"),
        (4, 4, 10, 1, "Main Riff"),
        (4, 12, 18, 1, "Main Riff"),
        (4, 31, 10, 11, "Variation Riff"),
        (4, 41, 10, 8, "Breakdown"),
        (5, 20, 10, 11, "Nasty Loop"),
        (5, 29, 10, 10, "Build Up"),
        (6, 20, 40, 5, "Slow Moving Drone"),
        (7, 12, 40, 1, "Sparks"),
    ];
    let (bar, th) = (16, 40);
    let Some(header) = page.find_mut("TrackHeaderPanel") else { return };
    for t in 0..8 {
        let mut h = load(skin, "Song/SongTrackHeaderPanel.skin");
        h.rect = Rect::new(0, 20 + t * th, 48, th);
        h.style = if t == 2 { "SongTrackPanelSelectedStyle".into() } else { format!("SongTrackPanelStyle{}", t % 2) };
        if let Some(l) = h.find_mut("TrackNumberLabel") {
            l.text = (t + 1).to_string();
            l.style = "TrackNumberStyle".into();
        }
        header.children.push(h);
    }
    let Some(view) = page.find_mut("TrackScrollView") else { return };
    let w = view.rect.w;
    view.children.push(timeline(Rect::new(0, 0, w, 20), bar, 4, 1, 4));
    for t in 0..8 {
        let style = if t == 2 {
            "SongLaneSelectedStyle"
        } else if t % 2 == 1 {
            "SongLaneAltStyle"
        } else {
            "SongLaneStyle"
        };
        view.children.push(panel(Rect::new(0, 20 + t * th, w, th), style));
        for x in (bar..w).step_by(bar as usize) {
            view.children.push(panel(Rect::new(x, 20 + t * th, 1, th), "GridLineStyle"));
        }
        view.children.push(panel(Rect::new(0, 20 + t * th + th - 1, w, 1), "GridDashStyle"));
    }
    for &(t, at, len, pat, name) in PARTS {
        let r = Rect::new(at * bar, 20 + t * th + 1, (len * bar).min(w - at * bar), th - 2);
        let style = if t == 6 { "SongPartSelectedBodyStyle" } else { "SongPartBodyStyle" };
        let mut p = label(r, style, &format!("{pat}\n{name}"));
        p.class = "CtrlMultiLineLabel".into();
        p.text = format!("\n{pat}\n{name}");
        p.set_attr("intLineHeight", "11".into());
        p.children.push(panel(Rect::new(0, r.h - 4, r.w, 4), "SongPartFootStyle"));
        view.children.push(p);
    }
}

/// The browser column between its header and footer: crumbs, the source
/// tree, the file list and the shortlist (BrowserTabs coordinates), as on
/// v1's pattern screen.
fn browser(tabs: &mut Ctrl) {
    let mut x = 1;
    for (i, (crumb, w)) in [("..", 21), ("Factory Presets", 78), ("Drum Kits", 100)].iter().enumerate() {
        let mut b = ctrl("CtrlButton", Rect::new(x, 37, *w, 18), "BrowserCrumbStyle", crumb);
        if i == 0 {
            b.set_attr("optArrowNotch", "false".into());
        }
        tabs.children.push(b);
        // The next crumb's notch takes this one's point.
        x += w - 5;
    }
    // (text, heading, icon)
    let tree: &[(&str, bool, &str)] = &[
        ("\u{25BC} CONTENT", true, ""),
        ("Einst Factory", false, "book"),
        ("\u{25BC} PLACES", true, ""),
        ("Einst", false, "home"),
        ("My Documents", false, "folder"),
        ("Desktop", false, "desktop"),
        ("\u{25BC} DRIVES", true, ""),
        ("C:\\", false, "drive"),
        ("D:\\", false, "drive"),
        ("E:\\", false, "drive"),
        ("F:\\", false, "disc"),
        ("Z:\\", false, "drive"),
        ("FAVOURITES", true, ""),
        ("SEARCHES", true, ""),
        ("SHORTLISTS", true, ""),
    ];
    for (i, (text, head, kind)) in tree.iter().enumerate() {
        let r = Rect::new(1, 60 + i as i32 * 20, 96, 20);
        let style = if *head {
            "BrowserTreeHeadStyle"
        } else if i == 1 {
            "BrowserTreeSelectedStyle"
        } else {
            "BrowserTreeItemStyle"
        };
        tabs.children.push(label(r, style, text));
        if !kind.is_empty() {
            tabs.children.push(icon(Rect::new(14, r.y, 20, 20), kind, i == 1));
        }
    }
    // The splitter between the tree and the list.
    tabs.children.push(panel(Rect::new(99, 59, 2, 520), "BrowserSplitStyle"));
    tabs.children.push(ctrl("CtrlButton", Rect::new(104, 59, 186, 18), "BrowserParentStyle", "Parent folder"));
    // (name, type, size, icon)
    let files = [("samples", "Folder", "\u{2013}", "folder"), ("Trance Kit", "Preset", "919KB", "preset")];
    let mut t = ctrl("CtrlTableView", Rect::new(104, 79, 190, 384), "BrowserListStyle", "");
    t.set_attr("columns", "Name \u{25BC}|Type|Size|\u{25BC}".into());
    t.set_attr("columnWidths", "86,46,41".into());
    t.set_attr("rows", files.iter().map(|(n, ty, sz, _)| format!("        {n}|{ty}|{sz}|")).collect::<Vec<_>>().join("\n"));
    t.set_attr("intSelectedRow", "0".into());
    for (i, (.., kind)) in files.iter().enumerate() {
        t.children.push(icon(Rect::new(2, 19 + i as i32 * 25, 22, 20), kind, false));
    }
    // The list's scroll track, the height of the list.
    t.children.push(panel(Rect::new(182, 17, 7, 367), "BrowserScrollStyle"));
    tabs.children.push(t);
    tabs.children.push(ctrl("CtrlTableView", Rect::new(104, 466, 190, 110), "BrowserListStyle", ""));
}

/// A browser icon (`kind`: book, home, folder, desktop, drive, disc,
/// preset), drawn by [`Preview`] in BrowserIconStyle's colours
/// (BrowserIconSelectedStyle on a selected row).
fn icon(r: Rect, kind: &str, selected: bool) -> Ctrl {
    let mut c = panel(r, if selected { "BrowserIconSelectedStyle" } else { "BrowserIconStyle" });
    c.overload = "BrowserIcon".into();
    c.set_attr("icon", kind.into());
    c
}

/// The browser icons, as the editor draws them.
fn draw_icon(r: &Renderer, p: &mut Painter, c: &Ctrl) {
    let rect = dreist_gui::paint::RectF::new(0.0, 0.0, c.rect.w as f32, c.rect.h as f32);
    let kind = c.attr("icon").unwrap_or("");
    let (kind, label) = if kind == "preset" { ("file", "PST") } else { (kind, "") };
    einst_editor::browsericons::draw(r, p, &c.style, rect, kind, label);
}

/// The editor draws the logo mark itself; this preview fills Einst's "E"
/// (`einst::LOGO_MARK`, a 50 x 50 design) in the logo style's colFill,
/// scaled to the control's height.
struct Preview;

impl CustomDraw for Preview {
    fn draw(&self, r: &Renderer, p: &mut Painter, c: &Ctrl) -> bool {
        if c.overload == "BrowserIcon" {
            draw_icon(r, p, c);
            return true;
        }
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

/// Absolute rects of every visible control (name, class, style, x, y, w, h).
fn dump_rects(c: &Ctrl, x: i32, y: i32, out: &mut String) {
    if !c.visible {
        return;
    }
    let (ax, ay) = (x + c.rect.x, y + c.rect.y);
    out.push_str(&format!("{}\t{}\t{}\t{}\t{}\t{}\t{}\n", c.name, c.class, c.style, ax, ay, c.rect.w, c.rect.h));
    for k in &c.children {
        dump_rects(k, ax, ay, out);
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

    // The window, with the pattern page (`root`), the song page
    // (`root-song`) and the engine mixer (`root-mixer`) on the screen.
    for (name, tab) in [("root", 0), ("root-song", 2), ("root-mixer", 6)] {
        let mut root = skin.load_skin("RootSkin.skin").expect("RootSkin.skin");
        root.rect = Rect::new(0, 0, 1135, 640);
        mount_all(&mut skin, &mut root, 0);
        if let Some(tabs) = root.find_mut("MainTabs") {
            if tab == 0 {
                let mut page = load(&mut skin, "Pattern Editor/GlobalControls.skin");
                mount_all(&mut skin, &mut page, 0);
                tabs.children.push(page);
                tabs.children.extend(pattern_grid(&mut skin));
            } else {
                let rel = PAGES.iter().find(|(n, _)| *n == if tab == 2 { "song" } else { "engine-mixer" }).map(|(_, r)| *r).unwrap_or_default();
                let mut page = load(&mut skin, rel);
                mount_all(&mut skin, &mut page, 0);
                if tab == 2 {
                    song_lanes(&mut skin, &mut page);
                }
                tabs.children.push(page);
            }
        }
        if let Some(tabs) = root.find_mut("BrowserTabs") {
            let mut header = load(&mut skin, "Browser/Header.skin");
            header.rect.y = 0;
            let mut footer = load(&mut skin, "Browser/Footer.skin");
            footer.rect.y = tabs.rect.h - footer.rect.h;
            tabs.children.push(header);
            tabs.children.push(footer);
            browser(tabs);
        }
        demo_state(&mut root);
        if let Some(g) = root.find_mut("MainTabButtons") {
            for (i, b) in g.children.iter_mut().enumerate() {
                b.on = i == tab;
            }
        }
        save(&skin, &mut fonts, &root, &out, name);
        if std::env::var_os("V1SKINS_RECTS").is_some() {
            let mut lines = String::new();
            dump_rects(&root, 0, 0, &mut lines);
            std::fs::write(out.join(format!("{name}.rects.tsv")), lines).expect("rects");
        }
    }

    for (name, rel) in PAGES {
        let mut page = load(&mut skin, rel);
        mount_all(&mut skin, &mut page, 0);
        match *name {
            "pattern" => {
                page.rect.h = 395;
                page.children.extend(pattern_grid(&mut skin));
            }
            "song" => song_lanes(&mut skin, &mut page),
            _ => {}
        }
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
        demo_state(&mut panel);
        let mut back = Ctrl { class: "CtrlPanel".into(), style: "ScreenLcdStyle".into(), rect: Rect::new(0, 0, panel.rect.w + 10, panel.rect.h + 10), visible: true, enabled: true, ..Ctrl::default() };
        panel.rect = Rect::new(5, 5, panel.rect.w, panel.rect.h);
        back.children.push(panel);
        save(&skin, &mut fonts, &back, &out, name);
    }
}
