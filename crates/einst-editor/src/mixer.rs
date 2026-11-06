//! The v1 mixer pages ([re:faf56f]): the selected channel's effect
//! chain on top (`FXPanel`, [`crate::fxchain::FxPanel`]) and the
//! channel strips fixed in the page skin below, with the MIX / SEND
//! (and the engine mixer's SETUP) view buttons.
//!
//! - Layer mixer ([re:de6382]): the 8 layers of the selected pad
//!   (`Channel0..7`, `LayerMixerStrip.skin`).
//! - Pad mixer ([re:d84457]): the selected engine's 16 pads (`PadChan0..15`,
//!   `MixerStripPad.skin`).
//! - Engine mixer: the 8 engines (`Channel0..7`, `EngineMixerStrip.skin`);
//!   SETUP shows engine pitch, swing template and depth and the tempo
//!   modifier.
//! - Global mixer ([re:51d49b]): 4 global aux returns and the master.
//!
//! Selection and the shown chain (ev_37b25110…, ev_06897cf5…,
//! ev_975c5a77…, ev_550d1ba0…, ev_705113971…, ev_e8192a36…): a press on
//! a strip outside its controls ([re:fcb381] [re:42d973], aux [re:4a3270])
//! selects it: an engine selects that engine ([re:1068c4]), a pad goes
//! through the pad selection (cmd toggles, shift extends), a layer becomes
//! the current layer, an aux or the master shows its chain ([re:070ea3] /
//! [re:d1e5bd]). The strip takes its `…Selected` background (other selected
//! pads / engines `…Secondary`, the rest alternate `…Odd` / `…Even`) and
//! the glue image joins it to the chain.
//!
//! Messages ([re:b8e0fd]): engine/pad parameter value at v0 with the
//! parameter at v1 and source at v2; layer parameter index at v0 and value
//! at v1; send levels with the aux at v0, level at v1; aux 0x9f and master
//! 0xa5/0xa6 gains at v0.

use dreist_engine::binding::id;
use dreist_formats::{EngineParam, LayerParam, Node, PadParam};
use dreist_gui::app::{App, UiEvent};
use dreist_gui::{Ctrl, Rect, Skin};

use crate::EditorError;
use crate::command::{Command, msg};
use crate::model::{ENGINES, LAYERS, Model};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Global,
    Pad,
    Layer,
    Engine,
}

