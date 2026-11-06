//! v1's Chorus voice (constructor ev_30d92afc, `UpdateSampleRate` ev_dbd1543d,
//! `Reset` ev_2bac9328, `ProcessBuffer` ev_41e7ae75, ports ev_f96cb250)
//! transcribed operation by operation (f32 throughout, the disassembly's
//! association where the pseudo-C lost it) and rendered against Dreist's
//! node (`dreist_dsp::fx::nodes::chorus`).
//!
//! One voice handles both channels: In1/In2 → Out1/Out2 through a single mono
//! delay line fed with (L + R)/2.
//!
//! The generator's start state comes from the shared seed routine, whose
//! body is not part of the transcribed functions; both sides start from
//! state 0 here (Dreist's node starts from 0), so the random walk is the
//! same sequence and only the arithmetic is compared.

use dreist_dsp::analog::sin_tab;
use dreist_dsp::fx::{Ctx, Node};
use einst_dsp::chorus::Chorus;

/// Switches that put one of Dreist's choices into the transcription, to
/// attribute each difference.
#[derive(Clone, Copy, Default)]
struct Ours {
    /// Max delay from 384 kHz instead of the current rate.
    max_rate: bool,
    /// Walk increment `f / 3` instead of `f · 0.33333334`.
    div3: bool,
    /// Cubic constant term `((c − a/3) − b/2) − d/6` instead of
    /// `((−b/2 + c) − a/3) − d/6`.
    cubic: bool,
    /// Tap mix `(y3/4 + 3y2/4) + y1` instead of `(y1 + 3y2/4) + y3/4`.
    mix: bool,
}

struct V1Chorus {
    o: Ours,
    idx: u32,         // +0x138
    max_delay: f32,   // +0x13c
    depth: f32,       // +0x140
    k_depth: f32,     // +0x144
    phase: f32,       // +0x148
    seed: u32,        // +0x14c (VRand state)
    walk: f32,        // +0x19c
    cur: [f32; 4],    // +0x1a0
    target: [f32; 4], // +0x1c0
    slope: [f32; 4],  // +0x1d0
    last_f: f32,      // +0x1e0
    buf: Vec<f32>,    // +0x1f0
    size: i32,        // +0x1f8
    mask: u32,        // +0x1fc
    alt: f32,         // +0x208
    k_acc: f32,       // +0x20c
    mono: f32,        // +0x210
    acc: f32,         // +0x214 (written, never read)
    wet: f32,         // +0x21c
}

impl V1Chorus {
    fn new(seed: u32, o: Ours) -> Self {
        V1Chorus {
            o,
            idx: 0,
            max_delay: 0.0,
            depth: 0.0,
            k_depth: 0.0,
            phase: 0.0,
            seed,
            walk: 0.0,
            cur: [0.0; 4],
            target: [0.0; 4],
            slope: [0.0; 4],
            last_f: 0.0,
            buf: Vec::new(),
            size: 0,
            mask: 0,
            alt: f32::from_bits(0x322bcc77),
            k_acc: 0.0,
            mono: 0.0,
            acc: 0.0,
            wet: 0.0,
        }
    }

    fn rand(&mut self) -> f32 {
        self.seed = self.seed.wrapping_mul(0xbb38433).wrapping_add(0x3619636b);
        f32::from_bits(self.seed >> 9 | 0x4000_0000) - 3.0
    }

    /// SongInfo +0x20 is the rate, +0x24 = 1.0 / +0x20 (`SetSampleRate`).
    fn update_sample_rate(&mut self, sr: f32) {
        let isr = 1.0f32 / sr;
        let r = if self.o.max_rate { 384000.0f32 } else { sr };
        let m = (r * 0.015 + 2.0 + (r * 0.015 - 2.0).abs()) * 0.5;
        self.max_delay = m;
        let mut size = 1i32;
        let mut k = (m + m + 0.5) as i32 - 1;
        while 0 < k {
            size *= 2;
            k >>= 1;
        }
        if self.size != size {
            self.size = size;
            self.buf = vec![0.0; size as usize];
            self.mask = (size - 1) as u32;
        }
        self.buf.iter_mut().for_each(|v| *v = 0.0);
        self.k_acc = isr * 251.32742;
        self.k_depth = isr * 6.2831855;
    }

