//! Tooltip texts ([re:5e5dd2]s, the panels' tooltip lambdas
//! and per-control text overrides): asked of the control under the
//! pointer, then its parents, after each pointer event
//! ([re:eb2a9b]). Controls without a provider show
//! none (there is no generic fallback); static texts are the skin
//! `displayName`s of the controls given a [re:d9055d].

use dreist_gui::Ctrl;
use dreist_gui::tooltip::Pos;

use crate::Editor;
use crate::conv::{Conv, bool_text, db15, db15_text, depth_text, long_percent_text, midi_text};

/// Buttons given a [re:103540] ("N: On" / "N: Off"; the
/// controls connected to a provider).
const BOOL_BUTTONS: [&str; 33] = [
    "Mute", "Solo", "MuteButton", "SoloButton", "QuantizeButton", "PlayButton", "RecButton", "RecordButton", "LearnButton", "MetroButton", "SongButton", "LoopButton", "PrePostButton", "PrePost",
    "GateButton", "RestartButton", "MonoButton", "EraseButton", "FullLevelButton", "MultiLayersButton", "LoopPreview", "ArmButton", "ThruButton", "AllLayers", "AllPads", "ShowSplits",
    "InteractionModeButton", "HitLoopModeButton", "ActiveButton", "PatternPower", "PowerButton", "RepeatButton", "Follow",
];

/// Fixed help texts of the pattern tool bar and
/// sampler tooltip lambdas.
const HELP: [(&str, &str); 7] = [
    ("StepLengthCombo", "Length of time each step lasts"),
    ("LengthBox", "Number of steps in the pattern"),
    ("PatternNameBox", "Name of the pattern"),
    ("DragEditButton", "When the mouse is dragged up and down over a step, which graph is changed"),
    ("Follow", "Set the follow action to occur once a sample has been captured"),
    ("TriggerMode", "Set the mode that is used to determine sampling start and stop conditions"),
    ("SourceList", "Set where the sampler takes its input from"),
];

impl Editor {
    /// Show the tooltip for the control at `path` (or the nearest parent
    /// that has one), else hide it.
    pub(crate) fn update_tooltip(&mut self, path: &[usize]) {
        let found = (0..=path.len()).rev().find_map(|n| self.tooltip_text(&path[..n]).map(|t| (n, t)));
        match found {
            Some((n, (text, pos))) => {
                let anchor = path[..n].to_vec();
                // [re:17719e]: the
                // path buttons' tooltips are the delayed kind.
                if self.app.ctrl(&anchor).is_some_and(|c| c.overload == "BrowserPathNavigator") {
                    let mut r = self.app.abs_rect(&anchor);
                    if let Some(b) = self.browser.as_ref().and_then(|b| b.nav_button_at(self.app.mouse_pos().0 - r.x as f32)) {
                        r = dreist_gui::ctrl::Rect::new(r.x + b.x as i32, r.y, b.w as i32, r.h);
                    }
                    self.app.set_tooltip_delayed(r, &text, pos);
                    return;
                }
                match self.tooltip_rect(&anchor) {
                    Some(r) => self.app.set_tooltip_at(r, &text, pos),
                    None => self.app.set_tooltip(&anchor, &text, pos),
                }
            }
            None => self.app.hide_tooltip(),
        }
    }

    /// A tooltip placed against something other than its control: the
    /// pattern graph's at the pointer's column, above the graph.
    fn tooltip_rect(&self, path: &[usize]) -> Option<dreist_gui::ctrl::Rect> {
        let c = self.app.ctrl(path)?;
        if c.overload == "PatternGraph" {
            let oy = self.app.origin(path).1;
            return Some(dreist_gui::ctrl::Rect::new(self.app.mouse_pos().0 as i32, oy - 2, 0, 0));
        }
        // [re:45e9a3]: the out rect's y = 10 pushes it down.
        if c.name == "PlusButton" {
            let mut r = self.app.abs_rect(path);
            r.y += 10;
            return Some(r);
        }
        None
    }

