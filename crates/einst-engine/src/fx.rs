//! Einst's copy (see docs/PROGRESS.md): `from_node` builds Geist v1's
//! own nodes through `einst_dsp::create`.
//!
//! Effect chains as stored in projects (`EffectChain::Serialize`,
//! `Effect::Serialize`) and their per-block parameter update
//! (`EffectChain::Update`). The DSP lives in `geist_dsp::fx`.

use geist_dsp::fx::{Ctx, Effect, EffectChain};
use geist_dsp::voice::SOURCES;
use geist_formats::params::{effect_param_names, effect_type_index};
use geist_formats::{Node, TransmodSource};

use crate::engine::{Row, param_sum};

struct SlotRows {
    is_active: bool,
    mix: Row,
    level: Row,
    trim: Row,
    params: Vec<Row>,
}

fn row(n: &Node, names: &[String], default: f32) -> Row {
    let mut r = [0.0; SOURCES];
    r[0] = names.iter().find_map(|k| n.f32(k)).unwrap_or(default);
    for (i, s) in TransmodSource::ALL.iter().enumerate().skip(1) {
        r[i] = names.iter().find_map(|k| n.f32(&format!("{k}_{}", s.name()))).unwrap_or(0.0);
    }
    r
}

/// What an effect message sets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FxTarget {
    Param(usize),
    Mix,
    Level,
    Trim,
}

/// A loaded chain with its parameter rows.
pub struct FxChain {
    pub chain: EffectChain,
    rows: Vec<Option<SlotRows>>,
}

impl FxChain {
    pub fn from_node(n: Option<&Node>, sample_rate: f32) -> FxChain {
        let mut chain = EffectChain::new();
        let mut rows: Vec<Option<SlotRows>> = (0..6).map(|_| None).collect();
        if let Some(n) = n {
            chain.active = n.bool("IsActive").unwrap_or(true);
            chain.mix = n.f32("ChainMix").unwrap_or(1.0);
            for e in n.children_of("Effect") {
                let Some(slot) = e.index().map(|i| i as usize).filter(|i| *i < 6) else { continue };
                let Some(kind) = e.attr("EffectType").and_then(effect_type_index) else { continue };
                if kind == 0 {
                    continue;
                }
                let params = effect_param_names(kind)
                    .iter()
                    .enumerate()
                    .map(|(k, name)| {
                        let stripped: String = name.chars().filter(|c| !c.is_whitespace()).collect();
                        row(e, &[stripped, format!("Parameter{k}")], 0.0)
                    })
                    .collect();
                let r =
                    SlotRows { is_active: e.bool("IsActive").unwrap_or(true), mix: row(e, &["Mix".into()], 0.5), level: row(e, &["Level".into()], 1.0), trim: row(e, &["Trim".into()], 1.0), params };
                let mut fx = Effect::new(kind, sample_rate);
                // Einst: Geist v1's own nodes (Gain, Vari-EQ, Envelope
                // Shaper; `einst_dsp`) replace Dreist's Geist2 ones.
                if let Some(mut node) = einst_dsp::create(kind) {
                    for &(port, v) in einst_dsp::internal_ports(kind) {
                        if let Some(p) = node.ports().get_mut(port) {
                            p.set01(v);
                        }
                    }
                    node.prepare_rate(sample_rate);
                    fx.node = Some(node);
                }
                fx.active = r.is_active;
                fx.mix = r.mix[0];
                fx.level = r.level[0];
                fx.trim_target = r.trim[0];
                for (k, p) in r.params.iter().enumerate() {
                    fx.push_parameter(k, p[0]);
                }
                chain.slots[slot] = Some(fx);
                rows[slot] = Some(r);
            }
        }
        FxChain { chain, rows }
    }

    pub fn is_empty(&self) -> bool {
        self.rows.iter().all(|r| r.is_none())
    }

