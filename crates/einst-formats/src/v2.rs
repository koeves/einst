//! v1 document tree → v2 document tree (what Dreist's `dreist_formats`
//! and `dreist_engine` read). See dreist-re `einst/docs/re/loading-v1.md`.
//!
//! v1 and v2 share the element shape (tag = class, `name` = instance
//! key) and most attribute names, so the conversion walks the tree and
//! rewrites only what differs:
//! * per-object `Ver` floats are dropped; layers get `version="3300"` and
//!   the root `version="3000"`.
//! * layers go through v2's legacy branch ([re:0cef81]
//!   for `Ver` < 3300, ev_5075e9a1): envelopes, modulation routings,
//!   velocity amounts and timestretch switches become v2 parameters.
//! * step lanes and track modifiers that v2 does not have are dropped
//!   (v2's [re:9b6bed] reads only its own lane names).
//! * output names and sample paths are rewritten to v2's forms.
//! * MIDI/host bindings go through [`crate::bindings`]. v1's `MIDIMap`
//!   holds nothing but its version (v1's serializer reads no items,
//!   ev_93038a78): the note map is always v1's built-in default, as v2's
//!   is, so it is dropped.

use dreist_formats::node::format_g16;
use dreist_formats::{LayerParam, Node, STEP_LANES, TransmodSource};

/// Layer version v2 writes ([re:87acba], 0xce4).
const LAYER_VERSION: u32 = 3300;
/// Root version of a v2 `.geist` ([re:8b0f28] in Dreist).
const ROOT_VERSION: u32 = 3000;

/// Convert a v1 `<root>` (project, kit, pad, engine, pattern or bank).
pub fn convert_root(v1: &Node) -> Node {
    let mut out = Node::new("root", None);
    out.version = Some(ROOT_VERSION);
    for (k, v) in &v1.attrs {
        match k.as_str() {
            // [re:7e3d37] gates on it; v2 reads it only below root version 3000.
            "StateVersion" => {}
            "PreviewOutput" => {
                out.set(k, output_name(v));
            }
            _ => {
                out.set(k, v);
            }
        }
    }
    out.children = v1
        .children
        .iter()
        .filter_map(|c| match c.class.as_str() {
            "BindingMap" => Some(crate::bindings::convert_binding_map(c, Some(v1)).0),
            _ => convert(c),
        })
        .collect();
    out
}

/// One object, recursively; `None` drops it.
fn convert(n: &Node) -> Option<Node> {
    Some(match n.class.as_str() {
        "MIDIMap" => return None,
        "BindingMap" => crate::bindings::convert_binding_map(n, None).0,
        "Layer" => layer(n),
        "Step" => step(n),
        "Track" => track(n),
        "AudioFileManager" => audio_files(n),
        _ => {
            let mut out = copy(n);
            out.children = n.children.iter().filter_map(convert).collect();
            out
        }
    })
}

/// Same element with `Ver` dropped and output names rewritten.
fn copy(n: &Node) -> Node {
    let mut out = Node { class: n.class.clone(), name: n.name.clone(), version: n.version, attrs: Vec::new(), children: Vec::new() };
    for (k, v) in &n.attrs {
        match k.as_str() {
            "Ver" | "FileMarkerVersion" => {}
            "OutputLocation" => {
                out.set(k, output_name(v));
            }
            _ => {
                out.set(k, v);
            }
        }
    }
    out
}

/// [re:59154a] (v1, ev_a80119bf) → v2 route
/// names. v1's four `Buss n` outputs are the global aux buses, which v2
/// calls `Aux n` [guess: same buses, renamed].
fn output_name(v: &str) -> String {
    match v.strip_prefix("Buss ") {
        Some(k) => format!("Aux {k}"),
        None => v.to_owned(),
    }
}

