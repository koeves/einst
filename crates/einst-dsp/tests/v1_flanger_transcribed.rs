//! v1's Flanger voice transcribed from the decompiled code (constructor
//! ev_ef1c8512, UpdateSampleRate ev_1cfc6d7b, Reset ev_90cf0902,
//! ProcessBuffer ev_0503858b, ports ev_abe7246f), operation for operation,
//! against the shared node. One v1 voice holds both channels (In1/In2,
//! two rings), so the transcription is stereo like the node.

use dreist_dsp::analog::sin_tab;
use dreist_dsp::fx::{Ctx, Node};
use einst_dsp::flanger::Flanger;

/// The voice's fields by offset.
struct V1 {
    idx: u32,       // +0x138
    max_delay: f32, // +0x13c (stream setup, not in the transcribed files)
    d_l: f32,       // +0x140
    d_r: f32,       // +0x144
    dc_l: f32,      // +0x148
    dc_r: f32,      // +0x14c
    phase: f32,     // +0x150
    buf_l: Vec<f32>,
    mask_l: u32, // +0x1b0 / +0x1bc
    buf_r: Vec<f32>,
    mask_r: u32, // +0x1d0 / +0x1dc
    alt: f32,    // +0x1e8
    k_dc: f32,   // +0x1ec
    time_out: f32,
    /// Diagnostic switches: use the node's operation order instead of v1's
    /// for the LFO read, the delay smoother and the cubic read.
    node_order: [bool; 3],
}

impl V1 {
    fn new(sr: f32) -> V1 {
        // Ring size / max delay as v1's stream setup (0x564020) computes
        // them from the current rate.
        let m = sr * 0.01;
        let m = ((m + 2.0) + (m - 2.0).abs()) * 0.5;
        let n = (m + m + 4.0 + 0.5) as i64;
        let size = (n as usize).next_power_of_two();
        V1 {
            idx: 0,
            max_delay: m,
            d_l: 0.0,
            d_r: 0.0,
            dc_l: 0.0,
            dc_r: 0.0,
            phase: 0.0,
            buf_l: vec![0.0; size],
            mask_l: size as u32 - 1,
            buf_r: vec![0.0; size],
            mask_r: size as u32 - 1,
            alt: f32::from_bits(0x322bcc77),
            k_dc: 0.0,
            time_out: 0.0,
            node_order: [false; 3],
        }
    }

    fn update_sample_rate(&mut self, isr: f32) {
        self.k_dc = ((isr * 251.32742 + 1.0) - (isr * 251.32742 - 1.0).abs()) * 0.5;
    }

    #[allow(dead_code)]
    fn reset(&mut self) {
        self.idx = 0;
        self.phase = 0.0;
        self.dc_l = 0.0;
        self.dc_r = 0.0;
        self.d_l = 0.0;
        self.d_r = 0.0;
        self.buf_l.fill(0.0);
        self.buf_r.fill(0.0);
    }

