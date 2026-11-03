//! the original v1's mono effect nodes, the ones v1 does not share with v2
//! (dreist-re `einst/docs/re/loading-v1.md`, "Effect types and the v1-only DSP"):
//!
//! | Effect | v1 class | v2 class (Dreist's port) |
//! |---|---|---|
//! | Gain | [re:6ffb08] | [re:3fed9b] |
//! | Vari-EQ | [re:5a35c5] | [re:928f80] |
//! | Envelope Shaper | [re:9679d7] | [re:bc6b0a] |
//!
//! and the shared effects whose v1 build differs in more than its 2^x
//! (dreist-re `einst/docs/re/effects-v1.md`): Channel Compressor, Noise
//! Gate, Parametric / DJ EQ, Filter, Delay, Comb Filter, Tin Can Verb and the
//! ladder filters. Effects that differ only by v1's fast 2^x use Dreist's
//! nodes with [`fast_exp2`] installed as `dreist_dsp::fx::exp2`
//! (`einst::install`).
//!
//! v1's [re:4ff8f4] ([re:5bcf80], ev_964c5456) runs a mono
//! node as two instances, one per channel, so each node here keeps one
//! state per channel. They implement `dreist_dsp::fx::Node`; the effect
//! chain builds them through [`create`] (`dreist_dsp::fx::install_factory`)
//! in place of Dreist's v2 nodes.

pub mod audio_freeze;
pub mod comb;
pub mod delay;
pub mod envelope_shaper;
pub mod gain;
pub mod ladder;
mod math;
pub mod noise_gate;
pub mod ssl_eq;
pub mod ssl_filter;
pub mod tin_can;
pub mod urei1176;
pub mod vari_eq;

use dreist_dsp::fx::Node;

/// Effect type indices as v2 / Dreist number them (`EFFECT_TYPES`).
pub const GAIN: usize = 1;
pub const STEREO_WIDTH: usize = 2;
pub const PARAMETRIC_EQ: usize = 3;
pub const DJ_EQ: usize = 4;
pub const VARI_EQ: usize = 5;
pub const CHANNEL_COMPRESSOR: usize = 8;
pub const BUS_COMPRESSOR: usize = 9;
pub const NOISE_GATE: usize = 11;
pub const ENVELOPE_SHAPER: usize = 12;
pub const DELAY: usize = 15;
pub const PHASER: usize = 18;
pub const DISTORTION: usize = 19;
pub const BIT_CRUSHER: usize = 23;
pub const FILTER: usize = 28;
pub const MODULATED_FILTER: usize = 29;
pub const COMB_FILTER: usize = 30;
pub const THREE_OH_THREE_FILTER: usize = 31;
pub const HOOVER_FILTER: usize = 32;
pub const GRANDAD_FILTER: usize = 33;
pub const LITTLE_FATTY_FILTER: usize = 34;
pub const CHILD_GENIUS_FILTER: usize = 35;
pub const JAPAN_FILTER: usize = 36;
pub const AUDIO_FREEZER: usize = 38;
pub const TIN_CAN_VERB: usize = 43;
pub const FREQUENCY_SHIFTER: usize = 25;

