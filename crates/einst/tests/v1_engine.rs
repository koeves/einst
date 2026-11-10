//! v1's engine timing and voice pools with Einst's rules installed.

use dreist_engine::pattern::{PatternData, StepData, TRACKS};
use dreist_engine::sequencer::{Clock, EngineState, EngineTiming, PadInfo, Rng, process_engine};

/// Sequenced note starts (absolute samples) of a four-on-the-floor track at
/// 120 BPM, 48 kHz, in 32-sample sub-blocks.
fn quarter_note_starts() -> Vec<f64> {
    einst::install();
    let mut p = PatternData::empty();
    for s in [0, 4, 8, 12] {
        p.tracks[0].steps[s] = StepData { on: true, ..StepData::off() };
    }
    let mut st = EngineState { gate: true, ..EngineState::default() };
    st.needs_resync = true;
    let timing = EngineTiming::default();
    let one_shot = [true; TRACKS];
    let links = [[false; TRACKS]; TRACKS];
    let pads = PadInfo { one_shot: &one_shot, links: &links };
    let mut rng = Rng::new(1);
    let (sr, bpm, block) = (48_000.0f32, 120.0, 32usize);
    let mut starts = Vec::new();
    let mut ppq = 0.0;
    for b in 0..2000 {
        let clock = Clock { bpm, sample_rate: sr, ppq_block: ppq, ppq_now: ppq };
        let mut out = Vec::new();
        process_engine(&mut st, Some(&p), &timing, &clock, 0.0, block as f64, &pads, &mut rng, &mut out, &mut Vec::new(), None);
        starts.extend(out.iter().map(|n| (b * block) as f64 + n.start));
        ppq += block as f64 * bpm / 60.0 / sr as f64;
    }
    starts
}

/// v1 [re:01fcac] reads each sub-block's window from `start − 1`: after
/// the first block a beat (24000 samples) starts 2 samples late (the
/// reference: 1).
#[test]
fn sequenced_notes_start_one_sample_after_the_reference() {
    let starts = quarter_note_starts();
    assert_eq!(starts.len(), 3, "{starts:?}");
    // The first window starts a sample before 0: the downbeat too starts at 1.
    assert_eq!(starts[0], 1.0);
    for (t, want) in starts[1..].iter().zip([24_002.0, 48_002.0]) {
        assert!((t - want).abs() < 0.01, "{starts:?}");
    }
}

/// v1 [re:70951b]: 128 normal and 8 timestretch voices.
#[test]
fn timestretch_pool_holds_eight_voices() {
    einst::install();
    assert_eq!(dreist_dsp::rules::get().timestretch_voices, 8);
}