    #[allow(clippy::too_many_arguments)]
    fn process_buffer(&mut self, sr: f32, isr: f32, p: &[f32; 8], in_l: &[f32], in_r: &[f32], out_l: &mut [f32], out_r: &mut [f32]) {
        let t = sin_tab();
        let tab = |i: i32| t[(1536 + i) as usize]; // DAT_00a64c00
        let tab1 = |i: i32| t[(1537 + i) as usize]; // DAT_00a64c04
        let frames = in_l.len();
        let mut f24 = sr * 0.5 * (p[2].abs() + p[2]);
        let mut f28 = self.max_delay;
        f24 = ((f24.abs() + f28) - (f24 - f28).abs()) * 0.5;
        let f31 = ((p[1].abs() + 1.0) - (p[1] - 1.0).abs()) * f28 * 0.5;
        f28 = p[4] * -0.5 + 0.5;
        let f21 = ((f28.abs() + 1.0) - (f28 - 1.0).abs()) * 0.5;
        f28 = p[5];
        let mut f22 = ((p[7].abs() + 1.0) - (p[7] - 1.0).abs()) * 0.5;
        let f23 = 1.0 - f22;
        let mut local_88 = 1.0f32;
        if ((((p[6].abs() + 1.0) - (p[6] - 1.0).abs()) * 0.5 + 0.5) as i32) != 0 {
            local_88 = -1.0;
        }
        let f26 = (((p[3].abs() + 1.0) - (p[3] - 1.0).abs()) * 0.5).sqrt();
        let f25 = (f26 * -0.95 + 1.0).sqrt();
        let mut f17 = isr * 0.5 * ((p[0].abs() + 5.0) - (p[0] - 5.0).abs());
        let f32_ = ((f17.abs() + 1.0) - (f17 - 1.0).abs()) * 0.5;
        f17 = f32_ * 8.0;
        let f29 = ((f17 + 1.0) - (f17 - 1.0).abs()) * 0.5;
        f17 = -self.alt;
        self.alt = f17;
        self.d_l += f17;
        let mut local_b0 = self.d_r + f17;
        self.d_r = local_b0;
        self.dc_l += f17;
        self.dc_r = f17 + self.dc_r;
        if frames > 0 {
            f22 *= 0.5;
            let mut i = 0usize;
            loop {
                let u11 = (self.idx.wrapping_add(1)) & self.mask_l;
                self.idx = u11;
                let f18 = (f32_ + self.phase) % 1.0;
                self.phase = f18;
                let i15 = (f18 * 2048.0).floor() as i64 as i32;
                let f17a = tab(i15);
                let f19 = ((((f28.abs() + 1.0) - (f28 - 1.0).abs()) * 0.5) + f18) % 1.0;
                let i12 = (f19 * 2048.0).floor() as i64 as i32;
                let f20a = tab(i12);
                let f30a = tab1(i12);
                let x1 = in_l[i];
                let x2 = in_r[i];
                let o = self.node_order;
                let rl = if o[0] { (tab1(i15) - f17a) * (f18 * 2048.0 - i15 as f32) + f17a + 0.5 } else { f17a + 0.5 + (tab1(i15) - f17a) * (f18 * 2048.0 - i15 as f32) };
                let rr = if o[0] { (f30a - f20a) * (f19 * 2048.0 - i12 as f32) + f20a + 0.5 } else { f20a + 0.5 + (f30a - f20a) * (f19 * 2048.0 - i12 as f32) };
                let dl = if o[1] { ((rl * f31 + f24) - self.d_l) * f29 + self.d_l } else { ((rl * f31 - self.d_l) + f24) * f29 + self.d_l };
                self.d_l = dl;
                local_b0 = if o[1] { ((rr * f31 + f24) - local_b0) * f29 + local_b0 } else { ((rr * f31 - local_b0) + f24) * f29 + local_b0 };
                self.d_r = local_b0;

                let mut fr = ((u11 as i32) as f32 - dl) - 2.0;
                let u14 = fr.floor() as i64 as u32;
                fr -= (u14 as i32) as f32;
                let m3 = self.mask_l;
                let b = &self.buf_l;
                let a0 = b[(u14.wrapping_sub(1) & m3) as i32 as usize];
                let a1 = b[(u14 & m3) as i32 as usize];
                let a2 = b[(u14.wrapping_add(1) & m3) as i32 as usize];
                let a3 = b[(u14.wrapping_add(2) & m3) as i32 as usize];
                let head = if o[2] { ((a2 - a0 * 0.33333334) - a1 * 0.5) - a3 * 0.16666667 } else { a1 * -0.5 + a2 + a0 * -0.33333334 + a3 * -0.16666667 };
                let yl = ((head + (((a2 + a0) * 0.5 - a1) + ((a1 - a2) * 0.5 + (a3 - a0) * 0.16666667) * fr) * fr) * fr + a1) * local_88;

                let mut f27 = ((u11 as i32) as f32 - local_b0) - 2.0;
                let u13 = f27.floor() as i64 as u32;
                f27 -= (u13 as i32) as f32;
                let mr = self.mask_r;
                let b = &self.buf_r;
                let a0 = b[(u13.wrapping_sub(1) & mr) as i32 as usize];
                let a1 = b[(u13 & mr) as i32 as usize];
                let a2 = b[(u13.wrapping_add(1) & mr) as i32 as usize];
                let a3 = b[(u13.wrapping_add(2) & mr) as i32 as usize];
                let head = if o[2] { ((a2 - a0 * 0.33333334) - a1 * 0.5) - a3 * 0.16666667 } else { a1 * -0.5 + a2 + a0 * -0.33333334 + a3 * -0.16666667 };
                let yr = ((head + (((a2 + a0) * 0.5 - a1) + ((a1 - a2) * 0.5 + (a3 - a0) * 0.16666667) * f27) * f27) * f27 + a1) * local_88;

                let mut s1 = f26 * yl + x1 * f25;
                s1 = ((s1 + 2.0).abs() - (s1 - 2.0).abs()) * 0.5;
                let mut s2 = f26 * yr + x2 * f25;
                s2 = ((s2 + 2.0).abs() - (s2 - 2.0).abs()) * 0.5;
                out_l[i] = ((1.0 - f21) * yl + f21 * yr + self.buf_l[(u11.wrapping_sub(2) & m3) as i32 as usize]) * f22 + f23 * x1;
                out_r[i] = (yl * f21 + yr * (1.0 - f21) + self.buf_r[(self.idx.wrapping_sub(2) & self.mask_r) as i32 as usize]) * f22 + f23 * x2;
                let w1 = ((s1.abs() * 0.0625 - 0.25) * s1 * s1 + 1.0) * s1 - self.dc_l;
                let w2 = ((s2.abs() * 0.0625 - 0.25) * s2 * s2 + 1.0) * s2 - self.dc_r;
                self.dc_l = w1 * self.k_dc + self.dc_l;
                self.dc_r = w2 * self.k_dc + self.dc_r;
                self.buf_l[(self.idx & self.mask_l) as i32 as usize] = w1;
                self.buf_r[(self.idx & self.mask_r) as i32 as usize] = w2;
                let it = i as i32;
                i += 1;
                if frames as i32 <= it + 1 {
                    break;
                }
                local_b0 = self.d_r;
            }
        }
        self.time_out = self.d_l * isr * 50.0;
    }
}

