//! v1's nodes against the decompiled formulas (dreist-re `einst/docs/re/loading-v1.md`).

use dreist_dsp::fx::{Ctx, Node};
use einst_dsp::envelope_shaper::EnvelopeShaper;
use einst_dsp::gain::Gain;
use einst_dsp::vari_eq::{Filter, VariEq};

fn run(node: &mut dyn Node, ctx: &Ctx, l: &[f32], r: &[f32]) -> (Vec<f32>, Vec<f32>) {
    let (mut ol, mut or) = (vec![0.0; l.len()], vec![0.0; r.len()]);
    for ((il, ir), (ol, or)) in l.chunks(32).zip(r.chunks(32)).zip(ol.chunks_mut(32).zip(or.chunks_mut(32))) {
        node.process(ctx, [il, ir], [ol, or]);
    }
    (ol, or)
}

/// [re:a21df5]: a linear ramp from the last gain to
/// max(fast2^x − 0.016367754, 0) across the block, then flat.
#[test]
fn gain_ramps_to_the_fast_exp2_level() {
    let ctx = Ctx::new(44_100.0);
    let mut g = Gain::new();
    g.ports()[0].value = 1.3;
    let target = (einst_dsp::fast_exp2(1.3) - 0.016367754).max(0.0);
    assert_eq!(Gain::target(1.3), target);
    // v1's approximation, not Dreist's exact exp2.
    assert!((target - (1.3f32.exp2() - 0.016367754)).abs() > 1e-5);
    let ones = vec![1.0f32; 64];
    let (l, r) = run(&mut g, &ctx, &ones, &ones);
    for i in 0..32 {
        let want = target * (i + 1) as f32 / 32.0;
        assert!((l[i] - want).abs() < 1e-5, "{i}: {} vs {want}", l[i]);
    }
    assert!(l[32..].iter().chain(&r[32..]).all(|v| (v - target).abs() < 1e-5));
    // Below the floor: silence (the clamp at -10 gives 2^-10 < 0.0164).
    g.ports()[0].value = -20.0;
    assert_eq!(Gain::target(-20.0), 0.0);
}

/// The RBJ cookbook forms v1's `SetCoeffs` cases compute, written out
/// independently: w = 2π·clamp(f, 0.001, 0.45), Q clamped 0.5..1000,
/// A = 10^(g/40).
fn cookbook(mode: i32, f: f64, q: f64, g: f64) -> [f64; 5] {
    let w = 2.0 * std::f64::consts::PI * f.clamp(0.001, 0.45);
    let q = q.clamp(0.5, 1000.0);
    let (sn, cs) = (w.sin(), w.cos());
    let al = sn / (2.0 * q);
    let a = 10f64.powf(g / 40.0);
    let (b, aa): ([f64; 3], [f64; 3]) = match mode {
        0 => ([1.0 + al * a, -2.0 * cs, 1.0 - al * a], [1.0 + al / a, -2.0 * cs, 1.0 - al / a]),
        1 => {
            let s = 2.0 * a.sqrt() * al;
            (
                [a * ((a + 1.0) - (a - 1.0) * cs + s), 2.0 * a * ((a - 1.0) - (a + 1.0) * cs), a * ((a + 1.0) - (a - 1.0) * cs - s)],
                [(a + 1.0) + (a - 1.0) * cs + s, -2.0 * ((a - 1.0) + (a + 1.0) * cs), (a + 1.0) + (a - 1.0) * cs - s],
            )
        }
        2 => {
            let s = 2.0 * a.sqrt() * al;
            (
                [a * ((a + 1.0) + (a - 1.0) * cs + s), -2.0 * a * ((a - 1.0) + (a + 1.0) * cs), a * ((a + 1.0) + (a - 1.0) * cs - s)],
                [(a + 1.0) - (a - 1.0) * cs + s, 2.0 * ((a - 1.0) - (a + 1.0) * cs), (a + 1.0) - (a - 1.0) * cs - s],
            )
        }
        3 => ([(1.0 - cs) / 2.0, 1.0 - cs, (1.0 - cs) / 2.0], [1.0 + al, -2.0 * cs, 1.0 - al]),
        4 => ([sn / 2.0, 0.0, -sn / 2.0], [1.0 + al, -2.0 * cs, 1.0 - al]),
        5 => ([al, 0.0, -al], [1.0 + al, -2.0 * cs, 1.0 - al]),
        6 => ([(1.0 + cs) / 2.0, -(1.0 + cs), (1.0 + cs) / 2.0], [1.0 + al, -2.0 * cs, 1.0 - al]),
        7 => ([1.0, -2.0 * cs, 1.0], [1.0 + al, -2.0 * cs, 1.0 - al]),
        _ => ([1.0 - al, -2.0 * cs, 1.0 + al], [1.0 + al, -2.0 * cs, 1.0 - al]),
    };
    [b[0] / aa[0], b[1] / aa[0], b[2] / aa[0], aa[1] / aa[0], aa[2] / aa[0]]
}

