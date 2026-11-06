//! [re:275da3] ("Gain"): `InitializePorts` [re:a18c5d] (ev_cfa257f1, one
//! port "Amp", amp-log, 0 dB, −oo..36 dB, the same table as v2's
//! [re:3fed9b]), `Voice::ProcessBuffer` [re:dba687] (ev_9b98b0fe), `Voice::Reset`
//! [re:bf87c8] (gain 0).
//!
//! Per block: the port value clamped to ±10, a = 2^x − 0.016367754 with v1's
//! [`fast_exp2`](crate::fast_exp2), floored at 0; the gain ramps linearly from
//! its last value to `a` over the block ([re:84e26c] = 1/block size).
//! Dreist's [re:3fed9b] port uses an exact `exp2`.

use dreist_dsp::fx::{Ctx, Node, Port, ports, ports_from};

pub struct Gain {
    ports: Vec<Port>,
    /// One voice per channel ([re:4ff8f4]); both ramp alike.
    gain: [f32; 2],
}

impl Gain {
    pub fn new() -> Gain {
        Gain { ports: ports_from(ports::GAIN), gain: [0.0; 2] }
    }

    /// The level a port value ramps to.
    pub fn target(x: f32) -> f32 {
        let x = x.clamp(-10.0, 10.0);
        let a = crate::fast_exp2(x) - 0.016367754;
        (a.abs() + a) * 0.5
    }
}

impl Default for Gain {
    fn default() -> Self {
        Gain::new()
    }
}

impl Node for Gain {
    fn ports(&mut self) -> &mut Vec<Port> {
        &mut self.ports
    }

    fn reset(&mut self) {
        self.gain = [0.0; 2];
    }

    fn process(&mut self, _ctx: &Ctx, inp: [&[f32]; 2], out: [&mut [f32]; 2]) {
        let n = inp[0].len();
        if n == 0 {
            return;
        }
        let target = Gain::target(self.ports[0].value);
        let k = 1.0 / n as f32;
        for (c, (i, o)) in inp.iter().zip(out).enumerate() {
            let start = self.gain[c];
            for (o, x) in o.iter_mut().zip(i.iter()) {
                self.gain[c] += (target - start) * k;
                *o = self.gain[c] * x;
            }
        }
    }
}