/// v1 sample paths ([re:5b6ca8], ev_e162a1c9) are
/// relative to `<doc>.samples/`: a leading `../` climbs out of it, and
/// `_RELATIVE_x` means `x` next to the document. Dreist resolves relative
/// names against the samples folder too, so only the prefix changes.
pub fn sample_path(v: &str) -> String {
    match v.strip_prefix("_RELATIVE_") {
        Some(rest) => format!("../{rest}"),
        None => v.to_owned(),
    }
}

fn audio_files(n: &Node) -> Node {
    let mut out = copy(n);
    out.children = n
        .children
        .iter()
        .map(|c| {
            let mut c = c.clone();
            if let Some(p) = c.attr("string").map(sample_path) {
                c.set("string", p);
            }
            c
        })
        .collect();
    out
}

/// [re:a71e0c] (v2): `State` plus lanes v2 knows. v1's
/// `Ampenvdepth1/2`, `Freeenvdepth1/2` lanes have no v2 counterpart.
fn step(n: &Node) -> Node {
    let mut out = Node { class: n.class.clone(), name: n.name.clone(), version: None, attrs: Vec::new(), children: Vec::new() };
    for (k, v) in &n.attrs {
        if k == "State" || STEP_LANES.contains(&k.as_str()) {
            out.set(k, v);
        }
    }
    out
}

/// `Track`: v2 reads the same `S%d` steps (`Ver` 2.2) and v1's older
/// `Steps` array (`Ver` 2.1, [re:ef2cfc]); the array
/// is unpacked into `S%d` children [guess: array children are the steps in
/// order]. Lane modifiers are kept for lanes v2 has.
fn track(n: &Node) -> Node {
    let mut out = copy(n);
    out.attrs.retain(|(k, _)| match ["Rand", "Scal", "Comp"].iter().find_map(|s| k.strip_suffix(s)) {
        Some(lane) => STEP_LANES.contains(&lane),
        None => k != "TrackVersion",
    });
    let legacy = n.attr("Ver").and_then(|v| v.parse::<f32>().ok()).is_some_and(|v| (v - 2.1).abs() >= 1e-6);
    let array = n.children.iter().find(|c| c.name.as_deref() == Some("Steps"));
    let steps: Vec<Node> = match array {
        Some(a) if legacy || n.attr("Ver").is_none() => a
            .children
            .iter()
            .filter(|c| c.class == "Step")
            .enumerate()
            .map(|(i, s)| {
                let mut s = step(s);
                s.name = Some(format!("S{i}"));
                s
            })
            .collect(),
        _ => n.children.iter().filter(|c| c.name.as_deref() != Some("Steps")).filter_map(convert).collect(),
    };
    out.children = steps;
    out
}

/// The v1 layer `Ver` as v2 reads it: `round(Ver·1000)`, 3300 when absent.
fn layer_version(n: &Node) -> u32 {
    n.attr("Ver").and_then(|v| v.parse::<f32>().ok()).map(|v| (v * 1000.0 + 0.5) as u32).unwrap_or(LAYER_VERSION)
}

/// Dropped v1 layer attributes. `SourceBPM` and `DestinationBPM` stay: the
/// engine's v1 timestretch ratio reads them ([re:d8985e]).
const V1_ONLY_LAYER_ATTRS: [&str; 5] = ["Ver", "V2C", "V2P", "UsesTimestretch", "TSTracksBPM"];

