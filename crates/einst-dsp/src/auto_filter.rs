//! Einst: Dreist's port of `VDSP::VFilterAutoS` (`dreist_dsp::fx::nodes::auto_filter`,
//! copied from Dreist at bc729de) with v1's differences (`docs/re/effects-v1.md`):
//!
//! - v1's `Voice::ProcessBuffer` 0x553a10 (ev_9fed2d28) takes the attack
//!   and release times, 2^(Attack − 1) and 2^(Release − 1), and the cutoff
//!   2^(clamp(pitch + envelope, ±10)) from v1's [`fast_exp2`](crate::fast_exp2)
//!   where Geist2 calls `exp2f`.
//! - The FM term is computed once per sample from the left side-chain
//!   input and used for both channels, as in v1 and in Geist2's
//!   `ProcessBuffer` 0x4df120 (one `exp2f` call and one 0.24 clamp per
//!   sample in each mode loop, from one buffer); Dreist's port takes it per
//!   channel.
//!
//! The rest matches, with GCC's folded constants: the attack time constant
//! −2127659.5/(t·8199999.5) = −1/((t/0.001)·8200·4.7e−7), the release
//! resistor t·1.8e6 = (t/0.1)·180000, the input drive d³·3.3333333 = d³/0.3,
//! the coefficient g²·k·0.16666667 = k·g²/6, the sine table read at
//! clamp(w)·1024 from `ms_sin_tab + 0x1000` with the clamp's halving folded
//! in; the diode solve (3 Newton steps from the previous currents,
//! 0.0475·ln 5.9e−9 at run time), the state-variable stage
//! `Process121NonlinSem2` 0x557410 (ev_2cbcc7ed, out of line in v1) and
//! `Voice::Reset` 0x553580 (ev_51bf7baa) are Geist2's.
//!
//! `VDSP::VFilterAutoS` (Modulated Filter; Geist2 `InitializePorts` 0x4ddf20,
//! `Voice::Reset` 0x4defd0, `PrepareStream` 0x4df080, `ProcessBuffer`
//! 0x4df120): Chamberlin state-variable filter with a polynomial soft
//! clipper and diode feedback, cutoff modulated by a diode/RC envelope
//! follower (Newton-solved per sample) and optional FM from the sidechain.
//! Modes: Low, Band, High, Peak, Notch.

use crate::fast_exp2;
use dreist_dsp::analog::{analog, lerp, lut_pos, sin_tab};
use dreist_dsp::fx::{Ctx, Node, Port, ports, ports_from};

/// The node's saturator: `((|c|·0.0016875 − 0.0225)·c² + 1)·c`, c clamped to ±6.6667.
#[inline]
fn sat(x: f32) -> f32 {
    let c = ((x + 6.6666665).abs() - (x - 6.6666665).abs()) * 0.5;
    ((c.abs() * 0.0016875 + -0.0225) * c * c + 1.0) * c
}

pub struct AutoFilter {
    ports: Vec<Port>,
    sr: f32,
    env: Vec<f32>,
    c_dc: f32,
    c_hp: f32,
    lp: [f32; 2],
    bp: [f32; 2],
    dc: [f32; 2],
    hp_in: [f32; 2],
    diode_i: [f32; 2],
    cap: f32,
    smooth: f32,
    drive: f32,
    out_drive: f32,
    pitch: f32,
    wet: f32,
    alt: f32,
    /// Envelope meter (control-out).
    pub meter: f32,
}

impl AutoFilter {
    pub fn new() -> AutoFilter {
        let mut ports = ports_from(ports::VFilterAutoS);
        ports[1].max = 4.0;
        AutoFilter { ports, env: Vec::with_capacity(256), ..AutoFilter::state() }
    }

