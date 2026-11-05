//! v1's Phaser voice transcribed from the decompiled code (Reset
//! ev_14ca6d2e, ProcessBuffer ev_c3cd471f, ports ev_c6edd600, modes
//! ev_426fcf89), operation for operation, against the shared node. The
//! voice is mono; the node runs one per channel.
//!
//! `ours` bits switch single operations to the node's order, to pin down
//! which one a difference comes from (0 = v1 throughout).

use dreist_dsp::analog::sin_tab;
use dreist_dsp::fx::{Ctx, Node};
use einst_dsp::phaser::Phaser;

const OURS_RS: u32 = 1; // rsqrt Newton step: y0³·g vs ((g·y0)·y0)·y0
const OURS_DRY: u32 = 2; // dry step: (1−m)·5·s·rs·rs vs (1−m)·s·rs·rs·5
const OURS_FT: u32 = 4; // exp2·tune·(2/sr) vs (2/sr)·tune·exp2
const OURS_LFO: u32 = 8; // (Δ·frac + t0) + 0.5 vs (t0 + 0.5) + Δ·frac
const OURS_V: u32 = 16; // c·f + ((f + c) − prev) vs ((f − prev) + c) + c·f
const OURS_ROUND: u32 = 32; // round() vs (int)(x + 0.5)
const OURS_ISR: u32 = 64; // 2/sr kept local vs re-read from +0x1c4 after a mode-change Reset (which zeroes it)

/// The voice's fields by byte offset (0x138 ..= 0x1d4).
struct V1 {
    m: [f32; 0x1d8 / 4],
    mode: i32, // +0x1c8
    ours: u32,
}

impl V1 {
    fn new(ours: u32) -> V1 {
        let mut v = V1 { m: [0.0; 0x1d8 / 4], mode: 0, ours };
        // +0x1d4 (alternating denormal guard) is set outside Reset; the
        // constructor is not in the dumps: the shared nodes' value.
        v.set(0x1d4, f32::from_bits(0x322bcc77));
        v.reset();
        v
    }

    fn g(&self, off: usize) -> f32 {
        self.m[off / 4]
    }

    fn set(&mut self, off: usize, x: f32) {
        self.m[off / 4] = x;
    }

    fn reset(&mut self) {
        for off in [
            400, 0x16c, 0x14c, 0x194, 0x170, 0x150, 0x198, 0x174, 0x154, 0x19c, 0x178, 0x158, 0x1a0, 0x17c, 0x15c, 0x1a4, 0x180, 0x160, 0x1a8, 0x184, 0x164, 0x1ac, 0x188, 0x168, 0x1b0, 0x18c, 0x1b4,
            0x1b8, 0x1bc,
        ] {
            self.set(off, 0.0);
        }
        self.set(0x148, f32::from_bits(0x2edbe6ff));
        self.set(0x138, f32::from_bits(0x3dcccccd));
        self.set(0x1d0, 0.0);
        self.set(0x140, 0.5);
        self.set(0x144, 0.5);
        self.set(0x1cc, 0.0);
        self.mode = 0;
        self.set(0x13c, 0.0);
        self.set(0x1c0, 0.0);
        self.set(0x1c4, 0.0);
    }

    fn sat(x: f32) -> f32 {
        let c = ((x + 2.0).abs() - (x - 2.0).abs()) * 0.5;
        ((c.abs() * 0.0625 - 0.25) * c * c + 1.0) * c
    }

