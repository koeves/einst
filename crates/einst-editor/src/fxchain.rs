//! Effect chain panel (v1 `EffectChain.skin`, [re:663534]): six fixed
//! slots (`EffectPanel.skin`: type and preset menus, active, mix, output
//! level) of which four show, scrolled by the slot bar, each holding its
//! effect's own skin (`Effects/<name>.skin`).
//!
//! [re:5a508e] binds `Control<n>` to effect
//! parameter n−1 (up to 12); changes post `e_setFXParameter` 0x59 with index
//! [engine, pad, layer, slot] and values [parameter, source, value]
//! ([re:b2575e]). Mix/output post 0x5c/0x5d with values [0, value, source].
//! A split fader sets the two parameters of the caption panels above (high)
//! and below (low) it.
//!
//! The slot's type menu ([re:0531cf]): No Effect (removes the slot's
//! effect), then one submenu per category ([re:364345], table
//! [re:2b5598]). The preset menu ([re:41a672]): Save
//! preset, Save as default, Revert to default, the type's presets (user
//! folder first, then system `Effects/<Type>/*.fxpreset`). Choosing a type
//! posts `e_setEffect` (100) [engine, pad, layer] with the slot and a new
//! `Effect` loaded from the type's `Default.fxpreset` ([re:02a5ef]); the port
//! edits the document and rebuilds. Submenus open as a second menu.

use dreist_engine::binding::id;
use dreist_formats::Node;
use dreist_formats::params::{EFFECT_TYPES, effect_option_strings, effect_param_names, effect_type_index};
use dreist_gui::app::{App, UiEvent};
use dreist_gui::{Ctrl, Rect, Skin};

use crate::command::{Command, msg};
use crate::model::Model;

pub const CATEGORIES: [&str; 6] = ["Distortion", "Dynamics", "EQ", "Filters", "FX", "Reverbs"];
#[cfg(test)]
use dreist_formats::params::effect_category as category;

/// System and user effect preset folders for a type.
fn preset_dirs(resources: &std::path::Path, kind: usize) -> Vec<std::path::PathBuf> {
    let name = EFFECT_TYPES[kind];
    let mut v = Vec::new();
    // [re:3b4eea].
    v.push(crate::options::documents_dir().join("Effects").join(name));
    if let Some(r) = resources.parent() {
        v.push(r.join("Effects").join(name));
    }
    v
}

/// The type's presets [re:a50ec4]: user presets, then system ones;
/// the folder scan [re:a7b56c] descends into subfolders
/// and skips `Default.fxpreset` (the type's default, not a preset).
pub fn presets(resources: &std::path::Path, kind: usize) -> Vec<std::path::PathBuf> {
    fn scan(d: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        let mut v: Vec<std::path::PathBuf> =
            std::fs::read_dir(d).into_iter().flatten().flatten().map(|e| e.path()).filter(|p| !p.file_name().is_some_and(|n| n.to_string_lossy().starts_with('.'))).collect();
        v.sort();
        for p in v {
            if p.is_dir() {
                scan(&p, out);
            } else if p.extension().is_some_and(|x| x.eq_ignore_ascii_case("fxpreset")) && p.file_stem().is_some_and(|n| n != "Default") {
                out.push(p);
            }
        }
    }
    let mut out = Vec::new();
    for d in preset_dirs(resources, kind) {
        scan(&d, &mut out);
    }
    out
}

/// An `Effect` node from a preset file, parameters renamed from
/// `Parameter<n>` to the document's names.
fn load_preset(path: &std::path::Path, kind: usize) -> Option<Node> {
    let root = Node::parse(&std::fs::read_to_string(path).ok()?).ok()?;
    let mut n = root.children.into_iter().find(|c| c.class == "Effect")?;
    let names = effect_param_names(kind);
    for (k, _) in n.attrs.iter_mut() {
        if let Some(i) = k.strip_prefix("Parameter").and_then(|i| i.parse::<usize>().ok()) {
            if let Some(pn) = names.get(i) {
                *k = pn.chars().filter(|c| !c.is_whitespace()).collect();
            }
        }
    }
    n.set("EffectType", EFFECT_TYPES[kind]);
    Some(n)
}

/// A new effect of `kind` (its `Default.fxpreset`, else bare).
fn new_effect(resources: &std::path::Path, kind: usize) -> Node {
    preset_dirs(resources, kind).into_iter().find_map(|d| load_preset(&d.join("Default.fxpreset"), kind)).unwrap_or_else(|| {
        let mut n = Node::new("Effect", None);
        n.set("EffectType", EFFECT_TYPES[kind]).set("IsActive", "true");
        n
    })
}

/// What a menu item does.
#[derive(Clone, Debug, PartialEq)]
enum Action {
    None,
    Revert,
    /// Rescan presets: the lists are read when each menu opens.
    Rescan,
    Preset(std::path::PathBuf),
    Remove,
    Category(usize),
    Type(usize),
    /// Option parameter item: (parameter, normalized value).
    Option(usize, f32),
    /// Type menu Save preset / Save as default.
    SavePreset,
    SaveDefault,
    /// Chain menu ([re:663534] [re:90cff5]).
    ChainSave,
    ChainClear,
    ChainLoad(std::path::PathBuf),
    /// A preset folder (the tokenised menu's submenu).
    ChainFolder(std::path::PathBuf),
    /// [re:b6dac9] / [re:0531cf].
    EffectCut,
    EffectCopy,
    EffectPaste,
}

/// The effect clipboard ([re:8a8bb9]).
static EFFECT_CLIPBOARD: std::sync::Mutex<Option<Node>> = std::sync::Mutex::new(None);

fn effect_clipboard() -> Option<Node> {
    EFFECT_CLIPBOARD.lock().ok().and_then(|c| c.clone())
}

/// A file the editor should ask for and write.
#[derive(Clone, Debug, PartialEq)]
pub enum SaveRequest {
    /// [re:d9c973].
    Chain(Target),
    /// An effect preset (`.fxpreset`) of the slot, in the type's user folder.
    Effect(Target, usize, usize),
}

/// [re:585af0] /
/// [re:db510f]: "Effect Chains" under the user documents folder and the
/// resources.
pub fn chain_dirs(resources: &std::path::Path) -> (Option<std::path::PathBuf>, Option<std::path::PathBuf>) {
    let user = Some(crate::options::documents_dir().join("Effect Chains"));
    (user, resources.parent().map(|r| r.join("Effect Chains")))
}

/// [re:33c153]: user `.fxchain` files, then
/// system ones (folders scanned recursively, sorted).
pub fn chain_presets(resources: &std::path::Path) -> Vec<std::path::PathBuf> {
    fn scan(d: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        let mut v: Vec<std::path::PathBuf> =
            std::fs::read_dir(d).into_iter().flatten().flatten().map(|e| e.path()).filter(|p| !p.file_name().is_some_and(|n| n.to_string_lossy().starts_with('.'))).collect();
        v.sort();
        for p in v {
            if p.is_dir() {
                scan(&p, out);
            } else if p.extension().is_some_and(|x| x == "fxchain") {
                out.push(p);
            }
        }
    }
    let (u, sys) = chain_dirs(resources);
    let mut out = Vec::new();
    for d in [u, sys].into_iter().flatten() {
        scan(&d, &mut out);
    }
    out
}

