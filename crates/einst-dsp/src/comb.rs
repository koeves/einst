//! Einst: Dreist's port of the Comb Filter (`dreist_dsp::fx::nodes::comb`,
//! copied from dreist-commons 69f4cc1) with v1's differences
//! (`Voice::ProcessBuffer` ev_8625fc5a, `Reset` ev_40393d54,
//! `UpdateSampleRate` ev_10501d80; dreist-re `einst/docs/re/effects-v1.md`):
//!
//! - both 2^x, the comb delay 2^(−pitch) and the tracking DC coefficient's
//!   2^(pitch − 2), are v1's [`fast_exp2`](crate::fast_exp2);
//! - the modulated delay and the cubic tap sum in v1's order:
//!   `((s − prev) + p) + (s·x)·fm` and `((−b/2 + c) − a/3) − d/6`.
//!
//! The rest (targets, ramps, saturator, feedback clip, DC trackers, reset)
//! is the same.
//!
//! [re:5939a7] (Comb Filter, v2: `Voice::PrepareStream` [re:72902e],
//! `ProcessBuffer` [re:f216da]): tuned feedback comb with cubic-interpolated
//! taps, saturated input, clipped feedback and audio-rate phase/frequency
//! modulation from its `ModIn` port. The effect runs it dual-mono
//! ([re:7441ff]): one instance per channel, each fed its
//! own channel on both `In` and `ModIn`.

use crate::fast_exp2;
use crate::math::{pow2_at_least, sat};
use dreist_dsp::fx::{Ctx, Node, Port, ports, ports_from};

/// 4-point cubic read at `pos`, summed in v1's order.
fn cubic_v1(buf: &[f32], mask: u32, pos: f32) -> f32 {
    let i = pos.floor() as i32 as u32;
    let t = pos - (i as i32) as f32;
    let a = buf[(i.wrapping_sub(1) & mask) as usize];
    let b = buf[(i & mask) as usize];
    let c = buf[(i.wrapping_add(1) & mask) as usize];
    let d = buf[(i.wrapping_add(2) & mask) as usize];
    ((((b * -0.5 + c) + a * -0.33333334) + d * -0.16666667) + (((c + a) * 0.5 - b) + ((b - c) * 0.5 + (d - a) * 0.16666667) * t) * t) * t + b
}

/// `(0xbe6a3000 − bits) >> 1` estimate plus one Newton step.
#[inline]
fn fast_rsqrt(x: f32) -> f32 {
    let y = f32::from_bits(0xbe6a3000u32.wrapping_sub(x.to_bits()) >> 1);
    y * 1.5 - y * y * y * x * 0.5
}

struct Voice {
    buf: Vec<f32>,
    mask: u32,
    idx: u32,
    max_delay: f32,
    k_delay: f32,
    k_dc: f32,
    alt: f32,
    delay: f32,
    fb: f32,
    gain_a: f32,
    gain_b: f32,
    delay_smooth: f32,
    dc: f32,
    pm_prev: f32,
}

impl Voice {
    fn new() -> Voice {
        Voice {
            buf: Vec::new(),
            mask: 0,
            idx: 0,
            max_delay: 0.0,
            k_delay: 0.0,
            k_dc: 0.0,
            alt: dreist_dsp::fx::nodes::ssl_filter::DENORMAL,
            delay: 1.0,
            fb: 0.0,
            gain_a: 1.0,
            gain_b: 1.0,
            delay_smooth: 1.0,
            dc: 0.0,
            pm_prev: 0.0,
        }
    }

    /// `PrepareStream`: buffer for [re:46f4da]+8 / 20 samples (power of two).
    fn prepare(&mut self, sr: f32) {
        let m = 384000.0f32 / 20.0;
        let m = ((m - 2.0).abs() + m + 2.0) * 0.5;
        let n = m.round() as i32;
        let size = pow2_at_least(n as i64);
        self.buf = vec![0.0; size];
        self.mask = size as u32 - 1;
        self.max_delay = (size as i32 - 2) as f32;
        let inv = 1.0 / sr;
        let a = inv * 376.99112;
        self.k_delay = ((a + 1.0) - (a - 1.0).abs()) * 0.5;
        let b = inv * 125.66371;
        self.k_dc = ((b + 1.0) - (b - 1.0).abs()) * 0.5;
    }