/// [re:87acba] (v2 [re:6f7ded], ev_5075e9a1) loading a v1 layer.
fn layer(n: &Node) -> Node {
    let ver = layer_version(n);
    let mut out = copy(n);
    out.version = Some(LAYER_VERSION);
    out.attrs.retain(|(k, _)| !V1_ONLY_LAYER_ATTRS.contains(&k.as_str()) && k != "TSMode" && !k.starts_with("Envelope"));
    if let Some(p) = n.attr("AudioFileName").map(sample_path) {
        out.set("AudioFileName", p);
    }
    let f = |k: &str| n.f32(k);
    let set = |out: &mut Node, k: &str, v: f32| {
        out.set(k, format_g16(v as f64));
    };
    if ver < LAYER_VERSION {
        // Velocity amounts become Velocity depths: 2v − 1.
        for (k, p) in [("V2C", LayerParam::FilterCutoff), ("V2P", LayerParam::CoarseTune)] {
            if let Some(v) = f(k) {
                set(&mut out, &velocity_attr(p), v + v - 1.0);
            }
        }
        let on = |k: &str| if n.bool(k).unwrap_or(false) { 1.0 } else { 0.0 };
        set(&mut out, LayerParam::TimestretchActive.name(), on("UsesTimestretch"));
        set(&mut out, LayerParam::TimestretchTracksBPM.name(), on("TSTracksBPM"));
        let fast = n.attr("TSMode").is_some_and(|m| m == "Fast");
        set(&mut out, LayerParam::TimestretchMode.name(), if fast { 1.0 } else { 0.0 });
        // v1's stretch is source over destination BPM ([re:d8985e]); as
        // the amount (the length in %), as the editor sets it.
        if n.attr("SourceBPM").is_some() || n.attr("DestinationBPM").is_some() {
            let (src, dst) = (f("SourceBPM").unwrap_or(120.0) as f64, f("DestinationBPM").unwrap_or(120.0) as f64);
            if src > 0.0 && dst > 0.0 && src != dst {
                let amount = LayerParam::TimestretchAmount.curve().from_units((100.0 * src / dst) as f32).clamp(0.0, 1.0);
                set(&mut out, LayerParam::TimestretchAmount.name(), amount);
            }
        }
    }
    // Below 3200 (v1 `Ver` < 3.2): the old cutoff/resonance scales. v1's own
    // remap (ev_7c48010c) and v2's are the same formula.
    if ver < 3200 {
        if let Some(c) = f("Cutoff") {
            let c2 = c * c;
            let v = ((c2 * c2 * 400.36362 + 0.5454545).ln() + 2.0794415) * 0.12022459;
            set(&mut out, "Cutoff", v.min(1.0).max(0.0));
        }
        if let Some(r) = f("Resonance") {
            set(&mut out, "Resonance", (r * 1.320132 - 0.11111111).min(1.0).max(0.0));
        }
    }
    if ver < LAYER_VERSION {
        envelopes(n, ver, &mut out);
        routings(n, &mut out);
    }
    out.children = n.children.iter().filter(|c| c.class != "LayerModulationRouting").filter_map(convert).collect();
    out
}

fn velocity_attr(p: LayerParam) -> String {
    TransmodSource::Velocity.attr(p.name()).unwrap_or_default()
}

/// `Envelope{e}Param{i}` → AmpEnv*/FreeEnv* (v2 params 14..29, same
/// order: Attack, Decay, Sustain, Hold, Release, three curves), clamped to
/// 0..1. Below 3000 the holds are set to −1 and the old default decay
/// (A 0, D 0.5, S 1, R 0.001) becomes 0.02, as in v1.
///
/// Hold follows v1, not v2 (owner's decision, 2026-11-03): a negative
/// Hold stays −1 instead of v2's clamp to 0. v1's loader then calls
/// [re:bd735d] (ev_97d7ba71), which derives the hold from
/// the sample; Dreist's loader does the same computation for a negative
/// hold (`dreist_engine::hold::reset_hold_time`, see dreist-re `einst/docs/re/loading-v1.md`).
fn envelopes(n: &Node, ver: u32, out: &mut Node) {
    const FIRST: [LayerParam; 2] = [LayerParam::AmpEnvAttack, LayerParam::FreeEnvAttack];
    for (e, first) in FIRST.iter().enumerate() {
        let base = *first as usize;
        let read = |i: usize| n.f32(&format!("Envelope{e}Param{i}")).map(|v| if i == 3 && v < 0.0 { -1.0 } else { v.min(1.0).max(0.0) });
        let mut vals: [Option<f32>; 8] = std::array::from_fn(read);
        if ver < 3000 {
            vals[3] = Some(-1.0);
            let is = |i: usize, x: f32| (vals[i].unwrap_or(LayerParam::ALL[base + i].default_value()) - x).abs() <= 1e-6;
            let old_default = is(0, 0.0) && is(1, 0.5) && is(2, 1.0) && is(4, 0.001);
            if old_default {
                vals[1] = Some(0.02);
            }
        }
        for (i, v) in vals.iter().enumerate() {
            if let Some(v) = v {
                out.set(LayerParam::ALL[base + i].name(), format_g16(*v as f64));
            }
        }
    }
}