/// The doc chain for a `.fxchain` file ([re:2c02b0]): effects
/// with `Parameter<n>` renamed to the document's names, `ChainPresetName`
/// = the file path.
pub fn load_chain(path: &std::path::Path) -> Option<Node> {
    let root = Node::parse(&std::fs::read_to_string(path).ok()?).ok()?;
    let mut chain = root.children.into_iter().find(|c| c.class == "EffectChain")?;
    for fx in chain.children.iter_mut().filter(|c| c.class == "Effect") {
        let kind = fx.attr("EffectType").and_then(effect_type_index).unwrap_or(0);
        let names = effect_param_names(kind);
        for (k, _) in fx.attrs.iter_mut() {
            if let Some(i) = k.strip_prefix("Parameter").and_then(|i| i.parse::<usize>().ok()) {
                if let Some(pn) = names.get(i) {
                    *k = pn.chars().filter(|c| !c.is_whitespace()).collect();
                }
            }
        }
        if fx.version.is_none() {
            fx.version = Some(2);
        }
    }
    set_chain_name(&mut chain, &path.to_string_lossy());
    chain.name = Some("EffectChain".into());
    Some(chain)
}

/// [re:376498].
pub fn set_chain_preset_name(chain: &mut Node, name: &str) {
    set_chain_name(chain, name);
}

/// An effect's `PresetName`.
pub fn set_effect_preset_name(n: &mut Node, name: &str) {
    n.children.retain(|c| c.name.as_deref() != Some("PresetName"));
    let mut pn = Node::new("FixedString", Some("PresetName"));
    pn.set("string", name);
    n.children.push(pn);
}

fn set_chain_name(chain: &mut Node, name: &str) {
    chain.children.retain(|c| c.name.as_deref() != Some("ChainPresetName"));
    let mut n = Node::new("FixedString", Some("ChainPresetName"));
    n.set("string", name);
    chain.children.push(n);
}

/// [re:103e36].
pub fn chain_name(chain: &Node) -> Option<String> {
    chain.children.iter().find(|c| c.name.as_deref() == Some("ChainPresetName")).and_then(|c| c.attr("string")).map(str::to_string)
}

/// Write a preset file: `<root>` around `node` (the document format, as
/// [re:bdd777] writes the current serializer's).
pub fn save_preset_file(path: &std::path::Path, node: &Node) -> std::io::Result<()> {
    let mut root = Node::new("root", None);
    root.children.push(node.clone());
    if let Some(d) = path.parent() {
        std::fs::create_dir_all(d)?;
    }
    dreist_formats::file::write_atomic(path, root.to_xml())
}

/// Effect type index → its skin (`Effects/<name>.skin`): v1's own, the
/// model filters one each; the types v1 lacked have skins authored in
/// v1's style (scripts/uigen/authored.py). Every type has one.
fn skin_for(kind: usize) -> Option<&'static str> {
    Some(match kind {
        1 => "Gain",
        2 => "StereoWidth",
        3 => "ParaEQ",
        4 => "DJEQ",
        5 => "VariEQ",
        6 => "HiLo",
        7 => "Enhancer",
        8 => "ChanComp",
        9 => "BusComp",
        10 => "VCAComp",
        11 => "NoiseGate",
        12 => "EnvShaper",
        13 => "DrumDynamics",
        14 => "Limiter",
        15 => "Delay",
        16 => "Flanger",
        17 => "Chorus",
        18 => "Phaser",
        19 => "Distortion",
        20 => "ShapedDistortion",
        21 => "Driver",
        22 => "Maul",
        23 => "Crusher",
        24 => "RingSting",
        25 => "Shifter",
        26 => "Amber",
        27 => "PhaseMod",
        28 => "Filter",
        29 => "ModFilter",
        30 => "Comb",
        31 => "Filter303",
        32 => "Filter106",
        33 => "FilterLadder",
        34 => "Filter101",
        35 => "FilterProdigy",
        36 => "FilterMS20",
        37 => "ClassicFilter",
        38 => "Freezer",
        39 => "BreverbHall",
        40 => "BreverbPlate",
        41 => "BreverbRoom",
        42 => "BreverbInverse",
        43 => "TinCan",
        44 => "Reverb",
        45 => "Bloom",
        46 => "DirtyDAC",
        47 => "AnalogRingMod",
        _ => return None,
    })
}

/// Effect slots per chain and how many the scroll view shows.
pub const SLOTS: usize = 6;
pub const VISIBLE_SLOTS: usize = 4;

/// The slot controls an empty slot hides.
const SLOT_CONTROLS: [&str; 8] = ["MixKnob", "ActiveButton", "Label1", "Label2", "LastPresetButton", "NextPresetButton", "PresetCombo", "OutputKnob"];

/// Bind an effect skin's split faders (`SplitFader`, two parameters): the
/// `Control<n>` caption panel above is the high handle, the one below the
/// low handle.
fn bind_split_faders(c: &mut Ctrl) {
    let panels: Vec<(i32, usize)> = c.children.iter().filter(|k| k.class == "CtrlPanel").filter_map(|k| Some((k.rect.y, k.name.strip_prefix("Control")?.parse::<usize>().ok()?))).collect();
    for k in c.children.iter_mut().filter(|k| k.overload == "SplitFader") {
        let (y0, y1) = (k.rect.y, k.rect.bottom());
        let above = panels.iter().filter(|(y, _)| *y < y0).max_by_key(|(y, _)| *y).map(|p| p.1);
        let below = panels.iter().filter(|(y, _)| *y >= y1).min_by_key(|(y, _)| *y).map(|p| p.1);
        if let (Some(hi), Some(lo)) = (above, below) {
            k.set_attr("paramHigh", hi.to_string());
            k.set_attr("paramLow", lo.to_string());
        }
    }
}

/// Which chain the panel shows: engine, pad or layer of the selection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    Engine(usize),
    Pad(usize, usize),
    Layer(usize, usize, usize),
    /// Global aux return (chain of engine −4, pad = aux).
    Aux(usize),
    /// Master channel (chain of engine −3).
    Master,
}

impl Target {
    pub(crate) fn index(self) -> [i32; 3] {
        match self {
            Target::Engine(e) => [e as i32, -1, -1],
            Target::Pad(e, p) => [e as i32, p as i32, -1],
            Target::Layer(e, p, l) => [e as i32, p as i32, l as i32],
            Target::Aux(a) => [-4, a as i32, -1],
            Target::Master => [-3, -1, -1],
        }
    }

    fn node(self, model: &Model) -> Option<&Node> {
        match self {
            Target::Engine(e) => model.engine_node(e),
            Target::Pad(e, p) => model.pad_node(e, p),
            Target::Layer(e, p, l) => model.layer_node(e, p, l),
            Target::Aux(a) => model.doc.children.iter().find(|c| c.class == "GlobalAux" && c.index() == Some(a as u32)),
            Target::Master => model.doc.children.iter().find(|c| c.class == "MasterChannel"),
        }
    }
}