/// v1's node for an effect type (v2 numbering), or `None` where v1 and
/// v2 share the DSP and Dreist's node stands.
pub fn create(kind: usize) -> Option<Box<dyn Node>> {
    Some(match kind {
        GAIN => Box::new(gain::Gain::new()),
        VARI_EQ => Box::new(vari_eq::VariEq::new()),
        CHANNEL_COMPRESSOR => Box::new(urei1176::Urei1176::new()),
        NOISE_GATE => Box::new(noise_gate::NoiseGate::new()),
        COMB_FILTER => Box::new(comb::Comb::new()),
        DELAY => Box::new(delay::Delay::new()),
        AUDIO_FREEZER => Box::new(audio_freeze::AudioFreeze::new()),
        TIN_CAN_VERB => Box::new(tin_can::TinCan::new()),
        PARAMETRIC_EQ | DJ_EQ => Box::new(ssl_eq::SslEq::new()),
        FILTER => Box::new(ssl_filter::SslFilter::new()),
        THREE_OH_THREE_FILTER => Box::new(ladder::Ladder::new(ladder::Kind::Tb303)),
        HOOVER_FILTER => Box::new(ladder::Ladder::new(ladder::Kind::Juno)),
        GRANDAD_FILTER => Box::new(ladder::Ladder::new(ladder::Kind::Moog)),
        CHILD_GENIUS_FILTER => Box::new(ladder::Ladder::new(ladder::Kind::Prodigy)),
        ENVELOPE_SHAPER => Box::new(envelope_shaper::EnvelopeShaper::new()),
        _ => return None,
    })
}

/// The hidden ports v1's [re:1160d2] ([re:17fdea],
/// ev_5107cdbc) sets for these types, as `(port, value 0..1)` in the
/// node's own (v1) port order.
pub fn internal_ports(kind: usize) -> &'static [(usize, f32)] {
    match kind {
        // v1 type 6: Mix, Active on, Q 0.5.
        CHANNEL_COMPRESSOR => &[(7, 1.0), (8, 1.0), (3, 0.5)],
        // v1 type 9: WetMix.
        ENVELOPE_SHAPER => &[(0, 1.0)],
        // v1 type 8: HPFreq, Preview, Sidechain off, WetMix 1.
        NOISE_GATE => &[(6, 0.0), (7, 0.0), (8, 0.0), (9, 1.0)],
        // Same hidden ports as v2 (`dreist_dsp` setup) for the copied nodes.
        FILTER => &[(4, 1.0)],
        DELAY => &[(1, 1.0), (7, 1.0), (4, 0.0)],
        AUDIO_FREEZER => &[(3, 1.0), (8, 0.0), (7, 0.0), (9, 1.0)],
        TIN_CAN_VERB => &[(6, 1.0), (5, 0.0)],
        PARAMETRIC_EQ => &[(3, 0.0), (7, 1.0), (11, 1.0), (15, 0.0), (16, 0.0), (17, 1.0)],
        DJ_EQ => &[(3, 1.0), (7, 0.0), (11, 1.0), (15, 1.0), (0, 0.0), (12, 0.0), (1, 0.3), (13, 0.75), (8, 0.5), (16, 0.0), (17, 1.0)],
        THREE_OH_THREE_FILTER | HOOVER_FILTER | CHILD_GENIUS_FILTER => &[(4, 0.33262002), (6, 0.0), (5, 0.0)],
        GRANDAD_FILTER => &[(4, 0.6010388), (5, 0.0), (6, 0.0)],
        _ => &[],
    }
}

/// v1's fast 2^x (inlined in [re:7bb6eb] and
/// [re:6953af]): `i = floor(x − 0.5)`,
/// `f = x − i` in 0.5..1.5, a cubic for 2^f (within 0.4 %), and `i` added to the float's
/// exponent bits.
pub fn fast_exp2(x: f32) -> f32 {
    let i = (x - 0.5).floor() as i32;
    let f = x - i as f32;
    let p = ((f * 0.079440236 + 0.22449434) * f + 0.69606566) * f + 1.0;
    f32::from_bits((p.to_bits() as i32).wrapping_add(i.wrapping_mul(0x80_0000)) as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fast_exp2_tracks_exp2() {
        for k in -400..=450 {
            let x = k as f32 / 100.0;
            let (a, b) = (fast_exp2(x), x.exp2());
            assert!(((a - b) / b).abs() < 4e-3, "{x}: {a} vs {b}");
        }
        // Integer exponents land on the polynomial's value at f = 1 (≈ 2).
        assert!((fast_exp2(3.0) / 8.0 - fast_exp2(1.0) / 2.0).abs() < 1e-6);
    }
}
