//! Standalone Einst (nih-plug's standalone wrapper: CoreAudio or WASAPI
//! through cpal, or JACK).
//!
//! The devices come from the audio / MIDI settings panels
//! (`einst_editor::devices`, options `Standalone*`); nih-plug's command
//! line flags (`--output-device`, `--midi-input`, ...) override them.

use einst_plugin::EinstPlugin;
use nih_plug::prelude::*;

/// Command line plus the stored device choices for flags not given.
fn args() -> Vec<String> {
    let mut args: Vec<String> = std::env::args().collect();
    let o = einst_editor::options::Options::load();
    let given = |args: &[String], flag: &str| args.iter().any(|a| a == flag || a.starts_with(&format!("{flag}=")));
    let s = einst_editor::devices::Settings::load(&o);
    // The system backend even when JACK is installed (nih-plug's "auto"
    // prefers JACK).
    let backend = if cfg!(windows) {
        "wasapi"
    } else if cfg!(target_os = "macos") {
        "core-audio"
    } else {
        "alsa"
    };
    let rate = if s.rate > 0 { Some(s.rate) } else { dreist_host::audio::output_sample_rate(&s.output) };
    for (flag, v) in [
        ("--backend", Some(backend.to_string())),
        ("--midi-input", Some(s.midi_in.clone())),
        ("--midi-output", Some(s.midi_out.clone())),
        ("--output-device", Some(s.output.clone())),
        ("--input-device", Some(s.input.clone())),
        // nih-plug defaults to 48 kHz; on a device running at another
        // rate the system resamples and hands over more frames than the
        // period size (558 for 512 at 44.1 kHz), which the wrapper
        // rejects. Default to the output device's own rate.
        ("--sample-rate", rate.map(|r| r.to_string())),
        ("--period-size", (s.buffer > 0).then(|| s.buffer.to_string())),
    ] {
        if let Some(v) = v.filter(|v| !v.is_empty()) {
            if !given(&args, flag) {
                args.push(flag.to_string());
                args.push(v);
            }
        }
    }
    args
}

fn main() {
    einst::install();
    nih_export_standalone_with_args::<EinstPlugin, _>(args());
}