/// The chain's current preset path.
fn chain_current(model: &Model, t: Target) -> Option<String> {
    t.node(model).and_then(|n| n.children.iter().find(|c| c.class == "EffectChain")).and_then(chain_name)
}

/// The document chain node of `t` (for saving).
pub fn chain_node(model: &Model, t: Target) -> Option<Node> {
    t.node(model).and_then(|n| n.children.iter().find(|c| c.class == "EffectChain")).cloned()
}

/// The effect node in `slot` of `t`.
pub fn effect_node(model: &Model, t: Target, slot: usize) -> Option<Node> {
    effects(model, t).into_iter().find(|(s, _, _)| *s == slot).map(|(_, _, n)| n)
}

/// The effects of a chain: (slot, type, node).
pub(crate) fn effects(model: &Model, t: Target) -> Vec<(usize, usize, Node)> {
    let Some(chain) = t.node(model).and_then(|n| n.children.iter().find(|c| c.class == "EffectChain")) else { return Vec::new() };
    let mut v: Vec<(usize, usize, Node)> = chain
        .children
        .iter()
        .filter(|c| c.class == "Effect")
        .filter_map(|c| {
            let slot = c.name.as_deref()?.strip_prefix("Effect")?.parse().ok()?;
            let kind = c.attr("EffectType").and_then(effect_type_index).unwrap_or(0);
            (kind != 0).then(|| (slot, kind, c.clone()))
        })
        .collect();
    v.sort_by_key(|x| x.0);
    v
}

/// The v1 chain panel (`EffectChain.skin` in the mixer's `FXPanel`): the
/// header (chain Active, Mix, preset menu and prev / next), the slot bar
/// (`ScrollBar`, one item per slot, the shown ones lit) and the scroll view
/// holding the six slots (`EffectChainFX.skin`, `EffectPanel.skin` each),
/// four at a time.
pub struct FxPanel {
    pub(crate) host: Vec<usize>,
    /// The six slot panels.
    slots: Vec<Vec<usize>>,
    /// The row of slots inside the scroll view.
    row: Option<Vec<usize>>,
    /// The slot bar.
    bar: Option<Vec<usize>>,
    /// Slot width (the row scrolls by whole slots).
    slot_w: i32,
    /// The first shown slot (0..=SLOTS − VISIBLE_SLOTS).
    pub(crate) scroll: usize,
    /// The slot bar's item labels (effect names).
    labels: std::cell::RefCell<Vec<String>>,
    fx_skins: Vec<(usize, Ctrl)>,
    /// What is built: target and the type in each slot.
    built: Option<(Target, Vec<usize>)>,
    /// Open menu: the slot it acts on (None = the chain) and its item actions.
    menu: Option<(Option<usize>, Vec<Action>)>,
    /// A press on the slot bar: (x, y, slot, dragging).
    bar_press: Option<(f32, f32, usize, bool)>,
    resources: std::path::PathBuf,
    /// A save the editor should run (file dialog + write).
    pub save_request: Option<SaveRequest>,
    /// Chain presets found by the last folder scan and when (the prev /
    /// next buttons' state; panels resync on every edit).
    chain_presets: std::cell::Cell<Option<(usize, std::time::Instant)>>,
}

impl FxPanel {
    /// The chain shown and its six slot panels.
    pub(crate) fn shown(&self) -> Option<(Target, &[Vec<usize>])> {
        self.built.as_ref().map(|(t, _)| (*t, &self.slots[..]))
    }

    /// Fill the mixer's chain panel at `host` (its `EffectChain.skin`
    /// already mounted) with the six slots.
    pub fn build(skin: &mut Skin, root: &mut Ctrl, host: Vec<usize>) -> Result<FxPanel, crate::EditorError> {
        let mut fx_skins = Vec::new();
        for k in 1..EFFECT_TYPES.len() {
            if let Some(mut c) = skin_for(k).and_then(|s| crate::front::load(skin, &format!("Effects/{s}.skin")).ok()) {
                bind_split_faders(&mut c);
                fx_skins.push((k, c));
            }
        }
        let mut h = &mut *root;
        for &i in &host {
            h = &mut h.children[i];
        }
        let mut slots = Vec::new();
        let mut row = None;
        let mut bar = None;
        let mut slot_w = 199;
        if let Some(sv) = h.path_of("ScrollView") {
            let mut row_c = crate::front::load(skin, "Effects/EffectChainFX.skin")?;
            crate::front::mount_all(skin, &mut row_c)?;
            for (i, s) in row_c.children.iter_mut().enumerate() {
                s.set_attr("fxSlot", i.to_string());
                s.set_attr("fxKind", "0".into());
                slot_w = s.rect.w;
                slots.push([host.as_slice(), &sv, &[0, i]].concat());
            }
            let mut v = &mut *h;
            for &i in &sv {
                v = &mut v.children[i];
            }
            v.children.clear();
            v.children.push(row_c);
            row = Some([host.as_slice(), &sv, &[0]].concat());
        }
        if let Some(b) = h.path_of("ScrollBar") {
            let mut c = &mut *h;
            for &i in &b {
                c = &mut c.children[i];
            }
            // Code-drawn and hit as a custom control.
            c.class = "CtrlPanel".into();
            c.set_attr("fxHost", format!("{host:?}"));
            bar = Some([host.as_slice(), &b].concat());
        }
        Ok(FxPanel {
            host,
            slots,
            row,
            bar,
            slot_w,
            scroll: 0,
            labels: Default::default(),
            fx_skins,
            built: None,
            menu: None,
            bar_press: None,
            resources: dreist_formats::paths::skin_dir(),
            save_request: None,
            chain_presets: Default::default(),
        })
    }

    /// Put each slot's effect skin in and show / hide the slot controls.
    fn rebuild(&mut self, app: &mut App, t: Target, kinds: &[usize]) {
        if self.built.as_ref().is_some_and(|(bt, bk)| *bt == t && bk == kinds) {
            return;
        }
        let old = self.built.take().map(|(_, k)| k);
        self.built = Some((t, kinds.to_vec()));
        for (i, path) in self.slots.iter().enumerate() {
            let kind = kinds.get(i).copied().unwrap_or(0);
            if old.as_ref().and_then(|o| o.get(i)) == Some(&kind) {
                continue;
            }
            let tpl = self.fx_skins.iter().find(|(k, _)| *k == kind).map(|(_, c)| c.clone());
            let Some(slot) = app.ctrl_mut(path) else { continue };
            slot.set_attr("fxKind", kind.to_string());
            slot.style = if kind == 0 { "EffectPanelBackgroundNoFX" } else { "EffectPanelBackground" }.into();
            // The card under the slot (the skin's decor, scripts/uigen
            // skins/effects.py): dark when empty, its inner edge darker and no
            // line under the menu band.
            for k in slot.children.iter_mut() {
                let base = k.style.strip_suffix("EmptyStyle").or_else(|| k.style.strip_suffix("Style")).unwrap_or("");
                if matches!(base, "FxCard" | "FxCardInner" | "FxCardHeadLine") {
                    k.style = format!("{base}{}", if kind == 0 { "EmptyStyle" } else { "Style" });
                }
            }
            if let Some(setup) = slot.find_mut("SetupPanel") {
                for k in setup.children.iter_mut() {
                    if SLOT_CONTROLS.contains(&k.name.as_str()) {
                        k.visible = kind != 0;
                        k.enabled = kind != 0;
                    }
                }
            }
            if let Some(inner) = slot.find_mut("EffectPanel") {
                inner.children.clear();
                if let Some(mut c) = tpl {
                    c.set_rect(Rect::new(0, 0, inner.rect.w, inner.rect.h));
                    inner.children.push(c);
                }
            }
        }
    }