    /// The shown transmod source's label and index.
    fn tooltip_source(&self) -> Option<(String, usize)> {
        let s = self.model.source;
        if s == 0 {
            return None;
        }
        let p = self.app.root.path_of(&format!("Src{s}"))?;
        let c = self.app.ctrl(&p)?;
        let label = if c.text.is_empty() { c.display_name.clone() } else { c.text.clone() };
        Some((label, s))
    }

    /// The tooltip of the control at `path` itself.
    pub(crate) fn tooltip_text(&self, path: &[usize]) -> Option<(String, Pos)> {
        let c = self.app.ctrl(path)?;
        if !c.visible || !c.enabled {
            return None;
        }
        let below = |t: String| Some((t, Pos::Below));
        let v = c.value;
        let name = c.name.as_str();
        let src = self.tooltip_source();
        // Layer and pad controls: v1's per-control providers under the
        // skin display name; a layer parameter without one shows none.
        if self.inspector.in_layer_or_pad_controls(path) || self.inspector.is_v2a(path, name) {
            if let Some(t) = crate::conv::v1_layer_tooltip(name, &c.display_name, v, c.on) {
                return below(t);
            }
            if self.inspector.layer_param_at(path, name).is_some() {
                return None;
            }
        }
        if let Some(t) = self.transmod.knob_tooltip(&self.app, &self.model, path, src.as_ref().map(|(l, s)| (l.as_str(), *s))) {
            return Some((t, Pos::Above));
        }
        if let Some(t) = self.mixers.iter().find_map(|m| m.tooltip(&self.app, &self.model, path, src.as_ref().map(|(l, s)| (l.as_str(), *s)))) {
            return below(t);
        }
        if let Some((t, pos)) = HELP.iter().find(|(n, _)| *n == name).map(|(_, t)| (t.to_string(), Pos::Below)) {
            return Some((t, pos));
        }
        match name {
            // [re:5cdc6f]: DB15 / CoarsePitch under the display name.
            "MasterSlider" => return below(db15_text(&c.display_name, v)),
            "MasterPitch" => return below(crate::conv::coarse_pitch_text(&c.display_name, v)),
            // [re:0592a8].
            "EngineSwingDepthKnob" => return below(long_percent_text(&c.display_name, v)),
            // v2 only (no such control in v1).
            "VarianceKnob" => return below(Conv::Depth.named("Variance", v)),
            "ScaleKnob" | "CompKnob" | "RandKnob" => return below(depth_text(&c.display_name, v)),
            "InstoSlider" | "Sensitivity" => return below(long_percent_text(&c.display_name, v)),
            "FullLevelKnob" => return below(midi_text(&c.display_name, v)),
            // [re:85987b]: LongPercentage under the display name.
            "MixKnob" | "OutputKnob" => return below(long_percent_text(&c.display_name, v)),
            // v2 only.
            "TrimKnob" => return below(Conv::Percent.named("Trim", v)),
            // [re:9fbf48] (with the start one above).
            "StartThresholdSlider" => return Some((format!("Start threshold: {}", db15(v)), Pos::Above)),
            "StopThresholdSlider" => return below(format!("Stop threshold: {}", db15(v))),
            "PreviewVolume" | "PreviewVolumeKnob" if !c.display_name.is_empty() => return below(db15_text(&c.display_name, v)),
            // [re:948d61].
            "SliceMode" => return below("If enabled, transients are detected and used to slice the file. If disabled the audio file is divided in to equal sections".into()),
            _ => {}
        }
        // v2 only: macros ([re:284b3d]; sources above).
        if let Some(k) = name.strip_prefix("Macro").and_then(|k| k.parse::<usize>().ok()).filter(|_| c.class.contains("Knob")) {
            return below(Conv::Percent.named(&format!("Macro {k}"), v));
        }
        if name.strip_prefix("Src").is_some_and(|k| k.parse::<usize>().is_ok()) {
            let t = if c.display_name.is_empty() { c.text.clone() } else { c.display_name.clone() };
            return Some((t, Pos::Above));
        }
        if let Some(t) = self.inspector.vsplit_tooltip(&self.model, c) {
            return below(t);
        }
        if c.overload == "TransmodSequencerGrid" {
            let ox = self.app.origin(path).0 as f32;
            return crate::transmod::seq_tooltip(&self.model, c, self.app.mouse_pos().0 - ox).map(|t| (t, Pos::Below));
        }
        if c.overload == "EditorPreviewWaveEditor" {
            let ox = self.app.origin(path).0 as f32;
            return self.slicer.wave_tooltip(&self.model, path, self.app.mouse_pos().0 - ox, c.rect.w as f32).map(|t| (t, Pos::Below));
        }
        if c.overload == "LayerWaveDisplay" {
            let (ox, oy) = self.app.origin(path);
            let (mx, my) = self.app.mouse_pos();
            return self.inspector.wave_tooltip(&self.app, &self.model, path, mx - ox as f32, my - oy as f32).map(|t| (t, Pos::Below));
        }
        if c.overload == "PatternGraph" {
            let ox = self.app.origin(path).0 as f32;
            return self.pattern.as_ref()?.graph_tooltip(&self.app, &self.model, self.app.mouse_pos().0 - ox).map(|t| (t, Pos::Above));
        }
        if c.overload == "PatternSelector" {
            let n: usize = name.strip_prefix("Key")?.parse().ok()?;
            return below(self.pattern_selector_tooltip(n));
        }
        if c.overload == "BrowserPathNavigator" {
            let (x, _) = self.app.mouse_pos();
            let ox = self.app.origin(path).0 as f32;
            return self.browser.as_ref()?.nav_button_at(x - ox).map(|b| (b.display, Pos::Below));
        }
        generic(c)
    }
}

