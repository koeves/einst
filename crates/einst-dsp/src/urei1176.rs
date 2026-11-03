//! Einst: `VDSP::VCompressorUrei1176S` (Channel Compressor) as v1 builds it.
//! A copy of Dreist's port (`dreist_dsp::fx::nodes::urei1176`, Dreist bc729de2)
//! with v1's code where it differs (`docs/re/effects-v1.md`):
//!
//! - `Urei1176Comp::Solve1176A` 0x53ed… (ev_9b3b4723) and `SolveDRCR2`
//!   (ev_7029d2a1) transcribed from v1's decompile: the same circuit
//!   equations as Geist2's with the constants folded by v1's compiler
//!   (`12500 = 0.001·10000·1250`, `2.20125e10 = 10000·2.20125e6`,
//!   1/4.40125e6 when the FET is off, `vcrit = ln(1.8228e13)·0.02577878`).
//! - `Urei1176Comp::process` (ev_adb9af66): the input gain
//!   `10^((in − 10)·0.05)` is applied flat for the block; Geist2 ramps it
//!   per sample.
//! - `Voice::ProcessBuffer` (ev_d402f63f): attack and release take v1's
//!   fast 2^x (`setAttackTime` = x·23636362 = x·5.2/2.2e−7, ev_f7e912e2;
//!   `setReleaseTime` = x·4545454.5 = x/2.2e−7, ev_8307faad).

use dreist_dsp::fx::{Ctx, Node, Port, ports, ports_from};

/// v1's `Solve1176A(v1, v2, x, vctl)` (ev_9b3b4723).
pub fn solve_1176a(p1: &mut f32, p2: &mut f32, x: f32, vctl: f32) {
    let f11 = vctl * 1.75 * -10000.0;
    let mut it = 0;
    let mut f9 = *p2;
    let mut f10 = *p1;
    loop {
        let mut f8 = f9 - (f9 - f10);
        let (mut f3, f4, f5, l1c);
        if 0.0 < f8 {
            let g3 = f9 + 1.7372;
            if g3 < 0.0 {
                f8 = 2.2720818e-07;
                f4 = -0.0;
                l1c = -0.0f32;
                f3 = 1.0;
                f5 = 0.0;
            } else {
                let (f6, lc);
                if g3 < f8 {
                    let a = (f8 * 0.0177211 + 1.0) * g3;
                    f6 = a * 0.00233242;
                    let b = g3 * g3 * 2.0666525e-05;
                    lc = (g3 * 0.00116621 * a - f6 * f9) - f8 * b;
                    f3 = b * 10000.0;
                    f8 = 1.0 / ((f6 * 12500.0 + f3 + 1.0) * 2.2e+06 + b * 2.20125e+10 + 2.20125e+06);
                } else {
                    let b4 = f8 * 0.0177211 + 1.0;
                    let f7 = (g3 + g3) - f8;
                    f6 = f8 * 0.00233242 * b4;
                    let b5 = f8 * 2.0666525e-05 * f7 + (g3 - f8) * 0.00233242 * b4;
                    lc = (f8 * 0.00116621 * b4 * f7 - f6 * f9) - f8 * b5;
                    f3 = b5 * 10000.0;
                    f8 = 1.0 / ((f6 * 12500.0 + f3 + 1.0) * 2.2e+06 + b5 * 2.20125e+10 + 2.20125e+06);
                }
                f4 = lc * -4.40125e+10;
                l1c = lc * -10000.0;
                f5 = f6 * f11;
                f3 += 1.0;
            }
        } else {
            let g5 = (f9 - f10) + 1.7372;
            if g5 < 0.0 {
                f8 = 2.2720818e-07;
                l1c = -0.0;
                f5 = 0.0;
                f3 = 1.0;
                f4 = -0.0;
            } else if g5 < -f8 {
                let a = g5 * 0.00116621 * (f8 * 0.0177211 - 1.0);
                let f6 = a + a;
                let b = g5 * 0.00233242 * (f8 * -0.0177211 + 1.0) + g5 * g5 * 2.0666525e-05;
                let lc = (g5 * a - f6 * f9) - f8 * b;
                f3 = b * 10000.0;
                f8 = 1.0 / ((f6 * 12500.0 + f3 + 1.0) * 2.2e+06 + b * 2.20125e+10 + 2.20125e+06);
                f4 = lc * -4.40125e+10;
                l1c = lc * -10000.0;
                f5 = f6 * f11;
                f3 += 1.0;
            } else {
                let b4 = f8 * -0.0177211 + 1.0;
                let f6 = g5 + g5 + f8;
                let f7 = f8 * 0.00233242 * b4;
                let b3 = f8 * -2.0666525e-05 * f6 + g5 * 0.00233242 * b4;
                let lc = (f8 * 0.00116621 * b4 * f6 - f7 * f9) - f8 * b3;
                f8 = 1.0 / ((f7 * 12500.0 + b3 * 10000.0 + 1.0) * 2.2e+06 + b3 * 2.20125e+10 + 2.20125e+06);
                l1c = lc * -10000.0;
                f5 = f11 * f7;
                f3 = b3 * 10000.0 + 1.0;
                f4 = lc * -4.40125e+10;
            }
        }
        *p1 = (f4 + x * 2.2e+06 + (f5 + x) * 2.20125e+06) * f8;
        let n2 = ((l1c + x) * 2.75e+06 + f3 * vctl * 1.75 * 2.20125e+06) * f8;
        *p2 = n2;
        let n1 = *p1;
        let more = if (n1 - f10).abs() <= (n1 * 0.001).abs() { (n2 * 0.001).abs() < (n2 - f9).abs() } else { true };
        it += 1;
        if it == 10 || !more {
            return;
        }
        f9 = n2;
        f10 = n1;
    }
}

