//! v1's sound rules on a single voice (Einst's rules installed).

use dreist_dsp::voice::{LAYER_PARAMS, NoteSetup, PlaybackMode, SOURCES, SampleData, Voice};
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
