//! `VDSP::VFilterEQ2` (Vari-EQ): `InitializePorts` 0x520370 (ev_6e577663;
//! Mode 0..8, Freq −4..4.5 frequency-pitch, Q 0.5..4 double-log, Gain
//! ±15 dB: the same table as Geist2's `VFilterEQ2S`), `CreateVoice` 0x51fe80
//! (ev_d61d4f58), `Voice::Reset` 0x51f920 (ev_9a6184d3),
//! `Voice::ProcessBuffer` 0x51fab0 (ev_b33db815) and v1's
//! `VFilterEQCommon::SetCoeffs` 0x521520 (ev_a7794e4b; the per-mode cases
//! from the disassembly of its jump table at 0x521664).
//!
//! Differences from Dreist's port of Geist2's `VFilterEQ2S`: the frequency
//! comes from v1's fast 2^x; the normalised frequency is clamped to
//! 0.001..0.45 (Geist2: 1e−5..0.45); the shelf and bell gain factor is
//! `powf(10, g·0.025)` (the bell's in double precision); every sample feeds
//! the x history an alternating ±1e−8 offset against denormals.

use dreist_dsp::fx::{Ctx, Node, Port, ports, ports_from};

/// `VFilterEQ2::ConvertMode`: Bell, Low Shelf, High Shelf, LPF, BPF, BPF 0dB, HPF, Notch, Allpass.
pub const MODES: usize = 9;

/// One voice: `VFilterEQCommon` (coefficients, key, history) and the
/// voice's anti-denormal term (`Voice + 0x1b8`, 1e−8 at `CreateVoice`).
#[derive(Debug, Clone, Copy)]
pub struct Filter {
    pub b0: f64,
    pub b1: f64,
    pub b2: f64,
    pub a1: f64,
    pub a2: f64,
    x1: f64,
    x2: f64,
    y1: f64,
    y2: f64,
    /// `(mode, f, q, gain)`; mode −1 = none yet.
    key: (i32, f32, f32, f32),
    denormal: f32,
}

impl Default for Filter {
    fn default() -> Filter {
        Filter { b0: 0.0, b1: 0.0, b2: 0.0, a1: 0.0, a2: 0.0, x1: 0.0, x2: 0.0, y1: 0.0, y2: 0.0, key: (-1, 0.0, 0.0, 0.0), denormal: f32::from_bits(0x322b_cc77) }
    }
}

impl Filter {
    /// `Voice::Reset`: key, coefficients and history; the denormal term stays.
    pub fn reset(&mut self) {
        *self = Filter { denormal: self.denormal, ..Filter::default() };
    }