impl Kind {
    /// The LCD page of the mixer.
    pub fn page(self) -> usize {
        match self {
            Kind::Layer => crate::front::LAYER_MIXER_PAGE,
            Kind::Pad => crate::front::PAD_MIXER_PAGE,
            Kind::Engine => crate::front::ENGINE_MIXER_PAGE,
            Kind::Global => crate::front::GLOBAL_MIXER_PAGE,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Channel {
    Engine(usize),
    Aux(usize),
    Master,
    Pad(usize),
    Layer(usize),
}

/// The engine strip swing menu's head items.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SwingRequest {
    /// Extract timing from engine k's current pattern.
    Extract(usize),
    ImportMidi,
    ImportGuru,
}

/// OutputCombo items, as the original v1 fills them ([re:eab727],
/// [re:59154a] long names): Parent, Master, Sub output 1..15, Buss 1..4.
pub fn output_items() -> Vec<String> {
    (0..21)
        .map(|i| match i {
            0 => "Parent".to_string(),
            1 => "Master".to_string(),
            2..=16 => format!("Sub output {}", i - 1),
            _ => format!("Buss {}", i - 16),
        })
        .collect()
}

/// Preview output pairs ([re:d51559]): Master, Sub output 1..15 (the
/// preview output is the 1-based item).
pub fn preview_output_items() -> Vec<String> {
    (0..16).map(|k| if k == 0 { "Master".to_string() } else { format!("Sub output {k}") }).collect()
}

/// The engine's output location index of item `i` of [`output_items`]
/// (v1's four busses are the global aux buses, 34..37).
pub fn output_location(i: usize) -> f32 {
    match i {
        0..=16 => i as f32,
        _ => (i.min(20) - 17 + 34) as f32,
    }
}

/// Item index of a stored `OutputLocation` name (unknown → Parent).
pub(crate) fn output_index(name: &str) -> usize {
    let num = |p: &str| name.strip_prefix(p).and_then(|n| n.trim().parse::<usize>().ok());
    match name {
        "Master" => 1,
        _ => match (num("Sub output "), num("Aux ")) {
            (Some(k @ 1..=15), _) => k + 1,
            (_, Some(k @ 1..=4)) => k + 16,
            _ => 0,
        },
    }
}

/// The aux send of a strip knob: `SendKnob<n>` (layer / engine strips) or
/// `Send<n>` (pad strips).
fn send_index(name: &str) -> Option<usize> {
    let n = name.strip_prefix("SendKnob").or_else(|| name.strip_prefix("Send"))?;
    n.parse().ok().filter(|_| n.len() == 1)
}

/// A strip control's role: `GainSlider`, `PanKnob`, `Pitch` or
/// `SendKnob<n>` (the strip skins' names differ).
fn canon(name: &str) -> String {
    match name {
        "EnginePitchKnob" => "Pitch".into(),
        n => match send_index(n) {
            Some(k) => format!("SendKnob{k}"),
            None => n.to_string(),
        },
    }
}

/// The strip controls each view shows (the rest of a strip always shows).
const MIX_VIEW: [&str; 5] = ["PanKnob", "GainSlider", "VUMeterLeft", "VUMeterRight", "OutputCombo"];
const SETUP_VIEW: [&str; 9] = ["EnginePitchLabel", "EnginePitchKnob", "SwingDepthLabel", "SwingLabel", "EngineSwingTemplateCombo", "SwingKnob", "TempoLabel", "EngineNumBox", "EngineDenomBox"];

/// Whether strip control `name` shows in view `mode` (Mix 0, Send 1,
/// Setup 2); `None` for controls every view shows.
fn shown_in(name: &str, mode: usize) -> Option<bool> {
    if name.starts_with("SendLabel") || send_index(name).is_some() {
        return Some(mode == 1);
    }
    if SETUP_VIEW.contains(&name) {
        return Some(mode == 2);
    }
    if name == "OutputCombo" {
        return Some(mode != 2);
    }
    MIX_VIEW.contains(&name).then_some(mode == 0)
}

/// An `AutoShorteningLabel`: its text cut to fit (`fitWidth`, else the
/// label's width) with an ellipsis.
pub(crate) fn draw_short_label(r: &dreist_gui::render::Renderer, p: &mut dreist_gui::paint::Painter, c: &Ctrl) -> bool {
    if c.overload != "AutoShorteningLabel" || c.class != "CtrlLabel" {
        return false;
    }
    let font = r.styles.font(&c.style);
    let avail = c.attr_f32("fitWidth", c.rect.w as f32 - 4.0);
    let mut text = c.text.clone();
    if p.text_width(&font, &text, 0.0) > avail {
        while !text.is_empty() && p.text_width(&font, &format!("{}\u{2026}", text.trim_end()), 0.0) > avail {
            text.pop();
        }
        text = format!("{}\u{2026}", text.trim_end());
    }
    let mut k = c.clone();
    k.overload.clear();
    k.text = text;
    k.rect = Rect::new(0, 0, c.rect.w, c.rect.h);
    r.draw(p, &k);
    true
}

pub struct Mixer {
    pub(crate) kind: Kind,
    pub(crate) page: Vec<usize>,
    /// The strips (host panels) and their channels.
    strips: Vec<(Channel, Vec<usize>)>,
    /// The glue images: path, rect in the skin.
    glue: Vec<(String, Vec<usize>, Rect)>,
    /// Mix (0), Send (1), Setup (2).
    mode: usize,
    /// Channel whose OutputCombo menu is open.
    output_menu: Option<Channel>,
    /// The output selector's right-click menu is open (its strip kind).
    route_all_menu: Option<Channel>,
    /// The swing template menu of an engine strip: engine, template paths.
    swing_menu: Option<(usize, Vec<std::path::PathBuf>)>,
    /// A swing menu head item for the editor (dialogs, the groove).
    pub swing_request: Option<SwingRequest>,
    fx: Option<crate::fxchain::FxPanel>,
    /// Global mixer: the aux or the master whose chain is shown.
    bus_view: Channel,
}

const STRIP_PREFIX: [(&str, &str); 4] = [("Layer", "LayerChannelBackground"), ("Pad", "PadChannelBackground"), ("Engine", "EngineChannelBackground"), ("Aux", "AuxChannelBackground")];

fn attr_f32(n: Option<&Node>, k: &str, d: f32) -> f32 {
    n.and_then(|n| n.f32(k)).unwrap_or(d).clamp(0.0, 1.0)
}

fn ctrl_at<'a>(root: &'a mut Ctrl, path: &[usize]) -> &'a mut Ctrl {
    let mut c = root;
    for &i in path {
        c = &mut c.children[i];
    }
    c
}

impl Mixer {
    /// The effect chain panel on top, if this page has one.
    pub(crate) fn fx_panel(&self) -> Option<&crate::fxchain::FxPanel> {
        self.fx.as_ref()
    }

    /// Mount the page's skin into `page` and find its strips and chain.
    pub fn build(skin: &mut Skin, root: &mut Ctrl, page: &[usize], kind: Kind) -> Result<Mixer, EditorError> {
        let host = ctrl_at(root, page);
        crate::front::mount_into(skin, host, crate::front::PAGES[kind.page()].1)?;
        crate::front::mount_all(skin, host)?;
        let mut strips = Vec::new();
        let mut glue = Vec::new();
        let mut fx_host = None;
        let num = |s: &str, p: &str| s.strip_prefix(p).and_then(|n| n.parse::<usize>().ok());
        for (i, c) in host.children.iter_mut().enumerate() {
            let path = [page, &[i]].concat();
            let ch = match kind {
                Kind::Layer => num(&c.name, "Channel").map(Channel::Layer),
                Kind::Engine => num(&c.name, "Channel").map(Channel::Engine),
                Kind::Pad => num(&c.name, "PadChan").map(Channel::Pad),
                Kind::Global => num(&c.name, "AuxChannel").map(Channel::Aux).or((c.name == "MasterChannel").then_some(Channel::Master)),
            };
            if let Some(ch) = ch {
                c.overload = "MixerStrip".into();
                c.set_attr("channel", format!("{ch:?}"));
                for n in ["NameLabel", "SwingLabel", "TempoLabel"] {
                    if let Some(l) = c.find_mut(n) {
                        l.passive = true;
                    }
                }
                // The name fits its strip.
                let fit = (c.rect.w - 6).to_string();
                if let Some(l) = c.find_mut("NameLabel") {
                    l.overload = "AutoShorteningLabel".into();
                    l.set_attr("fitWidth", fit);
                }
                strips.push((ch, path));
            } else if c.name.ends_with("Glue") {
                c.passive = true;
                glue.push((c.name.clone(), path, c.rect));
            } else if c.name == "FXPanel" {
                fx_host = Some(path);
            } else if c.name.starts_with("Glue") || c.name.ends_with("GlueMicro") {
                c.passive = true;
            }
        }
        let fx = match fx_host {
            Some(h) => Some(crate::fxchain::FxPanel::build(skin, root, h)?),
            None => None,
        };
        Ok(Mixer { kind, page: page.to_vec(), strips, glue, mode: 0, output_menu: None, route_all_menu: None, swing_menu: None, swing_request: None, fx, bus_view: Channel::Aux(0) })
    }