    fn reset(&mut self) {
        self.depth = 0.0;
        self.idx = 0;
        self.walk = 0.0;
        self.phase = 0.0;
        self.mono = 0.0;
        self.acc = 0.0;
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

    fn cubic(&self, pos: f32) -> f32 {
        let k = pos.floor() as i64 as i32;
        let t = pos - k as f32;
        let m = self.mask;
        let u = k as u32;
        let a = self.buf[(u.wrapping_sub(1) & m) as usize];
        let b = self.buf[(u & m) as usize];
        let c = self.buf[(u.wrapping_add(1) & m) as usize];
        let d = self.buf[(u.wrapping_add(2) & m) as usize];
        let c0 = if self.o.cubic { ((c - a * 0.33333334) - b * 0.5) - d * 0.16666667 } else { ((b * -0.5 + c) + a * -0.33333334) + d * -0.16666667 };
        let inner = (((c + a) * 0.5 - b) + ((b - c) * 0.5 + (d - a) * 0.16666667) * t) * t;
        (c0 + inner) * t + b
    }

    fn process(&mut self, sr: f32, p: [f32; 4], il: &[f32], ir: &[f32], ol: &mut [f32], or: &mut [f32]) {
        let isr = 1.0f32 / sr;
        let n = il.len();
        let inv_n = 1.0f32 / n as f32;
        let clip = |x: f32| ((x.abs() + 1.0) - (x - 1.0).abs()) * 0.5;
        let x = p[0] * isr;
        let f = ((x.abs() + 0.5) - (x - 0.5).abs()) * 0.5;
        let d3 = p[1] * p[1] * p[1];
        // The disassembly: ((|d| + 1) − |d − 1|) · (0.5 · md).
        let dtarget = ((d3.abs() + 1.0) - (d3 - 1.0).abs()) * (0.5 * self.max_delay);
        let cross = clip(p[2] * -0.5 + 0.5);
        let one_m_cross = 1.0 - cross;
        let dwet = (clip(p[3]) - self.wet) * inv_n;
        let f3 = if self.o.div3 { f / 3.0 } else { f * 0.33333334 };
        if f != self.last_f {
            self.last_f = f;
            for j in 0..4 {
                self.slope[j] = (self.target[j] - self.cur[j]) * f3;
            }
        }
        self.alt = -self.alt;
        self.depth += self.alt;
        self.acc += self.alt;
        let t = sin_tab();
        let mut wet = self.wet;
        for i in 0..n {
            wet += dwet;
            let omw = 1.0 - wet;
            self.phase += f;
            self.walk += f3;
            if 1.0 <= self.phase {
                self.phase -= 1.0;
            }
            if 1.0 <= self.walk {
                self.walk -= 1.0;
                for j in 0..4 {
                    self.target[j] = self.rand();
                }
                for j in 0..4 {
                    self.slope[j] = (self.target[j] - self.cur[j]) * f3;
                }
            }
            self.idx = (self.idx + 1) & self.mask;
            self.depth += (dtarget - self.depth) * self.k_depth;
            for j in 0..4 {
                self.cur[j] += self.slope[j];
            }
            let base = self.idx as i32 as f32 - self.max_delay;
            let (xl, xr) = (il[i], ir[i]);
            let ph = self.phase * 2048.0;
            let k = ph.floor() as i64 as i32;
            let fr = ph - k as f32;
            let k = k as usize;
            let s = (t[1024 + k + 1] - t[1024 + k]) * fr + t[1024 + k];
            let c = (t[1536 + k + 1] - t[1536 + k]) * fr + t[1536 + k];
            let d = self.depth;
            let y1 = self.cubic((s + self.cur[0]) * d + base);
            let y2 = self.cubic((c + self.cur[1]) * d + base);
            let y3 = self.cubic(base - (c + self.cur[2]) * d);
            let y4 = self.cubic(base - (s + self.cur[3]) * d);
            let (a, b) = if self.o.mix { (y3 * 0.25 + y2 * 0.75 + y1, y2 * 0.25 + y3 * 0.75 + y4) } else { (y1 + y2 * 0.75 + y3 * 0.25, y4 + y3 * 0.75 + y2 * 0.25) };
            let hw = 0.5 * wet;
            ol[i] = omw * xl + ((one_m_cross * a + cross * b) + xl) * hw;
            or[i] = omw * xr + ((cross * a + one_m_cross * b) + xr) * hw;
            self.mono = (xr + xl) * 0.5;
            self.acc += self.mono * self.k_acc;
            self.buf[(self.idx & self.mask) as usize] = self.mono;
        }
        self.wet = wet;
    }
}

fn signals(sr: f32, len: usize) -> Vec<(&'static str, Vec<f32>, Vec<f32>)> {
    let mut imp_l = vec![0.0f32; len];
    let mut imp_r = vec![0.0f32; len];
    imp_l[0] = 1.0;
    imp_r[777] = -0.8;
    let dur = len as f64 / sr as f64;
    let sweep: Vec<f32> = (0..len)
        .map(|i| {
            let tt = i as f64 / sr as f64;
            let k = (10_000.0f64 / 100.0).ln();
            let ph = 2.0 * std::f64::consts::PI * 100.0 * dur / k * ((k * tt / dur).exp() - 1.0);
            (ph.sin() * 0.7) as f32
        })
        .collect();
    let sweep_r: Vec<f32> = sweep.iter().map(|v| v * -0.5).collect();
    let mut s = 0x1234_5678u32;
    let mut noise = || {
        s = s.wrapping_mul(1664525).wrapping_add(1013904223);
        (s >> 8) as f32 / (1u32 << 24) as f32 * 2.0 - 1.0
    };
    let nl: Vec<f32> = (0..len).map(|_| noise()).collect();
    let nr: Vec<f32> = (0..len).map(|_| noise()).collect();
    vec![("impulse", imp_l, imp_r), ("sweep", sweep, sweep_r), ("noise", nl, nr)]
}

/// Settings (Freq Hz, Depth, Spread, WetMix), each port's range ends and
/// interior points; the second set is switched to half way through.
const SETTINGS: &[([f32; 4], [f32; 4])] = &[
    ([0.1, 0.5, 1.0, 1.0], [0.1, 0.5, 1.0, 1.0]),
    ([0.0, 0.0, 0.0, 0.0], [0.0, 0.0, 0.0, 0.0]),
    ([5.0, 1.0, 1.0, 1.0], [5.0, 1.0, 1.0, 1.0]),
    ([5.0, 1.0, 0.0, 0.6], [5.0, 1.0, 0.0, 0.6]),
    ([1.3, 0.7, 0.37, 0.8], [1.3, 0.7, 0.37, 0.8]),
    ([0.0, 1.0, 0.5, 1.0], [0.0, 1.0, 0.5, 1.0]),
    ([2.71, 0.23, 0.81, 0.33], [2.71, 0.23, 0.81, 0.33]),
    ([0.1, 0.5, 1.0, 1.0], [3.7, 0.9, 0.2, 0.5]),
    ([4.4, 0.05, 0.6, 0.0], [0.6, 0.95, 1.0, 1.0]),
    ([0.37, 0.6, 0.9, 1.0], [0.77, 0.6, 0.9, 1.0]),
    ([1.7, 0.8, 0.1, 0.9], [4.9, 0.4, 0.75, 0.7]),
];

struct Diff {
    max: f32,
    first: Option<usize>,
    count: usize,
}

fn compare(o: Ours, verbose: bool) -> (f32, usize) {
    let mut worst = 0.0f32;
    let mut mismatches = 0usize;
    for &sr in &[44_100.0f32, 48_000.0] {
        let len = (sr as usize) * 3 / 2;
        let sigs = signals(sr, len);
        for &block in &[64usize, 512] {
            for (name, l, r) in &sigs {
                for (si, &(p0, p1)) in SETTINGS.iter().enumerate() {
                    let ctx = Ctx::new(sr);
                    let mut ours = Chorus::new();
                    let mut v1 = V1Chorus::new(0, o);
                    v1.update_sample_rate(sr);
                    v1.reset();
                    let mut d = Diff { max: 0.0, first: None, count: 0 };
                    let (mut a, mut b, mut c, mut e) = (vec![0.0; block], vec![0.0; block], vec![0.0; block], vec![0.0; block]);
                    let mut pos = 0;
                    while pos < len {
                        let n = block.min(len - pos);
                        let p = if pos < len / 2 { p0 } else { p1 };
                        for (k, v) in p.iter().enumerate() {
                            ours.ports()[k].value = *v;
                        }
                        let (il, ir) = (&l[pos..pos + n], &r[pos..pos + n]);
                        ours.process(&ctx, [il, ir], [&mut a[..n], &mut b[..n]]);
                        v1.process(sr, p, il, ir, &mut c[..n], &mut e[..n]);
                        for i in 0..n {
                            for (x, y) in [(a[i], c[i]), (b[i], e[i])] {
                                if x.to_bits() != y.to_bits() {
                                    d.count += 1;
                                    d.first.get_or_insert(pos + i);
                                    d.max = d.max.max((x - y).abs());
                                }
                            }
                        }
                        pos += n;
                    }
                    if verbose && d.count > 0 {
                        println!("sr {sr} block {block} {name:8} set {si}: {} differing samples, first at {}, max |diff| {:e}", d.count, d.first.unwrap(), d.max);
                    }
                    worst = worst.max(d.max);
                    mismatches += d.count;
                }
            }
        }
    }
    (worst, mismatches)
}

/// Einst's Chorus is v1's: the faithful transcription (v1's line length,
/// walk increment, cubic and tap-mix order) renders bit-identically.
#[test]
fn chorus_against_v1_transcription() {
    let (w, m) = compare(Ours::default(), true);
    println!("faithful v1: max {w:e}, {m} differing samples");
    assert_eq!(m, 0);
}

/// Walk increment: v1 multiplies by 0.33333334, Dreist divides by 3. Over
/// the Freq range at 44.1 / 48 kHz the two round differently for some
/// rates; count them.
#[test]
fn walk_increment_div3_vs_mul() {
    for sr in [44_100.0f32, 48_000.0] {
        let isr = 1.0f32 / sr;
        let (mut n, mut diff) = (0, 0);
        let mut q = 0.0f32;
        while q <= 5.0 {
            let x = q * isr;
            let f = ((x.abs() + 0.5) - (x - 0.5).abs()) * 0.5;
            n += 1;
            if (f / 3.0).to_bits() != (f * 0.33333334).to_bits() {
                diff += 1;
            }
            q += 0.0001;
        }
        println!("sr {sr}: {diff} of {n} Freq values give a different f/3");
    }
}