    /// The running state as built (no ports or buffer).
    fn state() -> AutoFilter {
        AutoFilter {
            ports: Vec::new(),
            sr: 0.0,
            env: Vec::new(),
            c_dc: 0.0,
            c_hp: 0.0,
            lp: [1e-8, 1e-8],
            bp: [0.0, 0.0],
            dc: [0.0; 2],
            hp_in: [0.0; 2],
            diode_i: [1e-12, 1e-12],
            cap: 0.0,
            smooth: 0.0,
            drive: 0.0,
            out_drive: 0.0,
            // Voice::Reset leaves the pitch state at 1.0.
            pitch: 1.0,
            wet: 1.0,
            alt: dreist_dsp::fx::nodes::ssl_filter::DENORMAL,
            meter: 0.0,
        }
    }
}

impl Default for AutoFilter {
    fn default() -> Self {
        AutoFilter::new()
    }
}

impl Node for AutoFilter {
    fn ports(&mut self) -> &mut Vec<Port> {
        &mut self.ports
    }

    /// `Voice::Reset` 0x4defd0 (`state` is its field values); the rate
    /// coefficients and the voice's denormal offset are kept.
    fn reset(&mut self) {
        let mut env = std::mem::take(&mut self.env);
        env.fill(0.0);
        let fresh = AutoFilter { ports: std::mem::take(&mut self.ports), env, ..AutoFilter::state() };
        let (sr, c_dc, c_hp, alt) = (self.sr, self.c_dc, self.c_hp, self.alt);
        *self = AutoFilter { sr, c_dc, c_hp, alt, ..fresh };
    }