#[test]
fn vari_eq_coefficients_are_the_cookbook_with_v1_clamps() {
    for mode in 0..9 {
        for (f, q, g) in [(0.01f32, 0.7f32, 6.0f32), (0.2, 2.5, -9.0), (0.0004, 1.0, 3.0), (0.47, 4.0, 15.0)] {
            let mut flt = Filter::default();
            flt.set(mode, f, q, g);
            let got = [flt.b0, flt.b1, flt.b2, flt.a1, flt.a2];
            // v1 computes the gain factor from g·0.025 in float.
            let want = cookbook(mode, f as f64, q as f64, (g * 0.025) as f64 * 40.0);
            for k in 0..5 {
                assert!((got[k] - want[k]).abs() < 1e-6, "mode {mode} f {f} coef {k}: {} vs {}", got[k], want[k]);
            }
        }
    }
}

/// `Voice::ProcessBuffer`: the x history carries an alternating ±1e-8.
#[test]
fn vari_eq_feeds_an_alternating_denormal_offset() {
    let mut flt = Filter::default();
    // An allpass-free check: a bell at 0 dB is the identity filter.
    flt.set(0, 0.05, 1.0, 0.0);
    let x = [0.25f32, -0.5, 0.75, 0.0, 0.0, 0.0];
    let y: Vec<f32> = x.iter().map(|v| flt.tick(*v)).collect();
    for (a, b) in x.iter().zip(&y) {
        assert!((a - b).abs() < 1e-6, "{a} vs {b}");
    }
    // Silence in: the offset keeps the output from settling to exact zero.
    let mut z = Filter::default();
    z.set(3, 0.05, 0.7, 0.0);
    let tail: Vec<f32> = (0..8).map(|_| z.tick(0.0)).collect();
    assert!(tail.iter().any(|v| *v != 0.0) && tail.iter().all(|v| v.abs() < 1e-7), "{tail:?}");
}

#[test]
fn vari_eq_node_maps_ports_like_v1() {
    let mut eq = VariEq::new();
    let ports = eq.ports();
    ports[0].value = 9.3;
    ports[1].value = 0.5;
    ports[2].value = 1.0;
    ports[3].value = 4.0;
    let (mode, f, q, g) = VariEq::coeff_args(ports, 48_000.0);
    assert_eq!((mode, q, g), (8, 1.0, 4.0));
    assert!((f - einst_dsp::fast_exp2(1.5) * 400.0 / 48_000.0).abs() < 1e-9);
    let ctx = Ctx::new(48_000.0);
    let noise: Vec<f32> = (0..256).map(|i| ((i * 7919) % 97) as f32 / 97.0 - 0.5).collect();
    let (l, r) = run(&mut eq, &ctx, &noise, &noise);
    assert!(l.iter().all(|v| v.is_finite()));
    assert_eq!(l, r, "the same input through two voices gives the same output");
}