    /// v1's `VFilterEQCommon::SetCoeffs(mode, f/fs, Q, gain dB)`.
    pub fn set(&mut self, mode: i32, f: f32, q: f32, gain: f32) {
        if self.key.0 == mode {
            if f == self.key.1 && q == self.key.2 && gain == self.key.3 {
                return;
            }
        } else {
            // A mode change clears the coefficients and the history.
            self.reset();
        }
        self.key = (mode, f, q, gain);
        let fd = f as f64;
        let w = (((fd - 0.0010000000474974513).abs() + 0.4509999881265685) - (fd - 0.44999998807907104).abs()) * std::f64::consts::PI;
        let qd = q as f64;
        let q = (((qd - 0.5).abs() + 1000.5) - (qd - 1000.0).abs()) * 0.5;
        let sn = w.sin();
        let cs = w.cos();
        if !(0..=8).contains(&mode) {
            return;
        }
        let g = gain * 0.025;
        match mode {
            // Bell: A in double precision.
            0 => {
                let al = sn / (q + q);
                let a = 10f64.powf(g as f64);
                let d = al / a;
                let n = 1.0 / (d + 1.0);
                let au = al * a;
                self.b0 = (au + 1.0) * n;
                self.b1 = cs * -2.0 * n;
                self.b2 = (1.0 - au) * n;
                self.a1 = self.b1;
                self.a2 = (1.0 - d) * n;
            }
            // Low shelf.
            1 => {
                let a = 10f32.powf(g) as f64;
                let am = a - 1.0;
                let s = a.sqrt() / q * sn;
                let ap = a + 1.0;
                let d7 = am * cs + ap;
                let n = 1.0 / (s + d7);
                let d9 = (1.0 - a) * cs + ap;
                self.b0 = (s + d9) * a * n;
                self.b1 = (a + a) * ((-1.0 - a) * cs + am) * n;
                self.b2 = (d9 - s) * a * n;
                self.a1 = (cs * ap + am) * -2.0 * n;
                self.a2 = (d7 - s) * n;
            }
            // High shelf.
            2 => {
                let a = 10f32.powf(g) as f64;
                let s = a.sqrt() / q * sn;
                let ap = a + 1.0;
                let d7 = (1.0 - a) * cs + ap;
                let n = 1.0 / (s + d7);
                let am = a - 1.0;
                let d9 = am * cs + ap;
                self.b0 = (s + d9) * a * n;
                self.b1 = a * -2.0 * (ap * cs + am) * n;
                self.b2 = (d9 - s) * a * n;
                let t = (-1.0 - a) * cs + am;
                self.a1 = (t + t) * n;
                self.a2 = (d7 - s) * n;
            }
            // LPF / HPF share the tail.
            3 | 6 => {
                let al = sn / (q + q);
                let n = 1.0 / (al + 1.0);
                if mode == 3 {
                    let k = (1.0 - cs) * n;
                    self.b0 = 0.5 * k;
                    self.b1 = k;
                    self.b2 = 0.5 * k;
                } else {
                    let k = (cs + 1.0) * n;
                    self.b0 = 0.5 * k;
                    self.b1 = -k;
                    self.b2 = 0.5 * k;
                }
                self.a1 = cs * -2.0 * n;
                self.a2 = (1.0 - al) * n;
            }
            // BPF.
            4 => {
                let al = sn / (q + q);
                let n = 1.0 / (al + 1.0);
                let t = sn * n;
                self.b0 = 0.5 * t;
                self.b1 = 0.0;
                self.b2 = t * -0.5;
                self.a1 = cs * -2.0 * n;
                self.a2 = (1.0 - al) * n;
            }
            // BPF 0 dB.
            5 => {
                let al = sn / (q + q);
                let n = 1.0 / (al + 1.0);
                self.b0 = n * al;
                self.b1 = 0.0;
                self.b2 = -al * n;
                self.a1 = cs * -2.0 * n;
                self.a2 = (1.0 - al) * n;
            }
            // Notch.
            7 => {
                let al = sn / (q + q);
                let n = 1.0 / (al + 1.0);
                self.b0 = n;
                self.b1 = cs * -2.0 * n;
                self.b2 = n;
                self.a1 = self.b1;
                self.a2 = (1.0 - al) * n;
            }
            // Allpass.
            _ => {
                let al = sn / (q + q);
                let d = al + 1.0;
                let n = 1.0 / d;
                self.b0 = (1.0 - al) * n;
                self.b1 = cs * -2.0 * n;
                self.b2 = n * d;
                self.a1 = self.b1;
                self.a2 = self.b0;
            }
        }
    }

    /// `Voice::ProcessBuffer`'s per-sample step.
    #[inline]
    pub fn tick(&mut self, x: f32) -> f32 {
        let y = (((self.x2 * self.b2 + self.x1 * self.b1) - self.y1 * self.a1) - self.y2 * self.a2) + x as f64 * self.b0;
        self.x2 = self.x1;
        self.y2 = self.y1;
        let d = self.denormal;
        self.denormal = -d;
        self.x1 = (-d + x) as f64;
        self.y1 = y;
        y as f32
    }
}

pub struct VariEq {
    ports: Vec<Port>,
    /// One voice per channel (`VDSPNodeAdaptor::Process22`).
    f: [Filter; 2],
}

impl VariEq {
    pub fn new() -> VariEq {
        let mut ports = ports_from(ports::VFilterEQ2S);
        ports[0].max = (MODES - 1) as f64;
        VariEq { ports, f: [Filter::default(); 2] }
    }

    /// The block's `SetCoeffs` arguments from the port values:
    /// mode `trunc(clamp(m, 0, 8) + 0.5)`, f/fs = 2^(Freq + 1)·400/fs.
    pub fn coeff_args(ports: &[Port], sample_rate: f32) -> (i32, f32, f32, f32) {
        let m = ports[0].value;
        let mode = (((m.abs() + 8.0) - (m - 8.0).abs()) * 0.5 + 0.5) as i32;
        let f = crate::fast_exp2(ports[1].value + 1.0) * (1.0 / sample_rate) * 400.0;
        (mode, f, ports[2].value, ports[3].value)
    }
}

impl Default for VariEq {
    fn default() -> Self {
        VariEq::new()
    }
}

impl Node for VariEq {
    fn ports(&mut self) -> &mut Vec<Port> {
        &mut self.ports
    }

    fn reset(&mut self) {
        for f in &mut self.f {
            f.reset();
        }
    }

    fn process(&mut self, ctx: &Ctx, inp: [&[f32]; 2], out: [&mut [f32]; 2]) {
        let (mode, f, q, g) = VariEq::coeff_args(&self.ports, ctx.sample_rate);
        for (filter, (i, o)) in self.f.iter_mut().zip(inp.iter().zip(out)) {
            filter.set(mode, f, q, g);
            for (o, x) in o.iter_mut().zip(i.iter()) {
                *o = filter.tick(*x);
            }
        }
    }
}
