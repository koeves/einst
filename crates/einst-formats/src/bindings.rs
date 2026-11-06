//! v1 MIDI CC / host automation bindings → v2's `BindingMap`.
//!
//! The containers are the same in both versions: `HostBindingCount%d`,
//! `Auto%dBinding%d`, `MIDI%dCC%dBindingCount`, `MIDI%dCC%dBinding%d`
//! (v1's map serializer, ev_e4e57dbe), and a `Binding` keeps `Min Max MIDI
//! MajorIndex MinorIndex TargetMode EnterTargetMode EditorOnly`, the
//! `Path`/`Name` strings, the stuffer/popper names, the binding `Type` and
//! a `MessagePrototype` (v1's binding serializer, ev_1e06779d). What
//! differs is the prototype:
//!
//! * v1 writes a `Message` with `EngineIndex PadIndex LayerIndex
//!   PatternIndex TrackIndex StepIndex`, a `Type` name and three
//!   `MessageValue%d` (v1's message serializer, ev_aa4127e9); v2 reads
//!   `TMessage_I6_V3` with `Index0..5` in the same order.
//! * some messages carry their values differently, or exist only in v1;
//!   those are rewritten to the v2 message with the same effect, or
//!   dropped (see [`Dropped`]).
//!
//! v1 posts a binding's message as soon as the controller moves (no
//! smoothing, ev_1889e863), so converted bindings get
//! `NeedsIterp="false"`; v2 smooths by default.
//!
//! Message layouts below are v1's handlers: layer ev_74fa771b, pad
//! ev_d34b2311, engine ev_9e4f4344, effect ev_b041572e, pattern
//! ev_e3c8f67e, transport ev_724c7526, selection ev_d5b9a871, scenes
//! ev_2c6ed307, master ev_55e58374, global aux ev_cc4be154; dispatch by
//! type ev_afaaf16c.

use dreist_formats::node::format_g16;
use dreist_formats::{LayerParam, Node, STEP_LANES, TransmodSource};

/// A binding the conversion left out, and why.
#[derive(Debug, Clone, PartialEq)]
pub struct Dropped {
    /// `MIDI%dCC%dBinding%d` / `Auto%dBinding%d`.
    pub binding: String,
    /// The v1 message type.
    pub message: String,
    pub reason: &'static str,
}

/// v1's `Layer` parameter enum (`GetNameOfParameter`, ev_163ab894) in v2's
/// `LayerParam` terms. The values share one normalised scale: the layer
/// conversion copies these attributes unchanged.
const V1_LAYER_PARAMS: [LayerParam; 11] = [
    LayerParam::Gain,
    LayerParam::Pan,
    LayerParam::CoarseTune,
    LayerParam::FineTune,
    LayerParam::PreDelay,
    LayerParam::FilterCutoff,
    LayerParam::FilterResonance,
    LayerParam::FilterType,
    LayerParam::FilterDrive,
    LayerParam::FilterAlgorithm,
    LayerParam::FilterActive,
];

/// v1's step graph lanes (`GetNameOfStepGraphItem`, ev_3060d6bc) by v2
/// lane name; `None` for the four envelope-depth lanes v2 does not have.
/// From index 23 on, v1 continues with v2's `FX1Active .. FX6Param6`.
fn v1_lane(i: usize) -> Option<&'static str> {
    const HEAD: [Option<&str>; 23] = [
        Some("Velocity"),
        Some("Pan"),
        Some("Repeats"),
        Some("Shift"),
        Some("Reverse"),
        Some("CoarsePitch"),
        Some("FinePitch"),
        Some("StartPoint"),
        Some("Duration"),
        Some("LinkToPrevious"),
        Some("FilterCutoff"),
        Some("FilterResonance"),
        Some("FilterType"),
        Some("FilterDrive"),
        Some("ForceLayer"),
        Some("Send1"),
        Some("Send2"),
        Some("Send3"),
        Some("Send4"),
        None,
        None,
        None,
        None,
    ];
    let fx = STEP_LANES.iter().position(|l| *l == "FX1Active")?;
    match i {
        0..23 => HEAD[i],
        _ => STEP_LANES.get(fx + i - 23).copied(),
    }
}