/// v1 runs the Envelope Shaper as two mono instances: "Linked" never links
/// left and right. With Linked on, v1's node equals Dreist's with it off.
#[test]
fn envelope_shaper_never_links_channels() {
    use dreist_dsp::fx::nodes::envelope_shaper::EnvelopeShaper as V2;
    let ctx = Ctx::new(44_100.0);
    let set = |n: &mut dyn Node, linked: f32| {
        for (i, v) in [1.0, 0.9, 0.2, 0.6, 0.5, linked].iter().enumerate() {
            n.ports()[i].value = *v;
        }
    };
    let l: Vec<f32> = (0..4096).map(|i| if i % 1024 < 40 { 0.9 } else { 0.05 }).collect();
    let r: Vec<f32> = (0..4096).map(|i| if (i + 512) % 1024 < 40 { 0.9 } else { 0.05 }).collect();
    let mut v1 = EnvelopeShaper::new();
    set(&mut v1, 1.0);
    let (mut unlinked, mut linked) = (V2::new(), V2::new());
    set(&mut unlinked, 0.0);
    set(&mut linked, 1.0);
    let a = run(&mut v1, &ctx, &l, &r);
    let b = run(&mut unlinked, &ctx, &l, &r);
    let c = run(&mut linked, &ctx, &l, &r);
    assert_eq!(a, b);
    assert_ne!(a, c, "v2's linked detector differs");
}

fn noise(n: usize, seed: u32) -> Vec<f32> {
    let mut s = seed;
    (0..n)
        .map(|_| {
            s = s.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            (s >> 8) as f32 / (1u32 << 24) as f32 - 0.5
        })
        .collect()
}

/// v1's Noise Gate ports in v1 order reach Dreist's gate at v2's indices.
#[test]
fn noise_gate_takes_v1_port_order() {
    use dreist_dsp::fx::nodes::noise_gate::NoiseGate as V2;
    use einst_dsp::noise_gate::{NoiseGate, V1_TO_V2};
    let ctx = Ctx::new(44_100.0);
    let mut v1 = NoiseGate::new();
    let mut g2 = V2::new();
    // v1 values: Attack, Hold, Release, Threshold, Hysteresis, LPFreq, HPFreq, Preview, Sidechain, WetMix.
    let vals = [0.002f32, 0.05, 0.2, 0.1, 0.5, 8000.0, 30.0, 0.0, 0.0, 1.0];
    for (i, v) in vals.iter().enumerate() {
        v1.ports()[i].value = *v;
        g2.ports()[V1_TO_V2[i]].value = *v;
    }
    let (l, r) = (noise(4096, 3), noise(4096, 4));
    assert_eq!(run(&mut v1, &ctx, &l, &r), run(&mut g2, &ctx, &l, &r));
    // The v1 names sit where v1 puts them.
    assert_eq!(einst_dsp::internal_ports(einst_dsp::NOISE_GATE), &[(6, 0.0), (7, 0.0), (8, 0.0), (9, 1.0)]);
}