    /// A chain preset was saved somewhere: the panel counts presets again.
    pub fn forget_presets(&self) {
        if let Some(fx) = &self.fx {
            fx.forget_presets();
        }
    }

    pub fn take_save_request(&mut self) -> Option<crate::fxchain::SaveRequest> {
        self.fx.as_mut()?.save_request.take()
    }

    /// The chain the FX panel shows.
    pub(crate) fn fx_target(&self, model: &Model) -> crate::fxchain::Target {
        let e = model.engine;
        match self.kind {
            Kind::Global => match self.bus_view {
                Channel::Master => crate::fxchain::Target::Master,
                Channel::Aux(k) => crate::fxchain::Target::Aux(k),
                _ => crate::fxchain::Target::Aux(0),
            },
            Kind::Engine => crate::fxchain::Target::Engine(e),
            Kind::Pad => crate::fxchain::Target::Pad(e, model.pad[e]),
            Kind::Layer => crate::fxchain::Target::Layer(e, model.pad[e], model.layer[e]),
        }
    }

    /// The pad of pad strip `k` (the engine's visible pads).
    fn pad_of(model: &Model, k: usize) -> usize {
        model.visible_pads().0 + k
    }

    /// Whether `ch` shows its chain (primary selection) or is otherwise
    /// selected (other selected pads / engines).
    fn selection_of(&self, model: &Model, ch: Channel) -> (bool, bool) {
        let e = model.engine;
        match ch {
            Channel::Engine(k) => {
                let sel = crate::multipad::engine_selection(model) & (1u8 << k.min(7)) != 0;
                (k == e, sel && k != e)
            }
            Channel::Aux(_) | Channel::Master => (self.bus_view == ch, false),
            Channel::Pad(k) => {
                let (anchor, mask) = crate::multipad::selection(model, e);
                let p = Self::pad_of(model, k);
                (p == anchor, p != anchor && mask & (1u64 << p.min(63)) != 0)
            }
            Channel::Layer(k) => (k == model.layer[e], false),
        }
    }

    /// The document node of a channel.
    fn node(model: &Model, ch: Channel) -> Option<&Node> {
        let e = model.engine;
        match ch {
            Channel::Engine(k) => model.doc.child("Engine", &format!("Engine{k}")),
            Channel::Aux(k) => model.doc.child("GlobalAux", &format!("GlobalAux{k}")),
            Channel::Master => model.doc.child("MasterChannel", "MasterChannel"),
            Channel::Pad(k) => model.pad_node(e, Self::pad_of(model, k)),
            Channel::Layer(k) => model.layer_node(e, model.pad[e], k),
        }
    }