    /// `EffectChain::Update`: modulation vector `m` and, for pad chains, the
    /// triggering step's lanes.
    pub fn update(&mut self, m: &[f32; SOURCES], lanes: Option<&[f32]>) {
        for (n, (slot, rows)) in self.chain.slots.iter_mut().zip(&self.rows).enumerate() {
            let (Some(fx), Some(r)) = (slot.as_mut(), rows.as_ref()) else { continue };
            let lane = |i: usize| lanes.map(|l| l.get(i).copied().unwrap_or(1.0)).unwrap_or(1.0);
            fx.active = match lanes {
                Some(l) => match (l[39 + 9 * n] * 2.0 + 1.0).floor() as i32 {
                    3 => false,
                    1 => r.is_active,
                    _ => true,
                },
                None => r.is_active,
            };
            fx.mix = param_sum(&r.mix, m) * lane(40 + 9 * n);
            fx.level = param_sum(&r.level, m) * lane(41 + 9 * n);
            fx.trim_target = param_sum(&r.trim, m);
            for (k, p) in r.params.iter().enumerate().take(12) {
                let modulated = p[1..].iter().any(|d| *d != 0.0);
                let v = if modulated { param_sum(p, m) } else { p[0] };
                fx.push_parameter(k, v * lane(42 + 9 * n + k));
            }
        }
    }

    /// `Effect::SetParameter` / `SetMix` / `SetOutputLevel` / `SetInputTrim`
    /// (messages 0x59, 0x5c–0x5e): store into the slot's rows; the next
    /// `update` applies it.
    pub fn set(&mut self, slot: usize, target: FxTarget, src: usize, value: f32) {
        let Some(Some(r)) = self.rows.get_mut(slot) else { return };
        if src >= SOURCES {
            return;
        }
        let row = match target {
            FxTarget::Param(k) => match r.params.get_mut(k) {
                Some(p) => p,
                None => return,
            },
            FxTarget::Mix => &mut r.mix,
            FxTarget::Level => &mut r.level,
            FxTarget::Trim => &mut r.trim,
        };
        row[src] = value;
    }

    /// `Effect::SetActive` (message 0x5b).
    pub fn set_active(&mut self, slot: usize, active: bool) {
        if let Some(Some(r)) = self.rows.get_mut(slot) {
            r.is_active = active;
        }
    }

    pub fn process(&mut self, ctx: &Ctx, l: &mut [f32], r: &mut [f32]) {
        self.chain.process(ctx, l, r);
    }

    pub fn tail(&self, ctx: &Ctx) -> usize {
        self.chain.tail(ctx)
    }
}

/// Einst: chains build Geist v1's nodes for Gain, Vari-EQ and Envelope Shaper.
#[cfg(test)]
mod einst_tests {
    use super::*;

    #[test]
    fn chains_build_v1_nodes() {
        let n = Node::parse(
            r#"<EffectChain name="EffectChain" IsActive="true">
                <Effect name="Effect0" IsActive="true" Mix="1" Level="1" Parameter0="0.6" EffectType="Gain" />
                <Effect name="Effect1" IsActive="true" Mix="1" Level="1" Parameter0="0.5" Parameter1="0.5" Parameter2="0.5" Parameter3="0.5" Parameter4="1" EffectType="Envelope Shaper" />
            </EffectChain>"#,
        )
        .unwrap();
        let mut fx = FxChain::from_node(Some(&n), 44_100.0);
        let gain = fx.chain.slots[0].as_mut().unwrap();
        let x = gain.node.as_mut().unwrap().ports()[0].value;
        let ones = vec![1.0f32; 64];
        let (mut l, mut r) = (vec![0.0; 64], vec![0.0; 64]);
        gain.process(&Ctx::new(44_100.0), [&ones, &ones], [&mut l, &mut r]);
        // v1's fast 2^x, not Dreist's exact one.
        assert!((l[63] - einst_dsp::gain::Gain::target(x)).abs() < 1e-6, "{} vs {}", l[63], einst_dsp::gain::Gain::target(x));
        // v1's internal setup: the Envelope Shaper's WetMix at 1; Linked pushed as 1.
        let es = fx.chain.slots[1].as_mut().unwrap().node.as_mut().unwrap();
        assert_eq!(es.ports()[0].value, 1.0);
        assert_eq!(es.ports()[5].value, 1.0);
    }
}
