//! Einst: Dreist's port of the Filter effect (`dreist_dsp::fx::nodes::ssl_filter`,
//! copied from dreist-commons 69f4cc1) with v1's differences
//! (`Voice::UpdateCoeffs` ev_85d132f7, `Voice::Reset` ev_4e0ee1e1; dreist-re
//! `einst/docs/re/effects-v1.md`):
//!
//! - the HP and LP frequencies 2^(p + 1) are v1's [`fast_exp2`](crate::fast_exp2);
//! - `Voice::Reset` keeps the LP biquad (its tail rings on; v2 clears it).
//!
//! The rest is v2's: the HP one-pole/sine coefficients, the LP biquad
//! (Q 0.5..1000, f clamp; v1's lower clamp 0.001 against v2's 1e−5 never
//! applies at audio rates). v1's ±1e−8 denormal offset on the biquad
//! history and its summation order are not transcribed.
//!
//! [re:fb882e] (Filter; `InitializePorts` [re:007df0],
//! `Voice::UpdateCoeffs` [re:9c301b], `Voice::ProcessBuffer` [re:04067e]):
//! 3rd-order high-pass (one-pole + Chamberlin 2-pole) and a 12 dB/oct RBJ
//! low-pass, each switchable, with a wet/dry mix.

use crate::fast_exp2;
use dreist_dsp::fx::nodes::biquad::Biquad;
use dreist_dsp::fx::{Ctx, Node, Port, ports, ports_from};

/// [re:f2c879] voices start their anti-denormal offset at 1e-8 (`CreateVoice`).
pub const DENORMAL: f32 = 1e-8;

pub struct SslFilter {
    ports: Vec<Port>,
    first: bool,
    hp_key: f32,
    lp_key: f32,
    g2: f32,
    c1: f32,
    s1: [f32; 2],
    s2: [f32; 2],
    s3: [f32; 2],
    alt: f32,
    lp: [Biquad; 2],
}

impl SslFilter {
    pub fn new() -> SslFilter {
        SslFilter { ports: ports_from(ports::FILTER), first: true, hp_key: 0.0, lp_key: 0.0, g2: 0.0, c1: 0.0, s1: [0.0; 2], s2: [0.0; 2], s3: [0.0; 2], alt: DENORMAL, lp: [Biquad::default(); 2] }
    }

    fn update(&mut self, ctx: &Ctx) {
        let inv_sr = 1.0 / ctx.sample_rate;
        let hp = self.ports[1].value;
        if self.first || self.hp_key != hp {
            self.hp_key = hp;
            let f = fast_exp2(hp + 1.0) * inv_sr;
            let g = ((f.min(0.125) * std::f32::consts::PI) as f64).sin();
            self.g2 = (g + g) as f32;
            self.c1 = 1.0 - (f * -6.2831855).exp();
        }
        let lp = self.ports[3].value;
        if self.first || self.lp_key != lp {
            self.lp_key = lp;
            let f = fast_exp2(lp + 1.0) * inv_sr;
            for b in &mut self.lp {
                b.set(3, f, std::f32::consts::FRAC_1_SQRT_2, 0.0);
            }
        }
        self.first = false;
    }
}

impl Default for SslFilter {
    fn default() -> Self {
        SslFilter::new()
    }
}

impl Node for SslFilter {
    fn ports(&mut self) -> &mut Vec<Port> {
        &mut self.ports
    }

    /// `Voice::Reset` [re:2f591b]: coefficients, keys and filter history
    /// cleared, recomputed next block (the voice's denormal offset runs on).
    fn reset(&mut self) {
        self.first = true;
        self.hp_key = 0.0;
        self.lp_key = 0.0;
        self.g2 = 0.0;
        self.c1 = 0.0;
        self.s1 = [0.0; 2];
        self.s2 = [0.0; 2];
        self.s3 = [0.0; 2];
    }

    fn process(&mut self, ctx: &Ctx, inp: [&[f32]; 2], out: [&mut [f32]; 2]) {
        self.update(ctx);
        let hp_on = self.ports[0].value >= 0.5;
        let lp_on = self.ports[2].value >= 0.5;
        let w = self.ports[4].value;
        let wet = ((w.abs() + 1.0) - (w - 1.0).abs()) * 0.5;
        let a = self.alt;
        self.alt = -a;
        for c in 0..2 {
            self.s1[c] -= a;
            self.s2[c] -= a;
            self.s3[c] -= a;
        }
        let [ol, or] = out;
        for i in 0..inp[0].len() {
            let mut y = [inp[0][i], inp[1][i]];
            if hp_on {
                for c in 0..2 {
                    let h1 = y[c] - self.s1[c];
                    self.s1[c] += h1 * self.c1;
                    self.s2[c] += self.g2 * self.s3[c];
                    let h2 = (h1 - self.s2[c]) - self.s3[c];
                    self.s3[c] += h2 * self.g2;
                    y[c] = h2;
                }
            }
            if lp_on {
                y[0] = self.lp[0].tick(y[0]);
                y[1] = self.lp[1].tick(y[1]);
            }
            ol[i] = inp[0][i] * (1.0 - wet) + wet * y[0];
            or[i] = inp[1][i] * (1.0 - wet) + y[1] * wet;
        }
    }
}

/// The editor plot of part `part` for port values `ports`: 0 the low-pass,
/// 1 the high-pass. [re:d9b038] draws the part's response when its active
/// port is on, else the flat 0 dB line. The high-pass is shown as v1's
/// display biquad ([re:426c2d]: an RBJ high-pass, Q 1/√2, at the HP
/// frequency), not the 3rd-order filter the audio runs through.
pub fn display_part(ports: &[f32], part: usize, sample_rate: f32) -> [f32; crate::eq_display::POINTS] {
    let p = |i: usize| ports.get(i).copied().unwrap_or(0.0);
    let (on, freq, mode) = match part {
        0 => (2, 3, 3),
        1 => (0, 1, 6),
        _ => return crate::eq_display::curve(&[], sample_rate),
    };
    if p(on) < 0.5 {
        return crate::eq_display::curve(&[], sample_rate);
    }
    let mut f = crate::vari_eq::Filter::default();
    f.set(mode, (1.0 / sample_rate) * fast_exp2(p(freq) + 1.0), std::f32::consts::FRAC_1_SQRT_2, 0.0);
    crate::eq_display::curve(&[[f.b0, f.b1, f.b2, f.a1, f.a2]], sample_rate)
}
