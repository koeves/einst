//! `VDSP::VNoiseGateS` (Noise Gate) in v1's port layout. v1's
//! `InitializePorts` (ev_df3a50d3) orders the ports Attack, Hold, Release,
//! Threshold, Hysteresis, LPFreq, HPFreq, Preview, Sidechain, WetMix;
//! Geist2's (and Dreist's port, `geist_dsp::fx::nodes::noise_gate`) puts
//! Sidechain and WetMix at 4 and 5. v1 maps front-panel parameter n to port
//! n (`Effect::PushParameterValue`, ev_a735042f), so a v1 "Hysteresis" or
//! "LPF" value must reach those ports, not Sidechain/WetMix.
//!
//! This node exposes v1's layout and feeds Dreist's node. The gate's own
//! processing is Dreist's; v1's `Voice::ProcessBuffer` uses v1's fast 2^x
//! and differs in some constants, which is still to be compared
//! (`docs/re/effects-v1.md`).

use geist_dsp::fx::nodes::noise_gate::NoiseGate as Geist2Gate;
use geist_dsp::fx::{Ctx, Node, Port};

/// Geist2 port index for each v1 port.
pub const V1_TO_GEIST2: [usize; 10] = [0, 1, 2, 3, 6, 7, 8, 9, 4, 5];

pub struct NoiseGate {
    inner: Geist2Gate,
    ports: Vec<Port>,
}

impl NoiseGate {
    pub fn new() -> NoiseGate {
        let mut inner = Geist2Gate::new();
        let g2 = inner.ports().clone();
        let ports = V1_TO_GEIST2.iter().map(|&i| g2[i]).collect();
        NoiseGate { inner, ports }
    }
}

impl Default for NoiseGate {
    fn default() -> Self {
        NoiseGate::new()
    }
}

impl Node for NoiseGate {
    fn ports(&mut self) -> &mut Vec<Port> {
        &mut self.ports
    }

    fn reset(&mut self) {
        self.inner.reset();
    }

    fn prepare_rate(&mut self, sample_rate: f32) {
        self.inner.prepare_rate(sample_rate);
    }

    fn process(&mut self, ctx: &Ctx, inp: [&[f32]; 2], out: [&mut [f32]; 2]) {
        let inner = self.inner.ports();
        for (v1, &g2) in self.ports.iter().zip(&V1_TO_GEIST2) {
            inner[g2].value = v1.value;
        }
        self.inner.process(ctx, inp, out);
    }

    fn port_value(&self, index: usize) -> Option<f32> {
        self.ports.get(index).map(|p| p.value)
    }
}