/// v1's Channel Compressor: Dreist's circuit (v1's solvers agree with
/// v2's to float-folding precision) with a flat per-block input gain and
/// v1's fast 2^x for attack and release.
#[test]
fn channel_compressor_matches_dreist_but_for_gain_ramp_and_fast_exp2() {
    use dreist_dsp::fx::nodes::urei1176::Urei1176 as V2;
    let ctx = Ctx::new(48_000.0);
    let mut v1 = einst_dsp::urei1176::Urei1176::new();
    let mut g2 = V2::new();
    // 0 In dB, 1 Attack, 2 Release, 3 Q, 5 Ratio, 6 Out dB, 7 Mix, 8 Active.
    // In at 0 dB: v2's input ramp starts at 0 dB, so only v1's solvers and
    // the fast 2^x (compensated on v2's side) remain.
    let set = [(0usize, 0.0f32), (1, 0.4), (2, 1.6), (3, 0.5), (5, 2.0), (6, 3.0), (7, 1.0), (8, 1.0)];
    for (i, v) in set {
        v1.ports()[i].value = v;
        g2.ports()[i].value = if i == 1 || i == 2 { einst_dsp::fast_exp2(v - 1.0).log2() + 1.0 } else { v };
    }
    let n = 32;
    let sig: Vec<f32> = (0..n * 200).map(|i| (i as f32 * std::f32::consts::TAU * 180.0 / 48_000.0).sin() * if i < n * 60 { 0.1 } else { 2.0 }).collect();
    let (mut worst, mut peak) = (0.0f32, 0.0f32);
    for x in sig.chunks(n) {
        let (mut l1, mut r1, mut l2, mut r2) = (vec![0.0; n], vec![0.0; n], vec![0.0; n], vec![0.0; n]);
        v1.process(&ctx, [x, x], [&mut l1, &mut r1]);
        g2.process(&ctx, [x, x], [&mut l2, &mut r2]);
        worst = worst.max(l1.iter().zip(&l2).map(|(a, b)| (a - b).abs()).fold(0.0f32, f32::max));
        peak = peak.max(l2.iter().fold(0.0f32, |m, v| m.max(v.abs())));
        assert!(l1.iter().all(|v| v.is_finite()));
    }
    // Both solvers stop at a 0.1 % step, so folding differences can shift an
    // iteration: within 0.1 % of the output level.
    assert!(worst < peak * 1e-3, "{worst} of {peak}");
    // A gain change: v1 jumps to it for the whole block, v2 ramps.
    v1.ports()[0].value = 12.0;
    g2.ports()[0].value = 12.0;
    let x = &sig[..n];
    let (mut l1, mut r1, mut l2, mut r2) = (vec![0.0; n], vec![0.0; n], vec![0.0; n], vec![0.0; n]);
    v1.process(&ctx, [x, x], [&mut l1, &mut r1]);
    g2.process(&ctx, [x, x], [&mut l2, &mut r2]);
    assert!((l1[1] - l2[1]).abs() > 1e-4, "{} vs {}", l1[1], l2[1]);
}

/// v1's solvers equal v2's algebra: the FET stage's "off" shortcut is
/// 1/4.40125e6, and `vcrit` is ln(1.8228e13)·0.02577878 = 0.787129.
#[test]
fn channel_compressor_solver_constants() {
    let (mut a, mut b) = (0.0f32, -3.0f32);
    einst_dsp::urei1176::solve_1176a(&mut a, &mut b, 0.0, 0.0);
    assert!(a.is_finite() && b.is_finite());
    assert!((f32::from_bits(0x5584_a0f3).ln() * 0.02577878 - 0.7871292).abs() < 1e-6);
    assert!((1.0f32 / 4.40125e6 - 2.2720818e-07).abs() < 1e-13);
}

/// Comb Filter ports: 0 Drive, 1 Mode, 2 Pitch, 3 Feedback, 4 PM, 5 FM.
/// With PM on (the modulated path) v1's comb is Dreist's at the pitch whose
/// exact 2^(−pitch) is v1's fast one, up to float rounding (v1 sums the tap
/// and the modulated delay in another order).
#[test]
fn comb_differs_only_by_fast_exp2_and_rounding() {
    use dreist_dsp::fx::nodes::comb::Comb;
    let ctx = Ctx::new(48_000.0);
    // Where v1's fast 2^x is furthest off (0.38 %).
    let pitch = 1.51;
    let set = |n: &mut dyn Node, p: f32| {
        for (i, v) in [0.6, 1.0, p, 0.8, 0.3, 0.2].into_iter().enumerate() {
            n.ports()[i].value = v;
        }
    };
    let mut v1 = einst_dsp::comb::Comb::new();
    let (mut g2, mut g2_exact) = (Comb::new(), Comb::new());
    set(&mut v1, pitch);
    set(&mut g2, -einst_dsp::fast_exp2(-pitch).log2());
    set(&mut g2_exact, pitch);
    let (mut worst, mut worst_exact, mut peak) = (0.0f32, 0.0f32, 0.0f32);
    for b in 0..30 {
        let (l, r) = (noise(512, 20 + b), noise(512, 200 + b));
        let a = run(&mut v1, &ctx, &l, &r);
        let g = run(&mut g2, &ctx, &l, &r);
        let e = run(&mut g2_exact, &ctx, &l, &r);
        for k in 0..512 {
            worst = worst.max((a.0[k] - g.0[k]).abs()).max((a.1[k] - g.1[k]).abs());
            worst_exact = worst_exact.max((a.0[k] - e.0[k]).abs());
            peak = peak.max(a.0[k].abs());
        }
    }
    assert!(worst < peak * 1e-3, "worst {worst}, peak {peak}");
    assert!(worst_exact > worst * 10.0, "exact {worst_exact} vs {worst}");
}

