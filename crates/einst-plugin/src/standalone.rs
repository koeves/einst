//! The standalone app's side of the audio / MIDI settings panels
//! (`einst_editor::devices`): audio devices and MIDI ports through
//! `dreist_host` (cpal, midir), and an audio restart that relaunches the app
//! with the current document (nih-plug opens its devices once, at start-up).

use einst_editor::devices::{AudioSetup, DeviceHost, MidiSetup};

/// Environment variable naming a session file (`Session` as JSON) the
/// relaunched app starts from.
pub const SESSION_ENV: &str = "EINST_SESSION_FILE";

pub struct Host;

impl DeviceHost for Host {
    fn audio(&self, output: &str, input: &str) -> AudioSetup {
        dreist_host::audio::setup(output, input)
    }

    fn midi(&self) -> MidiSetup {
        dreist_host::midi::setup("Einst")
    }

    fn restart(&self, doc: &dreist_formats::Node, samples: Option<&std::path::Path>) {
        let session = crate::Session { doc: doc.to_xml(), samples: samples.map(|p| p.to_string_lossy().into_owned()) };
        dreist_host::session::relaunch(SESSION_ENV, "einst-session", &session);
    }
}

/// The session a relaunch handed over (read once, then removed).
pub fn take_session() -> Option<crate::Session> {
    dreist_host::session::take(SESSION_ENV)
}