    /// How many chain presets exist (folders rescanned at most every two
    /// seconds, or after [`FxPanel::forget_presets`]).
    fn chain_preset_count(&self) -> usize {
        match self.chain_presets.get() {
            Some((n, at)) if at.elapsed() < std::time::Duration::from_secs(2) => n,
            _ => {
                let n = chain_presets(&self.resources).len();
                self.chain_presets.set(Some((n, std::time::Instant::now())));
                n
            }
        }
    }

    /// A preset was saved: count again.
    pub fn forget_presets(&self) {
        self.chain_presets.set(None);
    }

    pub fn sync(&mut self, app: &mut App, model: &Model, t: Target) {
        let fx = effects(model, t);
        let mut kinds = vec![0; SLOTS];
        for (s, k, _) in &fx {
            if let Some(v) = kinds.get_mut(*s) {
                *v = *k;
            }
        }
        self.rebuild(app, t, &kinds);
        *self.labels.borrow_mut() = kinds.iter().map(|&k| if k == 0 { "No Effect".to_string() } else { dreist_formats::params::effect_label(k).to_string() }).collect();
        self.scroll = self.scroll.min(SLOTS - VISIBLE_SLOTS);
        let x = -(self.scroll as i32) * self.slot_w;
        if let Some(r) = self.row.as_ref().and_then(|p| app.ctrl_mut(p)) {
            if r.rect.x != x {
                let nr = Rect::new(x, r.rect.y, r.rect.w, r.rect.h);
                r.set_rect(nr);
            }
        }
        if let Some(b) = self.bar.as_ref().and_then(|p| app.ctrl_mut(p)) {
            // The bar repaints when the labels or the scroll change.
            let sig = format!("{}|{}", self.scroll, self.labels.borrow().join("|"));
            if b.text != sig {
                b.text = sig;
            }
        }
        // Chain header: Active / Mix, the preset name and the next /
        // previous buttons [re:e9c3be] — prev / next enabled with two or
        // more presets while the chain is active.
        let chain = chain_node(model, t);
        let chain_active = chain.as_ref().and_then(|c| c.attr("IsActive")).map(str::trim) != Some("false");
        let chain_mix = chain.as_ref().and_then(|c| c.f32("ChainMix")).unwrap_or(1.0).clamp(0.0, 1.0);
        let chain_label = chain.as_ref().and_then(chain_name).and_then(|n| std::path::Path::new(&n).file_stem().map(|s| s.to_string_lossy().into_owned())).unwrap_or_else(|| "Default".into());
        let stepping = chain_active && self.chain_preset_count() >= 2;
        let Some(host) = app.ctrl_mut(&self.host) else { return };
        fn header(c: &mut Ctrl, f: &mut dyn FnMut(&mut Ctrl)) {
            for k in c.children.iter_mut() {
                if k.attr("fxSlot").is_some() {
                    continue;
                }
                f(k);
                header(k, f);
            }
        }
        header(host, &mut |k: &mut Ctrl| match k.name.as_str() {
            "ActiveButton" => k.on = chain_active,
            "MixKnob" => k.value = chain_mix,
            "PresetCombo" => k.text = chain_label.clone(),
            "PresetPrev" | "PresetNext" => k.enabled = stepping,
            _ => {}
        });
        host.walk_mut(&mut |c: &mut Ctrl| {
            let (Some(slot), Some(kind)) = (c.attr("fxSlot").and_then(|v| v.parse::<usize>().ok()), c.attr("fxKind").and_then(|v| v.parse::<usize>().ok())) else { return };
            let Some((_, _, node)) = fx.iter().find(|(s, _, _)| *s == slot) else {
                if let Some(k) = c.find_mut("EffectTypeCombo") {
                    k.text = "No Effect".into();
                }
                return;
            };
            let names = effect_param_names(kind);
            let key_of = |n: usize| -> Option<String> { names.get(n.wrapping_sub(1)).map(|pn| pn.chars().filter(|ch| !ch.is_whitespace()).collect()) };
            let get = |k: &str, d: f32| node.f32(k).unwrap_or(d).clamp(0.0, 1.0);
            let active = node.attr("IsActive").map(str::trim) != Some("false");
            let preset = node.children.iter().find(|c| c.name.as_deref() == Some("PresetName")).and_then(|c| c.attr("string")).map(str::to_string).unwrap_or_else(|| "Default".into());
            c.walk_mut(&mut |k: &mut Ctrl| {
                if k.overload == "SplitFader" {
                    for (attr, key) in [("valueLow", "paramLow"), ("valueHigh", "paramHigh")] {
                        if let Some(p) = k.attr(key).and_then(|v| v.parse::<usize>().ok()).and_then(key_of) {
                            k.set_attr(attr, get(&p, 0.5).to_string());
                        }
                    }
                    return;
                }
                if let Some(n) = k.name.strip_prefix("Control").and_then(|n| n.parse::<usize>().ok()) {
                    if let Some(key) = key_of(n) {
                        let v = get(&key, 0.5);
                        if k.class == "CtrlPopupButton" {
                            // The popup item round(v·(n − 1)).
                            if let Some(items) = effect_option_strings(kind, n - 1) {
                                let i = ((v * (items.len() - 1) as f32).round() as usize).min(items.len() - 1);
                                k.text = items[i].to_string();
                            }
                        } else if k.class == "CtrlButton" {
                            k.on = v >= 0.5;
                        } else if k.class != "CtrlPanel" {
                            k.value = v;
                        }
                    }
                    return;
                }
                match k.name.as_str() {
                    "EffectTypeCombo" => k.text = dreist_formats::params::effect_label(kind).to_string(),
                    "PresetCombo" => k.text = preset.clone(),
                    "MixKnob" => k.value = get("Mix", 1.0),
                    "OutputKnob" => k.value = get("Level", 1.0),
                    "ActiveButton" => k.on = active,
                    _ => {}
                }
            });
            crate::fxplot::sync(c, kind, node, model.sample_rate.unwrap_or(crate::fxplot::DEFAULT_SAMPLE_RATE));
        });
    }

