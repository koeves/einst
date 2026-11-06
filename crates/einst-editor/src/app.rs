//! What this editor knows about Einst beyond the shared profile
//! (`dreist_formats::profile`): the About text, the settings folder, the
//! effect menu and how documents (v1 files among them) are read.

use std::path::Path;

use dreist_formats::Node;

/// The About text (three lines).
pub const ABOUT: &str = "Einst: drum sampler, sequencer and performance instrument,\na rebuild of the original v1 instrument.\nNative for macOS, written in Rust.";

/// Settings folder name (`~/Library/Application Support/<name>`).
pub const SETTINGS_DIR: &str = "Einst";

/// Earlier settings folder names, moved to [`SETTINGS_DIR`] on first use.
pub const LEGACY_SETTINGS_DIRS: &[&str] = &[];

/// Environment variable naming a separate settings folder.
pub const SETTINGS_ENV: &str = "EINST_SETTINGS_DIR";

/// Further extensions the Open dialog offers besides projects.
pub const OPEN_ALSO: &[&str] = einst::OPEN_ALSO;

/// v1's effect menu: the types offered in category `c` (0..6).
pub fn effect_menu(c: usize) -> Vec<usize> {
    einst::effect_menu(c)
}

/// A document file's root element, converted when it is a v1 file.
pub fn read_document(path: &Path) -> Option<Node> {
    einst::v1io::read_document(path)
}

/// A project to open and whether it was converted (v1 files are).
pub fn read_project(path: &Path) -> Option<(Node, bool)> {
    einst::v1io::read_project(path)
}

/// The Open dialog's extensions: projects of every family (Einst's first),
/// then [`OPEN_ALSO`].
pub fn open_extensions() -> Vec<&'static str> {
    let mut v = dreist_formats::profile::extensions_of(dreist_formats::profile::DocKind::Project);
    v.extend(OPEN_ALSO);
    v
}

/// Tests run under Einst's profile (and effect nodes, rules, voice model).
#[cfg(test)]
pub(crate) fn test_setup() {
    einst::install();
}
