//! The original v1's Phaser: Dreist's node (Geist2's, [re:297af4]) as v1
//! builds it, from v1's `Voice::Reset` (ev_14ca6d2e) and `ProcessBuffer`
//! (0x52c0a0), transcribed and render-compared sample by sample
//! (`tests/v1_phaser_transcribed.rs`):
//!
//! - v1 keeps `2/sr` in a field that `Reset` zeroes and re-reads it after
//!   the reset a mode change triggers: on that block (and the first, since
//!   Reset sets mode 0) the frequency ramps toward 0, reaching the pitch
//!   only on the next block;
//! - the pitch is `(2/sr · tune) · 2^x`;
//! - the LFO table read adds the 0.5 to the first entry before the slope;
//! - the swept coefficient's argument is `((f − prev) + c) + c·f`;
//! - the 1/√g Newton step is `y0·1.5 + g·y0·y0·y0·−0.5`;
//! - the dry ramp target is `(1 − mix)·sgn·rs·rs·5`;
//! - Mode rounds by adding 0.5 and truncating.

use crate::math::{clip01, sat};
use dreist_dsp::analog::sin_tab;
use dreist_dsp::fx::{Ctx, Node, Port, ports, ports_from};

/// The alternating anti-denormal offset's bits (as Dreist's nodes).
const ALT: u32 = 0x322bcc77;

/// Per-mode coefficient polynomials `(((a·v + b)·v + c)·v + d)·v − 1`.
const POLY: [[f32; 4]; 8] = [
    [-0.49724257, -0.0028277636, 2.4676685, -7.705447e-06],
    [0.09575251, -1.3119804, 0.001682329, 3.1415472],
    [-2.5111454, 3.0190501, -2.4265928, 3.14055],
    [-42.52696, 39.453728, -18.940275, 7.5505123],
    [-20.616514, 20.549196, -10.952777, 5.427524],
    [-166.76474, 132.36517, -47.11987, 11.511031],
    [-58.97007, 52.01872, -22.277859, 7.530122],
    [-354.18213, 259.99634, -80.07754, 15.178016],
];

#[derive(Clone)]
struct Voice {
    alt: f32,
    f: f32,
    fb_sign: f32,
    dry: f32,
    wet: f32,
    x1: [f32; 8],
    y: [f32; 8],
    fb: f32,
    dc: f32,
    mode: u32,
    phase: f32,
    lfo: f32,
    spare: [f32; 2],
}

impl Voice {
    fn new() -> Voice {
        let mut v = Voice { alt: f32::from_bits(ALT), f: 0.0, fb_sign: 0.0, dry: 0.0, wet: 0.0, x1: [0.0; 8], y: [0.0; 8], fb: 0.0, dc: 0.0, mode: 0, phase: 0.0, lfo: 0.0, spare: [0.0; 2] };
        v.reset();
        v
    }

    fn reset(&mut self) {
        self.x1 = [0.0; 8];
        self.y = [0.0; 8];
        self.x1[0] = f32::from_bits(0x2edbe6ff);
        self.spare = [0.0; 2];
        self.fb = 0.0;
        self.dc = 0.0;
        self.lfo = 0.0;
        self.f = 0.1;
        self.fb_sign = 0.0;
        self.dry = 0.5;
        self.wet = 0.5;
        self.mode = 0;
        self.phase = 0.0;
    }