    /// The slot bar ([re:663534] `EditorEffectPanelScrollbar`): one item
    /// per slot with its effect's name, the shown slots lit.
    pub fn draw(&self, r: &dreist_gui::render::Renderer, p: &mut dreist_gui::paint::Painter, c: &Ctrl) -> bool {
        use dreist_gui::paint::RectF;
        use dreist_gui::style::{HAlign, VAlign};
        if c.overload != "EditorEffectPanelScrollbar" || c.attr("fxHost") != Some(&format!("{:?}", self.host)) {
            return false;
        }
        let s = r.styles;
        let (w, h) = (c.rect.w as f32, c.rect.h as f32);
        let iw = (w / SLOTS as f32).floor();
        let scroll = c.text.split('|').next().and_then(|v| v.parse::<usize>().ok()).unwrap_or(self.scroll);
        for (i, label) in self.labels.borrow().iter().enumerate() {
            // (v1: items the bar's full height bar a 2 px top and 1 px bottom
            // margin, shaded top to bottom)
            let rc = RectF::new(i as f32 * iw + 1.0, 2.0, iw - 2.0, h - 3.0);
            // `FxSlotItemStyle`, the shown slots `FxSlotItemLitStyle`.
            let st = if (scroll..scroll + VISIBLE_SLOTS).contains(&i) { "FxSlotItemLitStyle" } else { "FxSlotItemStyle" };
            let text = s.colour(st, "colText", dreist_gui::Colour::rgba(18, 25, 60, 255));
            let fill = s.colour(st, "colFill", dreist_gui::Colour::rgba(200, 205, 215, 255));
            match (s.raw(st, "colFillGradientTo").map(|v| s.parse_colour(v)), dreist_gui::paint::rounded_rect_path(rc, 0.0, [false; 4])) {
                (Some(to), Some(path)) => p.fill_path_gradient(&path, fill, dreist_gui::paint::Gradient { to, horizontal: false, radial: None }),
                _ => p.fill_rect(rc, fill),
            }
            p.stroke_rect(rc.inset(0.5), s.colour(st, "colBorder", text), 1.0);
            // (v1 shortens long names in the middle: "Chann...essor")
            let font = s.font(st);
            let label = p.fonts.shorten(&font, label, rc.w - 4.0, dreist_gui::style::Shorten::Middle);
            p.text(RectF::new(rc.x + 2.0, rc.y, rc.w - 4.0, rc.h), &label, text, &font, HAlign::Center, VAlign::Center);
        }
        true
    }

    /// Show slot `i` (scrolling as little as possible).
    fn reveal(&mut self, i: usize) {
        if i < self.scroll {
            self.scroll = i;
        } else if i >= self.scroll + VISIBLE_SLOTS {
            self.scroll = i + 1 - VISIBLE_SLOTS;
        }
        self.scroll = self.scroll.min(SLOTS - VISIBLE_SLOTS);
    }

    /// The (slot, kind) of the effect panel containing `path`.
    /// The control at `path` is inside one of this panel's effects.
    pub fn contains(&self, app: &App, path: &[usize]) -> bool {
        path.starts_with(&self.host) && self.slot_of(app, path).is_some()
    }

    /// Sources modulating the effect parameter knob at `path`
    /// ([re:2f9e16], [re:a50db4]).
    pub fn modulation_mask(&self, app: &App, model: &Model, path: &[usize]) -> Option<u32> {
        if !path.starts_with(&self.host) {
            return None;
        }
        let (slot, kind) = self.slot_of(app, path)?;
        let n = app.ctrl(path)?.name.strip_prefix("Control")?.parse::<usize>().ok()?;
        let key: String = effect_param_names(kind).get(n.checked_sub(1)?)?.chars().filter(|c| !c.is_whitespace()).collect();
        let (t, _) = self.built.as_ref()?;
        let node = effect_node(model, *t, slot);
        Some(crate::model::modulation_mask(node.as_ref(), &key))
    }

    /// An effect parameter knob's tooltip ([re:c57189]
    ///  [re:c57189]): "<param>: <value text>"
    /// and, with a transmod source shown and a non-zero depth, " -> " and
    /// the modulated value.
    pub fn tooltip(&self, app: &App, model: &Model, path: &[usize], source: Option<usize>) -> Option<String> {
        let (slot, kind) = self.slot_of(app, path)?;
        let c = app.ctrl(path)?;
        if !c.class.contains("Knob") && !c.class.contains("Slider") {
            return None;
        }
        let n = c.name.strip_prefix("Control")?.parse::<usize>().ok()?.checked_sub(1)?;
        let name = effect_param_names(kind).get(n)?;
        let key: String = name.chars().filter(|c| !c.is_whitespace()).collect();
        let (t, _) = self.built.as_ref()?;
        let node = effect_node(model, *t, slot);
        let sync = node.as_ref().and_then(|e| e.f32("Sync")).is_none_or(|v| v >= 0.5);
        let mut text = format!("{name}: {}", crate::fxtext::text(kind, n, c.value, sync)?);
        if let Some(s) = source {
            let d = node.as_ref().and_then(|e| dreist_engine::engine::doc::attr_name(&key, s).and_then(|a| e.f32(&a))).unwrap_or(0.0);
            if d.abs() >= 1e-6 {
                text.push_str(" -> ");
                text.push_str(&crate::fxtext::text(kind, n, (c.value + d).clamp(0.0, 1.0), sync)?);
            }
        }
        Some(text)
    }

    /// A knob double click's reset value: [re:1f365c]
    /// [re:1f365c] (Mix 0.5, Trim / Output 1.0) and [re:2f9e16]
    /// [re:6a5c33] (the value the type's default preset gave the
    /// parameter, [re:1cda27]).
    pub fn reset_value(&self, app: &App, path: &[usize], name: &str) -> Option<f32> {
        if !path.starts_with(&self.host) {
            return None;
        }
        let (_, kind) = self.slot_of(app, path)?;
        match name {
            "MixKnob" => Some(0.5),
            "TrimKnob" | "OutputKnob" => Some(1.0),
            _ => {
                let n = name.strip_prefix("Control")?.parse::<usize>().ok()?;
                let key: String = effect_param_names(kind).get(n.checked_sub(1)?)?.chars().filter(|c| !c.is_whitespace()).collect();
                Some(new_effect(&self.resources, kind).f32(&key).unwrap_or(0.5).clamp(0.0, 1.0))
            }
        }
    }

    fn slot_of(&self, app: &App, path: &[usize]) -> Option<(usize, usize)> {
        if !path.starts_with(&self.host) {
            return None;
        }
        (self.host.len()..=path.len()).rev().find_map(|n| {
            let c = app.ctrl(&path[..n])?;
            Some((c.attr("fxSlot")?.parse().ok()?, c.attr("fxKind")?.parse().ok()?))
        })
    }

    pub(crate) fn chain_mut(model: &mut Model, t: Target) -> &mut Node {
        let n = match t {
            Target::Engine(e) => model.engine_mut(e),
            Target::Pad(e, p) => model.pad_mut(e, p),
            Target::Layer(e, p, l) => model.layer_mut(e, p, l),
            Target::Aux(a) => model.doc.child_mut_or_insert("GlobalAux", &format!("GlobalAux{a}")),
            Target::Master => model.doc.child_mut_or_insert("MasterChannel", "MasterChannel"),
        };
        if !n.children.iter().any(|c| c.class == "EffectChain") {
            let mut c = Node::new("EffectChain", Some("EffectChain"));
            c.version = Some(2100);
            n.children.push(c);
        }
        n.child_mut_or_insert("EffectChain", "EffectChain")
    }