    fn process(&mut self, ctx: &Ctx, inp: [&[f32]; 2], out: [&mut [f32]; 2]) {
        let n = inp[0].len();
        if self.sr != ctx.sample_rate {
            // PrepareStream: 10 Hz DC blocker and 2.4 Hz envelope high-pass.
            self.sr = ctx.sample_rate;
            let inv = 1.0 / self.sr;
            self.c_dc = 1.0 - (inv * -62.831856).exp();
            self.c_hp = 1.0 - (inv * -15.079646).exp();
        }
        let p = |i: usize| self.ports[i].value;
        let drive_in = (p(0).abs() + p(0)) * 0.5;
        let drive_out = (p(4).abs() + p(4)) * 0.5;
        let mode = (((p(1).abs() + 4.0) - (p(1) - 4.0).abs()) * 0.5).round() as u32;
        let (pitch_t, res, fm, depth) = (p(2), p(3), p(11), p(10));
        let attack = fast_exp2(p(8) - 1.0);
        let release = fast_exp2(p(9) - 1.0);
        let wet_t = p(5);
        let inv_n = 1.0 / n as f32;
        let inv_sr = 1.0 / self.sr;
        let [ol, or] = out;
        // Anti-denormal offset flips every block.
        let a = self.alt;
        self.alt = -a;
        for c in 0..2 {
            self.lp[c] -= a;
            self.bp[c] -= a;
            self.dc[c] -= a;
        }
        self.smooth -= a;
        self.cap -= a;
        let k_att = ((-1.0 / ((attack / 0.001) * 8200.0 * 4.7e-07)) * inv_sr).exp();
        if p(6) < 0.5 {
            ol.copy_from_slice(inp[0]);
            or.copy_from_slice(inp[1]);
            self.meter = 0.0;
            return;
        }
        let w0 = ctx.tune * inv_sr;
        self.env.clear();
        self.env.resize(n, 0.0);
        let depth10 = depth * 10.0;
        let mut meter = 0.0f32;
        let mut v = self.cap;
        for i in 0..n {
            let mut drv = [0.0f32; 2];
            for c in 0..2 {
                let e = inp[c][i] - self.hp_in[c];
                self.hp_in[c] += e * self.c_hp;
                let x = (e * depth10 + e * depth10).abs();
                drv[c] = sat(x.min(6.6666665)) - v;
            }
            let mut cur = [0.0f32; 2];
            if drv[0] > 0.0 || drv[1] > 0.0 {
                cur = [self.diode_i[0].max(0.0), self.diode_i[1].max(0.0)];
                for _ in 0..3 {
                    let s = (cur[0] + cur[1]) * 820.0;
                    let next: [f32; 2] = std::array::from_fn(|c| {
                        let f = ((cur[c] + 5.9e-9).ln() * 0.0475 + s + 0.900044858) - drv[c];
                        let d = 0.0475 / (cur[c] + 5.9e-9) + 820.0;
                        (cur[c] - f / d).max(0.0)
                    });
                    let moved = (next[0] * 0.001 <= (next[0] - cur[0]).abs()) || (next[1] * 0.001 <= (next[1] - cur[1]).abs());
                    cur = next;
                    if !moved {
                        break;
                    }
                }
            }
            self.diode_i = cur;
            let leak = v / ((release / 0.1) * 180000.0) + v / 3_300_000.0;
            v += ((cur[1] + cur[0] - leak) / 4.7e-07) * inv_sr;
            self.smooth = (v - self.smooth) * (1.0 - k_att) + self.smooth;
            let env = ((self.smooth + 1.0).log2() - 1.0) * depth10;
            if meter.abs() < env.abs() {
                meter = env;
            }
            self.env[i] = env;
        }
        self.cap = v;
        self.meter = meter;
        if mode > 4 {
            return;
        }
        let fm5 = {
            let f = (fm.abs() + fm) * 0.5;
            f * f * 5.0
        };
        let r = ((res.abs() + res) * 0.5).sqrt();
        let k = 2.0 - (r + r);
        let d_drive = ((drive_in * drive_in * drive_in) / 0.3 - self.drive) * inv_n;
        let d_out = (drive_out * drive_out * drive_out * 0.3 - self.out_drive) * inv_n;
        let d_wet = (((wet_t.abs() + 1.0) - (wet_t - 1.0).abs()) * 0.5 - self.wet) * inv_n;
        let d_pitch = (pitch_t - self.pitch) * inv_n;
        let st = sin_tab();
        let dl = &analog().diodelim;
        for i in 0..n {
            self.wet += d_wet;
            self.drive += d_drive;
            self.out_drive += d_out;
            self.pitch += d_pitch;
            let pc = self.pitch + self.env[i];
            let f = ((pc + 10.0).abs() - (pc - 10.0).abs()) * 0.5;
            let f = fast_exp2(f);
            // One coefficient for both channels, FM from the left input.
            let w = (fm5 * inp[0][i] * f * w0 + f * w0) * 0.5;
            let wc = ((w.abs() + 0.24) - (w - 0.24).abs()) * 0.5;
            // `ms_sin_tab + 0x1000` indexed by `floor(wc·2048)` directly
            // (no table-centre offset; wc is clamped to 0..0.24).
            let v = wc * 2048.0;
            let iv = v.floor();
            let j = 1024 + iv as usize;
            let g = ((st[j + 1] - st[j]) * (v - iv) + st[j]) * 4.0;
            let t = (k * g * g) / 6.0;
            let (kd, gg) = (k - t, g - t);
            let mut y = [0.0f32; 2];
            for c in 0..2 {
                let x = inp[c][i];
                let bp = self.bp[c];
                self.lp[c] += sat(bp) * gg;
                let lp = self.lp[c];
                let hp = lerp(dl, lut_pos(bp * 32.0)) + (((x * self.drive - lp) - bp * kd) - bp);
                let hs = sat(hp);
                self.bp[c] = hs * gg + bp;
                let o = match mode {
                    0 => sat(lp),
                    1 => sat(self.bp[c]),
                    2 => hs,
                    3 => sat(lp - hp),
                    _ => sat(hp + lp),
                } * self.out_drive
                    - self.dc[c];
                self.dc[c] += self.c_dc * o;
                y[c] = o;
            }
            ol[i] = (1.0 - self.wet) * inp[0][i] + y[0] * self.wet;
            or[i] = (1.0 - self.wet) * inp[1][i] + y[1] * self.wet;
        }
    }
}