    #[allow(clippy::too_many_arguments)]
    fn process(&mut self, ctx: &Ctx, x: &[f32], out: &mut [f32], drive: f32, mode: i32, pitch: f32, fb: f32, pm: f32, fm: f32) {
        let n = x.len();
        let inv_n = 1.0 / n as f32;
        let sr = ctx.sample_rate;
        let inv_tune = 1.0 / ctx.tune;
        let one = fb + 1.0;
        let y = fast_rsqrt(one.abs());
        let norm = if one.is_sign_negative() { -1.0 } else { 1.0 } * y * y;
        let d = self.alt;
        self.alt = -d;
        self.delay_smooth -= d;
        let w0 = (self.idx & self.mask) as usize;
        self.buf[w0] -= d;
        self.dc -= d;
        let modulated = 1e-6 <= fm.abs() + pm.abs();
        let (t_delay, t_fb, t_a, t_b) =
            if mode == 1 { (fast_exp2(-pitch) * inv_tune * 0.5 * sr, -fb, norm * 0.5 * norm, norm * -0.5) } else { (fast_exp2(-pitch) * inv_tune * sr, fb, norm * norm * 0.5, norm * 0.5) };
        let s = (self.delay, self.fb, self.gain_a, self.gain_b);
        let step = (t_delay - s.0, t_fb - s.1, t_a - s.2, t_b - s.3);
        // Frequency-tracking DC coefficient of the unmodulated "+" mode.
        let k_track = {
            let f = fast_exp2(pitch + -2.0) * ctx.tune * (1.0 / sr);
            ((f.abs() + 0.25) - (f - 0.25).abs()) * 0.5
        };
        let m = self.mask;
        for i in 0..n {
            self.delay += inv_n * step.0;
            self.gain_a += inv_n * step.2;
            self.gain_b += inv_n * step.3;
            self.delay_smooth = (self.delay - self.delay_smooth) * self.k_delay + self.delay_smooth;
            self.fb += inv_n * step.1;
            let dl = if modulated {
                let prev = self.pm_prev;
                let p = x[i] * pm;
                self.pm_prev = p;
                ((self.delay_smooth - prev) + p) + self.delay_smooth * x[i] * fm
            } else {
                self.delay_smooth
            };
            self.idx = self.idx.wrapping_add(1) & m;
            let w = self.idx;
            let c = sat(x[i] * drive);
            let lim = self.max_delay;
            let pos = (w as i32 as f32 - ((dl.abs() + lim) - (dl - lim).abs()) * 0.5) + -2.0;
            let tap = cubic_v1(&self.buf, self.mask, pos);
            let e = tap - self.dc;
            let k = if !modulated && mode == 0 { k_track } else { self.k_dc };
            self.dc += k * e;
            out[i] = self.gain_b * tap + self.gain_a * self.buf[(w.wrapping_sub(2) & m) as usize];
            let v = e * self.fb + c;
            let v = ((v + 1.5).abs() - (v - 1.5).abs()) * 0.5;
            self.buf[w as usize] = v * -0.14814815 * v * v + v;
        }
    }

    fn reset(&mut self) {
        self.buf.fill(0.0);
        self.delay_smooth = 1.0;
        self.dc = 0.0;
        self.pm_prev = 0.0;
        self.delay = 1.0;
        self.fb = 0.0;
        self.gain_a = 1.0;
        self.gain_b = 1.0;
    }
}

pub struct Comb {
    ports: Vec<Port>,
    sr: f32,
    voices: [Voice; 2],
}

impl Comb {
    pub fn new() -> Comb {
        let mut ports = ports_from(ports::COMB_FILTER);
        // Mode: float-form, max = [re:0cac93] item count (2).
        ports[1] = Port::new(0.0, 0.0, 2.0);
        Comb { ports, sr: 0.0, voices: [Voice::new(), Voice::new()] }
    }
}

impl Default for Comb {
    fn default() -> Self {
        Comb::new()
    }
}

impl Comb {
    /// `PrepareStream` on a rate change.
    fn prepare(&mut self, sr: f32) {
        if self.sr != sr {
            self.sr = sr;
            for v in &mut self.voices {
                v.prepare(sr);
            }
        }
    }
}

impl Node for Comb {
    fn ports(&mut self) -> &mut Vec<Port> {
        &mut self.ports
    }

    fn prepare_rate(&mut self, sr: f32) {
        self.prepare(sr);
    }

    fn process(&mut self, ctx: &Ctx, inp: [&[f32]; 2], out: [&mut [f32]; 2]) {
        self.prepare(ctx.sample_rate);
        if inp[0].is_empty() {
            return;
        }
        let p = |i: usize| self.ports[i].value;
        let dv = p(0);
        let dv = dv * dv * dv;
        let drive = (dv.abs() + dv) * 0.5;
        let m = p(1);
        let mode = (((m.abs() + 1.0) - (m - 1.0).abs()) * 0.5).round() as i32;
        let r = (p(3).abs() + p(3)) * 0.5;
        let fb = r * r * 1.025;
        let fmc = ((p(5).abs() + 1.0) - (p(5) - 1.0).abs()) * 0.5;
        let pmc = ((p(4).abs() + 1.0) - (p(4) - 1.0).abs()) * 0.5;
        let pm = pmc * pmc * 6.2831855;
        let fm = fmc * fmc * 6.2831855 * 6.2831855;
        let pitch = p(2);
        for (k, o) in out.into_iter().enumerate() {
            self.voices[k].process(ctx, inp[k], o, drive, mode, pitch, fb, pm, fm);
        }
    }

    fn reset(&mut self) {
        for v in &mut self.voices {
            v.reset();
        }
    }
}
