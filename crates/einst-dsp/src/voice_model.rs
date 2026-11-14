//! The original v1's voice parameters (installed through
//! `dreist_dsp::model`), from v1's `UpdatePitchParameters` (ev_6c6802b2),
//! `UpdatePitchModulation` (ev_c7cae932), the pitch and filter envelope
//! generation (ev_63291d64, ev_eba20…), `UpdateFilterModulation` /
//! `UpdateParameters` (ev_9db2af34, ev_b0193aee), velocity → pitch
//! (ev_eeba243d) and `ProcessAudio` (ev_29992d9a):
//!
//! - pitch: whole-semitone terms (coarse, pad/engine/master, the step's
//!   coarse lane), fine through the step's fine lane, velocity → pitch as
//!   `round(24·(2a − 1)·vel) − 12` (mirrored below 0.5), the total clamped to
//!   ±12, step `2^(S/12)` (exact);
//! - pitch envelope: the routed envelopes averaged, times the averaged
//!   routing depth in whole semitones, added per sample through v1's fast
//!   2^x;
//! - cutoff (with v1's velocity curve), resonance and drive through the
//!   step lanes, then pushed toward 0 or 1 by the averaged routed envelopes'
//!   block maximum (over all but the last sample) times the averaged depth.
//!
//! Routings: a converted v1 layer keeps its four routings (0/1 the amp
//! envelope's, 2/3 the free envelope's), and the step its four routing-depth
//! lanes. Per routing r to a destination, the depth is
//! `ModulateValuePosNeg(depth_r, lane_r)`, counted when it is not neutral;
//! the destination takes the mean of the counted depths and of their
//! envelopes. Pitch reads the lanes' note-start variants
//! (`UpdatePitchModulation` [re:fad3cb]), the filter the stored lanes
//! (`UpdateFilterModulation` [re:eceb52]). Layers without routings (v2
//! documents) use the envelope columns of CoarseTune, FilterCutoff,
//! FilterResonance and FilterDrive (`einst_formats::v2`'s depth `2d − 1`).

use dreist_dsp::convert::{cutoff_with_velocity, fine_semitones, modulate_pos_neg, semitones};
use dreist_dsp::model::{ModelInput, ModelOutput, VoiceModel};
use dreist_dsp::voice::{lane, lp};

/// Transmod source columns of the converted layer.
const AMP_ENV: usize = 1;
const FREE_ENV: usize = 2;
const VELOCITY: usize = 3;

pub struct V1Voice;

/// v1's `ConvertPitchUsingVelocity`: `a` the raw velocity-to-pitch amount.
pub fn velocity_pitch(s: f64, a: f32, vel: f32) -> f64 {
    let mut s = s as f32;
    let round = |f: f32| if f < 0.0 { f - 0.5 } else { f + 0.5 } as i32 as f32;
    if (a - 0.5).abs() >= 1e-6 {
        if a > 0.5 {
            let f = (((a + a) - 1.0) * vel) as f64 * 24.0;
            s = s + round(f as f32) + -12.0;
        } else if a < 0.5 {
            let f = ((a * -2.0 + 1.0) * vel) as f64 * 24.0;
            s = s - (round(f as f32) + -12.0);
        }
    }
    (s as f64).clamp(-12.0, 12.0)
}

/// Routed envelopes of a parameter: up to four (depth 0..1, buffer) pairs,
/// kept in place (this runs per voice every sub-block on the audio thread).
struct Routed<'a> {
    depths: [f32; 4],
    bufs: [&'a [f32]; 4],
    n: usize,
}

impl<'a> Routed<'a> {
    fn new() -> Routed<'a> {
        Routed { depths: [0.0; 4], bufs: [&[]; 4], n: 0 }
    }

    fn push(&mut self, depth: f32, buf: &'a [f32]) {
        self.depths[self.n] = depth;
        self.bufs[self.n] = buf;
        self.n += 1;
    }

    fn depths(&self) -> &[f32] {
        &self.depths[..self.n]
    }

    fn bufs(&self) -> &[&'a [f32]] {
        &self.bufs[..self.n]
    }
}