fn render_blocks(n: &mut dyn Node, ports: &[(usize, f32)], blocks: u32) -> (Vec<f32>, Vec<f32>) {
    let ctx = Ctx::new(48_000.0);
    for &(i, v) in ports {
        n.ports()[i].value = v;
    }
    let (mut l, mut r) = (Vec::new(), Vec::new());
    for b in 0..blocks {
        let (x, y) = run(n, &ctx, &noise(256, 300 + b), &noise(256, 400 + b));
        l.extend(x);
        r.extend(y);
    }
    (l, r)
}

fn rms_diff(a: &(Vec<f32>, Vec<f32>), b: &(Vec<f32>, Vec<f32>)) -> (f32, f32) {
    let d = a.0.iter().zip(&b.0).map(|(x, y)| (x - y) * (x - y)).sum::<f32>();
    let e = a.0.iter().map(|x| x * x).sum::<f32>();
    ((d / a.0.len() as f32).sqrt(), (e / a.0.len() as f32).sqrt())
}

/// Grandad and Child Genius differ from Dreist's only by v1's fast 2^x
/// (cutoff, warp, drive): close, not equal.
#[test]
fn warped_filters_stay_close_to_dreist() {
    use dreist_dsp::fx::nodes::ladder::{Kind, Ladder};
    let ports = [(0, 0.3), (1, 3.0), (2, -2.3), (3, 0.4), (4, 0.2), (5, 0.0), (6, 0.0)];
    for (name, v1, v2) in [("Grandad", einst_dsp::ladder::Kind::Moog, Kind::Moog), ("Child Genius", einst_dsp::ladder::Kind::Prodigy, Kind::Prodigy)] {
        let a = render_blocks(&mut einst_dsp::ladder::Ladder::new(v1), &ports, 20);
        let b = render_blocks(&mut Ladder::new(v2), &ports, 20);
        let (d, e) = rms_diff(&a, &b);
        assert!(d > 0.0 && d < e * 0.2, "{name}: {d} vs {e}");
    }
}

/// Three Oh! Three: v1 clamps drive and output to 2.4 (3.0 sounds as 2.4);
/// Hoover: v1's mode 0 is its own row, v2's equals mode 1.
#[test]
fn tb303_drive_clamp_and_hoover_mode_0() {
    use dreist_dsp::fx::nodes::ladder::{Kind, Ladder};
    use einst_dsp::ladder::{Kind as V1Kind, Ladder as V1Ladder};
    let tb = |drive: f32| render_blocks(&mut V1Ladder::new(V1Kind::Tb303), &[(0, drive), (1, 1.0), (2, -2.0), (3, 0.3), (4, drive), (5, 0.0), (6, 0.0)], 10);
    assert_eq!(tb(3.0), tb(2.4));
    let dreist_tb = |drive: f32| render_blocks(&mut Ladder::new(Kind::Tb303), &[(0, drive), (1, 1.0), (2, -2.0), (3, 0.3), (4, drive), (5, 0.0), (6, 0.0)], 10);
    assert_ne!(dreist_tb(3.0), dreist_tb(2.4));
    let juno = |mode: f32| render_blocks(&mut V1Ladder::new(V1Kind::Juno), &[(0, 0.5), (1, mode), (2, -2.0), (3, 0.3), (4, 0.5), (5, 0.0), (6, 0.0)], 10);
    assert_ne!(juno(0.0), juno(1.0));
    let dreist_juno = |mode: f32| render_blocks(&mut Ladder::new(Kind::Juno), &[(0, 0.5), (1, mode), (2, -2.0), (3, 0.3), (4, 0.5), (5, 0.0), (6, 0.0)], 10);
    assert_eq!(dreist_juno(0.0), dreist_juno(1.0));
}

