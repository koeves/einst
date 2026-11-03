//! Shared harness pieces: the demo presets and the stress scenarios that
//! rewrite their layers and steps.

#![allow(dead_code)]

use std::path::{Path, PathBuf};

use geist_engine::audio::SampleCache;
use geist_engine::{Geist, RenderOptions};
use geist_formats::Node;
use geist_testkit::Stats;

pub fn presets() -> Vec<PathBuf> {
    let home = std::env::var("HOME").unwrap_or_default();
    let content = std::env::var("GEIST2_CONTENT").unwrap_or_else(|_| format!("{home}/Documents/FXpansion/Geist2/Content"));
    let Ok(rd) = std::fs::read_dir(PathBuf::from(content).join("Geist2 Demo Factory/Presets")) else { return Vec::new() };
    let mut v: Vec<PathBuf> = rd.flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|e| e == "geist")).collect();
    v.sort();
    v
}

/// Deterministic values in 0..1.
pub struct Lcg(pub u32);

impl Lcg {
    pub fn next(&mut self) -> f32 {
        self.0 = self.0.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (self.0 >> 8) as f32 / 16_777_216.0
    }
}

pub fn walk(n: &mut Node, f: &mut dyn FnMut(&mut Node)) {
    f(n);
    for c in &mut n.children {
        walk(c, f);
    }
}

/// Every layer node with its (pad, layer) numbers.
pub fn layers(doc: &mut Node, mut f: impl FnMut(&mut Node, usize, usize)) {
    for e in doc.children.iter_mut().filter(|c| c.class == "Engine") {
        for p in e.children.iter_mut().filter(|c| c.class == "Pad") {
            let pi = p.index().unwrap_or(0) as usize;
            for l in p.children.iter_mut().filter(|c| c.class == "Layer") {
                let li = l.index().unwrap_or(0) as usize;
                f(l, pi, li);
            }
        }
    }
}

pub type Scenario = (&'static str, fn(&mut Node));

pub fn scenarios() -> Vec<Scenario> {
    vec![
        ("plain", |_| {}),
        ("filters", |doc| {
            layers(doc, |l, p, _| {
                l.set("FilterActive", 1.0)
                    .set("FilterAlgorithm", (p % 4) as f32 / 3.0)
                    .set("FilterType", ((p * 7) % 10) as f32 / 10.0)
                    .set("FilterResonance", 0.95)
                    .set("FilterCutoff", 0.25 + 0.1 * (p % 7) as f32)
                    .set("FilterDrive", 0.8);
            })
        }),
        ("loops", |doc| {
            layers(doc, |l, p, _| {
                l.set("PlaybackMode", if p % 3 == 0 { "Gated" } else { "Looping" })
                    .set("LoopInRelease", (p % 2) as f32)
                    .set("AltLoopMode", ((p / 2) % 2) as f32)
                    .set("XFadeLoopTime", 0.3)
                    .set("SampleLoopStartPoint", 0.2)
                    .set("SampleLoopEndPoint", 0.9);
            })
        }),
        ("pitch", |doc| {
            layers(doc, |l, p, _| {
                l.set("CoarseTune", if p % 2 == 0 { 0.0 } else { 1.0 })
                    .set("Octave", if p % 4 < 2 { 0.0 } else { 1.0 })
                    .set("FineTune", 0.9)
                    .set("TimestretchActive", (p % 2) as f32)
                    .set("TimestretchAmount", if p % 3 == 0 { 0.05 } else { 0.95 })
                    .set("TimestretchMode", ((p / 3) % 2) as f32);
            })
        }),
        ("envelopes", |doc| {
            let mut r = Lcg(42);
            layers(doc, |l, p, _| {
                for k in ["AmpEnvAttack", "AmpEnvDecay", "AmpEnvSustain", "AmpEnvHold", "AmpEnvRelease", "FreeEnvAttack", "FreeEnvDecay", "FreeEnvSustain", "FreeEnvHold", "FreeEnvRelease"] {
                    l.set(k, r.next());
                }
                for k in ["AmpEnvAttackCurve", "AmpEnvDecayCurve", "AmpEnvReleaseCurve", "FreeEnvAttackCurve", "FreeEnvDecayCurve", "FreeEnvReleaseCurve"] {
                    l.set(k, if r.next() < 0.5 { 0.02 } else { 0.98 });
                }
                if p % 2 == 1 {
                    l.set("PlaybackMode", "OneShotLengthBound");
                }
            })
        }),
        ("vintage", |doc| {
            // The vintage sampler path, forward and reversed, transposed.
            layers(doc, |l, p, _| {
                l.set("SynthesisMode", if p % 2 == 0 { "VintageSampler" } else { "Sampler" }).set("IsReversed", if p % 3 == 0 { "true" } else { "false" }).set("CoarseTune", [0.5, 0.0, 1.0][p % 3]);
            })
        }),
        ("regions", |doc| {
            // Tiny and inverted sample regions, start offsets near the end.
            layers(doc, |l, p, _| {
                let (s, e) = [(0.0, 0.0), (0.0, 0.002), (0.99, 1.0), (0.5, 0.1), (1.0, 1.0), (0.3, 0.7)][p % 6];
                l.set("SampleStartPoint", s).set("SampleEndPoint", e);
            })
        }),
        ("lanes", |doc| {
            let mut r = Lcg(7);
            walk(doc, &mut |n| {
                if n.class == "Step" && n.attr("State").is_some_and(|v| v.trim() == "true") {
                    for lane in geist_formats::params::STEP_LANES {
                        n.set(lane, r.next());
                    }
                }
            })
        }),
    ]
}

/// Render `seconds` of `doc` at `sr` in host blocks of `block`.
pub fn render(doc: &Node, samples: &Path, sr: f32, block: usize, seconds: f32) -> (Vec<f32>, Vec<f32>) {
    let mut cache = SampleCache::default();
    let mut g = Geist::from_document(doc, samples, sr, RenderOptions::default(), &mut cache).expect("document builds");
    geist_engine::render(&mut g, (sr * seconds) as usize, block)
}

pub fn doc_for(preset: &Path, scenario: fn(&mut Node)) -> Node {
    let mut doc = geist_formats::Project::load(preset).expect("preset loads").raw;
    scenario(&mut doc);
    doc
}

pub fn name(p: &Path) -> String {
    p.file_stem().unwrap().to_string_lossy().into_owned()
}

/// A render that is not finite or peaks past +40 dB is broken.
pub fn sane(tag: &str, l: &[f32], r: &[f32]) -> Option<String> {
    let s = Stats::of(l.iter().chain(r));
    if !s.finite {
        Some(format!("{tag}: non-finite output"))
    } else if s.peak > 100.0 {
        Some(format!("{tag}: peak {:.1}", s.peak))
    } else {
        None
    }
}

/// A built (stopped) engine for `doc`.
pub fn engine(doc: &Node, samples: &Path, sr: f32) -> Geist {
    let mut cache = SampleCache::default();
    Geist::from_document(doc, samples, sr, RenderOptions::default(), &mut cache).expect("document builds")
}