    pub fn sync(&mut self, app: &mut App, model: &Model) {
        if self.kind != Kind::Engine && self.mode == 2 {
            self.mode = 0;
        }
        let t = self.fx_target(model);
        if let Some(fx) = &mut self.fx {
            fx.sync(app, model, t);
        }
        let mode = self.mode;
        // View buttons: the engine mixer's ModeGroup, the others' Mix /
        // Send buttons; the link buttons.
        if let Some(page) = app.ctrl_mut(&self.page) {
            for c in page.children.iter_mut() {
                match c.name.as_str() {
                    "ModeGroup" => {
                        for (i, b) in c.children.iter_mut().enumerate() {
                            b.on = i == mode;
                        }
                    }
                    "MixButton" => c.on = mode == 0,
                    "SendButton" => c.on = mode == 1,
                    "AllLayers" => c.on = model.all_layers,
                    "AllPads" => c.on = model.all_pads,
                    "AllGroups" => c.on = model.all_rows,
                    _ => {}
                }
            }
        }
        let swing_names: Vec<String> = (0..ENGINES)
            .map(|k| {
                model
                    .engine_node(k)
                    .and_then(|n| n.children.iter().find(|c| c.class == "SwingTemplate"))
                    .map(|n| crate::swing::name_of(Some(n), std::path::Path::new("")))
                    .unwrap_or_else(|| "None".into())
            })
            .collect();
        let editing = app.is_editing();
        let mut selected_rect = None;
        for (i, (ch, path)) in self.strips.clone().into_iter().enumerate() {
            let label = self.label(model, ch);
            let node = Self::node(model, ch);
            let gain_key = match ch {
                Channel::Layer(_) => LayerParam::Gain.name(),
                _ => "Gain",
            };
            let gain_default = match ch {
                Channel::Layer(_) => LayerParam::Gain.default_value(),
                _ => dreist_formats::params::UNITY_GAIN,
            };
            let gain = attr_f32(node, gain_key, gain_default);
            let pan = attr_f32(node, if matches!(ch, Channel::Layer(_)) { LayerParam::Pan.name() } else { "Pan" }, 0.5);
            let pitch = attr_f32(node, "Pitch", 0.5);
            let (mute, solo) = (node.is_some_and(|n| n.truthy("Mute")), node.is_some_and(|n| n.truthy("Solo")));
            // [re:5575fd]: PrePostButton = [re:892cf2] (`IsPreFade`); no
            // attribute: the option `AuxesArePreFade` default.
            let pre_fade = match node.and_then(|n| n.attr("IsPreFade")) {
                Some(_) => node.is_some_and(|n| n.truthy("IsPreFade")),
                None => crate::options::Options::load().bool("AuxesArePreFade", false),
            };
            let sends: Vec<f32> = (0..4).map(|k| attr_f32(node, &dreist_formats::params::aux_send_name(k), 0.0)).collect();
            let output = output_items()[output_index(node.and_then(|n| n.attr("OutputLocation")).unwrap_or("Parent"))].clone();
            let (num, den, depth) = (
                node.and_then(|n| n.u32("TempoNumerator")).unwrap_or(1).clamp(1, 16),
                node.and_then(|n| n.u32("TempoDenominator")).unwrap_or(1).clamp(1, 16),
                node.and_then(|n| n.f32("SwingTemplateDepth")).unwrap_or(0.0).clamp(0.0, 1.0),
            );
            let swing = match ch {
                Channel::Engine(k) => swing_names[k].clone(),
                _ => String::new(),
            };
            let (sel, sub) = self.selection_of(model, ch);
            let style = match ch {
                Channel::Master => if sel { "MasterChannelBackgroundSelected" } else { "MasterChannelBackground" }.to_string(),
                _ => {
                    let tag = format!("{ch:?}");
                    let prefix = STRIP_PREFIX.iter().find(|(p, _)| tag.starts_with(p)).map(|(_, s)| *s).unwrap_or("LayerChannelBackground");
                    let suffix = if sel {
                        "Selected"
                    } else if sub {
                        "Secondary"
                    } else if i % 2 == 0 {
                        "Odd"
                    } else {
                        "Even"
                    };
                    format!("{prefix}{suffix}")
                }
            };
            let has_views = !matches!(ch, Channel::Aux(_) | Channel::Master);
            let Some(strip) = app.ctrl_mut(&path) else { continue };
            if sel {
                selected_rect = Some((ch, strip.rect));
            }
            strip.style = style;
            strip.walk_mut(&mut |k: &mut Ctrl| {
                if has_views {
                    if let Some(v) = shown_in(&k.name, mode) {
                        k.visible = v;
                    }
                }
                match k.name.as_str() {
                    "NameLabel" => k.text = label.clone(),
                    "GainSlider" => k.value = gain,
                    "PanKnob" => k.value = pan,
                    "EnginePitchKnob" => k.value = pitch,
                    "SwingKnob" => k.value = depth,
                    "EngineSwingTemplateCombo" => k.text = swing.clone(),
                    "EngineNumBox" if !editing => k.text = num.to_string(),
                    "EngineDenomBox" if !editing => k.text = den.to_string(),
                    "MuteButton" => k.on = mute,
                    "SoloButton" => k.on = solo,
                    "PreButton" => k.on = pre_fade,
                    "PostButton" => k.on = !pre_fade,
                    "OutputCombo" => k.text = output.clone(),
                    n => {
                        if let Some(s) = send_index(n) {
                            k.value = sends.get(s).copied().unwrap_or(0.0);
                        }
                    }
                }
            });
        }
        // The glue joins the selected strip to the chain.
        for (name, path, rect) in &self.glue {
            let Some(g) = app.ctrl_mut(path) else { continue };
            match selected_rect {
                Some((ch, r)) => {
                    let master = name == "MasterGlue";
                    g.visible = master == (ch == Channel::Master);
                    let dx = if master {
                        2
                    } else if matches!(ch, Channel::Aux(_)) {
                        0
                    } else {
                        1
                    };
                    let nr = Rect::new(r.x + dx, rect.y, rect.w, rect.h);
                    if g.rect != nr {
                        g.set_rect(nr);
                    }
                }
                None => g.visible = false,
            }
        }
    }

    fn label(&self, model: &Model, ch: Channel) -> String {
        let e = model.engine;
        match ch {
            Channel::Engine(k) => model.engine_name(k),
            Channel::Aux(k) => model
                .doc
                .children
                .iter()
                .find(|c| c.class == "GlobalAux" && c.name.as_deref() == Some(&format!("GlobalAux{k}")))
                .and_then(|n| n.attr("AuxName"))
                .map(str::to_string)
                .unwrap_or_else(|| format!("Aux {}", k + 1)),
            Channel::Master => "Master".into(),
            Channel::Pad(k) => {
                let p = Self::pad_of(model, k);
                model.pad_name(e, p).unwrap_or_else(|| format!("Pad {}", p + 1))
            }
            Channel::Layer(k) => model.layer_name(e, model.pad[e], k).unwrap_or_else(|| format!("Layer {}", k + 1)),
        }
    }

    /// Code-drawn parts (the FX chain's scroll bar, shortened labels).
    pub fn draw(&self, r: &dreist_gui::render::Renderer, p: &mut dreist_gui::paint::Painter, c: &Ctrl) -> bool {
        self.fx.as_ref().is_some_and(|f| f.draw(r, p, c))
    }

