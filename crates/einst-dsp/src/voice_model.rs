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
//! Routings come from the converted layer: v1's routings 0/1 (amp
//! envelope) and 2/3 (free envelope) are the amp / free envelope columns of
//! CoarseTune (pitch), FilterCutoff, FilterResonance and FilterDrive as
//! depth `2d − 1` (`einst_formats::v2`). v1's per-step routing-depth lanes
//! are not carried by the converted document, so they count as neutral.

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

/// The routed envelopes of a parameter: (raw depths 0..1, buffers).
fn routed<'a>(input: &'a ModelInput, param: usize) -> (Vec<f32>, Vec<&'a [f32]>) {
    let mut depths = Vec::new();
    let mut bufs = Vec::new();
    for (src, buf) in [(AMP_ENV, input.amp_env), (FREE_ENV, input.free_env)] {
        let col = input.layer[param][src];
        let d = (col + 1.0) * 0.5;
        if (d - 0.5).abs() >= 1e-6 {
            depths.push(d);
            bufs.push(buf);
        }
    }
    (depths, bufs)
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
        let (depths, bufs) = routed(input, lp::COARSE);
        let n = input.amp_env.len();
        if !depths.is_empty() && n > 0 {
            let d = semitones(mean(&depths), true) as f32;
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
        let env_mod = |x: f32, param: usize| -> f32 {
            let (depths, bufs) = routed(input, param);
            if depths.is_empty() || n < 2 {
                return x;
            }
            let k = 1.0 / bufs.len() as f32;
            // v1's maximum covers all but the block's last sample.
            let m = (0..n - 1).map(|i| bufs.iter().map(|b| b[i]).sum::<f32>() * k).fold(0.0f32, f32::max);
            modulate_pos_neg(x, ((mean(&depths) * 2.0 - 1.0) * m + 1.0) * 0.5)
        };
        out.cutoff = env_mod(c, lp::CUTOFF);
        out.resonance = env_mod(r, lp::RESONANCE);
        out.drive = env_mod(dr, lp::DRIVE);
        true
    }
}
