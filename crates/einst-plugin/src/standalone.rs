//! The standalone app's side of the audio / MIDI settings panels
//! (`einst_editor::devices`): audio devices through cpal, MIDI ports
//! through midir, and an audio restart that relaunches the app with the
//! current document (nih-plug opens its devices once, at start-up).

use cpal::traits::{DeviceTrait, HostTrait};
use einst_editor::devices::{AudioSetup, DeviceHost, MidiSetup};

/// Rates offered when a device supports them.
const RATES: [u32; 6] = [44100, 48000, 88200, 96000, 176400, 192000];

/// Environment variable naming a session file (`Session` as JSON) the
/// relaunched app starts from.
pub const SESSION_ENV: &str = "EINST_SESSION_FILE";

pub struct Host;

fn named(devs: impl Iterator<Item = cpal::Device>, name: &str) -> Option<cpal::Device> {
    devs.into_iter().find(|d| d.name().is_ok_and(|n| n == name))
}

impl DeviceHost for Host {
    fn audio(&self, output: &str, input: &str) -> AudioSetup {
        let host = cpal::default_host();
        let outputs: Vec<String> = host.output_devices().map(|d| d.filter_map(|d| d.name().ok()).collect()).unwrap_or_default();
        let inputs: Vec<String> = host.input_devices().map(|d| d.filter_map(|d| d.name().ok()).collect()).unwrap_or_default();
        let default = host.default_output_device();
        let default_output = default.as_ref().and_then(|d| d.name().ok()).unwrap_or_default();
        let out = if output.is_empty() { default } else { host.output_devices().ok().and_then(|d| named(d, output)) };
        let (mut rates, mut default_rate, mut out_channels) = (Vec::new(), 0, 0);
        if let Some(d) = &out {
            if let Ok(cfgs) = d.supported_output_configs() {
                let cfgs: Vec<_> = cfgs.collect();
                rates = RATES.iter().copied().filter(|r| cfgs.iter().any(|c| (c.min_sample_rate().0..=c.max_sample_rate().0).contains(r))).collect();
                out_channels = cfgs.iter().map(|c| c.channels()).max().unwrap_or(0);
            }
            default_rate = d.default_output_config().map(|c| c.sample_rate().0).unwrap_or(0);
        }
        let in_channels = if input.is_empty() { 0 } else { host.input_devices().ok().and_then(|d| named(d, input)).and_then(|d| d.default_input_config().ok()).map(|c| c.channels()).unwrap_or(0) };
        AudioSetup { outputs, inputs, default_output, rates, default_rate, out_channels, in_channels }
    }

    fn midi(&self) -> MidiSetup {
        let inputs = midir::MidiInput::new("Einst").map(|m| m.ports().iter().filter_map(|p| m.port_name(p).ok()).collect()).unwrap_or_default();
        let outputs = midir::MidiOutput::new("Einst").map(|m| m.ports().iter().filter_map(|p| m.port_name(p).ok()).collect()).unwrap_or_default();
        MidiSetup { inputs, outputs }
    }

    fn restart(&self, doc: &geist_formats::Node, samples: Option<&std::path::Path>) {
        let session = crate::Session { doc: doc.to_xml(), samples: samples.map(|p| p.to_string_lossy().into_owned()) };
        let file = std::env::temp_dir().join(format!("einst-session-{}.json", std::process::id()));
        let Ok(json) = serde_json::to_string(&session) else { return };
        if std::fs::write(&file, json).is_err() {
            return;
        }
        let Ok(exe) = std::env::current_exe() else { return };
        if std::process::Command::new(exe).env(SESSION_ENV, &file).spawn().is_ok() {
            std::process::exit(0);
        }
    }
}

/// The session a relaunch handed over (read once, then removed).
pub fn take_session() -> Option<crate::Session> {
    let file = std::env::var_os(SESSION_ENV)?;
    let s = std::fs::read_to_string(&file).ok()?;
    let _ = std::fs::remove_file(&file);
    serde_json::from_str(&s).ok()
}