    /// `p`: port values; `isr` = SongInfo+0x24, `inv_n` = +0x2c.
    fn process_buffer(&mut self, isr: f32, tune: f32, p: &[f32; 9], x: &[f32], out: &mut [f32]) {
        let frames = x.len() as i32;
        let inv_n = 1.0f32 / frames as f32;
        self.set(0x1c4, isr + isr);
        let f13 = p[0] * p[0] * p[0];
        let mut f12 = (f13.abs() + f13) * 0.5;
        self.set(0x1c0, f12);
        let mc = ((p[1].abs() + 7.0) - (p[1] - 7.0).abs()) * 0.5;
        let mode = if self.ours & OURS_ROUND != 0 { mc.round() as i32 } else { (mc + 0.5) as i32 };
        let mut f13 = p[2];
        let f11 = ((p[3].abs() + p[3]) * 0.5).sqrt().sqrt();
        let mut f10 = ((p[6].abs() + 1.0) - (p[6] - 1.0).abs()) * 0.5;
        let mut f16 = p[7];
        let mut f18 = p[4];
        let f9 = p[5];
        let f15 = (f11.abs() + 1.0).abs();
        let mut f4 = f32::from_bits(0xbe6a3000u32.wrapping_sub(f15.to_bits()) >> 1);
        f4 = if self.ours & OURS_RS != 0 { f4 * 1.5 - f4 * f4 * f4 * f15 * 0.5 } else { f4 * 1.5 + f15 * f4 * f4 * f4 * -0.5 };
        f12 *= 0.2;
        let sgn = ((((f11.abs() + 1.0).to_bits() as i32) >> 31) * 2 + 1) as f32;
        f4 = if self.ours & OURS_DRY != 0 { ((1.0 - f10) * 5.0 * sgn * f4 * f4 - self.g(0x140)) * inv_n } else { ((1.0 - f10) * sgn * f4 * f4 * 5.0 - self.g(0x140)) * inv_n };
        f10 = inv_n * (f10 * 5.0 - self.g(0x144));
        if mode != self.mode {
            self.reset();
        }
        self.mode = mode;
        // Re-read from the field: a mode-change Reset above zeroed it.
        let f15 = if self.ours & OURS_ISR != 0 { isr + isr } else { self.g(0x1c4) };
        let f7 = (f13 - 0.5).floor();
        f13 -= (f7 as i32) as f32;
        let e = f32::from_bits(((((f13 * 0.079440236 + 0.22449434) * f13 + 0.69606566) * f13 + 1.0).to_bits() as i32).wrapping_add((f7 as i32).wrapping_mul(0x800000)) as u32);
        f13 = if self.ours & OURS_FT != 0 { e * tune * f15 } else { f15 * tune * e };
        f13 = (((f13 + 0.16666667).abs() - (f13 - 0.16666667).abs()) * 0.5 - self.g(0x138)) * inv_n;
        let f15 = -self.g(0x1d4);
        self.set(0x1d4, f15);
        for off in [0x148, 0x16c, 0x154, 0x178, 0x164] {
            self.set(off, self.g(off) + f15);
        }
        self.set(0x188, f15 + self.g(0x188));
        f16 = ((f16.abs() + 1.0) - (f16 - 1.0).abs()) * 0.5;
        assert!(1e-06 <= ((f16 * f16 * 6.2831855).abs() + 39.47842).abs());
        let f16 = ((f9.abs() + 1.0) - (f9 - 1.0).abs()) * 0.5;
        f18 = isr * 0.5 * ((f18.abs() + 5.0) - (f18 - 5.0).abs());
        f18 = ((f18.abs() + 1.0) - (f18 - 1.0).abs()) * 0.5;
        // Per-mode unrolled cases: same body, coefficients and stage count.
        let (poly, stages): ([f32; 4], usize) = match mode {
            0 => ([-0.49724257, -0.0028277636, 2.4676685, -7.705447e-06], 2),
            1 => ([0.09575251, -1.3119804, 0.001682329, 3.1415472], 2),
            2 => ([-2.5111454, 3.0190501, -2.4265928, 3.14055], 4),
            3 => ([-42.52696, 39.453728, -18.940275, 7.5505123], 4),
            4 => ([-20.616514, 20.549196, -10.952777, 5.427524], 6),
            5 => ([-166.76474, 132.36517, -47.11987, 11.511031], 6),
            6 => ([-58.97007, 52.01872, -22.277859, 7.530122], 8),
            7 => ([-354.18213, 259.99634, -80.07754, 15.178016], 8),
            _ => return,
        };
        let odd = mode & 1 == 1;
        self.set(0x13c, if odd { f11 } else { -f11 });
        let t = &sin_tab()[1536..]; // DAT_00a64c00
        for i in 0..frames.max(0) as usize {
            let f8 = (f18 + self.g(0x1cc)) % 1.0;
            self.set(0x1cc, f8);
            let k = (f8 * 2048.0).floor() as i64 as i32;
            let t0 = t[k as usize];
            let t1 = t[k as usize + 1];
            let f14 = f13 + self.g(0x138);
            self.set(0x138, f14);
            self.set(0x140, f4 + self.g(0x140));
            let wet_old = self.g(0x144);
            self.set(0x144, f10 + wet_old);
            let prev = self.g(0x1d0);
            let c = if self.ours & OURS_LFO != 0 { ((t1 - t0) * (f8 * 2048.0 - k as f32) + t0 + 0.5) * f16 } else { (t0 + 0.5 + (t1 - t0) * (f8 * 2048.0 - k as f32)) * f16 };
            self.set(0x1d0, c);
            let v = if self.ours & OURS_V != 0 { c * f14 + ((f14 + c) - prev) } else { (f14 - prev) + c + c * f14 };
            let v = (((v * poly[0] + poly[1]) * v + poly[2]) * v + poly[3]) * v - 1.0;
            let coef = ((v + 1.0).abs() - (v - 1.0).abs()) * 0.5;
            let s = Self::sat(f12 * x[i]);
            let u = Self::sat(s - self.g(0x1b4) * self.g(0x13c));
            self.set(0x16c, u);
            let mut inp = u;
            for st in 0..stages {
                let y = (inp - self.g(0x170 + 4 * st)) * coef + self.g(0x148 + 4 * st);
                self.set(0x170 + 4 * st, y);
                self.set(0x148 + 4 * st, inp);
                inp = y;
            }
            let fb = inp - self.g(0x1b8);
            self.set(0x1b4, fb);
            self.set(0x1b8, f14 * 0.1 * fb + self.g(0x1b8));
            let last = self.g(0x170 + 4 * (stages - 1));
            out[i] = if odd { s * self.g(0x140) - (f10 + wet_old) * last } else { s * self.g(0x140) + (f10 + wet_old) * last };
        }
    }
}