/// `ModRouting0..3`: routings 0 and 1 are the amp envelope's, 2 and 3 the
/// free envelope's. A depth d ≠ 0.5 to a destination becomes that source's
/// depth 2d − 1 on Cutoff, Resonance, FilterDrive or CoarseTune (Pitch).
/// Two routings of one envelope to one destination average their depths,
/// as v1's voice does (`UpdatePitchModulation` ev_c7cae932).
fn routings(n: &Node, out: &mut Node) {
    let mut acc: Vec<(String, f32, u32)> = Vec::new();
    for i in 0..4u32 {
        let name = format!("ModRouting{i}");
        let Some(r) = n.child("LayerModulationRouting", &name) else { continue };
        let depth = r.f32("ModulationDepth").unwrap_or(0.5);
        let dest = r.attr("ModulationDestination").unwrap_or("No destination");
        if (depth - 0.5).abs() < 1e-6 {
            continue;
        }
        let param = match dest {
            "Cutoff" => LayerParam::FilterCutoff,
            "Resonance" => LayerParam::FilterResonance,
            "Drive" => LayerParam::FilterDrive,
            "Pitch" => LayerParam::CoarseTune,
            _ => continue,
        };
        let src = if i < 2 { TransmodSource::AmpEnvelope } else { TransmodSource::FreeEnvelope };
        if let Some(k) = src.attr(param.name()) {
            match acc.iter_mut().find(|(key, ..)| *key == k) {
                Some(e) => {
                    e.1 += depth;
                    e.2 += 1;
                }
                None => acc.push((k, depth, 1)),
            }
        }
    }
    for (k, sum, count) in acc {
        let d = sum / count as f32;
        out.set(&k, format_g16((d + d - 1.0).min(1.0).max(-1.0) as f64));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_paths() {
        assert_eq!(sample_path("Kick.wav"), "Kick.wav");
        assert_eq!(sample_path("_RELATIVE_Drums\\Kick.wav"), "../Drums\\Kick.wav");
        assert_eq!(sample_path("../x.wav"), "../x.wav");
    }

    #[test]
    fn outputs() {
        assert_eq!(output_name("Buss 2"), "Aux 2");
        assert_eq!(output_name("Sub output 3"), "Sub output 3");
    }

    /// Two routings of one envelope to one destination average (v1).
    #[test]
    fn duplicate_routings_average() {
        let n = Node::parse(
            r#"<Layer name="Layer0" Ver="3.2">
                <LayerModulationRouting name="ModRouting0" Ver="1" ModulationDepth="0.9" ModulationDestination="Cutoff" />
                <LayerModulationRouting name="ModRouting1" Ver="1" ModulationDepth="0.7" ModulationDestination="Cutoff" />
                <LayerModulationRouting name="ModRouting2" Ver="1" ModulationDepth="0.25" ModulationDestination="Pitch" />
            </Layer>"#,
        )
        .unwrap();
        let mut out = Node::new("Layer", None);
        routings(&n, &mut out);
        let k = TransmodSource::AmpEnvelope.attr(LayerParam::FilterCutoff.name()).unwrap();
        assert!((out.f32(&k).unwrap() - 0.6).abs() < 1e-6);
        let k = TransmodSource::FreeEnvelope.attr(LayerParam::CoarseTune.name()).unwrap();
        assert!((out.f32(&k).unwrap() + 0.5).abs() < 1e-6);
    }
}