/// v1's gate opens on a key between the hysteresis level and the
/// threshold (v2's stays closed there once its hold has run out).
#[test]
fn noise_gate_opens_over_the_hysteresis_level() {
    use dreist_dsp::fx::nodes::noise_gate::NoiseGate;
    let ctx = Ctx::new(48_000.0);
    // v1 ports: 0 att, 1 hold, 2 rel, 3 thr, 4 hyst, 5 LP, 6 HP, 7 preview, 8 sc, 9 wet.
    let mut v1 = einst_dsp::noise_gate::NoiseGate::new();
    let mut v2 = NoiseGate::new();
    let v1_ports = [(0, -6.0), (1, -6.0), (2, -4.0), (3, 0.5), (4, 0.25), (5, 13.0), (6, 3.0), (7, 0.0), (8, 0.0), (9, 1.0)];
    for (i, v) in v1_ports {
        v1.ports()[i].value = v;
        v2.ports()[einst_dsp::noise_gate::V1_TO_V2[i]].value = v;
    }
    // A 1 kHz tone at 0.3: over hysteresis·threshold (0.125), under the threshold.
    let x: Vec<f32> = (0..4800).map(|i| (i as f32 * 2.0 * std::f32::consts::PI * 1000.0 / 48_000.0).sin() * 0.3).collect();
    let a = run(&mut v1, &ctx, &x, &x);
    let b = run(&mut v2, &ctx, &x, &x);
    let tail = |v: &[f32]| (v[2400..].iter().map(|s| s * s).sum::<f32>() / 2400.0).sqrt();
    assert!(tail(&a.0) > 0.15, "v1 open: {}", tail(&a.0));
    assert!(tail(&b.0) < 0.01, "v2 closed: {}", tail(&b.0));
}

/// EQ and Filter: v1's fast 2^x for the frequencies (integer exponents
/// match Dreist's), and a reset that keeps the filter tails.
#[test]
fn eq_and_filter_fast_exp2_and_tails_over_reset() {
    use dreist_dsp::fx::nodes::ssl_eq::SslEq;
    use dreist_dsp::fx::nodes::ssl_filter::SslFilter;
    // EQ band 1 (LMF): 4 Q, 5 freq, 6 gain, 7 on; wet 17.
    let eq = [(4, 0.5), (5, 9.0), (6, 0.8), (7, 1.0), (17, 1.0)];
    let a = render_blocks(&mut einst_dsp::ssl_eq::SslEq::new(), &eq, 10);
    let b = render_blocks(&mut SslEq::new(), &eq, 10);
    let (d, e) = rms_diff(&a, &b);
    assert!(d < e * 1e-4, "EQ integer exponent: {d} vs {e}");
    // Filter: 0 HP in, 1 HP freq, 2 LP in, 3 LP freq, 4 wet.
    let fl = [(0, 1.0), (1, 6.0), (2, 1.0), (3, 11.0), (4, 1.0)];
    let a = render_blocks(&mut einst_dsp::ssl_filter::SslFilter::new(), &fl, 10);
    let b = render_blocks(&mut SslFilter::new(), &fl, 10);
    let (d, e) = rms_diff(&a, &b);
    assert!(d < e * 1e-4, "Filter integer exponents: {d} vs {e}");
    // After a reset, silence in: v1's tails ring on, v2's do not.
    let ctx = Ctx::new(48_000.0);
    let tail = |n: &mut dyn Node| {
        let _ = render_blocks(n, &[], 4);
        n.reset();
        let z = vec![0.0f32; 64];
        run(n, &ctx, &z, &z).0.iter().map(|v| v.abs()).fold(0.0f32, f32::max)
    };
    let mut v1 = einst_dsp::ssl_filter::SslFilter::new();
    let mut v2 = SslFilter::new();
    for (i, v) in fl {
        v1.ports()[i].value = v;
        v2.ports()[i].value = v;
    }
    // (v2 leaves only its ±1e−8 denormal offset.)
    assert!(tail(&mut v1) > 1e-4);
    assert!(tail(&mut v2) < 1e-6);
}