/// [re:d9055d] / [re:103540] on buttons that carry a
/// display name.
fn generic(c: &Ctrl) -> Option<(String, Pos)> {
    if !matches!(c.class.as_str(), "CtrlButton" | "CtrlPopupButton") || c.display_name.is_empty() {
        return None;
    }
    let bool_button = BOOL_BUTTONS.contains(&c.name.as_str()) || c.name.strip_prefix("Control").is_some_and(|k| k.parse::<usize>().is_ok());
    if bool_button && c.class == "CtrlButton" {
        return Some((bool_text(&c.display_name, c.on), Pos::Below));
    }
    Some((c.display_name.clone(), Pos::Below))
}

impl Editor {
    /// [re:812db4]: "<name> | Steps: n |
    /// Res: r | MIDI: <key> (Note: n)" (an unused slot: "Pattern n", 16,
    /// 1/16); the note is [re:e0f870]
    /// [re:e0f870] = 12 + pattern.
    fn pattern_selector_tooltip(&self, n: usize) -> String {
        let e = self.model.engine;
        let node = self.model.pattern_node(e, n);
        let name = Some(self.model.pattern_name(e, n)).filter(|s| node.is_some() && !s.is_empty()).unwrap_or_else(|| format!("Pattern {}", n + 1));
        let steps = node.and_then(|p| p.attr("PatternSteps")).and_then(|v| v.trim().parse::<i32>().ok()).unwrap_or(16);
        let res = node.and_then(|p| p.attr("Resolution")).unwrap_or("1/16").to_string();
        let note = (12 + n).min(127) as u8;
        let mode = crate::options::Options::load().int("MIDINumbering", 1).clamp(0, 2) as u8;
        format!("{name} | Steps: {steps} | Res: {res} | MIDI: {} (Note: {note})", crate::midiconsole::key_name(note, mode, false))
    }
}
