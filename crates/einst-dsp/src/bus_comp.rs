//! [re:68112c] (Bus Compressor, v1 type 7) around
//! [re:8a3887], compared with v2's, which Dreist ports as
//! `dreist_dsp::fx::nodes::bus_comp`.
//!
//! v1's [re:ca4b18] is v2's circuit: the constructor (ev_15cdcb65), the
//! setters (`setRatio` ev_a170b69d, `setRelease` ev_a8999690,
//! `setSampleRate` ev_746a5beb, `setSidechainHP` ev_af0d4af3) and `process`
//! (ev_53f3c025) have the same values and update equations, with GCC's
//! folded reciprocals (·3.030303e−7 = /3.3e6, ·2127659.5 = /4.7e−7,
//! ·147058.83 = /6.8e−6, ·1.0989011e−5 = /91000, ·1.3333333e−6 = /750000,
//! ·0.12820512 = /7.8, ·0.0060975607 = /164, 10^(−0.39·g) = 10^(−7.8·g/20),
//! 10^(−8.2·m) = 10^(−164·m/20)). The two-diode solve `RDI012`
//! (ev_32d287e3) is the same Newton iteration as v2's inlined one: from
//! 1e−12 A, at most 12 steps, 0.1 % convergence, currents clamped at 0
//! (v1 computes 0.0475·ln 5.9e−9 at run time, v2 folds it to
//! −0.900044858). After the loop v1 updates its gain-reduction meter
//! (×0.3815, smoothed by exp(−31.4159·n/sr)), which Dreist does not model.
//!
//! The one audible difference is in `Voice::ProcessBuffer` (ev_4014c08a):
//! the side-chain high-pass frequency 2^(HPFreq + 1) is v1's
//! [`fast_exp2`](crate::fast_exp2). This node is Dreist's, fed the port
//! value whose exact 2^x is v1's fast one.

use dreist_dsp::fx::nodes::bus_comp::BusComp as V2BusComp;
use dreist_dsp::fx::{Ctx, Node, Port};

/// The side-chain HP frequency port (log2(Hz) − 1).
pub const HP_FREQ: usize = 8;

pub struct BusComp {
    inner: V2BusComp,
}

impl BusComp {
    pub fn new() -> BusComp {
        BusComp { inner: V2BusComp::new() }
    }
}

impl Default for BusComp {
    fn default() -> Self {
        BusComp::new()
    }
}

impl Node for BusComp {
    fn ports(&mut self) -> &mut Vec<Port> {
        self.inner.ports()
    }

    fn process(&mut self, ctx: &Ctx, inp: [&[f32]; 2], out: [&mut [f32]; 2]) {
        let hp = self.inner.ports()[HP_FREQ].value;
        self.inner.ports()[HP_FREQ].value = crate::fast_exp2(hp + 1.0).log2() - 1.0;
        self.inner.process(ctx, inp, out);
        self.inner.ports()[HP_FREQ].value = hp;
    }
}