fn signals(n: usize, sr: f32) -> Vec<(&'static str, Vec<f32>, Vec<f32>)> {
    let mut imp = vec![0.0f32; n];
    imp[0] = 1.0;
    let imp_r = imp.iter().map(|v| v * 0.5).collect();
    let (f0, f1) = (100.0f64, 10_000.0f64);
    let dur = n as f64 / sr as f64;
    let k = (f1 / f0).ln();
    let sweep: Vec<f32> = (0..n)
        .map(|i| {
            let t = i as f64 / sr as f64;
            (0.8 * (2.0 * std::f64::consts::PI * f0 * dur / k * ((t / dur * k).exp() - 1.0)).sin()) as f32
        })
        .collect();
    let sweep_r: Vec<f32> = sweep.iter().map(|v| -0.7 * v).collect();
    let mut s = 0x1234_5678u32;
    let mut rnd = || {
        s ^= s << 13;
        s ^= s >> 17;
        s ^= s << 5;
        (s as f32 / u32::MAX as f32) * 2.0 - 1.0
    };
    let nl: Vec<f32> = (0..n).map(|_| rnd()).collect();
    let nr: Vec<f32> = (0..n).map(|_| rnd()).collect();
    vec![("impulse", imp, imp_r), ("sweep", sweep, sweep_r), ("noise", nl, nr)]
}

