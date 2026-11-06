//! v1's sound rules on a single voice (Einst's rules installed): the
//! mirrored, per-block reverse, the end fade from the sample that reaches
//! it, one pan stage, and the timestretch ratio cap.

use dreist_dsp::voice::{LAYER_PARAMS, NoteSetup, PlaybackMode, SOURCES, SampleData, Voice, lp};
use std::sync::Arc;

const SR: f32 = 44_100.0;

/// A one-shot layer at unity: instant attack, sustain 1.
fn setup(frames: u32) -> NoteSetup {
    let mut layer = [[0.0; SOURCES]; LAYER_PARAMS];
    for (p, row) in layer.iter_mut().enumerate() {
        row[0] = match p {
            0 | 2 => 0.5f32.cbrt(),
            8 | 16 | 24 => 1.0,
            11 | 12 => 0.3,
            15 | 23 => 0.02,
            17 | 25 => 1.0,
            18 | 26 => 0.001,
            7 | 9 | 10 | 13 | 14 | 22 | 34..=36 | 38 | 40 | 41 => 0.0,
            _ => 0.5,
        };
    }
    NoteSetup {
        layer,
        playback: PlaybackMode::OneShot,
        vintage: false,
        start: 0,
        end: frames,
        loop_start: 0,
        loop_end: frames,
        loop_len: frames,
        step: None,
        midi_velocity: Some(127),
        pad_v2a: 1.0,
        semitone_offset: 0.0,
        output_scale: 1.0,
        env_tracks: false,
        reversed: false,
        file_bpm: 0.0,
        stretch_bpm: None,
        routings: None,
        routing_raw: [0.5; 4],
    }
}

fn play(src: Vec<f32>, s: NoteSetup, total: usize, block: usize, mut each: impl FnMut(usize, &mut Voice)) -> (Vec<f32>, Vec<f32>, Option<usize>) {
    einst::install();
    let mut v = Voice::new(SR);
    v.start(Arc::new(SampleData { left: src, right: None, sample_rate: SR }), s);
    let (mut l, mut r) = (vec![0.0; total], vec![0.0; total]);
    let mut stopped = None;
    for (k, (a, b)) in l.chunks_mut(block).zip(r.chunks_mut(block)).enumerate() {
        each(k, &mut v);
        v.process(a, b);
        if stopped.is_none() && !v.is_active() {
            stopped = Some(k * block);
        }
    }
    (l, r, stopped)
}

/// v1 [re:fece0b] reads a reversed note as the mirror of its forward
/// position, and [re:18d53d] takes the layer's reverse status every block:
/// switching it mid-note jumps to the forward read point.
#[test]
fn reverse_mirrors_and_follows_the_layer() {
    let n = 4000;
    let src: Vec<f32> = (0..n).map(|i| i as f32 / n as f32).collect();
    let mut s = setup(n as u32);
    s.reversed = true;
    let (l, _, _) = play(src.clone(), s, 2048, 64, |k, v| {
        if k == 16 {
            v.set_reversed(false);
        }
    });
    // The end marker is the last frame (n − 1): position p reads frame
    // n − 1 − p.
    for i in [100, 500, 1000] {
        assert!((l[i] - src[n - 1 - i]).abs() < 1e-6, "frame {i}: {} vs {}", l[i], src[n - 1 - i]);
    }
    for i in [1030, 1500, 2000] {
        assert!((l[i] - src[i]).abs() < 1e-6, "frame {i}: {} vs {}", l[i], src[i]);
    }
}

/// v1's end fade ([re:fece0b], [re:093661]) starts at the sample after
/// which end ≤ position + 34, inside its block, and runs to the end
/// marker: 31 steps of 1/31 here.
#[test]
fn end_fade_starts_at_the_sample() {
    let n = 1000;
    let (l, r, stopped) = play(vec![0.5; n], setup(n as u32), 2048, 64, |_, _| {});
    // End marker 999: the position after frame 964 is 965, 999 ≤ 965 + 34;
    // the fade starts at frame 964, mid-block (the reference reaches its
    // condition two frames later and fades from the block start, 960).
    assert!((l[960] - 0.5).abs() < 1e-4 && (l[964] - 0.5).abs() < 1e-4, "{} {}", l[960], l[964]);
    for k in 0..31 {
        let want = 0.5 * (1.0 - k as f32 / 31.0);
        assert!((l[964 + k] - want).abs() < 1e-4, "frame {}: {} vs {want}", 964 + k, l[964 + k]);
    }
    assert!(l[997..].iter().all(|x| *x == 0.0));
    assert_eq!(l, r);
    assert!(stopped.is_some_and(|s| s < 1024));
}

/// v1 pans once, at the layer ([re:f87c3f]): the voice ignores the step's
/// Pan lane.
#[test]
fn voice_ignores_the_pan_lane() {
    let n = 3000;
    let mut s = setup(n as u32);
    let mut lanes: dreist_dsp::voice::Lanes = std::array::from_fn(dreist_formats::params::step_lane_default);
    lanes[dreist_dsp::voice::lane::PAN] = 0.0;
    s.step = Some(lanes);
    let (l, r, _) = play((0..n).map(|i| (i as f32 * 0.05).sin()).collect(), s, 2048, 64, |_, _| {});
    assert!(l.iter().any(|x| x.abs() > 0.1));
    assert_eq!(l, r);
}

/// v1's stretch ratio is at most 4 ([re:d8985e]): the amount's top is ×4
/// (v1's cubic curve); tracking, the layer's source BPM over the host's,
/// capped the same.
#[test]
fn timestretch_ratio_is_capped_at_four() {
    let n = 8000;
    let src: Vec<f32> = (0..n).map(|i| (i as f32 * 0.06).sin() * 0.5).collect();
    let stop = |amount: f32, bpm: Option<(f32, f32)>| {
        let mut s = setup(n as u32);
        s.layer[lp::TS_ACTIVE][0] = 1.0;
        s.layer[lp::TS_AMOUNT][0] = amount;
        if bpm.is_some() {
            s.layer[lp::TS_TRACKS_BPM][0] = 1.0;
        }
        s.stretch_bpm = bpm;
        let mut v = Voice::new(SR);
        v.prepare_timestretch();
        play(src.clone(), s, 8 * n, 64, |_, _| {}).2.expect("voice ends")
    };
    let four = stop(1.0, None);
    assert!(stop(0.95, None) < four, "×3.2 is shorter than ×4");
    // Tracking at the host's 120 BPM: 240 → ×2, 960 → capped at ×4.
    let two = stop(0.5, Some((240.0, 120.0)));
    assert!(four > two * 3 / 2, "{four} vs {two}");
    assert_eq!(stop(0.5, Some((960.0, 120.0))), four);
}