/// v1's routings to `dest` (0 pitch, 1 cutoff, 2 resonance, 3 drive) with
/// the routing-depth `lanes`: the counted depths (0..1) and their slots'
/// envelopes; `None` for a layer without v1 routings.
fn slot_routed<'a>(input: &'a ModelInput, dest: u8, lanes: &[f32]) -> Option<Routed<'a>> {
    let slots = input.routings?;
    let mut out = Routed::new();
    for (r, slot) in slots.iter().enumerate() {
        let Some((d, depth)) = *slot else { continue };
        let v = modulate_pos_neg(depth, lanes[r]);
        if d == dest && (v - 0.5).abs() >= 1e-6 {
            out.push(v, if r < 2 { input.amp_env } else { input.free_env });
        }
    }
    Some(out)
}

/// The routed envelopes of a parameter: (raw depths 0..1, buffers), from the
/// v1 routings when the layer has them (`dest`, `lanes`), else from the
/// envelope columns.
fn routed<'a>(input: &'a ModelInput, param: usize, dest: u8, lanes: &[f32]) -> Routed<'a> {
    if let Some(r) = slot_routed(input, dest, lanes) {
        return r;
    }
    let mut out = Routed::new();
    for (src, buf) in [(AMP_ENV, input.amp_env), (FREE_ENV, input.free_env)] {
        let col = input.layer[param][src];
        let d = (col + 1.0) * 0.5;
        if (d - 0.5).abs() >= 1e-6 {
            out.push(d, buf);
        }
    }
    out
}

fn mean(v: &[f32]) -> f32 {
    v.iter().sum::<f32>() / v.len() as f32
}

