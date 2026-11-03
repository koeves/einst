//! v1 document tree → Geist2 document tree (what Dreist's `geist_formats`
//! and `geist_engine` read). See `docs/re/loading-v1.md`.
//!
//! v1 and Geist2 share the element shape (tag = class, `name` = instance
//! key) and most attribute names, so the conversion walks the tree and
//! rewrites only what differs:
//! * per-object `Ver` floats are dropped; layers get `version="3300"` and
//!   the root `version="3000"`.
//! * layers go through Geist2's legacy branch (`Layer::Serialize`
//!   for `Ver` < 3300, ev_5075e9a1): envelopes, modulation routings,
//!   velocity amounts and timestretch switches become Geist2 parameters.
//! * step lanes and track modifiers that Geist2 does not have are dropped
//!   (Geist2's `Step::Serialize` reads only its own lane names).
//! * output names and sample paths are rewritten to Geist2's forms.
//! * MIDI/host bindings are dropped (v1's `Binding` message format is not
//!   traced).

use geist_formats::node::format_g16;
use geist_formats::{LayerParam, Node, STEP_LANES, TransmodSource};

/// Layer version Geist2 writes (`Layer::Serialize`, 0xce4).
const LAYER_VERSION: u32 = 3300;
/// Root version of a Geist2 `.geist` (`Geist::empty` in Dreist).
const ROOT_VERSION: u32 = 3000;

/// Convert a v1 `<root>` (project, kit, pad, engine, pattern or bank).
pub fn convert_root(v1: &Node) -> Node {
    let mut out = Node::new("root", None);
    out.version = Some(ROOT_VERSION);
    for (k, v) in &v1.attrs {
        match k.as_str() {
            // `Grid::LoadEntireState` gates on it; Geist2 reads it only below root version 3000.
            "StateVersion" => {}
            "PreviewOutput" => {
                out.set(k, output_name(v));
            }
            _ => {
                out.set(k, v);
            }
        }
    }
    out.children = v1.children.iter().filter_map(convert).collect();
    out
}

/// One object, recursively; `None` drops it.
fn convert(n: &Node) -> Option<Node> {
    Some(match n.class.as_str() {
        "BindingMap" | "MIDIMap" => return None,
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

/// `AudioOutputs::GetNameOfOutputLocation` (v1, ev_a80119bf) → Geist2 route
/// names. v1's four `Buss n` outputs are the global aux buses, which Geist2
/// calls `Aux n` [guess: same buses, renamed].
fn output_name(v: &str) -> String {
    match v.strip_prefix("Buss ") {
        Some(k) => format!("Aux {k}"),
        None => v.to_owned(),
    }
}

/// v1 sample paths (`XMLFile::ConvertSamplePath`, ev_e162a1c9) are
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

/// `Step::Serialize` (Geist2): `State` plus lanes Geist2 knows. v1's
/// `Ampenvdepth1/2`, `Freeenvdepth1/2` lanes have no Geist2 counterpart.
fn step(n: &Node) -> Node {
    let mut out = Node { class: n.class.clone(), name: n.name.clone(), version: None, attrs: Vec::new(), children: Vec::new() };
    for (k, v) in &n.attrs {
        if k == "State" || STEP_LANES.contains(&k.as_str()) {
            out.set(k, v);
        }
    }
    out
}

/// `Track`: Geist2 reads the same `S%d` steps (`Ver` 2.2) and v1's older
/// `Steps` array (`Ver` 2.1, `Track::SerializeFromData`); the array
/// is unpacked into `S%d` children [guess: array children are the steps in
/// order]. Lane modifiers are kept for lanes Geist2 has.
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

/// The v1 layer `Ver` as Geist2 reads it: `round(Ver·1000)`, 3300 when absent.
fn layer_version(n: &Node) -> u32 {
    n.attr("Ver").and_then(|v| v.parse::<f32>().ok()).map(|v| (v * 1000.0 + 0.5) as u32).unwrap_or(LAYER_VERSION)
}

const V1_ONLY_LAYER_ATTRS: [&str; 7] = ["Ver", "V2C", "V2P", "UsesTimestretch", "TSTracksBPM", "SourceBPM", "DestinationBPM"];

/// `Layer::Serialize` (Geist2 0x71c10, ev_5075e9a1) loading a v1 layer.
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
    }
    // Below 3200 (v1 `Ver` < 3.2): the old cutoff/resonance scales. v1's own
    // remap (ev_7c48010c) and Geist2's are the same formula.
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

/// `Envelope{e}Param{i}` → AmpEnv*/FreeEnv* (Geist2 params 14..29, same
/// order: Attack, Decay, Sustain, Hold, Release, three curves), clamped to
/// 0..1. Below 3000 the holds are set to −1 and the old default decay
/// (A 0, D 0.5, S 1, R 0.001) becomes 0.02, as in v1.
fn envelopes(n: &Node, ver: u32, out: &mut Node) {
    const FIRST: [LayerParam; 2] = [LayerParam::AmpEnvAttack, LayerParam::FreeEnvAttack];
    for (e, first) in FIRST.iter().enumerate() {
        let base = *first as usize;
        // TODO(hold, pending the owner's decision): this clamps a v1 3.x layer's
        // Hold of -1 to 0 as Geist2 does. v1 itself keeps -1 and recomputes the
        // hold from the sample (`ResetHoldTime`); leaving `vals[3]` at -1 would
        // make Dreist do the same. See docs/re/loading-v1.md, "Guesses".
        let mut vals: [Option<f32>; 8] = std::array::from_fn(|i| n.f32(&format!("Envelope{e}Param{i}")).map(|v| v.min(1.0).max(0.0)));
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
fn routings(n: &Node, out: &mut Node) {
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
            out.set(&k, format_g16((depth + depth - 1.0).min(1.0).max(-1.0) as f64));
        }
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
}
