//! Geist v1's mono effect nodes, the ones v1 does not share with Geist2
//! (`docs/re/loading-v1.md`, "Effect types and the v1-only DSP"):
//!
//! | Effect | v1 class | Geist2 class (Dreist's port) |
//! |---|---|---|
//! | Gain | `VAmpM` | `VAmpS` |
//! | Vari-EQ | `VFilterEQ2` | `VFilterEQ2S` |
//! | Envelope Shaper | `VDSPTransientDesigner` | `VDSPTransientDesignerS` |
//!
//! and the shared classes whose v1 build differs (`docs/re/effects-v1.md`):
//! Stereo Width, Phaser and Frequency Shifter (v1's fast 2^x), Noise Gate
//! (v1's port order), Channel Compressor (flat input gain, fast 2^x), Bus
//! Compressor (fast 2^x for the side-chain high-pass), Modulated Filter
//! (fast 2^x, FM from the left input only).
//!
//! v1's `VDSPNodeAdaptor::Process22` (0x4f3840, ev_964c5456) runs a mono
//! node as two instances, one per channel, so each node here keeps one
//! state per channel. They implement `dreist_dsp::fx::Node`; the engine
//! (`einst-engine`'s `fx.rs`) builds them through [`create`] in place of
//! Dreist's Geist2 nodes.

pub mod auto_filter;
pub mod bus_comp;
pub mod envelope_shaper;
pub mod freq_shifter;
pub mod gain;
mod math;
pub mod noise_gate;
pub mod phaser;
pub mod stereo;
pub mod urei1176;
pub mod vari_eq;

use dreist_dsp::fx::Node;

/// Effect type indices as Geist2 / Dreist number them (`EFFECT_TYPES`).
pub const GAIN: usize = 1;
pub const STEREO_WIDTH: usize = 2;
pub const VARI_EQ: usize = 5;
pub const CHANNEL_COMPRESSOR: usize = 8;
pub const BUS_COMPRESSOR: usize = 9;
pub const NOISE_GATE: usize = 11;
pub const ENVELOPE_SHAPER: usize = 12;
pub const PHASER: usize = 18;
pub const MODULATED_FILTER: usize = 29;
pub const FREQUENCY_SHIFTER: usize = 25;

/// v1's node for an effect type (Geist2 numbering), or `None` where v1 and
/// Geist2 share the DSP and Dreist's node stands.
pub fn create(kind: usize) -> Option<Box<dyn Node>> {
    Some(match kind {
        GAIN => Box::new(gain::Gain::new()),
        STEREO_WIDTH => Box::new(stereo::StereoMixer::new()),
        VARI_EQ => Box::new(vari_eq::VariEq::new()),
        CHANNEL_COMPRESSOR => Box::new(urei1176::Urei1176::new()),
        BUS_COMPRESSOR => Box::new(bus_comp::BusComp::new()),
        NOISE_GATE => Box::new(noise_gate::NoiseGate::new()),
        PHASER => Box::new(phaser::Phaser::new()),
        MODULATED_FILTER => Box::new(auto_filter::AutoFilter::new()),
        FREQUENCY_SHIFTER => Box::new(freq_shifter::FreqShifter::new()),
        ENVELOPE_SHAPER => Box::new(envelope_shaper::EnvelopeShaper::new()),
        _ => return None,
    })
}

/// The hidden ports v1's `Effect::SetupInternalParameters` (0x2f0250,
/// ev_5107cdbc) sets for these types, as `(port, value 0..1)` in the
/// node's own (v1) port order.
pub fn internal_ports(kind: usize) -> &'static [(usize, f32)] {
    match kind {
        // v1 type 6: Mix, Active on, Q 0.5.
        CHANNEL_COMPRESSOR => &[(7, 1.0), (8, 1.0), (3, 0.5)],
        // v1 type 7: Mix, Active on, external side-chain off.
        BUS_COMPRESSOR => &[(5, 1.0), (6, 1.0), (7, 0.0)],
        // v1 type 9: WetMix.
        ENVELOPE_SHAPER => &[(0, 1.0)],
        // v1 type 8: HPFreq, Preview, Sidechain off, WetMix 1.
        NOISE_GATE => &[(6, 0.0), (7, 0.0), (8, 0.0), (9, 1.0)],
        // v1 type 13.
        PHASER => &[(6, 1.0), (7, 0.0), (8, 0.0)],
        // v1 type 19: Mix, Active on, external side-chain off, FM off.
        MODULATED_FILTER => &[(5, 1.0), (6, 1.0), (7, 0.0), (11, 0.0)],
        // v1 type 17.
        FREQUENCY_SHIFTER => &[(2, 1.0)],
        _ => &[],
    }
}

/// v1's fast 2^x (inlined in `VAmpM::Voice::ProcessBuffer` 0x5369d0 and
/// `VFilterEQ2::Voice::ProcessBuffer` 0x51fab0): `i = floor(x − 0.5)`,
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