/// v1's `SolveDRCR2(v1, v2, v3, in_l, in_r, attack, release, c, scale, sr)` (ev_7029d2a1).
#[allow(clippy::too_many_arguments)]
pub fn solve_drcr2(p1: &mut f32, p2: &mut f32, p3: &mut f32, p4: f32, p5: f32, p6: f32, p7: f32, p8: f32, p9: f32, p10: f32) {
    // `vcrit` (a function static): ln(1.8228e13) · 0.02577878.
    let vcrit = f32::from_bits(0x5584_a0f3).ln() * 0.02577878;
    let f16 = p10 * p8;
    let f17 = -f16 * *p3;
    let f9 = p7 * f16 * p6;
    let f11 = p7 + p7 + p6 + f9;
    let f10 = f16 * p7 + 1.0;
    let mut it = 0;
    let mut l40 = *p3;
    let mut l3c = *p1;
    loop {
        let f1 = *p2;
        let f8 = ((vcrit + (p4 - l3c)) - ((p4 - l3c) - vcrit).abs()) * 0.5;
        let d = p5 - f1;
        let f7 = ((vcrit + d) - (d - vcrit).abs()) * 0.5;
        let e4 = (f8 * 38.791595).exp();
        let e5 = (f7 * 38.791595).exp();
        let f15 = e4 * 1e-15 * 38.791595;
        let f14 = e5 * 1e-15 * 38.791595;
        let f13 = (e4 * 1e-15 - 1e-15) - f8 * f15;
        let f4 = (e5 * 1e-15 - 1e-15) - f7 * f14;
        let f19 = p6 * f15 + 1.0;
        let f18 = p6 * f14;
        let g = (f18 + 1.0) * f19;
        let f12 = f14 * f15 * p6;
        let inv = 1.0 / ((f12 + f12 + f14 + f15 + f16 * g) * p7 + g);
        let f7b = (f11 * f14 + f10) * p6 + p7;
        let f8b = p5 * f14;
        *p1 = (p7 * f8b + p9 + p9 * f18 + (f17 * (-1.0 - f18) + f4) * p7 + p4 * f15 * f7b + f7b * f13) * inv;
        *p2 = ((f9 + p7 + p6) * f8b + p9 + (((f16 * p6 + 2.0) * p5 * f18 + p4) * p7 + (p5 * f18 + p9) * p6) * f15 + ((f11 * f15 + f10) * p6 + p7) * f4 + ((-1.0 - p6 * f15) * f17 + f13) * p7) * inv;
        let n3 = ((f12 + f14 + f15) * p6 * p9 + p9 + (f8b + f4 + (-1.0 - f18) * f19 * f17 + ((f4 + f14 * (p5 + p4)) * p6 + p4) * f15 + f13 + f13 * f18) * p7) * inv;
        *p3 = n3;
        let n1 = *p1;
        let more = (n1 * 0.001).abs() < (n1 - l3c).abs() || (*p2 * 0.001).abs() < (*p2 - f1).abs() || (n3 * 0.001).abs() < (n3 - l40).abs();
        if it == 9 || !more {
            return;
        }
        it += 1;
        l40 = n3;
        l3c = n1;
    }
}

/// `Urei1176Comp` state.
struct Comp {
    inv_sr: f32,
    alt: f32,
    ratio: u32,
    attack: f32,
    release: f32,
    q: f32,
    meter: [f32; 3],
    in_db: f32,
    out_db: f32,
    mix: f32,
    hp: [f32; 2],
    fet: [[f32; 2]; 2],
    det: [f32; 3],
}