/// v2 lane index of v1 lane `i`.
fn lane(i: f32) -> Option<f32> {
    if i < 0.0 {
        return None;
    }
    let name = v1_lane(i as usize)?;
    STEP_LANES.iter().position(|l| *l == name).map(|k| k as f32)
}

/// v1 message types with no v2 counterpart left after the rewrites below
/// (v1's message type table minus v2's names).
const V1_ONLY: [&str; 49] = [
    "e_resetGrid", "e_prepareForPatternWaveExport", "e_cleanUpPatternWaveExport", "e_setPadGain", "e_setPadPan", "e_setPadVA", "e_clonePadUpwards", "e_clonePadDownwards", "e_clonePadCentered",
    "e_movePadTo", "e_copyPadTo", "e_pastePad", "e_loadPad", "e_selectLayer", "e_setLayerEnvelopeType", "e_setLayerEnvelopeParameter", "e_setLayerTimestretchActive", "e_setLayerTimestretchMode",
    "e_setLayerTimestretchTracking", "e_setLayerTimestretchSourceBPM", "e_setLayerTimestretchDestinationBPM", "e_setLayerTimestretchTimestretch", "e_setLayerV2C", "e_setLayerV2P", "e_loadLayer",
    "e_pasteLayer", "e_setEngineGain", "e_setEnginePitch", "e_setEnginePan", "e_loadEngineAudio", "e_loadEnginePatterns", "e_pastePattern", "e_copyPatternTo", "e_movePatternTo", "e_setGraphScaling",
    "e_setGraphCompression", "e_deallocateSampleUsingModifiers", "e_switchToPreviousPanel", "e_switchToPadLayersPanel", "e_updateSelectView", "e_updateSelectedEngineChanged",
    "e_updateSelectedPadChanged", "e_updateSelectedLayerChanged", "e_controlSurfaceSetStepOn", "e_controlSurfaceSetStepOff", "e_controlSurfaceSetGraphValue", "e_controlSurfaceCopyPattern",
    "e_updateCompEQChain", "e_updateEffectChainEQ",
];

/// v1's prototype as read from the file.
struct V1Message {
    kind: String,
    index: [i32; 6],
    /// `MessageValue0..2` as written (`VValue` nodes).
    values: [Option<Node>; 3],
}

const V1_INDEX: [&str; 6] = ["EngineIndex", "PadIndex", "LayerIndex", "PatternIndex", "TrackIndex", "StepIndex"];

impl V1Message {
    fn read(n: &Node) -> V1Message {
        let kind = n.children.iter().find(|c| c.name.as_deref() == Some("Type")).and_then(|c| c.attr("string")).unwrap_or("").to_owned();
        // v2-shaped prototypes (`Index%d`) are read too.
        let index = std::array::from_fn(|i| n.attr(V1_INDEX[i]).or_else(|| n.attr(&format!("Index{i}"))).and_then(|v| v.parse().ok()).unwrap_or(-1));
        let values = std::array::from_fn(|i| n.child("VValue", &format!("MessageValue{i}")).cloned());
        V1Message { kind, index, values }
    }

    fn value(&self, i: usize) -> f32 {
        match self.values[i].as_ref().and_then(|v| v.attr("value")) {
            Some("true") => 1.0,
            Some("false") => 0.0,
            Some(v) => v.parse().unwrap_or(0.0),
            None => 0.0,
        }
    }
}

/// One `MessageValue%d`.
enum Value {
    /// v1's node, unchanged.
    Keep(usize),
    Int(i32),
    Float(f32),
}

/// The v2 message a v1 prototype becomes.
struct Target {
    kind: String,
    index: [i32; 6],
    values: Vec<Value>,
    /// v2 value slot for v1 slot 0..2 (`None`: binding that slot is dropped).
    slots: [Option<usize>; 3],
    /// v2 value = `scale · v1 value + offset` for the stuffed value.
    scale: f32,
    offset: f32,
}

