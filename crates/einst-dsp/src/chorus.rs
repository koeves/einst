//! The original v1's Chorus: Dreist's node (Geist2's, [re:b7f410]) as v1
//! builds it, from v1's `Voice` code (constructor, `UpdateSampleRate`,
//! `Reset`, `ProcessBuffer` 0x53a240), transcribed and render-compared
//! sample by sample (`tests/v1_chorus_transcribed.rs`):
//!
//! - the line's length is 15 ms at the current rate (v1's
//!   `UpdateSampleRate` reads the processing rate; Geist2 sizes it for
//!   384 kHz, about 130 ms at 44.1 kHz: the delay and the depth scale);
//! - the walk increment is `f·0.33333334` (Geist2 `f/3`);
//! - the taps mix `(y1 + y2·0.75) + y3·0.25` and `(y4 + y3·0.75) + y2·0.25`;
//! - the cubic read groups its constant term left to right
//!   ([`crate::math::cubic`]).

use crate::math::{clip01, cubic, pow2_at_least};
use dreist_dsp::analog::sin_tab;
use dreist_dsp::fx::{Ctx, Node, Port, ports, ports_from};

/// The alternating anti-denormal offset's bits (as Dreist's nodes).
const ALT: u32 = 0x322bcc77;

pub struct Chorus {
    ports: Vec<Port>,
    alt: f32,
    idx: u32,
    phase: f32,
    walk_phase: f32,
    seed: u32,
    target: [f32; 4],
    cur: [f32; 4],
    slope: [f32; 4],
    last_f: f32,
    depth: f32,
    mono: f32,
    wet: f32,
    max_delay: f32,
    k_depth: f32,
    buf: Vec<f32>,
    mask: u32,
    sr: f32,
}

impl Chorus {
    pub fn new() -> Chorus {
        let mut c = Chorus {
            ports: ports_from(ports::CHORUS),
            alt: f32::from_bits(ALT),
            idx: 0,
            phase: 0.0,
            walk_phase: 0.0,
            seed: 0,
            target: [0.0; 4],
            cur: [0.0; 4],
            slope: [0.0; 4],
            last_f: 0.0,
            depth: 0.0,
            mono: 0.0,
            wet: 1.0,
            max_delay: 0.0,
            k_depth: 0.0,
            buf: Vec::new(),
            mask: 0,
            sr: 0.0,
        };
        c.reset();
        c
    }

    fn rand(&mut self) -> f32 {
        self.seed = self.seed.wrapping_mul(0xbb38433).wrapping_add(0x3619636b);
        f32::from_bits(self.seed >> 9 | 0x4000_0000) + -3.0
    }

    fn prepare(&mut self, sr: f32) {
        // v1: 15 ms at the processing rate.
        let m = sr * 0.015;
        let m = ((m + -2.0).abs() + m + 2.0) * 0.5;
        self.max_delay = m;
        let n = (m + m).round() as i32;
        let size = pow2_at_least(n as i64);
        self.buf = vec![0.0; size];
        self.mask = size as u32 - 1;
        let isr = 1.0 / sr;
        self.k_depth = isr * 6.2831855;
    }
}

impl Default for Chorus {
    fn default() -> Self {
        Chorus::new()
    }
}

impl Node for Chorus {
    fn ports(&mut self) -> &mut Vec<Port> {
        &mut self.ports
    }

    fn prepare_rate(&mut self, sr: f32) {
        if self.sr != sr {
            self.sr = sr;
            self.prepare(sr);
        }
    }

    fn process(&mut self, ctx: &Ctx, inp: [&[f32]; 2], out: [&mut [f32]; 2]) {
        if self.sr != ctx.sample_rate {
            self.sr = ctx.sample_rate;
            self.prepare(self.sr);
        }
        let p = |i: usize| self.ports[i].value;
        let isr = 1.0 / ctx.sample_rate;
        let fr = p(0) * isr;
        let f = ((fr.abs() + 0.5) - (fr + -0.5).abs()) * 0.5;
        let d3 = p(1) * p(1) * p(1);
        let f3 = f * 0.33333334;
        let n = inp[0].len();
        let dwet = (clip01(p(3)) - self.wet) * (1.0 / n.max(1) as f32);
        if self.last_f != f {
            self.last_f = f;
            for j in 0..4 {
                self.slope[j] = f3 * (self.target[j] - self.cur[j]);
            }
        }
        let a = self.alt;
        self.alt = -a;
        self.depth -= a;
        let cross = clip01(0.5 - p(2) * 0.5);
        let t = sin_tab();
        let m = self.mask;
        let md = self.max_delay;
        let dtarget = clip01(d3) * md;
        let [ol, or] = out;
        for i in 0..n {
            self.wet += dwet;
            self.phase += f;
            self.walk_phase += f3;
            if 1.0 <= self.phase {
                self.phase += -1.0;
            }
            if 1.0 <= self.walk_phase {
                self.walk_phase += -1.0;
                for j in 0..4 {
                    self.target[j] = self.rand();
                }
                for j in 0..4 {
                    self.slope[j] = (self.target[j] - self.cur[j]) * f3;
                }
            }
            self.idx = (self.idx + 1) & m;
            let w = self.idx;
            self.depth = (dtarget - self.depth) * self.k_depth + self.depth;
            for j in 0..4 {
                self.cur[j] = self.slope[j] + self.cur[j];
            }
            let base = w as i32 as f32 - md;
            let x = self.phase * 2048.0;
            let k = x.floor() as i32;
            let fr = x - k as f32;
            let s = (t[(1024 + k) as usize + 1] - t[(1024 + k) as usize]) * fr + t[(1024 + k) as usize];
            let c = (t[(1536 + k) as usize + 1] - t[(1536 + k) as usize]) * fr + t[(1536 + k) as usize];
            let d = self.depth;
            let y1 = cubic(&self.buf, m, (self.cur[0] + s) * d + base);
            let y2 = cubic(&self.buf, m, (self.cur[1] + c) * d + base);
            let y3 = cubic(&self.buf, m, base - (self.cur[2] + c) * d);
            let y4 = cubic(&self.buf, m, base - (self.cur[3] + s) * d);
            let a_ = y1 + y2 * 0.75 + y3 * 0.25;
            let b_ = y4 + y3 * 0.75 + y2 * 0.25;
            let (xl, xr) = (inp[0][i], inp[1][i]);
            ol[i] = (1.0 - self.wet) * xl + (cross * b_ + (1.0 - cross) * a_ + xl) * self.wet * 0.5;
            or[i] = (1.0 - self.wet) * xr + (b_ * (1.0 - cross) + a_ * cross + xr) * self.wet * 0.5;
            self.mono = (xl + xr) * 0.5;
            self.buf[w as usize] = self.mono;
        }
    }

    /// Reset [re:a569df]: four new targets, then four new current offsets.
    fn reset(&mut self) {
        self.depth = 0.0;
        self.idx = 0;
        self.walk_phase = 0.0;
        self.phase = 0.0;
        self.mono = 0.0;
        self.last_f = 0.0;
        self.wet = 1.0;
        for j in 0..4 {
            self.target[j] = self.rand();
        }
        for j in 0..4 {
            self.cur[j] = self.rand();
        }
        self.slope = [0.0; 4];
    }
}