    fn open_menu(&mut self, app: &mut App, path: &[usize], slot: Option<usize>, items: Vec<(String, Action)>) {
        let (labels, actions): (Vec<String>, Vec<Action>) = items.into_iter().unzip();
        app.open_menu(path, labels, None);
        self.menu = Some((slot, actions));
    }

    /// The chain menu: Save preset, Clear chain, then the user and the
    /// system presets (folders as submenus — the menu is tokenised on '/').
    fn chain_menu(&self, current: Option<&str>) -> Vec<(String, Action)> {
        let mut v = vec![("Save preset".to_string(), Action::ChainSave), ("Clear chain".to_string(), Action::ChainClear)];
        let (u, sys) = chain_dirs(&self.resources);
        for d in [u, sys].into_iter().flatten() {
            let items = self.chain_menu_dir(&d, current);
            if !items.is_empty() {
                v.push((String::new(), Action::None));
                v.extend(items);
            }
        }
        v
    }

    fn chain_menu_dir(&self, dir: &std::path::Path, current: Option<&str>) -> Vec<(String, Action)> {
        let mut v: Vec<std::path::PathBuf> =
            std::fs::read_dir(dir).into_iter().flatten().flatten().map(|e| e.path()).filter(|p| !p.file_name().is_some_and(|n| n.to_string_lossy().starts_with('.'))).collect();
        v.sort();
        v.into_iter()
            .filter_map(|p| {
                let name = p.file_stem()?.to_string_lossy().into_owned();
                if p.is_dir() {
                    Some((format!("{} ▸", p.file_name()?.to_string_lossy()), Action::ChainFolder(p)))
                } else if p.extension().is_some_and(|x| x == "fxchain") {
                    let mark = if current == Some(&*p.to_string_lossy()) { "✓ " } else { "" };
                    Some((format!("{mark}{name}"), Action::ChainLoad(p)))
                } else {
                    None
                }
            })
            .collect()
    }

    fn categories() -> Vec<(String, Action)> {
        CATEGORIES.iter().enumerate().map(|(i, c)| (format!("{c} ▸"), Action::Category(i))).collect()
    }

    /// The slot's preset menu ([re:41a672], v1's [re:41a672]): Save preset,
    /// Save as default; Revert to default, Rescan presets; the type's
    /// presets.
    fn type_menu(&self, kind: usize, current: Option<&str>) -> Vec<(String, Action)> {
        let mut v = vec![
            ("Save preset".to_string(), Action::SavePreset),
            ("Save as default".to_string(), Action::SaveDefault),
            (String::new(), Action::None),
            ("Revert to default".to_string(), Action::Revert),
            ("Rescan presets".to_string(), Action::Rescan),
            (String::new(), Action::None),
        ];
        let ps = presets(&self.resources, kind);
        for p in &ps {
            let name = p.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
            let mark = if current == Some(name.as_str()) { "✓ " } else { "" };
            v.push((format!("{mark}{name}"), Action::Preset(p.clone())));
        }
        v
    }

    fn apply(&mut self, model: &mut Model, t: Target, slot: Option<usize>, action: &Action, out: &mut Vec<Command>) {
        let res = self.resources.clone();
        let chain = Self::chain_mut(model, t);
        let find = |chain: &Node, s: usize| chain.children.iter().position(|c| c.class == "Effect" && c.name.as_deref() == Some(&format!("Effect{s}")));
        let kind_of = |chain: &Node, s: usize| find(chain, s).and_then(|i| chain.children[i].attr("EffectType")).and_then(effect_type_index).unwrap_or(0);
        let put = |chain: &mut Node, s: usize, mut n: Node| {
            n.name = Some(format!("Effect{s}"));
            n.version = Some(2);
            match find(chain, s) {
                Some(i) => chain.children[i] = n,
                None => chain.children.push(n),
            }
        };
        match (action, slot) {
            (Action::Type(k), Some(s)) => put(chain, s, new_effect(&res, *k)),
            (Action::Type(k), None) => {
                let s = chain.children.iter().filter(|c| c.class == "Effect").filter_map(|c| c.name.as_deref()?.strip_prefix("Effect")?.parse::<usize>().ok()).max().map_or(0, |m| m + 1);
                put(chain, s, new_effect(&res, *k));
            }
            (Action::Revert, Some(s)) => {
                let k = kind_of(chain, s);
                put(chain, s, new_effect(&res, k));
            }
            (Action::Preset(path), Some(s)) => {
                let k = kind_of(chain, s);
                if let Some(n) = load_preset(path, k) {
                    put(chain, s, n);
                }
            }
            (Action::Remove, Some(s)) => {
                if let Some(i) = find(chain, s) {
                    chain.children.remove(i);
                }
            }
            (Action::EffectCopy | Action::EffectCut, Some(s)) => {
                let Some(i) = find(chain, s) else { return };
                if let Ok(mut c) = EFFECT_CLIPBOARD.lock() {
                    *c = Some(chain.children[i].clone());
                }
                if *action == Action::EffectCopy {
                    return;
                }
                chain.children.remove(i);
            }
            (Action::EffectPaste, slot) => {
                let Some(n) = effect_clipboard() else { return };
                let s = slot.unwrap_or_else(|| {
                    chain.children.iter().filter(|c| c.class == "Effect").filter_map(|c| c.name.as_deref()?.strip_prefix("Effect")?.parse::<usize>().ok()).max().map_or(0, |m| m + 1)
                });
                put(chain, s, n);
            }
            (Action::SavePreset, Some(s)) => {
                self.save_request = Some(SaveRequest::Effect(t, s, kind_of(chain, s)));
                return;
            }
            // Save as default: the type's user `Default.fxpreset`.
            (Action::SaveDefault, Some(s)) => {
                let k = kind_of(chain, s);
                if let (Some(i), Some(d)) = (find(chain, s), preset_dirs(&res, k).into_iter().next()) {
                    let mut n = chain.children[i].clone();
                    n.name = Some("Effect".into());
                    n.children.retain(|c| c.name.as_deref() != Some("PresetName"));
                    let mut pn = Node::new("FixedString", Some("PresetName"));
                    pn.set("string", "Default");
                    n.children.push(pn);
                    let _ = save_preset_file(&d.join("Default.fxpreset"), &n);
                }
                return;
            }
            (Action::ChainSave, _) => {
                self.save_request = Some(SaveRequest::Chain(t));
                return;
            }
            // Clear: an empty chain.
            (Action::ChainClear, _) => {
                chain.children.retain(|c| c.class != "Effect" && c.name.as_deref() != Some("ChainPresetName"));
            }
            (Action::ChainLoad(path), _) => {
                let Some(n) = load_chain(path) else { return };
                let version = chain.version;
                *chain = n;
                chain.version = version.or(Some(2100));
            }
            _ => return,
        }
        out.push(model.rebuild_command());
    }