impl Target {
    fn keep(m: &V1Message) -> Target {
        Target { kind: m.kind.clone(), index: m.index, values: (0..3).map(Value::Keep).collect(), slots: [Some(0), Some(1), Some(2)], scale: 1.0, offset: 0.0 }
    }

    fn new(kind: &str, index: [i32; 6], values: Vec<Value>, slots: [Option<usize>; 3]) -> Target {
        Target { kind: kind.to_owned(), index, values, slots, scale: 1.0, offset: 0.0 }
    }

    fn affine(mut self, scale: f32, offset: f32) -> Target {
        self.scale = scale;
        self.offset = offset;
        self
    }

    fn node(&self, v1: &V1Message) -> Node {
        let mut n = Node::new("TMessage_I6_V3", Some("MessagePrototype"));
        for (i, x) in self.index.iter().enumerate() {
            n.set(&format!("Index{i}"), x);
        }
        let mut t = Node::new("VString", Some("Type"));
        t.set("string", &self.kind);
        n.children.push(t);
        for (i, v) in self.values.iter().enumerate() {
            let name = format!("MessageValue{i}");
            let mut c = Node::new("VValue", Some(&name));
            match v {
                Value::Keep(k) => match &v1.values[*k] {
                    Some(o) => {
                        c.attrs = o.attrs.clone();
                        c.attrs.retain(|(a, _)| a != "name");
                    }
                    None => {
                        c.set("type", "unknown");
                    }
                },
                Value::Int(x) => {
                    c.set("type", "int").set("value", x);
                }
                Value::Float(x) => {
                    c.set("type", "float").set("value", format_g16(*x as f64));
                }
            }
            n.children.push(c);
        }
        n
    }
}

/// `SET_LAYER_PARAMETER` [engine, pad, layer] [param, value, source] with
/// the value in slot 1.
fn layer_param(m: &V1Message, p: LayerParam, source: TransmodSource, from: usize) -> Target {
    let mut slots = [None; 3];
    slots[from] = Some(1);
    Target::new("e_setLayerParameter", m.index, vec![Value::Int(p as i32), Value::Float(m.value(from)), Value::Int(source as i32)], slots)
}

/// `e_set{Pad,Engine}Parameter` [engine(, pad)] [value, param, source].
fn object_param(m: &V1Message, kind: &str, param: i32) -> Target {
    Target::new(kind, m.index, vec![Value::Keep(0), Value::Int(param), Value::Int(0)], [Some(0), None, None])
}

/// The v1 layer `m` addresses, in the v1 document.
fn v1_layer<'a>(root: &'a Node, m: &V1Message) -> Option<&'a Node> {
    let pick = |n: &'a Node, class: &str, i: i32| -> Option<&'a Node> {
        let i = u32::try_from(i).ok()?;
        n.children_of(class).find(|c| c.index() == Some(i))
    };
    let e = pick(root, "Engine", m.index[0])?;
    let p = pick(e, "Pad", m.index[1])?;
    pick(p, "Layer", m.index[2])
}

/// v1 routing destinations as the layer conversion maps them.
fn routing_param(dest: &str) -> Option<LayerParam> {
    Some(match dest {
        "Cutoff" => LayerParam::FilterCutoff,
        "Resonance" => LayerParam::FilterResonance,
        "Drive" => LayerParam::FilterDrive,
        "Pitch" => LayerParam::CoarseTune,
        _ => return None,
    })
}

