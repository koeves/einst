//! Standalone Einst (nih-plug's standalone wrapper: CoreAudio, WASAPI or ALSA
//! through cpal, or JACK).
//!
//! The devices come from the audio / MIDI settings panels
//! (`einst_editor::devices`, options `Standalone*`); nih-plug's command
//! line flags (`--output-device`, `--midi-input`, ...) override them.

use einst_plugin::EinstPlugin;
use nih_plug::prelude::*;

/// Command line plus the stored device choices for flags not given.
fn args(mut args: Vec<String>) -> Vec<String> {
    let o = einst_editor::options::Options::load();
    let given = |args: &[String], flag: &str| {
        // nih-plug's one-letter forms of the flags set here.
        let short = match flag {
            "--backend" => Some("-b"),
            "--sample-rate" => Some("-r"),
            "--period-size" => Some("-p"),
            _ => None,
        };
        args.iter().any(|a| a == flag || a.starts_with(&format!("{flag}=")) || short.is_some_and(|s| a.starts_with(s)))
    };
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
        // The default buffer is nih-plug's choice: 512, or 1024 when PulseAudio
        // is the sound server (its ALSA plugin underruns at 512).
        ("--period-size", (s.buffer > 0 && s.buffer != einst_editor::devices::DEFAULT_BUFFER).then(|| s.buffer.to_string())),
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
    // Debug runs (`--debug`, `--play <file>`, ...; `dreist_engine::diag`).
    let (debug, rest) = match dreist_engine::diag::DebugArgs::split(std::env::args().collect()) {
        Ok(split) => split,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(2);
        }
    };
    if debug.debug {
        if let Some(song) = &debug.play {
            if let Err(e) = hand_over(song) {
                eprintln!("{}: {e}", song.display());
                std::process::exit(2);
            }
        }
    }
    let args = args(rest);
    if debug.debug {
        dreist_engine::diag::start_run("Einst", env!("CARGO_PKG_VERSION"), debug, &args);
    }
    nih_export_standalone_with_args::<EinstPlugin, _>(args);
    if let Some(status) = dreist_engine::diag::end_run() {
        std::process::exit(status);
    }
}

/// Start with the project at `song` (the session file an audio restart
/// hands over).
fn hand_over(song: &std::path::Path) -> Result<(), String> {
    let doc = dreist_formats::Project::load(song).map_err(|e| e.to_string())?.raw;
    let samples = dreist_formats::model::samples_dir(song);
    let session = einst_plugin::Session { doc: doc.to_xml(), samples: Some(samples.to_string_lossy().into_owned()) };
    let file = std::env::temp_dir().join(format!("einst-debug-{}.json", std::process::id()));
    std::fs::write(&file, dreist_host::serde_json::to_string(&session).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    // SAFETY: set before any other thread starts.
    unsafe { std::env::set_var(einst_plugin::standalone::SESSION_ENV, &file) };
    Ok(())
}