    /// Step through the type's presets (next / previous).
    fn step_preset(&mut self, model: &mut Model, t: Target, slot: usize, kind: usize, dir: i32, out: &mut Vec<Command>) {
        let ps = presets(&self.resources, kind);
        if ps.is_empty() {
            return;
        }
        let current = effects(model, t)
            .into_iter()
            .find(|(s, _, _)| *s == slot)
            .and_then(|(_, _, n)| n.children.iter().find(|c| c.name.as_deref() == Some("PresetName")).and_then(|c| c.attr("string")).map(str::to_string));
        let i = current.and_then(|c| ps.iter().position(|p| p.file_stem().is_some_and(|s| s.to_string_lossy() == c)));
        let next = match i {
            Some(i) => (i as i32 + dir).rem_euclid(ps.len() as i32) as usize,
            None => 0,
        };
        self.apply(model, t, Some(slot), &Action::Preset(ps[next].clone()), out);
    }

    pub fn handle(&mut self, app: &mut App, model: &mut Model, ev: &UiEvent, out: &mut Vec<Command>) -> bool {
        let Some((t, _)) = self.built.clone() else { return false };
        let [e, p, l] = t.index();
        match ev {
            UiEvent::MenuSelected { anchor, index, .. } if anchor.starts_with(&self.host) => {
                let Some((slot, actions)) = self.menu.take() else { return false };
                let Some(action) = actions.get(*index).cloned() else { return true };
                if let Action::Option(param, value) = action {
                    let index = [e, p, l, slot.unwrap_or(0) as i32];
                    out.push(msg(id::SET_FX_PARAMETER, &index, &[param as f32, 0.0, value]));
                    return true;
                }
                if let Action::ChainFolder(dir) = &action {
                    let items = self.chain_menu_dir(dir, chain_current(model, t).as_deref());
                    self.open_menu(app, anchor, slot, items);
                    return true;
                }
                if let Action::Category(c) = action {
                    let items = crate::app::effect_menu(c).into_iter().map(|k| (dreist_formats::params::effect_label(k).to_string(), Action::Type(k))).collect();
                    self.open_menu(app, anchor, slot, items);
                } else {
                    self.apply(model, t, slot, &action, out);
                }
                return true;
            }
            // Right press on an effect panel's background: Cut / Copy /
            // Paste ([re:b6dac9]; not over
            // its controls); on an empty slot's Add button: Paste
            // ([re:0531cf] [re:f54923]).
            // The row scrolls by slots: the wheel over it, a press or drag
            // on the slot bar.
            UiEvent::Wheel { path, dy, .. } if path.starts_with(&self.host) => {
                self.scroll = if *dy > 0.0 { self.scroll.saturating_sub(1) } else { (self.scroll + 1).min(SLOTS - VISIBLE_SLOTS) };
                return true;
            }
            // [re:9af496]: a press on an effect's block shows it; [re:2a566b]:
            // past 5 px the block is dragged, and [re:3132d4] the release
            // moves the effect to the slot under it ([re:1be64d]).
            UiEvent::Mouse { path, kind, x, y, .. } if self.bar.as_ref() == Some(path) => {
                use dreist_gui::app::MouseKind;
                let w = app.ctrl(path).map(|c| c.rect.w as f32).unwrap_or(1.0).max(1.0);
                let i = ((x / w * SLOTS as f32).floor().max(0.0) as usize).min(SLOTS - 1);
                match kind {
                    MouseKind::Down => {
                        self.bar_press = Some((*x, *y, i, false));
                        self.reveal(i);
                    }
                    MouseKind::Drag => {
                        if let Some(p) = self.bar_press.as_mut() {
                            p.3 |= (x - p.0).abs() >= 6.0 || (y - p.1).abs() >= 6.0;
                        }
                    }
                    MouseKind::Up => {
                        if let Some((_, _, from, true)) = self.bar_press.take() {
                            if crate::fxmove::move_effect(model, t, from, i) {
                                out.push(model.rebuild_command());
                                self.reveal(i);
                            }
                        }
                        self.bar_press = None;
                    }
                    MouseKind::RightDown => return false,
                }
                return true;
            }
            // Right press on an effect panel's background: Cut / Copy /
            // Paste ([re:b6dac9]; not over its controls); on an empty slot:
            // Paste ([re:0531cf] [re:f54923]).
            UiEvent::RightClicked { path, .. } | UiEvent::Mouse { path, kind: dreist_gui::app::MouseKind::RightDown, .. } if path.starts_with(&self.host) => {
                let Some(c) = app.ctrl(path) else { return false };
                let paste = dreist_gui::menu::item("Paste", effect_clipboard().is_some());
                let interactive = matches!(c.class.as_str(), "CtrlButton" | "CtrlKnob" | "CtrlPopupButton" | "CtrlTransmodKnob" | "CtrlPolyPlot" | "CtrlSlider" | "CtrlRangeSlider");
                let Some((slot, kind)) = self.slot_of(app, path).filter(|_| !interactive) else { return false };
                // [re:f54923]: Cut / Copy (with an effect) and Paste.
                let has = kind != 0;
                let items = vec![(dreist_gui::menu::item("Cut", has), Action::EffectCut), (dreist_gui::menu::item("Copy", has), Action::EffectCopy), (paste, Action::EffectPaste)];
                self.open_menu(app, path, Some(slot), items);
                return true;
            }
            // A split fader moved one of its two parameters.
            UiEvent::ValueChanged { path, value, .. } if path.starts_with(&self.host) && app.ctrl(path).is_some_and(|c| c.overload == "SplitFader") => {
                let Some((slot, _)) = self.slot_of(app, path) else { return false };
                let c = app.ctrl(path).unwrap();
                let key = if c.attr("dragPart") == Some("low") { "paramLow" } else { "paramHigh" };
                let Some(n) = c.attr(key).and_then(|v| v.parse::<usize>().ok()) else { return true };
                out.push(msg(id::SET_FX_PARAMETER, &[e, p, l, slot as i32], &[(n - 1) as f32, 0.0, *value]));
                return true;
            }
            // Option parameters ([re:5a508e]:
            // popup items from the parameter's option strings;
            // [re:5a508e] sends the value item/(count − 1)).
            UiEvent::Clicked { path, name, .. } if path.starts_with(&self.host) && name.starts_with("Control") && app.ctrl(path).is_some_and(|c| c.class == "CtrlPopupButton") => {
                let (Some((slot, kind)), Some(n)) = (self.slot_of(app, path), name.strip_prefix("Control").and_then(|n| n.parse::<usize>().ok())) else { return false };
                let Some(items) = effect_option_strings(kind, n - 1) else { return true };
                let last = (items.len() - 1).max(1) as f32;
                let list = items.iter().enumerate().map(|(i, s)| (s.to_string(), Action::Option(n - 1, i as f32 / last))).collect();
                self.open_menu(app, path, Some(slot), list);
                return true;
            }
            // Chain preset menu and stepping ([re:663534]).
            UiEvent::Clicked { path, name, .. } if path.starts_with(&self.host) && matches!(name.as_str(), "PresetCombo" | "PresetPrev" | "PresetNext") && self.slot_of(app, path).is_none() => {
                let current = chain_current(model, t);
                if name == "PresetCombo" {
                    let items = self.chain_menu(current.as_deref());
                    self.open_menu(app, path, None, items);
                    return true;
                }
                let ps = chain_presets(&self.resources);
                if ps.is_empty() {
                    return true;
                }
                let i = current.and_then(|c| ps.iter().position(|p| p.to_string_lossy() == c));
                let next = match (i, name == "PresetNext") {
                    (Some(i), true) => (i + 1) % ps.len(),
                    (Some(i), false) => (i + ps.len() - 1) % ps.len(),
                    (None, _) => 0,
                };
                self.apply(model, t, None, &Action::ChainLoad(ps[next].clone()), out);
                return true;
            }
            UiEvent::Clicked { path, name, .. } if path.starts_with(&self.host) && matches!(name.as_str(), "EffectTypeCombo" | "PresetCombo" | "LastPresetButton" | "NextPresetButton") => {
                let Some((slot, kind)) = self.slot_of(app, path) else { return false };
                match name.as_str() {
                    // [re:0531cf]: type 0's name first, on every slot.
                    "EffectTypeCombo" => {
                        let mut items = vec![("No Effect".to_string(), if kind == 0 { Action::None } else { Action::Remove })];
                        items.extend(Self::categories());
                        self.open_menu(app, path, Some(slot), items);
                    }
                    "PresetCombo" => {
                        let current = effects(model, t)
                            .into_iter()
                            .find(|(s, _, _)| *s == slot)
                            .and_then(|(_, _, n)| n.children.iter().find(|c| c.name.as_deref() == Some("PresetName")).and_then(|c| c.attr("string")).map(str::to_string));
                        let items = self.type_menu(kind, current.as_deref());
                        self.open_menu(app, path, Some(slot), items);
                    }
                    "LastPresetButton" => self.step_preset(model, t, slot, kind, -1, out),
                    _ => self.step_preset(model, t, slot, kind, 1, out),
                }
                return true;
            }
            // The chain header's own Active button / Mix knob
            // ([re:e31ebb]): 0x60 / 0x62 [e, p, l] [value].
            UiEvent::Clicked { path, name, on, .. } if name == "ActiveButton" && path.starts_with(&self.host) && self.slot_of(app, path).is_none() => {
                out.push(msg(id::SET_FX_CHAIN_ACTIVE, &[e, p, l], &[*on as i32 as f32]));
                return true;
            }
            UiEvent::ValueChanged { path, name, value, .. } if name == "MixKnob" && path.starts_with(&self.host) && self.slot_of(app, path).is_none() => {
                out.push(msg(id::SET_FX_CHAIN_MIX, &[e, p, l], &[value.clamp(0.0, 1.0)]));
                return true;
            }
            _ => {}
        }
        match ev {
            UiEvent::ValueChanged { path, name, value, .. } => {
                let Some((slot, _)) = self.slot_of(app, path) else { return false };
                let index = [e, p, l, slot as i32];
                if let Some(n) = name.strip_prefix("Control").and_then(|n| n.parse::<usize>().ok()) {
                    out.push(msg(id::SET_FX_PARAMETER, &index, &[(n - 1) as f32, 0.0, *value]));
                    return true;
                }
                let kind = match name.as_str() {
                    "TrimKnob" => id::SET_FX_TRIM,
                    "MixKnob" => id::SET_FX_MIX,
                    "OutputKnob" => id::SET_FX_LEVEL,
                    _ => return false,
                };
                out.push(msg(kind, &index, &[0.0, *value, 0.0]));
                true
            }
            UiEvent::Clicked { path, name, on, .. } => {
                let Some((slot, _)) = self.slot_of(app, path) else { return false };
                let index = [e, p, l, slot as i32];
                if let Some(n) = name.strip_prefix("Control").and_then(|n| n.parse::<usize>().ok()) {
                    out.push(msg(id::SET_FX_PARAMETER, &index, &[(n - 1) as f32, 0.0, if *on { 1.0 } else { 0.0 }]));
                    return true;
                }
                if name == "ActiveButton" {
                    out.push(msg(id::SET_FX_ACTIVE, &index, &[0.0, *on as i32 as f32]));
                    return true;
                }
                false
            }
            _ => false,
        }
    }
}