/// The v2 message for a v1 prototype, or why there is none.
fn target(m: &V1Message, root: Option<&Node>) -> Result<Target, &'static str> {
    use LayerParam as L;
    use TransmodSource as S;
    Ok(match m.kind.as_str() {
        // [param, value]: v1's parameter enum, v2's source column 0.
        "e_setLayerParameter" => {
            let p = m.value(0);
            let p = V1_LAYER_PARAMS.get(p as usize).filter(|_| p >= 0.0).ok_or("layer parameter v1 does not have")?;
            layer_param(m, *p, S::ActualParameter, 1)
        }
        // [envelope, parameter, value]: amp/free envelope, v2 parameters
        // AmpEnvAttack.. / FreeEnvAttack.. in the same order.
        "e_setLayerEnvelopeParameter" => {
            let (e, i) = (m.value(0) as i32, m.value(1) as i32);
            if !(0..2).contains(&e) || !(0..8).contains(&i) {
                return Err("envelope parameter out of range");
            }
            let p = L::ALL[L::AmpEnvAttack as usize + (e * 8 + i) as usize];
            layer_param(m, p, S::ActualParameter, 2)
        }
        // Velocity amounts (layer +0x578 / +0x57c): v2's Velocity depth
        // 2v − 1, as the layer conversion does.
        "e_setLayerV2C" => layer_param(m, L::FilterCutoff, S::Velocity, 0).affine(2.0, -1.0),
        "e_setLayerV2P" => layer_param(m, L::CoarseTune, S::Velocity, 0).affine(2.0, -1.0),
        // Timestretch switches (+0x5a8, +0x5a9) and mode (+0x5b4, 1 = Fast)
        // are v2 layer parameters.
        "e_setLayerTimestretchActive" => layer_param(m, L::TimestretchActive, S::ActualParameter, 0),
        "e_setLayerTimestretchTracking" => layer_param(m, L::TimestretchTracksBPM, S::ActualParameter, 0),
        "e_setLayerTimestretchMode" => {
            let mut t = layer_param(m, L::TimestretchMode, S::ActualParameter, 0);
            t.values[1] = Value::Float(if m.value(0) as i32 == 1 { 1.0 } else { 0.0 });
            t
        }
        "e_setLayerTimestretchSourceBPM" | "e_setLayerTimestretchDestinationBPM" => return Err("v2 layers have no source/destination BPM"),
        // v1's stretch setter turns the value into a destination BPM along
        // a cubic; v2's stretch amount uses a different curve.
        "e_setLayerTimestretchTimestretch" => return Err("v1's stretch curve has no v2 equivalent"),
        "e_setLayerEnvelopeType" => return Err("v2 envelopes have no type"),
        // [routing, depth]: the depth of the routing's destination for its
        // envelope (routings 0, 1 amp, 2, 3 free), 2d − 1.
        "e_setLayerModulationDepth" => {
            let i = m.value(0) as i32;
            let layer = root.and_then(|r| v1_layer(r, m)).ok_or("modulation routing of a layer not in the document")?;
            let r = layer.child("LayerModulationRouting", &format!("ModRouting{i}")).ok_or("no such modulation routing")?;
            let p = routing_param(r.attr("ModulationDestination").unwrap_or("")).ok_or("routing without a v2 destination")?;
            let src = if i < 2 { S::AmpEnvelope } else { S::FreeEnvelope };
            layer_param(m, p, src, 1).affine(2.0, -1.0)
        }
        "e_setLayerModulationDestination" => return Err("v2 has no modulation routings"),
        "e_setPadGain" => object_param(m, "e_setPadParameter", 0),
        "e_setPadPan" => object_param(m, "e_setPadParameter", 1),
        "e_setPadVA" => object_param(m, "e_setPadParameter", 2),
        "e_setEngineGain" => object_param(m, "e_setEngineParameter", 0),
        "e_setEnginePan" => object_param(m, "e_setEngineParameter", 1),
        "e_setEnginePitch" => object_param(m, "e_setEngineParameter", 2),
        // v1 [slot, param, value]; v2 takes the slot from Index3 and reads
        // [param, source, value].
        "e_setFXParameter" => {
            let mut index = m.index;
            index[3] = m.value(0) as i32;
            Target::new(&m.kind, index, vec![Value::Keep(1), Value::Int(0), Value::Keep(2)], [None, None, Some(2)])
        }
        // v1 [slot, value]; v2 [slot, value, source], slot also in Index3.
        "e_setFXMix" | "e_setFXLevel" | "e_setFXActive" => {
            let mut index = m.index;
            index[3] = m.value(0) as i32;
            Target::new(&m.kind, index, vec![Value::Keep(0), Value::Keep(1), Value::Int(0)], [None, Some(1), None])
        }
        // Lanes renumbered: [value, lane] / [amount, lane] / [lane, ..].
        "e_setInstoMIDIGraphValue" | "e_setGraphRandomisation" => {
            let l = lane(m.value(1)).ok_or("step lane v2 does not have")?;
            let mut t = Target::keep(m);
            t.values[1] = Value::Int(l as i32);
            t.slots[1] = None;
            t
        }
        "e_setStepGraphState" | "e_clearTrackAutomation" => {
            let l = lane(m.value(0)).ok_or("step lane v2 does not have")?;
            let mut t = Target::keep(m);
            t.values[0] = Value::Int(l as i32);
            t.slots[0] = None;
            t
        }
        "e_setGraphScaling" | "e_setGraphCompression" => return Err("v2 has no message for track scaling/compression"),
        "" => return Err("prototype without a type"),
        k if V1_ONLY.contains(&k) => return Err("no v2 message"),
        // The rest share v2's name and value layout.
        _ => Target::keep(m),
    })
}