impl Comp {
    fn new() -> Comp {
        Comp {
            inv_sr: f32::from_bits(0x37be37c6),
            alt: 1e-10,
            ratio: 0,
            attack: 8200.0,
            release: 560000.0,
            q: 0.5,
            meter: [1.0; 3],
            in_db: 0.0,
            out_db: 1.0,
            mix: 1.0,
            hp: [0.0; 2],
            fet: [[0.0; 2]; 2],
            det: [-1.6452; 3],
        }
    }

    /// `process` 0x4e5ca0.
    fn process(&mut self, sr: f32, inp: [&[f32]; 2], out: [&mut [f32]; 2]) {
        self.inv_sr = 1.0 / sr;
        let n = inp[0].len();
        // v1: both gains as 10^(x·0.05), the input's flat for the block.
        let in_g = 10f32.powf((self.in_db - 10.0) * 0.05);
        let mut out_g = 10f32.powf((self.out_db - 5.0) * 0.05);
        let (mut off, mut scale, mut gain) = (0.0f32, 0.0f32, 0.0f32);
        if self.ratio < 5 {
            let r1 = (1.0 - self.q) * 2000.0;
            let r2 = self.q * 2000.0;
            let a = r2 * 13900.0;
            let f8 = (a + 3.9e7) * 150.0;
            let f3 = (a + 585000.0 + 4.05e7) * r1;
            let s = r2 + r1;
            let t = s + 10000.0;
            let (den, num_s, num_o);
            match self.ratio {
                0 => {
                    den = (t * 47000.0 * 3900.0 + s * 47000.0 * 10000.0 + f8 + f3) * 4730.0 + (f8 + f3) * 47000.0;
                    num_s = ((a + 5.8032e7) * 47000.0 + 2.76705e9) * r1 * -10.0;
                    num_o = ((a + 5.7447e7 + 2.085e6) * r1 + f8) * 47000.0 * -10.0;
                    gain = 0.16607773;
                }
                1 => {
                    den = (t * 47000.0 * 3900.0 + s * 47000.0 * 10000.0 + f8 + f3) * 4260.0 + (f8 + f3) * 47000.0 + (t * 3900.0 + s * 10000.0) * 2.40922e7;
                    num_s = ((a + 5.8032e7) * 47000.0 + 1.030068e10) * r1 * -10.0;
                    num_o = ((a + 5.5614e7 + 2.085e6) * r1 + t * 470.0 * 3900.0 + s * 470.0 * 10000.0 + f8) * 47000.0 * -10.0;
                    gain = 0.36395758;
                }
                2 => {
                    den = (t * 47000.0 * 3900.0 + s * 47000.0 * 10000.0 + f8 + f3) * 3700.0 + (f8 + f3) * 47000.0 + (t * 3900.0 + s * 10000.0) * 5.2221e7;
                    num_s = ((a + 5.8032e7) * 47000.0 + 1.70274e10) * r1 * -10.0;
                    num_o = ((a + 5.343e7 + 2.085e6) * r1 + t * 1030.0 * 3900.0 + s * 1030.0 * 10000.0 + f8) * 47000.0 * -10.0;
                    gain = 0.56183743;
                }
                3 => {
                    den = (t * 47000.0 * 3900.0 + s * 47000.0 * 10000.0 + f8 + f3) * 2200.0 + (f8 + f3) * 47000.0 + (t * 3900.0 + s * 10000.0) * 1.24476e8;
                    num_s = ((a + 5.8032e7) * 47000.0 + 2.29944e10) * r1 * -10.0;
                    num_o = ((a + 4.758e7 + 2.085e6) * r1 + t * 2530.0 * 3900.0 + s * 2530.0 * 10000.0 + f8) * 47000.0 * -10.0;
                    gain = 0.80212015;
                }
                _ => {
                    den = (t * 47000.0 * 3900.0 + s * 47000.0 * 10000.0 + f8 + f3) * 2200.0 + (f8 + f3) * 47000.0;
                    num_s = ((a + 4.8165e7) * 47000.0 + 1.287e9) * r1 * -10.0;
                    num_o = ((a + 4.758e7 + 2.085e6) * r1 + f8) * 47000.0 * -10.0;
                    gain = 0.4563107;
                }
            }
            scale = num_s / den;
            off = num_o / den;
        }
        let k_hp = (self.inv_sr * -62.831856).exp();
        let d = self.alt;
        self.alt = -d;
        self.det[2] -= d;
        self.hp[0] -= d;
        self.hp[1] -= d;
        self.meter[0] -= d;
        self.meter[1] -= d;
        let mut peak = -1e10f32;
        let [ol, or] = out;
        if n > 0 {
            let rel = self.release * 0.5;
            out_g *= self.mix;
            let dry = 1.0 - self.mix;
            for i in 0..n {
                let vctl = self.det[2];
                let g = in_g;
                let (xl, xr) = (inp[0][i], inp[1][i]);
                let [fl, fr] = &mut self.fet;
                let [a, b] = fl;
                solve_1176a(a, b, xl * g, vctl);
                let [a, b] = fr;
                solve_1176a(a, b, g * xr, vctl);
                let hl = fl[0] - self.hp[0];
                let hr = fr[0] - self.hp[1];
                self.hp[0] += (1.0 - k_hp) * hl;
                self.hp[1] += (1.0 - k_hp) * hr;
                let sl = hl * 21.0;
                let sr_ = hr * 21.0;
                let al = (8.912509 * sl * gain).abs();
                let ar = (8.912509 * sr_ * gain).abs();
                let [d1, d2, d3] = &mut self.det;
                solve_drcr2(d1, d2, d3, ((al + 10.0) - (al - 10.0).abs()) * 0.5 + off, ((ar + 10.0) - (ar - 10.0).abs()) * 0.5 + off, self.attack, rel, 4.4e-7, scale, sr);
                ol[i] = xl * dry + sl * out_g;
                or[i] = xr * dry + sr_ * out_g;
                peak = if self.det[2] <= peak { peak } else { self.det[2] };
            }
        }
        let k = (self.inv_sr * -31.415928 * n as f32).exp();
        let m1 = ((1.0 - (scale - peak) / scale) - self.meter[0]) * (1.0 - k) + self.meter[0];
        self.meter[0] = m1;
        self.meter[1] += (m1 - self.meter[1]) * (1.0 - k);
        let m = self.meter[1];
        self.meter[2] = ((m.abs() + 1.0) - (m - 1.0).abs()) * 0.5;
    }
}