impl VoiceModel for V1Voice {
    /// Layers that use modulation sources v1 does not have (anything but
    /// the amp and free envelopes and velocity: a v2 project's modulators)
    /// keep the reference computation.
    fn update(&self, input: &ModelInput, out: &mut ModelOutput) -> bool {
        let l = input.layer;
        if l.iter().any(|row| row[VELOCITY + 1..].iter().any(|d| *d != 0.0)) {
            return false;
        }
        let lanes = input.lanes;
        // Pitch.
        let fine = fine_semitones(modulate_pos_neg(l[lp::FINE][0], lanes[lane::FINE]));
        let s = input.outer_semitones + fine + semitones(l[lp::COARSE][0], true) + input.semitone_offset + semitones(lanes[lane::COARSE], true);
        let a = (l[lp::COARSE][VELOCITY] + 1.0) * 0.5;
        let s = velocity_pitch(s, a, input.velocity);
        out.semitones = s;
        out.step = 2f64.powf(s / 12.0) * input.base;
        // Pitch envelope, per sample.
        out.steps.clear();
        let variants = &lanes[lane::ROUTING_DEPTH..lane::ROUTING_DEPTH + 4];
        let rt = routed(input, lp::COARSE, 0, variants);
        let (depths, bufs) = (rt.depths(), rt.bufs());
        let n = input.amp_env.len();
        if !depths.is_empty() && n > 0 {
            let d = semitones(mean(depths), true) as f32;
            if d.abs() >= 1e-6 {
                let k = 1.0 / bufs.len() as f32;
                for i in 0..n {
                    let e = bufs.iter().map(|b| b[i]).sum::<f32>() * k;
                    let st = crate::fast_exp2((e * d + s as f32) / 12.0) as f64 * input.base;
                    out.steps.push(st as f32 as f64);
                }
            }
        }
        // Filter.
        let va = (l[lp::CUTOFF][VELOCITY] + 1.0) * 0.5;
        let c = modulate_pos_neg(cutoff_with_velocity(l[lp::CUTOFF][0], va, input.velocity), lanes[lane::CUTOFF]);
        let r = modulate_pos_neg(l[lp::RESONANCE][0], lanes[lane::RESONANCE]);
        let dr = modulate_pos_neg(l[lp::DRIVE][0], lanes[lane::DRIVE]);
        let env_mod = |x: f32, param: usize, dest: u8| -> f32 {
            let rt = routed(input, param, dest, input.routing_raw);
            let (depths, bufs) = (rt.depths(), rt.bufs());
            if depths.is_empty() || n < 2 {
                return x;
            }
            let k = 1.0 / bufs.len() as f32;
            // v1's maximum covers all but the block's last sample.
            let m = (0..n - 1).map(|i| bufs.iter().map(|b| b[i]).sum::<f32>() * k).fold(0.0f32, f32::max);
            modulate_pos_neg(x, ((mean(depths) * 2.0 - 1.0) * m + 1.0) * 0.5)
        };
        out.cutoff = env_mod(c, lp::CUTOFF, 1);
        out.resonance = env_mod(r, lp::RESONANCE, 2);
        out.drive = env_mod(dr, lp::DRIVE, 3);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dreist_dsp::voice::{LAYER_PARAMS, Lanes, centre_row};

    /// One block of the v1 model for a layer with routing `slots`.
    fn run(slots: Option<[Option<(u8, f32)>; 4]>, variants: [f32; 4], raw: [f32; 4]) -> ModelOutput {
        let layer = [centre_row(); LAYER_PARAMS];
        let mut lanes: Lanes = [0.5; 97];
        lanes[lane::ROUTING_DEPTH..lane::ROUTING_DEPTH + 4].copy_from_slice(&variants);
        let (amp, free) = ([1.0f32; 8], [0.25f32; 8]);
        let input = ModelInput {
            layer: &layer,
            lanes: &lanes,
            velocity: 1.0,
            amp_env: &amp,
            free_env: &free,
            outer_semitones: 0.0,
            semitone_offset: 0.0,
            base: 1.0,
            routings: slots.as_ref(),
            routing_raw: &raw,
        };
        let mut out = ModelOutput::default();
        assert!(V1Voice.update(&input, &mut out));
        out
    }

    /// Filter: the stored lanes scale each routing, counted routings
    /// average (two amp routings and the free one), neutral ones drop out
    /// ([re:eceb52]); the variants do not touch it.
    #[test]
    fn filter_routes_by_slot_with_stored_lanes() {
        let slots = Some([Some((1, 0.9)), Some((1, 0.7)), Some((1, 0.5)), None]);
        let n = [0.5; 4];
        let base = run(None, n, n).cutoff;
        let a = run(slots, n, n).cutoff;
        assert!(a > base, "the routings open the filter");
        // The 0.5 routing does not count: same as the two alone.
        assert_eq!(a, run(Some([Some((1, 0.9)), Some((1, 0.7)), None, None]), n, n).cutoff);
        // A stored lane at 0 neutralises routing 0's depth (MPN(0.9, 0)).
        let r = run(slots, n, [0.0, 0.5, 0.5, 0.5]).cutoff;
        assert_ne!(r, a);
        assert_eq!(run(slots, [0.0, 0.5, 0.5, 0.5], n).cutoff, a, "variants are pitch's");
    }

    /// Pitch: the lane variants scale each routing ([re:fad3cb]); a slot to
    /// another destination does not count.
    #[test]
    fn pitch_routes_by_slot_with_variants() {
        let slots = Some([None, None, Some((0, 1.0)), Some((1, 0.9))]);
        let n = [0.5; 4];
        let up = run(slots, n, n);
        assert!(!up.steps.is_empty() && up.steps.iter().all(|s| *s > 1.0));
        let damped = run(slots, [0.5, 0.5, 0.25, 0.5], n);
        assert!(damped.steps.first().is_none_or(|s| *s < up.steps[0]), "{:?}", damped.steps);
        assert_eq!(run(slots, n, [0.5, 0.5, 0.25, 0.5]).steps, up.steps, "stored lanes are the filter's");
    }
}