fn slot_of(stuffer: &str) -> Option<(usize, bool)> {
    let rest = stuffer.strip_prefix("Binding::StuffToValue").or_else(|| stuffer.strip_prefix("Binding::GetValueFromValue"))?;
    let (digit, bool_) = match rest.strip_suffix("AsBool") {
        Some(d) => (d, true),
        None => (rest, false),
    };
    Some((digit.parse().ok()?, bool_))
}

fn string_leaf(class: &str, name: &str, value: &str) -> Node {
    let mut n = Node::new(class, Some(name));
    n.set("string", value);
    n
}

/// One binding, or why it was dropped.
fn binding(b: &Node, root: Option<&Node>) -> Result<Node, (String, &'static str)> {
    let s = |k: &str| b.children.iter().find(|c| c.name.as_deref() == Some(k)).and_then(|c| c.attr("string"));
    let proto = b.children.iter().find(|c| c.name.as_deref() == Some("MessagePrototype")).ok_or((String::new(), "binding without a message"))?;
    let m = V1Message::read(proto);
    let t = target(&m, root).map_err(|r| (m.kind.clone(), r))?;
    let stuffer = s("MessageStuffer").unwrap_or("Binding::StuffToValue0");
    let (from, as_bool) = slot_of(stuffer).ok_or((m.kind.clone(), "unknown message stuffer"))?;
    let to = t.slots.get(from).copied().flatten().ok_or((m.kind.clone(), "binding drives a value v2 keeps fixed"))?;
    let kind = s("Type").unwrap_or("Continuous");
    let affine = t.scale != 1.0 || t.offset != 0.0;
    if affine && (kind != "Continuous" || as_bool) {
        return Err((m.kind.clone(), "rescaled value on a switching binding"));
    }
    let mut out = Node::new("Binding", b.name.as_deref());
    for (k, v) in &b.attrs {
        match k.as_str() {
            "Min" | "Max" if affine => {
                let x: f32 = v.parse().unwrap_or(if k == "Min" { 0.0 } else { 1.0 });
                out.set(k, format_g16((x * t.scale + t.offset) as f64));
            }
            "name" | "Ver" | "NeedsIterp" => {}
            _ => {
                out.set(k, v);
            }
        }
    }
    if affine {
        for (k, d) in [("Min", 0.0f32), ("Max", 1.0)] {
            if out.attr(k).is_none() {
                out.set(k, format_g16((d * t.scale + t.offset) as f64));
            }
        }
    }
    out.set("NeedsIterp", "false");
    let bool_suffix = if as_bool && to == 0 { "AsBool" } else { "" };
    out.children.push(string_leaf("VFixedString_64", "Path", s("Path").unwrap_or("")));
    out.children.push(string_leaf("VFixedString_64", "Name", s("Name").unwrap_or("")));
    out.children.push(string_leaf("VString", "MessageStuffer", &format!("Binding::StuffToValue{to}{bool_suffix}")));
    out.children.push(string_leaf("VString", "MessagePopper", &format!("Binding::GetValueFromValue{to}{bool_suffix}")));
    out.children.push(t.node(&m));
    out.children.push(string_leaf("VString", "Type", kind));
    Ok(out)
}

