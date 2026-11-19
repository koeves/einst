//! What this editor knows about Einst beyond the shared profile
//! (`dreist_formats::profile`): the About text, the
//! effect menu and how documents (v1 files among them) are read.

use std::path::Path;

use dreist_formats::Node;

/// The About text (three lines).
pub const ABOUT: &str = "Einst: drum sampler, sequencer and performance instrument,\na rebuild of the original v1 instrument.\nNative for macOS, Windows and Linux, written in Rust.";

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

/// What Einst adds to the shared editor parts: v1's own effect value texts
/// (the beat repeater's length in whole beats 1..16, [re:8ea5d7]).
pub(crate) fn install() {
    use dreist_editor_core::fxtext::{C, Fx};
    dreist_editor_core::fxtext::install_rows(&[(38, 1, 1.0, 16.0, Fx::Conv(C::Int))]);
}

/// Tests run under Einst's profile (and effect nodes, rules, voice model).
#[cfg(test)]
pub(crate) fn test_setup() {
    einst::install();
    install();
}
