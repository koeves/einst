//! [re:07d815] (Envelope Shaper): `InitializePorts`
//! [re:b35229] (WetMix, Attack, Sustain, Gain, Sensitivity, Linked; a Meter
//! out; mono `In1`/`Out1`), `Voice::ProcessBuffer` [re:d0685b] (ev_ed247fe9)
//! around a [re:73432e] ([re:1800b5], `CreateNewTransientDesigner`
//! [re:c01c22]).
//!
//! v1's [re:1800b5] is the transient designer v2's [re:bc6b0a]
//! wraps, which Dreist ports as `dreist_dsp::fx::nodes::envelope_shaper`:
//! [re:ee961d] (ev_d7897614) gives the same time
//! constants (e.g. exp(−1/(2π·sr·39000·100e−9)) = exp(−40.809/sr)),
//! `ConvertParameter01ToParameterValue` [re:d0530c] (ev_8c86f1ce) the same
//! ranges (Attack ±15, Sustain ±24, Gain ±12 dB) and [re:fbc6c0]
//! [re:dbfaaa] the same constants (compared, not re-transcribed line by line).
//!
//! What differs is the wiring: v1's node is mono, so [re:4ff8f4]
//!  runs one instance per channel, each with its own detector.
//! [re:d018db] (ev_8c1c8c76) links channels only across the
//! channels of one instance, so "Linked" never links left and right in v1.
//! This node is Dreist's with Linked held off. v1's voice has no `Reset` of
//! its own (the base [re:1740cf] is empty), so a panic keeps the
//! parameters where Dreist's v2 voice resets them.

use dreist_dsp::fx::nodes::envelope_shaper::EnvelopeShaper as V2Shaper;
use dreist_dsp::fx::{Ctx, Node, Port};

/// The Linked port.
pub const LINKED: usize = 5;

pub struct EnvelopeShaper {
    inner: V2Shaper,
}

impl EnvelopeShaper {
    pub fn new() -> EnvelopeShaper {
        EnvelopeShaper { inner: V2Shaper::new() }
    }
}

impl Default for EnvelopeShaper {
    fn default() -> Self {
        EnvelopeShaper::new()
    }
}

impl Node for EnvelopeShaper {
    fn ports(&mut self) -> &mut Vec<Port> {
        self.inner.ports()
    }

    /// Empty, as v1's voice.
    fn reset(&mut self) {}

    fn process(&mut self, ctx: &Ctx, inp: [&[f32]; 2], out: [&mut [f32]; 2]) {
        if let Some(p) = self.inner.ports().get_mut(LINKED) {
            p.value = 0.0;
        }
        self.inner.process(ctx, inp, out);
    }
}
