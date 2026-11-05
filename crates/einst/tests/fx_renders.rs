//! Per-effect renders of v1's 32 effect types through the effect chain's
//! `Effect` (Einst's nodes installed): an impulse, a 40 Hz–16 kHz sine
//! sweep and seeded noise, each at three front-panel settings (every
//! parameter at 0.1, at the type's default, at 0.9), at 44.1 kHz. Golden
//! set `fx` (`UPDATE_GOLDEN=1` rewrites it): any change to an effect's DSP,
//! its parameter mapping or its hidden ports shows here.

use dreist_dsp::fx::{Ctx, Effect};

const SR: f32 = 44_100.0;
const N: usize = 44_100 / 2;

fn signals() -> [(&'static str, Vec<f32>); 3] {
    let mut impulse = vec![0.0f32; N];
    impulse[0] = 1.0;
    let mut phase = 0.0f64;
    let sweep = (0..N)
        .map(|i| {
            let f = 40.0 * (16_000.0f64 / 40.0).powf(i as f64 / N as f64);
            phase += std::f64::consts::TAU * f / SR as f64;
            (0.5 * phase.sin()) as f32
        })
        .collect();
    let mut s = 0x1234_5678u32;
    let noise = (0..N)
        .map(|_| {
            s ^= s << 13;
            s ^= s >> 17;
            s ^= s << 5;
            (s as f32 / u32::MAX as f32 - 0.5) * 0.5
        })
        .collect();
    [("impulse", impulse), ("sweep", sweep), ("noise", noise)]
}

/// Render `x` (both channels) through a fresh effect of `kind` with every
/// front-panel parameter at `v` (`None`: the type's defaults), wet only.
fn render(kind: usize, v: Option<f32>, x: &[f32]) -> Vec<f32> {
    let ctx = Ctx::new(SR);
    let mut fx = Effect::new(kind, SR);
    fx.active = true;
    fx.mix = 1.0;
    if let Some(v) = v {
        for p in 0..dreist_formats::params::effect_param_names(kind).len() {
            fx.push_parameter(p, v);
        }
    }
    let (mut l, mut r) = (vec![0.0f32; x.len()], vec![0.0f32; x.len()]);
    for ((xi, li), ri) in x.chunks(256).zip(l.chunks_mut(256)).zip(r.chunks_mut(256)) {
        fx.process(&ctx, [xi, xi], [li, ri]);
    }
    l.into_iter().chain(r).collect()
}

#[test]
fn effects_match_fingerprints() {
    einst::install();
    let mut golden = dreist_testkit::Golden::open("fx");
    let kinds: Vec<usize> = (0..6).flat_map(einst::effect_menu).collect();
    assert_eq!(kinds.len(), 32);
    let sigs = signals();
    for kind in kinds {
        let name = dreist_formats::params::EFFECT_TYPES[kind].replace(' ', "_");
        for (sname, x) in &sigs {
            for (vname, v) in [("low", Some(0.1)), ("default", None), ("high", Some(0.9))] {
                let y = render(kind, v, x);
                assert!(y.iter().all(|s| s.is_finite()), "{name} {sname} {vname}: not finite");
                golden.check(&format!("{name}-{sname}-{vname}"), dreist_testkit::fnv(y.iter()), &y);
            }
        }
    }
    golden.finish();
}