    /// The control at `path` belongs to an effect in this mixer's chain.
    pub fn is_effect_control(&self, app: &App, path: &[usize]) -> bool {
        self.fx.as_ref().is_some_and(|f| f.contains(app, path))
    }

    /// The conversion of a strip knob's typed edit.
    pub fn edit_curve(&self, app: &App, path: &[usize]) -> Option<dreist_formats::params::Curve> {
        use dreist_formats::params::Curve;
        if self.is_effect_control(app, path) {
            return Some(Curve::Linear(0.0, 100.0));
        }
        let (ch, name) = self.hit(app, path)?;
        Some(match (ch, name.as_str()) {
            (_, "GainSlider") => Curve::Decibels,
            (_, "PanKnob") => Curve::Linear(-1.0, 1.0),
            (Channel::Layer(_), "Pitch") => LayerParam::CoarseTune.curve(),
            (Channel::Engine(_), "Pitch") => EngineParam::Pitch.curve(),
            (_, "Pitch") => PadParam::Pitch.curve(),
            (_, "SwingKnob") => Curve::Linear(0.0, 100.0),
            (_, n) if n.starts_with("SendKnob") => Curve::Decibels,
            _ => return None,
        })
    }

    /// Sources modulating the strip knob at `path` (engine / pad / layer
    /// parameters and aux sends), and effect parameters in the chain.
    pub fn modulation_mask(&self, app: &App, model: &Model, path: &[usize]) -> Option<u32> {
        if let Some(m) = self.fx.as_ref().and_then(|f| f.modulation_mask(app, model, path)) {
            return Some(m);
        }
        let (ch, name) = self.hit(app, path)?;
        let node = match ch {
            Channel::Engine(_) | Channel::Pad(_) | Channel::Layer(_) => Self::node(model, ch),
            _ => return None,
        };
        let key = Self::param_key(ch, &name)?;
        Some(crate::model::modulation_mask(node, &key))
    }

    /// The document attribute of a strip knob.
    fn param_key(ch: Channel, name: &str) -> Option<String> {
        Some(match (ch, name) {
            (Channel::Layer(_), "GainSlider") => LayerParam::Gain.name().to_string(),
            (Channel::Layer(_), "PanKnob") => LayerParam::Pan.name().to_string(),
            (Channel::Layer(_), "Pitch") => LayerParam::CoarseTune.name().to_string(),
            (_, "GainSlider") => "Gain".into(),
            (_, "PanKnob") => "Pan".into(),
            (_, "Pitch") => "Pitch".into(),
            (_, n) => dreist_formats::params::aux_send_name(n.strip_prefix("SendKnob")?.parse().ok()?),
        })
    }

    /// The strip knob tooltip ([re:fcb381] [re:dafd34] and the derived
    /// strips): Decibels "Gain", Pan "Pan", Percent "Send n", Semitones
    /// "Pitch"; the master gain is a DB15 provider of its display name.
    pub fn tooltip(&self, app: &App, model: &Model, path: &[usize], source: Option<(&str, usize)>) -> Option<String> {
        use crate::conv::{Conv, db15_text, transmod_text};
        if let Some(f) = self.fx.as_ref().filter(|f| f.contains(app, path)) {
            return f.tooltip(app, model, path, source.map(|(_, s)| s));
        }
        let (ch, name) = self.hit(app, path)?;
        let c = app.ctrl(path)?;
        let v = c.value;
        let (conv, label) = match name.as_str() {
            "GainSlider" => (Conv::Decibels, "Gain".to_string()),
            "PanKnob" => (Conv::Pan, "Pan".into()),
            "Pitch" => (Conv::Semitones, "Pitch".into()),
            "SwingKnob" => (Conv::Percent, "Swing".into()),
            n => (Conv::Percent, format!("Send {}", n.strip_prefix("SendKnob")?.parse::<usize>().ok()? + 1)),
        };
        match ch {
            Channel::Master if name == "GainSlider" => return Some(db15_text(&c.display_name, v)),
            Channel::Master | Channel::Aux(_) => return (name == "GainSlider").then(|| conv.named(&label, v)),
            _ => {}
        }
        if name == "SwingKnob" {
            return Some(conv.named(&label, v));
        }
        let node = Self::node(model, ch);
        let key = Self::param_key(ch, &name)?;
        let src = source.map(|(l, s)| (l, node.and_then(|n| dreist_engine::engine::doc::attr_name(&key, s).and_then(|a| n.f32(&a))).unwrap_or(0.0)));
        Some(transmod_text(conv, &label, v, src))
    }

    /// A knob double click's reset value: the chain's effects, then
    /// [re:7b12f8] (Gain ∛0.5, Pan 0.5, Pitch 0.5, sends 0, swing depth 0);
    /// aux strips ([re:223b19] [re:c32b02]) only reset the gain.
    pub fn reset_value(&self, app: &App, path: &[usize]) -> Option<f32> {
        let name = app.ctrl(path)?.name.clone();
        if let Some(v) = self.fx.as_ref().and_then(|f| f.reset_value(app, path, &name)) {
            return Some(v);
        }
        let (ch, name) = self.hit(app, path)?;
        match (ch, name.as_str()) {
            (_, "GainSlider") => Some(dreist_formats::params::UNITY_GAIN),
            (Channel::Aux(_), _) => None,
            (_, "PanKnob") => Some(0.5),
            // The pad / engine / master strips override the default to 0.5
            // ([re:06731b], [re:0a2673]; master [re:81d585]).
            (_, "Pitch") => Some(0.5),
            (_, "SwingKnob") => Some(0.0),
            (_, n) if n.starts_with("SendKnob") => Some(0.0),
            _ => None,
        }
    }

