//! v1's nodes against the decompiled formulas (`docs/re/loading-v1.md`).

use einst_dsp::envelope_shaper::EnvelopeShaper;
use einst_dsp::gain::Gain;
use einst_dsp::vari_eq::{Filter, VariEq};
use geist_dsp::fx::{Ctx, Node};

fn run(node: &mut dyn Node, ctx: &Ctx, l: &[f32], r: &[f32]) -> (Vec<f32>, Vec<f32>) {
    let (mut ol, mut or) = (vec![0.0; l.len()], vec![0.0; r.len()]);
    for ((il, ir), (ol, or)) in l.chunks(32).zip(r.chunks(32)).zip(ol.chunks_mut(32).zip(or.chunks_mut(32))) {
        node.process(ctx, [il, ir], [ol, or]);
    }
    (ol, or)
}

/// `VAmpM::Voice::ProcessBuffer`: a linear ramp from the last gain to
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
    use geist_dsp::fx::nodes::envelope_shaper::EnvelopeShaper as Geist2;
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
    let (mut unlinked, mut linked) = (Geist2::new(), Geist2::new());
    set(&mut unlinked, 0.0);
    set(&mut linked, 1.0);
    let a = run(&mut v1, &ctx, &l, &r);
    let b = run(&mut unlinked, &ctx, &l, &r);
    let c = run(&mut linked, &ctx, &l, &r);
    assert_eq!(a, b);
    assert_ne!(a, c, "Geist2's linked detector differs");
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

/// The copied nodes differ from Dreist's only in the one 2^x: Dreist's
/// node fed log2(fast2^v) at that port gives v1's output.
fn matches_dreist_with_fast_exp2(mut v1: Box<dyn Node>, mut g2: Box<dyn Node>, exp_port: usize, set: &[(usize, f32)]) {
    let ctx = Ctx::new(48_000.0);
    for &(i, v) in set {
        v1.ports()[i].value = v;
        g2.ports()[i].value = if i == exp_port { einst_dsp::fast_exp2(v).log2() } else { v };
    }
    let (l, r) = (noise(2048, 1), noise(2048, 2));
    let a = run(v1.as_mut(), &ctx, &l, &r);
    let b = run(g2.as_mut(), &ctx, &l, &r);
    let worst = a.0.iter().chain(&a.1).zip(b.0.iter().chain(&b.1)).map(|(x, y)| (x - y).abs()).fold(0.0f32, f32::max);
    assert!(worst < 1e-4, "worst difference {worst}");
    // And it is not Dreist's exact exp2.
    let mut g2_exact = g2;
    for &(i, v) in set {
        g2_exact.ports()[i].value = v;
    }
    let c = run(g2_exact.as_mut(), &ctx, &l, &r);
    assert_ne!(a, c);
}

#[test]
fn phaser_differs_only_by_fast_exp2() {
    use geist_dsp::fx::nodes::phaser::Phaser;
    // Ports: 0 Drive, 1 Mode, 2 Pitch (the 2^x), 3 Resonance, 4 Rate, 5 Depth, 6 Mix.
    let set = [(0, 1.0), (1, 2.0), (2, 1.37), (3, 0.6), (4, 1.0), (5, 0.7), (6, 1.0), (7, 0.0), (8, 0.0)];
    matches_dreist_with_fast_exp2(Box::new(einst_dsp::phaser::Phaser::new()), Box::new(Phaser::new()), 2, &set);
}

#[test]
fn freq_shifter_differs_only_by_fast_exp2() {
    use geist_dsp::fx::nodes::freq_shifter::FreqShifter;
    let set = [(0, 0.63), (1, 1.0), (2, 1.0)];
    matches_dreist_with_fast_exp2(Box::new(einst_dsp::freq_shifter::FreqShifter::new()), Box::new(FreqShifter::new()), 0, &set);
}

#[test]
fn stereo_width_gain_uses_fast_exp2() {
    let ctx = Ctx::new(44_100.0);
    let mut s = einst_dsp::stereo::StereoMixer::new();
    s.ports()[0].value = 1.3;
    let ones = vec![1.0f32; 64];
    let (l, _) = run(&mut s, &ctx, &ones, &ones);
    // Mono input, full width: the level settles at v1's Gain target.
    assert!((l[63] - Gain::target(1.3)).abs() < 1e-3, "{} vs {}", l[63], Gain::target(1.3));
}

/// v1's Noise Gate ports in v1 order reach Dreist's gate at Geist2's indices.
#[test]
fn noise_gate_takes_v1_port_order() {
    use einst_dsp::noise_gate::{NoiseGate, V1_TO_GEIST2};
    use geist_dsp::fx::nodes::noise_gate::NoiseGate as Geist2;
    let ctx = Ctx::new(44_100.0);
    let mut v1 = NoiseGate::new();
    let mut g2 = Geist2::new();
    // v1 values: Attack, Hold, Release, Threshold, Hysteresis, LPFreq, HPFreq, Preview, Sidechain, WetMix.
    let vals = [0.002f32, 0.05, 0.2, 0.1, 0.5, 8000.0, 30.0, 0.0, 0.0, 1.0];
    for (i, v) in vals.iter().enumerate() {
        v1.ports()[i].value = *v;
        g2.ports()[V1_TO_GEIST2[i]].value = *v;
    }
    let (l, r) = (noise(4096, 3), noise(4096, 4));
    assert_eq!(run(&mut v1, &ctx, &l, &r), run(&mut g2, &ctx, &l, &r));
    // The v1 names sit where v1 puts them.
    assert_eq!(einst_dsp::internal_ports(einst_dsp::NOISE_GATE), &[(6, 0.0), (7, 0.0), (8, 0.0), (9, 1.0)]);
}