/// Binding child name → (MIDI channel and CC, or host parameter).
fn key(name: &str) -> Option<(bool, usize, usize)> {
    if let Some(r) = name.strip_prefix("MIDI") {
        let (ch, r) = r.split_once("CC")?;
        let (cc, _) = r.split_once("Binding")?;
        return Some((true, ch.parse().ok()?, cc.parse().ok()?));
    }
    let (p, _) = name.strip_prefix("Auto")?.split_once("Binding")?;
    Some((false, p.parse().ok()?, 0))
}

/// v1 `BindingMap` → v2 `BindingMap`, with the bindings left out. `root`
/// is the v1 document (for modulation routings); bindings under one key
/// are renumbered in order and the counts rewritten.
pub fn convert_binding_map(map: &Node, root: Option<&Node>) -> (Node, Vec<Dropped>) {
    let mut out = Node::new("BindingMap", map.name.as_deref());
    out.version = Some(2);
    // Counts are rebuilt below.
    for (k, v) in &map.attrs {
        if !(k.starts_with("HostBindingCount") || (k.starts_with("MIDI") && k.ends_with("BindingCount")) || k == "Ver") {
            out.set(k, v);
        }
    }
    let mut dropped = Vec::new();
    let mut counts: Vec<((bool, usize, usize), usize)> = Vec::new();
    for c in &map.children {
        let Some(k) = c.name.as_deref().and_then(key) else {
            // `HostParameters` and anything else: kept.
            out.children.push(c.clone());
            continue;
        };
        match binding(c, root) {
            Ok(mut b) => {
                let n = match counts.iter_mut().find(|(x, _)| *x == k) {
                    Some((_, n)) => {
                        *n += 1;
                        *n - 1
                    }
                    None => {
                        counts.push((k, 1));
                        0
                    }
                };
                b.name = Some(match k {
                    (true, ch, cc) => format!("MIDI{ch}CC{cc}Binding{n}"),
                    (false, p, _) => format!("Auto{p}Binding{n}"),
                });
                out.children.push(b);
            }
            Err((message, reason)) => dropped.push(Dropped { binding: c.name.clone().unwrap_or_default(), message, reason }),
        }
    }
    for ((midi, a, b), n) in counts {
        match midi {
            true => out.set(&format!("MIDI{a}CC{b}BindingCount"), n),
            false => out.set(&format!("HostBindingCount{a}"), n),
        };
    }
    (out, dropped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lanes() {
        assert_eq!(lane(0.0), Some(0.0));
        let at = |n: &str| STEP_LANES.iter().position(|l| *l == n).unwrap() as f32;
        assert_eq!(lane(5.0), Some(at("CoarsePitch")));
        assert_eq!(lane(14.0), Some(at("ForceLayer")));
        assert_eq!(lane(15.0), Some(at("Send1")));
        assert_eq!(lane(19.0), None);
        assert_eq!(lane(23.0), Some(at("FX1Active")));
        assert_eq!(lane(76.0), Some(at("FX6Param6")));
        assert_eq!(lane(77.0), None);
    }

    #[test]
    fn stuffers() {
        assert_eq!(slot_of("Binding::StuffToValue2"), Some((2, false)));
        assert_eq!(slot_of("Binding::StuffToValue0AsBool"), Some((0, true)));
        assert_eq!(slot_of("x"), None);
    }
}