fn settings() -> Vec<[f32; 9]> {
    // Drive, Mode, Pitch, Res, Freq, Depth, WetMix, PM, FM
    let base = [1.0f32, 5.0, 0.0, 0.5, 0.1, 0.2, 0.5, 0.0, 0.0];
    let ranges: [&[f32]; 9] = [
        &[0.0, 0.5, 2.0, 3.0],
        &[0.0, 1.0, 2.0, 3.0, 4.0, 6.0, 7.0, 8.0, 2.4, 2.5, 6.6],
        &[-3.0, -1.5, 1.0, 2.2, 3.0],
        &[0.0, 0.25, 0.9, 1.0, 1.01],
        &[0.0, 1.0, 2.5, 5.0],
        &[0.0, 0.5, 1.0],
        &[0.0, 0.3, 1.0],
        &[1.0],
        &[1.0],
    ];
    let mut out = vec![base];
    for (port, vals) in ranges.iter().enumerate() {
        for v in *vals {
            let mut s = base;
            s[port] = *v;
            out.push(s);
        }
    }
    for mode in 0..8 {
        out.push([2.0, mode as f32, 1.5, 1.01, 3.0, 1.0, 0.8, 0.0, 0.0]);
        out.push([0.7, mode as f32, -2.0, 0.9, 0.5, 0.6, 1.0, 0.0, 0.0]);
    }
    out
}

/// Runs every case with the transcription in `ours` order; returns
/// (differing cases, total, max abs diff, first reports).
fn compare(ours: u32, n: usize) -> (usize, usize, f32, Vec<String>) {
    let mut worst = 0.0f32;
    let mut first = Vec::new();
    let (mut cases, mut differing) = (0, 0);
    for sr in [44_100.0f32, 48_000.0] {
        let isr = 1.0 / sr;
        let ctx = Ctx::new(sr);
        let sets = settings();
        for (name, l, r) in signals(n, sr) {
            for (si, s) in sets.iter().enumerate() {
                for block in [64usize, 512] {
                    // Also one run that switches settings every half
                    // (mode-change reset, ramps from non-default gains).
                    for switch in [false, true] {
                        cases += 1;
                        let mut v1 = [V1::new(ours), V1::new(ours)];
                        let mut node = Phaser::new();
                        let (mut al, mut ar) = (vec![0.0f32; n], vec![0.0f32; n]);
                        let (mut bl, mut br) = (vec![0.0f32; n], vec![0.0f32; n]);
                        for start in (0..n).step_by(block) {
                            let end = (start + block).min(n);
                            let cur = if switch && start >= n / 2 { &sets[(si * 7 + 3) % sets.len()] } else { s };
                            for (i, v) in cur.iter().enumerate() {
                                node.ports()[i].value = *v;
                            }
                            v1[0].process_buffer(isr, ctx.tune, cur, &l[start..end], &mut al[start..end]);
                            v1[1].process_buffer(isr, ctx.tune, cur, &r[start..end], &mut ar[start..end]);
                            node.process(&ctx, [&l[start..end], &r[start..end]], [&mut bl[start..end], &mut br[start..end]]);
                        }
                        let mut case_max = 0.0f32;
                        let mut first_idx = None;
                        for i in 0..n {
                            for (a, b) in [(al[i], bl[i]), (ar[i], br[i])] {
                                if a.to_bits() != b.to_bits() {
                                    first_idx.get_or_insert(i);
                                    case_max = case_max.max((a - b).abs());
                                }
                            }
                        }
                        if let Some(i) = first_idx {
                            differing += 1;
                            worst = worst.max(case_max);
                            if first.len() < 30 {
                                first.push(format!("sr {sr} {name} set {si} {s:?} block {block} switch {switch}: first diff at {i} ({} vs {}), max {case_max:e}", al[i], bl[i]));
                            }
                        }
                    }
                }
            }
        }
    }
    (differing, cases, worst, first)
}

#[test]
fn phaser_matches_the_v1_transcription() {
    dreist_dsp::fx::install_exp2(einst_dsp::fast_exp2);
    let (differing, cases, worst, first) = compare(0, 12_000);
    for f in &first {
        println!("{f}");
    }
    println!("{differing}/{cases} cases differ, max abs diff {worst:e}");
    assert_eq!(differing, 0, "{differing}/{cases} cases differ, max abs diff {worst:e}");
}