/// Show / hide the controls an effect panel folds away
/// ([re:1f365c]): Active, Next / Last preset and the knob labeller.
pub fn fold_controls(panel: &mut Ctrl, show: bool) {
    fn walk(c: &mut Ctrl, show: bool) {
        for k in &mut c.children {
            if matches!(k.name.as_str(), "ActiveButton" | "LastPresetButton" | "NextPresetButton") || k.children.iter().any(|g| g.name == "OutputKnob") {
                k.visible = show;
                k.enabled = show;
            }
            if k.attr("fxSlot").is_none() && k.name != "EffectPanel" {
                walk(k, show);
            }
        }
    }
    walk(panel, show);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn categories_and_presets() {
        crate::app::test_setup();
        assert_eq!(CATEGORIES[category(effect_type_index("Bit Crusher").unwrap())], "Distortion");
        assert_eq!(CATEGORIES[category(effect_type_index("Breverb Hall").unwrap())], "Reverbs");
        assert_eq!(CATEGORIES[category(effect_type_index("Comb Filter").unwrap())], "Filters");
        let res = dreist_formats::paths::skin_dir();
        let k = effect_type_index("Bit Crusher").unwrap();
        let ps = presets(&res, k);
        if ps.is_empty() {
            return;
        }
        let n = load_preset(&ps[0], k).unwrap();
        assert_eq!(n.attr("EffectType"), Some("Bit Crusher"));
        assert!(n.attrs.iter().all(|(a, _)| !a.starts_with("Parameter")), "{:?}", n.attrs);
    }

    #[test]
    fn chain_presets_load_and_save() {
        crate::app::test_setup();
        let res = dreist_formats::paths::skin_dir();
        let ps = chain_presets(&res);
        let Some(p) = ps.iter().find(|p| p.ends_with("Auxiliary Sends/Delays/Giant Delayed Hall.fxchain")) else { return };
        let c = load_chain(p).unwrap();
        let fx: Vec<&Node> = c.children.iter().filter(|n| n.class == "Effect").collect();
        assert_eq!(fx.len(), 3);
        // Delay has six named parameters (Gain has fewer: its unused
        // Parameter<n> stay, harmless).
        assert!(fx[1].attrs.iter().all(|(a, _)| !a.starts_with("Parameter")), "{:?}", fx[1].attrs);
        assert_eq!(chain_name(&c).as_deref(), Some(&*p.to_string_lossy()));
        let dir = std::env::temp_dir().join(format!("dreist-chain-{}", std::process::id()));
        let out = dir.join("x.fxchain");
        save_preset_file(&out, &c).unwrap();
        let back = load_chain(&out).unwrap();
        assert_eq!(back.children.iter().filter(|n| n.class == "Effect").count(), 3);
        let _ = std::fs::remove_dir_all(dir);
    }
}