    /// The strip at `path` (or containing it).
    fn strip_of(&self, path: &[usize]) -> Option<(Channel, &[usize])> {
        self.strips.iter().find(|(_, s)| path.starts_with(s)).map(|(c, s)| (*c, s.as_slice()))
    }

    /// The channel and control role for an event path inside a strip.
    fn hit(&self, app: &App, path: &[usize]) -> Option<(Channel, String)> {
        let (ch, s) = self.strip_of(path)?;
        if path.len() <= s.len() {
            return None;
        }
        Some((ch, canon(&app.ctrl(path)?.name)))
    }

    /// The pad whose strip background (or a passive child: label, panel)
    /// is at `path`: [re:bfddf4] [re:c455d6] opens the pad menu there
    /// (right clicks on the knobs keep their own menus).
    pub fn pad_strip_at(&self, app: &App, path: &[usize]) -> Option<usize> {
        if !app.ctrl(&self.page).is_some_and(|c| c.visible) {
            return None;
        }
        let (Channel::Pad(k), s) = self.strip_of(path)? else { return None };
        let interactive = (s.len()..=path.len()).any(|n| {
            app.ctrl(&path[..n]).is_some_and(|c| matches!(c.class.as_str(), "CtrlButton" | "CtrlPopupButton" | "CtrlKnob" | "CtrlSlider" | "CtrlPanSlider" | "CtrlScrollBar" | "CtrlValueBox"))
        });
        (!interactive).then_some(k)
    }

    /// A strip press: select the channel and show its chain.
    fn select(&mut self, model: &mut Model, ch: Channel, cmd: bool, shift: bool, out: &mut Vec<Command>) {
        let e = model.engine;
        match ch {
            Channel::Engine(k) => {
                // [re:1068c4]: [re:352eb9] (cmd toggles, shift extends) →
                // `e_setEngineSelections` 0x43.
                let cur = crate::multipad::select_engine(model, k, cmd, shift);
                model.engine = cur;
                model.doc.set("CurrentEngine", cur);
                out.push(Command::SelectEngine(cur));
            }
            Channel::Aux(_) | Channel::Master => self.bus_view = ch,
            Channel::Pad(k) => {
                let p = Self::pad_of(model, k);
                out.push(crate::multipad::select(model, e, p, cmd, shift));
            }
            Channel::Layer(k) => model.layer[e] = k,
        }
    }