/// Delay: v1 caps the time at 8 s at the current rate; with integer
/// exponents, no swing and a fixed time, it matches Dreist's.
#[test]
fn delay_max_length_and_fast_exp2() {
    use dreist_dsp::fx::nodes::delay::Delay;
    // 0 time, 1 mode, 2 feedback, 3 swing, 4 mono, 5 low cut, 6 high cut, 7 mix.
    let p = [(0, 0.0), (1, 0.0), (2, 0.5), (3, 0.0), (4, 0.0), (5, 5.0), (6, 12.0), (7, 0.7)];
    let a = render_blocks(&mut einst_dsp::delay::Delay::new(), &p, 30);
    let b = render_blocks(&mut Delay::new(), &p, 30);
    let (d, e) = rms_diff(&a, &b);
    assert!(d < e * 1e-3, "{d} vs {e}");
    // 10 s requested (2^(log2(10)+1−1)): v1 reaches its 8 s cap, v2 does not.
    let ctx = Ctx::new(48_000.0);
    let time = 10f32.log2() + 1.0;
    let mut v1 = einst_dsp::delay::Delay::new();
    let mut v2 = Delay::new();
    for (i, v) in [(0, time), (1, 0.0), (2, 0.0), (3, 0.0), (4, 0.0), (5, 0.0), (6, 13.0), (7, 1.0)] {
        v1.ports()[i].value = v;
        v2.ports()[i].value = v;
    }
    let impulse: Vec<f32> = (0..512).map(|i| if i == 0 { 1.0 } else { 0.0 }).collect();
    let z = vec![0.0f32; 512];
    let first_echo = |n: &mut dyn Node| {
        // The time glides to its target first; then one impulse.
        for _ in 0..2000 {
            run(n, &ctx, &z, &z);
        }
        let mut t = 0usize;
        for b in 0..1100 {
            let x = if b == 0 { &impulse } else { &z };
            let (l, _) = run(n, &ctx, x, x);
            if let Some(k) = l.iter().position(|v| v.abs() > 0.05) {
                if b > 0 || k > 0 {
                    t = b * 512 + k;
                    break;
                }
            }
        }
        t
    };
    let (t1, t2) = (first_echo(&mut v1), first_echo(&mut v2));
    assert!((t1 as f32 - 8.0 * 48_000.0).abs() < 600.0, "v1 echo at {t1}");
    assert!(t2 as f32 > 9.0 * 48_000.0 || t2 == 0, "v2 echo at {t2}");
}

/// Tin Can Verb: v1 has no crossfeed, so a left-only input leaves the
/// right output silent (v2 feeds 0.2 of each side into the other).
#[test]
fn tin_can_has_no_crossfeed() {
    use dreist_dsp::fx::nodes::tin_can::TinCan;
    let ctx = Ctx::new(48_000.0);
    let x = noise(512, 9);
    let z = vec![0.0f32; 512];
    let right = |n: &mut dyn Node| {
        n.ports()[6].value = 1.0;
        let mut m = 0.0f32;
        for _ in 0..20 {
            m = m.max(run(n, &ctx, &x, &z).1.iter().map(|v| v.abs()).fold(0.0, f32::max));
        }
        m
    };
    assert!(right(&mut einst_dsp::tin_can::TinCan::new()) < 1e-6);
    assert!(right(&mut TinCan::new()) > 1e-3);
}