    #[allow(clippy::too_many_arguments)]
    fn process(&mut self, ctx: &Ctx, p: &[f32], x: &[f32], out: &mut [f32]) {
        let isr2 = (1.0 / ctx.sample_rate) * 2.0;
        let d3 = p[0] * p[0] * p[0];
        let drive = (d3.abs() + d3) * 0.5;
        let mode = (((p[1].abs() + 7.0) - (p[1] + -7.0).abs()) * 0.5 + 0.5) as u32;
        let fb = ((p[3].abs() + p[3]) * 0.5).sqrt().sqrt();
        let mix = clip01(p[6]);
        let g = fb.abs() + 1.0;
        let y0 = f32::from_bits(0xbe6a3000u32.wrapping_sub(g.to_bits()) >> 1);
        let rs = y0 * 1.5 + g * y0 * y0 * y0 * -0.5;
        let n = x.len();
        let inv_n = 1.0 / n.max(1) as f32;
        // The ramp steps use the gains read before a mode-change reset.
        let (dry0, wet0) = (self.dry, self.wet);
        let changed = self.mode != mode;
        if changed {
            self.reset();
        }
        let drive = drive * 0.2;
        let sgn = ((g.to_bits() as i32 >> 30) | 1) as f32;
        let d_dry = ((1.0 - mix) * sgn * rs * rs * 5.0 - dry0) * inv_n;
        self.mode = mode;
        let d_wet = (mix * 5.0 - wet0) * inv_n;
        // v1 re-reads 2/sr from the field the reset zeroed.
        let ft = if changed { 0.0 } else { isr2 * ctx.tune * dreist_dsp::fx::exp2(p[2]) };
        let a = self.alt;
        self.alt = -a;
        self.x1[0] -= a;
        self.spare[0] -= a;
        self.x1[3] -= a;
        self.y[2] -= a;
        self.x1[7] -= a;
        self.y[6] -= a;
        let df = (((ft + 0.16666667).abs() - (ft + -0.16666667).abs()) * 0.5 - self.f) * inv_n;
        if mode >= 8 {
            return;
        }
        let depth = clip01(p[5]);
        let rate = clip01(((p[4].abs() + 5.0) - (p[4] + -5.0).abs()) * 0.5 * (1.0 / ctx.sample_rate));
        let stages = (mode as usize / 2 + 1) * 2;
        let odd = mode & 1 == 1;
        self.fb_sign = if odd { fb } else { -fb };
        let [c0, c1, c2, c3] = POLY[mode as usize];
        let t = sin_tab();
        for i in 0..n {
            self.phase = (self.phase + rate) % 1.0;
            let xk = self.phase * 2048.0;
            let k = xk.floor() as i32;
            let j = (1536 + k) as usize;
            self.f += df;
            let dry_old = self.dry;
            self.dry += d_dry;
            let wet_old = self.wet;
            self.wet += d_wet;
            let prev = self.lfo;
            let c = (t[j] + 0.5 + (t[j + 1] - t[j]) * (xk - k as f32)) * depth;
            self.lfo = c;
            let v = (self.f - prev) + c + c * self.f;
            let coef = (((v * c0 + c1) * v + c2) * v + c3) * v + -1.0;
            let s = sat(x[i] * drive);
            let u = sat(s - self.fb_sign * self.fb);
            let coef = ((coef + 1.0).abs() - (coef + -1.0).abs()) * 0.5;
            self.spare[1] = u;
            let mut inp = u;
            for st in 0..stages {
                let y = (inp - self.y[st]) * coef + self.x1[st];
                self.y[st] = y;
                self.x1[st] = inp;
                inp = y;
            }
            self.fb = inp - self.dc;
            self.dc = self.f * 0.1 * self.fb + self.dc;
            let dry = dry_old + d_dry;
            let wet = wet_old + d_wet;
            out[i] = if odd { s * dry - wet * inp } else { s * dry + wet * inp };
        }
    }
}

pub struct Phaser {
    ports: Vec<Port>,
    voices: [Voice; 2],
}

impl Phaser {
    pub fn new() -> Phaser {
        let mut ports = ports_from(ports::PHASER);
        // Float-form enum: max = item count (8).
        ports[1] = Port::new(5.0, 0.0, 8.0);
        Phaser { ports, voices: [Voice::new(), Voice::new()] }
    }
}

impl Default for Phaser {
    fn default() -> Self {
        Phaser::new()
    }
}

impl Node for Phaser {
    fn ports(&mut self) -> &mut Vec<Port> {
        &mut self.ports
    }

    fn process(&mut self, ctx: &Ctx, inp: [&[f32]; 2], out: [&mut [f32]; 2]) {
        let p = dreist_dsp::fx::port_values(&self.ports);
        for (k, o) in out.into_iter().enumerate() {
            self.voices[k].process(ctx, &p, inp[k], o);
        }
    }

    fn reset(&mut self) {
        for v in &mut self.voices {
            v.reset();
        }
    }
}