    pub fn handle(&mut self, app: &mut App, model: &mut Model, ev: &UiEvent, out: &mut Vec<Command>) -> bool {
        if let Some(fx) = &mut self.fx {
            if fx.handle(app, model, ev, out) {
                return true;
            }
        }
        let e = model.engine as i32;
        let p = model.pad[model.engine] as i32;
        if let UiEvent::MenuSelected { index, .. } = ev {
            if let Some((k, paths)) = self.swing_menu.take() {
                // [re:832a12]: the three head items post 0x10f (extract timing
                // of engine k's current pattern), 0x110 / 0x111 (imports).
                match *index {
                    0 => self.swing_request = Some(SwingRequest::Extract(k)),
                    1 => self.swing_request = Some(SwingRequest::ImportMidi),
                    2 => self.swing_request = Some(SwingRequest::ImportGuru),
                    _ => {}
                }
                let Some(path) = index.checked_sub(4).and_then(|i| paths.get(i)) else { return true };
                if let Some(n) = crate::swing::load(path) {
                    let sw = dreist_engine::pattern::Swing::from_node(&n);
                    out.push(Command::SwingTemplate(k, Box::new(n), Box::new(sw)));
                }
                return true;
            }
            // [re:703e6d] [re:60ed40], [re:b8ab71] [re:2788cc], [re:ab1a4e]
            // [re:c9fdd9]: route every pad / layer / engine to parent (0),
            // master (1) or "Sub output n" in order (i + 2).
            if let Some(ch) = self.route_all_menu.take() {
                let engine = matches!(ch, Channel::Engine(_));
                let choice = if engine { *index + 1 } else { *index };
                let v = |i: usize| match choice {
                    0 => 0.0,
                    1 => 1.0,
                    _ => (i + 2) as f32,
                };
                match ch {
                    Channel::Engine(_) => out.extend((0..ENGINES).map(|k| msg(id::SET_ENGINE_OUTPUT, &[k as i32], &[v(k)]))),
                    Channel::Pad(_) => out.extend((0..64).map(|k| msg(id::SET_PAD_OUTPUT, &[e, k as i32], &[v(k)]))),
                    Channel::Layer(_) => out.extend((0..LAYERS).map(|k| msg(id::SET_LAYER_OUTPUT, &[e, p, k as i32], &[v(k)]))),
                    _ => {}
                }
                return true;
            }
            if let Some(ch) = self.output_menu.take() {
                let v = output_location(if matches!(ch, Channel::Aux(_)) { *index + 1 } else { *index });
                let cmd = match ch {
                    Channel::Engine(k) => msg(id::SET_ENGINE_OUTPUT, &[k as i32], &[v]),
                    Channel::Pad(k) => msg(id::SET_PAD_OUTPUT, &[e, Self::pad_of(model, k) as i32], &[v]),
                    Channel::Layer(k) => msg(id::SET_LAYER_OUTPUT, &[e, p, k as i32], &[v]),
                    Channel::Aux(k) => msg(id::SET_GLOBAL_AUX_OUTPUT, &[k as i32], &[v]),
                    Channel::Master => return true,
                };
                out.push(cmd);
                return true;
            }
        }
        if let UiEvent::Mouse { path, overload, kind: dreist_gui::app::MouseKind::Down, .. } = ev {
            if overload == "MixerStrip" {
                let Some((ch, s)) = self.strip_of(path) else { return false };
                if path.len() != s.len() {
                    return false;
                }
                let (cmd, shift) = (app.mods.cmd, app.mods.shift);
                self.select(model, ch, cmd, shift, out);
                return true;
            }
        }
        if let UiEvent::RightClicked { path, name } = ev {
            if name == "OutputCombo" {
                let Some((ch, _)) = self.hit(app, path) else { return false };
                let items: Vec<String> = match ch {
                    Channel::Engine(_) => vec!["Set all to master".into(), "Output sequentially".into()],
                    Channel::Pad(_) | Channel::Layer(_) => vec!["Set all to parent".into(), "Set all to master".into(), "Output sequentially".into()],
                    _ => return false,
                };
                app.open_menu(path, items, None);
                self.route_all_menu = Some(ch);
                return true;
            }
        }
        if let UiEvent::Clicked { path, name, .. } = ev {
            if name == "OutputCombo" {
                let Some((ch, _)) = self.hit(app, path) else { return false };
                let current = app.ctrl(path).map(|c| c.text.clone()).unwrap_or_default();
                // [re:3ec96a]: an aux return offers Master and the subs.
                let items: Vec<String> = if matches!(ch, Channel::Aux(_)) { output_items()[1..17].to_vec() } else { output_items() };
                let checked = items.iter().position(|t| *t == current);
                app.open_menu(path, items, checked);
                self.output_menu = Some(ch);
                return true;
            }
            // The engine strip's swing template menu ([re:6012ce]): Extract
            // timing from current pattern, Import MIDI template, Import Guru
            // swing/shuffle, then the templates (current checked) →
            // `e_setEngineSwingTemplate`.
            if name == "EngineSwingTemplateCombo" {
                let Some((Channel::Engine(k), _)) = self.hit(app, path) else { return false };
                let paths = crate::swing::template_paths(&dreist_formats::paths::skin_dir());
                let current = model.engine_node(k).and_then(|n| n.children.iter().find(|c| c.class == "SwingTemplate")).map(|n| crate::swing::name_of(Some(n), std::path::Path::new("")));
                let names: Vec<String> = paths.iter().map(|p| crate::swing::name_of(crate::swing::load(p).as_ref(), p)).collect();
                let checked = names.iter().position(|l| Some(l) == current.as_ref()).map(|i| i + 4);
                let mut labels = vec!["Extract timing from current pattern".to_string(), "Import MIDI template".into(), "Import Guru swing/shuffle".into(), String::new()];
                labels.extend(names);
                app.open_menu(path, labels, checked);
                self.swing_menu = Some((k, paths));
                return true;
            }
        }
        // Tempo modifier (`e_setEngineTempoNum` 0x93 / `Denom` 0x94, 1..16).
        if let UiEvent::TextCommitted { path, name, text } = ev {
            if name == "EngineNumBox" || name == "EngineDenomBox" {
                let Some((Channel::Engine(k), _)) = self.hit(app, path) else { return false };
                let Ok(v) = text.trim().parse::<i32>() else { return true };
                let kind = if name == "EngineNumBox" { id::SET_ENGINE_TEMPO_NUM } else { id::SET_ENGINE_TEMPO_DENOM };
                out.push(msg(kind, &[k as i32], &[v.clamp(1, 16) as f32]));
                return true;
            }
        }
        match ev {
            UiEvent::ValueChanged { path, value, .. } => {
                let Some((ch, name)) = self.hit(app, path) else { return false };
                let v = *value;
                let cmd = match (ch, name.as_str()) {
                    (Channel::Engine(k), "GainSlider") => msg(id::SET_ENGINE_PARAMETER, &[k as i32], &[v, EngineParam::Gain as u8 as f32, 0.0]),
                    (Channel::Engine(k), "PanKnob") => msg(id::SET_ENGINE_PARAMETER, &[k as i32], &[v, EngineParam::Pan as u8 as f32, 0.0]),
                    (Channel::Engine(k), "Pitch") => msg(id::SET_ENGINE_PARAMETER, &[k as i32], &[v, EngineParam::Pitch as u8 as f32, 0.0]),
                    // `e_setEngineSwingTemplateDepth` 0x95.
                    (Channel::Engine(k), "SwingKnob") => msg(id::SET_ENGINE_SWING_TEMPLATE_DEPTH, &[k as i32], &[v.clamp(0.0, 1.0)]),
                    (Channel::Pad(k), "GainSlider") => msg(id::SET_PAD_PARAMETER, &[e, Self::pad_of(model, k) as i32], &[v, PadParam::Gain as u8 as f32, 0.0]),
                    (Channel::Pad(k), "PanKnob") => msg(id::SET_PAD_PARAMETER, &[e, Self::pad_of(model, k) as i32], &[v, PadParam::Pan as u8 as f32, 0.0]),
                    (Channel::Layer(k), "GainSlider") => msg(id::SET_LAYER_PARAMETER, &[e, p, k as i32], &[LayerParam::Gain as u8 as f32, v, 0.0]),
                    (Channel::Layer(k), "PanKnob") => msg(id::SET_LAYER_PARAMETER, &[e, p, k as i32], &[LayerParam::Pan as u8 as f32, v, 0.0]),
                    (Channel::Aux(k), "GainSlider") => msg(id::SET_GLOBAL_AUX_GAIN, &[k as i32], &[v]),
                    (Channel::Master, "GainSlider") => msg(id::SET_MASTER_CHANNEL_GAIN, &[], &[v]),
                    (ch, n) if n.starts_with("SendKnob") => {
                        let a = n[8..].parse::<i32>().unwrap_or(0);
                        match ch {
                            Channel::Engine(k) => msg(id::SET_ENGINE_SEND_LEVEL, &[k as i32], &[a as f32, v, 0.0]),
                            Channel::Pad(k) => msg(id::SET_PAD_SEND_LEVEL, &[e, Self::pad_of(model, k) as i32], &[a as f32, v, 0.0]),
                            Channel::Layer(k) => msg(id::SET_LAYER_SEND_DEPTH, &[e, p, k as i32], &[a as f32, v, 0.0]),
                            _ => return false,
                        }
                    }
                    _ => return false,
                };
                out.push(cmd);
                true
            }
            UiEvent::Clicked { path, name, on, .. } => {
                // View and link buttons on the page.
                if path.len() > self.page.len() && path.starts_with(&self.page) && self.strip_of(path).is_none() {
                    let parent = app.ctrl(&path[..path.len() - 1]).map(|c| c.name.clone()).unwrap_or_default();
                    if parent == "ModeGroup" {
                        self.mode = *path.last().unwrap();
                        return true;
                    }
                    match name.as_str() {
                        "MixButton" => self.mode = 0,
                        "SendButton" => self.mode = 1,
                        // [re:281f54]: `e_setModifiers` 0x26 [all layers,
                        // all pads, false].
                        "AllLayers" | "AllPads" | "AllGroups" => {
                            match name.as_str() {
                                "AllPads" => {
                                    model.all_pads = *on;
                                    model.all_rows &= !*on;
                                }
                                "AllGroups" => {
                                    model.all_rows = *on;
                                    model.all_pads &= !*on;
                                }
                                _ => model.all_layers = *on,
                            }
                            out.push(msg(id::SET_MODIFIERS, &[], &[model.all_layers as i32 as f32, model.all_pads as i32 as f32, model.all_rows as i32 as f32]));
                        }
                        _ => return false,
                    }
                    return true;
                }
                let Some((ch, _)) = self.hit(app, path) else { return false };
                let v = *on as i32 as f32;
                let pad = |k: usize| [e, Self::pad_of(model, k) as i32];
                let cmd = match (ch, name.as_str()) {
                    (Channel::Engine(k), "MuteButton") => msg(id::MUTE_ENGINE, &[k as i32], &[v]),
                    (Channel::Engine(k), "SoloButton") => msg(id::SOLO_ENGINE, &[k as i32], &[v]),
                    (Channel::Pad(k), "MuteButton") => msg(id::MUTE_PAD, &pad(k), &[v]),
                    (Channel::Pad(k), "SoloButton") => msg(id::SOLO_PAD, &pad(k), &[v]),
                    (Channel::Layer(k), "MuteButton") => msg(id::SET_LAYER_MUTE, &[e, p, k as i32], &[v]),
                    (Channel::Layer(k), "SoloButton") => msg(id::SET_LAYER_SOLO, &[e, p, k as i32], &[v]),
                    (Channel::Aux(k), "MuteButton") => msg(id::MUTE_GLOBAL_AUX, &[k as i32], &[v]),
                    (Channel::Aux(k), "SoloButton") => msg(id::SOLO_GLOBAL_AUX, &[k as i32], &[v]),
                    // [re:0a9148] [re:a4e09c]: 0xa3 [aux] [pre] (the Pre /
                    // Post radio pair).
                    (Channel::Aux(k), "PreButton") => msg(id::SET_GLOBAL_AUX_PRE_POST, &[k as i32], &[1.0]),
                    (Channel::Aux(k), "PostButton") => msg(id::SET_GLOBAL_AUX_PRE_POST, &[k as i32], &[0.0]),
                    _ => return false,
                };
                out.push(cmd);
                true
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn strip_control_roles() {
        crate::app::test_setup();
        assert_eq!(super::send_index("Send2"), Some(2));
        assert_eq!(super::send_index("SendKnob3"), Some(3));
        assert_eq!(super::send_index("SendLabel0"), None);
        assert_eq!(super::send_index("SendButton"), None);
        assert_eq!(super::canon("EnginePitchKnob"), "Pitch");
        assert_eq!(super::canon("Send1"), "SendKnob1");
        assert_eq!(super::shown_in("SendLabel2", 1), Some(true));
        assert_eq!(super::shown_in("PanKnob", 1), Some(false));
        assert_eq!(super::shown_in("SwingKnob", 2), Some(true));
        assert_eq!(super::shown_in("OutputCombo", 2), Some(false));
        assert_eq!(super::shown_in("MuteButton", 2), None);
    }
}
