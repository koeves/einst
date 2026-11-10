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

/// v1 [re:c0679e]: while recording, each sub-block writes the held graph
/// values into the current step, every sub-block, until the pattern wraps
/// (which clears them); not recording, nothing is written.
#[test]
fn graph_record_rewrites_the_current_step_until_the_wrap() {
    einst::install();
    let mut p = PatternData::empty();
    p.tracks[2].steps[0] = StepData { on: true, ..StepData::off() };
    let mut st = EngineState { gate: true, ..EngineState::default() };
    st.needs_resync = true;
    st.recording = true;
    let timing = EngineTiming::default();
    let one_shot = [true; TRACKS];
    let links = [[false; TRACKS]; TRACKS];
    let pads = PadInfo { one_shot: &one_shot, links: &links };
    let mut rng = Rng::new(1);
    let (sr, bpm, block) = (48_000.0f32, 120.0, 32usize);
    let lane = 11;
    // One 16th = 6000 samples. The current step is the window start
    // (2 samples before the block, see above) rounded to a 16th, so a
    // 16-step pattern wraps when that reaches 15.5 steps (93000).
    let mut ppq = 0.0;
    let mut writes = Vec::new();
    for b in 0..4000 {
        if b == 10 {
            st.set_graph_record(2, lane, 64);
        }
        let clock = Clock { bpm, sample_rate: sr, ppq_block: ppq, ppq_now: ppq };
        process_engine(&mut st, Some(&p), &timing, &clock, 0.0, block as f64, &pads, &mut rng, &mut Vec::new(), &mut Vec::new(), None);
        for w in st.graph_writes.drain(..) {
            writes.push((b, w));
        }
        ppq += block as f64 * bpm / 60.0 / sr as f64;
    }
    // Written from block 10 on, every block, into the step then current.
    assert_eq!(writes.first().map(|w| w.0), Some(10));
    let last = writes.last().unwrap().0;
    assert_eq!(writes.len(), last - 10 + 1, "one write per block");
    for (b, (t, step, l, v)) in &writes {
        assert_eq!((*t, *l, *v), (2, lane, 64.0 / 127.0));
        let at = (b * block) as f64 - 2.0;
        assert_eq!(*step as usize, (at / 6000.0).round() as usize, "block {b}");
    }
    // The wrap clears the held values: the last write is the last block
    // whose window starts before 93000.
    assert_eq!(last, (93_000 + 2 - 1) / block);
    assert!(!st.graph_rec_dirty[2]);

    // Not recording: held values are not written.
    let mut st2 = EngineState { gate: true, ..EngineState::default() };
    st2.needs_resync = true;
    st2.set_graph_record(2, lane, 64);
    let clock = Clock { bpm, sample_rate: sr, ppq_block: 0.0, ppq_now: 0.0 };
    process_engine(&mut st2, Some(&p), &timing, &clock, 0.0, block as f64, &pads, &mut rng, &mut Vec::new(), &mut Vec::new(), None);
    assert!(st2.graph_writes.is_empty());
}
