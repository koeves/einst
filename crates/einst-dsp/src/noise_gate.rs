//! Einst: Dreist's Noise Gate (`dreist_dsp::fx::nodes::noise_gate`, copied
//! from dreist-commons 69f4cc1) in v1's form (`Voice::ProcessBuffer`
//! ev_9a683fda, `InitializePorts` ev_df3a50d3; dreist-re
//! `einst/docs/re/effects-v1.md`):
//!
//! - v1's port order: Attack, Hold, Release, Threshold, Hysteresis, LPFreq,
//!   HPFreq, Preview, Sidechain, WetMix (v2 puts Sidechain and WetMix at 4
//!   and 5);
//! - every 2^x (attack, hold, release, key LP/HP) is v1's
//!   [`fast_exp2`](crate::fast_exp2);
//! - v1's hold: rearmed while the key is over the threshold or, with the
//!   hold running, over the hysteresis level; counted down by one; the
//!   gate opens while holding or over the hysteresis level. v2 counts down
//!   by two, rearms only above 1 and closes when the hold has run out
//!   (checked in v2's disassembly).
//!
//! The side-chain inputs are not modelled (as in Dreist).
//!
//! [re:8fc789] (Noise Gate, kind 11; `Voice::Reset` [re:fa4788],
//! `Voice::ProcessBuffer` [re:bacb48]): key = band-passed (LP/HP one-poles)
//! input, peak envelope with release, hold counter with hysteresis, gain
//! attack/release; Preview outputs the key.

use crate::fast_exp2;
use dreist_dsp::fx::nodes::blocks::ALT;
use dreist_dsp::fx::{Ctx, Node, Port, ports, ports_from};

/// v2 port index for each v1 port.
pub const V1_TO_V2: [usize; 10] = [0, 1, 2, 3, 6, 7, 8, 9, 4, 5];

pub struct NoiseGate {
    ports: Vec<Port>,
    alt: f32,
    hold: i32,
    env: f32,
    gain: f32,
    lp: [f32; 2],
    hp: [f32; 2],
    /// +0x140..+0x14c: offset by the alternator, never otherwise used.
    spare: [f32; 4],
}

impl NoiseGate {
    pub fn new() -> NoiseGate {
        let v2 = ports_from(ports::NOISE_GATE);
        NoiseGate::with_ports(V1_TO_V2.iter().map(|&i| v2[i]).collect())
    }

    /// As built around `ports` (no allocation).
    fn with_ports(ports: Vec<Port>) -> NoiseGate {
        NoiseGate { ports, alt: f32::from_bits(ALT), hold: 0, env: 0.0, gain: 0.0, lp: [0.0; 2], hp: [0.0; 2], spare: [0.0; 4] }
    }
}

impl Default for NoiseGate {
    fn default() -> Self {
        NoiseGate::new()
    }
}

impl Node for NoiseGate {
    fn ports(&mut self) -> &mut Vec<Port> {
        &mut self.ports
    }

    fn process(&mut self, ctx: &Ctx, inp: [&[f32]; 2], out: [&mut [f32]; 2]) {
        let p = |i: usize| self.ports[i].value;
        let sr = ctx.sample_rate;
        let isr = 1.0 / sr;
        let att = fast_exp2(p(0) + -1.0);
        let hold_n = (fast_exp2(p(1) + -1.0) * sr).round() as i32;
        let thr = 10f32.powf((p(3).log10() * 20.0) / 20.0);
        let hyst = 10f32.powf((p(4).log10() * 20.0) / 20.0);
        let ka = (-6.2831855 / (att * sr)).exp();
        let f_lp = fast_exp2(p(5) + 1.0);
        let f_hp = fast_exp2(p(6) + 1.0);
        let k_hp = 1.0 - ((f_hp + f_hp) * 0.5 * -6.2831855 * isr).exp();
        let n = inp[0].len();
        if n == 0 {
            return;
        }
        let preview = p(7) >= 0.5;
        let mix = ((p(9).abs() + 1.0) - (p(9) - 1.0).abs()) * 0.5;
        let e_lp = (isr * (f_lp + f_lp) * -0.5 * 6.2831855).exp();
        let rel = fast_exp2(p(2) + -1.0);
        let kr = 1.0 - (1.0 - (-6.2831855 / (sr * rel)).exp());
        let [ol, or] = out;
        for i in 0..n {
            let a = self.alt;
            self.alt = -a;
            for v in self.lp.iter_mut().chain(&mut self.hp).chain(&mut self.spare) {
                *v -= a;
            }
            let key = [inp[0][i], inp[1][i]];
            let mut b = [0.0f32; 2];
            for c in 0..2 {
                self.lp[c] = (key[c] - self.lp[c]) * (1.0 - e_lp) + self.lp[c];
                let d = self.lp[c] - self.hp[c];
                self.hp[c] = k_hp * d + self.hp[c];
                b[c] = d * (1.0 - k_hp);
            }
            let (al, ar) = (b[0].abs(), b[1].abs());
            let peak = if ar <= al { al } else { ar };
            let decayed = self.env * kr;
            self.env = if peak <= decayed { decayed } else { peak };
            // v1: rearm while the key is over the threshold, or over the
            // hysteresis level with the hold still running; else count down
            // by one (saturating where v1 would wrap after 2^31 samples).
            // The gate opens while holding or over the hysteresis level.
            let open_level = self.env > hyst * thr;
            if peak > thr || (open_level && self.hold > 0) {
                self.hold = hold_n;
            } else {
                self.hold = self.hold.saturating_sub(1);
            }
            if self.hold >= 0 || open_level {
                self.gain = (1.0 - self.gain) * (1.0 - ka) + self.gain;
            } else {
                self.gain *= kr;
            }
            if preview {
                ol[i] = b[0];
                or[i] = b[1];
            } else {
                let (x, y) = (inp[0][i], inp[1][i]);
                ol[i] = x * mix * self.gain + (1.0 - mix) * x;
                or[i] = y * mix * self.gain + (1.0 - mix) * y;
            }
        }
    }

    fn reset(&mut self) {
        let alt = self.alt;
        *self = NoiseGate::with_ports(std::mem::take(&mut self.ports));
        self.alt = alt;
    }
}
