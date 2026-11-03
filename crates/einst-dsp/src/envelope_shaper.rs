//! `VDSP::VDSPTransientDesigner` (Envelope Shaper): `InitializePorts`
//! 0x31ddc0 (WetMix, Attack, Sustain, Gain, Sensitivity, Linked; a Meter
//! out; mono `In1`/`Out1`), `Voice::ProcessBuffer` 0x31e370 (ev_ed247fe9)
//! around a `TransientDesigner` (`TDImpl`, `CreateNewTransientDesigner`
//! 0x321010).
//!
//! v1's `TDImpl` is the transient designer Geist2's `VDSPTransientDesignerS`
//! wraps, which Dreist ports as `geist_dsp::fx::nodes::envelope_shaper`:
//! `TDChannel::BuildCoeffs` 0x31f560 (ev_d7897614) gives the same time
//! constants (e.g. exp(−1/(2π·sr·39000·100e−9)) = exp(−40.809/sr)),
//! `ConvertParameter01ToParameterValue` 0x31f1d0 (ev_8c86f1ce) the same
//! ranges (Attack ±15, Sustain ±24, Gain ±12 dB) and `TDChannel::Process`
//! 0x31fb40 the same constants (compared, not re-transcribed line by line).
//!
//! What differs is the wiring: v1's node is mono, so `VDSPNodeAdaptor::
//! Process22` runs one instance per channel, each with its own detector.
//! `TDImpl::Process` 0x320830 (ev_8c1c8c76) links channels only across the
//! channels of one instance, so "Linked" never links left and right in v1.
//! This node is Dreist's with Linked held off. v1's voice has no `Reset` of
//! its own (the base `VDSPVoice::Reset` is empty), so a panic keeps the
//! parameters where Dreist's Geist2 voice resets them.

use geist_dsp::fx::nodes::envelope_shaper::EnvelopeShaper as Geist2Shaper;
use geist_dsp::fx::{Ctx, Node, Port};

/// The Linked port.
pub const LINKED: usize = 5;

pub struct EnvelopeShaper {
    inner: Geist2Shaper,
}

impl EnvelopeShaper {
    pub fn new() -> EnvelopeShaper {
        EnvelopeShaper { inner: Geist2Shaper::new() }
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