fn signals(n: usize, sr: f32) -> Vec<(&'static str, Vec<f32>, Vec<f32>)> {
    let mut imp = vec![0.0f32; n];
    imp[0] = 1.0;
    let imp_r = imp.clone();
    // Exponential sine sweep 100 Hz .. 10 kHz (f64 phase, f32 samples).
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

fn settings() -> Vec<[f32; 8]> {
    // Freq, Depth, Pos, Feedback, Spread, Phase, Mode, WetMix
    let base = [0.1f32, 0.2, 0.0, 0.2, 1.0, 0.5, 0.0, 1.0];
    let ranges: [[f32; 5]; 8] = [
        [0.0, 2.5, 5.0, 7.3, -1.0],
        [0.0, 0.5, 1.0, 1.4, -0.3],
        [0.0, 0.005, 0.01, 0.03, -0.004],
        [0.0, 0.5, 1.0, 1.7, -0.6],
        [0.0, 0.5, 1.0, 1.2, -0.1],
        [0.0, 0.25, 1.0, 1.6, -0.2],
        [0.0, 1.0, 2.0, 0.4, 0.6],
        [0.0, 0.5, 1.0, 1.3, -0.2],
    ];
    let mut out = vec![base];
    for (port, vals) in ranges.iter().enumerate() {
        for v in vals {
            let mut s = base;
            s[port] = *v;
            out.push(s);
        }
    }
    // Combined extremes.
    out.push([5.0, 1.0, 0.01, 1.0, 0.0, 1.0, 1.0, 0.5]);
    out.push([2.5, 0.5, 0.005, 0.5, 0.5, 0.25, 0.0, 0.5]);
    out.push([0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
    out.push([4.0, 1.0, 0.0, 0.95, 0.3, 0.75, 1.0, 1.0]);
    out
}

fn compare(node_order: [bool; 3]) -> (usize, usize, f32, Vec<String>) {
    let n = 24_000;
    let mut worst = 0.0f32;
    let mut first: Vec<String> = Vec::new();
    let mut cases = 0;
    let mut differing = 0;
    for sr in [44_100.0f32, 48_000.0] {
        let isr = 1.0 / sr;
        let ctx = Ctx::new(sr);
        for (name, l, r) in signals(n, sr) {
            for (si, s) in settings().iter().enumerate() {
                for block in [64usize, 512] {
                    cases += 1;
                    let mut v1 = V1::new(sr);
                    v1.node_order = node_order;
                    v1.update_sample_rate(isr);
                    let mut node = Flanger::new();
                    node.prepare_rate(sr);
                    for (i, v) in s.iter().enumerate() {
                        node.ports()[i].value = *v;
                    }
                    let (mut al, mut ar) = (vec![0.0f32; n], vec![0.0f32; n]);
                    let (mut bl, mut br) = (vec![0.0f32; n], vec![0.0f32; n]);
                    for start in (0..n).step_by(block) {
                        let end = (start + block).min(n);
                        v1.process_buffer(sr, isr, s, &l[start..end], &r[start..end], &mut al[start..end], &mut ar[start..end]);
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
                        if first.len() < 40 {
                            first.push(format!("sr {sr} {name} set {si} {s:?} block {block}: first diff at {i} ({} vs {}), max {case_max:e}", al[i], bl[i]));
                        }
                    }
                }
            }
        }
    }
    (differing, cases, worst, first)
}

/// Einst's Flanger is v1's: the faithful transcription renders
/// bit-identically.
#[test]
fn flanger_matches_the_v1_transcription() {
    let (differing, cases, worst, first) = compare([false; 3]);
    assert_eq!(differing, 0, "{differing}/{cases} cases differ, max abs diff {worst:e}: {:?}", first.iter().take(5).collect::<Vec<_>>());
}