pub struct Urei1176 {
    ports: Vec<Port>,
    comp: Comp,
}

impl Urei1176 {
    pub fn new() -> Urei1176 {
        let mut ports = ports_from(ports::VCompressorUrei1176S);
        // Ratio: int ctor over `ConvertRatioMode` (5 items).
        ports[5] = Port::new(0.0, 0.0, 4.0);
        Urei1176 { ports, comp: Comp::new() }
    }

    /// Gain-reduction meter (`ControlOut`, Voice + 0x17c).
    pub fn meter(&self) -> f32 {
        self.comp.meter[2]
    }
}

impl Default for Urei1176 {
    fn default() -> Self {
        Urei1176::new()
    }
}

impl Node for Urei1176 {
    fn ports(&mut self) -> &mut Vec<Port> {
        &mut self.ports
    }

    fn process(&mut self, ctx: &Ctx, inp: [&[f32]; 2], out: [&mut [f32]; 2]) {
        let p = |i: usize| self.ports[i].value;
        let c = &mut self.comp;
        c.in_db = p(0);
        c.out_db = p(6);
        c.attack = crate::fast_exp2(p(1) - 1.0) * 23636362.0;
        c.release = crate::fast_exp2(p(2) - 1.0) * 4545454.5;
        let q = p(3);
        c.q = if q < 0.0001 { 0.0001 } else { q.min(0.9999) };
        let r = p(5);
        c.ratio = (((r.abs() + 4.0) - (r - 4.0).abs()) * 0.5).round() as u32;
        let m = p(7);
        c.mix = ((m.abs() + 1.0) - (m - 1.0).abs()) * 0.5;
        if p(8) >= 0.5 {
            c.process(ctx.sample_rate, inp, out);
        } else {
            let [ol, or] = out;
            ol.copy_from_slice(inp[0]);
            or.copy_from_slice(inp[1]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compresses_loud_signal() {
        let sr = 48000.0;
        let ctx = Ctx::new(sr);
        let run = |amp: f32| {
            let mut u = Urei1176::new();
            u.ports[0].value = 20.0;
            let n = 256;
            let mut last = 0.0f32;
            for b in 0..40 {
                let x: Vec<f32> = (0..n).map(|i| ((b * n + i) as f32 * std::f32::consts::TAU * 200.0 / sr).sin() * amp).collect();
                let (mut l, mut r) = (vec![0.0; n], vec![0.0; n]);
                u.process(&ctx, [&x, &x], [&mut l, &mut r]);
                assert!(l.iter().all(|v| v.is_finite()));
                last = l.iter().fold(0.0f32, |m, v| m.max(v.abs()));
            }
            last
        };
        let (quiet, loud) = (run(0.01), run(0.5));
        // 34 dB more input should give much less than 34 dB more output.
        assert!(loud / quiet < 20.0, "quiet {quiet} loud {loud}");
        assert!(loud > quiet);
    }
}
