//! Einst's app profile on dreist-commons (`dreist_formats::profile`): its
//! names and theme, its resources (this repository's `assets/ui` and the v1
//! factory data in `assets/v1/System`), the `.einst*` file family, v1's
//! effect menu, v1 documents read through `einst_formats` and v1's effect
//! nodes from `einst_dsp`. [`install`] it before any window, document or
//! effect is created.

pub mod v1io;

use dreist_formats::profile::{EINST, Profile, Resources};

pub static PROFILE: Profile = Profile {
    product: "Einst",
    about: "Einst: drum sampler, sequencer and performance instrument, a rebuild of the original v1 instrument on Dreist. Native for macOS, written in Rust.",
    theme: "Einst",
    ui_family: "Einst Sans",
    icon_family: "Einst-Icons",
    settings_dir: "Einst",
    legacy_settings_dirs: &[],
    settings_env: "EINST_SETTINGS_DIR",
    ui: Resources { env: "EINST_UI", bundle_dir: "Einst", dev_dirs: &[concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/ui")] },
    factory: Resources { env: "EINST_RESOURCES", bundle_dir: "Factory", dev_dirs: &[concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/v1/System")] },
    save: EINST,
    // v1 parts open as projects (into engine 0).
    open_also: &[
        dreist_formats::profile::LEGACY.kit,
        dreist_formats::profile::LEGACY.pad,
        dreist_formats::profile::LEGACY.engine,
        dreist_formats::profile::LEGACY.pattern,
        dreist_formats::profile::LEGACY.bank,
    ],
    effect_menu,
    read_document: v1io::read_document,
    read_project: v1io::read_project,
};

/// v1's effect menu: its 32 types in v1's order and categories.
pub fn effect_menu(category: usize) -> Vec<usize> {
    einst_formats::v1::effect_menu(category).filter_map(dreist_formats::params::effect_type_index).collect()
}

/// Install Einst's profile and effect nodes.
pub fn install() {
    dreist_formats::profile::install(&PROFILE);
    dreist_dsp::fx::install_factory(dreist_dsp::fx::NodeFactory { create: einst_dsp::create, internal_ports: einst_dsp::internal_ports });
}
