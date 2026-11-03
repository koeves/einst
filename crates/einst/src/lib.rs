//! Einst's app profile on dreist-commons (`dreist_formats::profile`): its
//! names and theme, its resources (this repository's `assets/ui` and the v1
//! factory data in `assets/v1/System`), the `.einst*` file family, v1's
//! effect menu, v1 documents read through `einst_formats` and v1's effect
//! nodes from `einst_dsp`. [`install`] it before any window, document or
//! effect is created.

pub mod v1io;

use dreist_formats::profile::{EINST, Profile, Resources};

/// Einst's logo mark, an "E" like its app icon: a stem and three bars.
pub const LOGO_MARK: &[&[(f32, f32)]] = &[
    &[(9.0, 9.0), (16.0, 9.0), (16.0, 41.0), (9.0, 41.0)],
    &[(16.0, 9.0), (41.0, 9.0), (41.0, 16.0), (16.0, 16.0)],
    &[(16.0, 21.5), (36.0, 21.5), (36.0, 28.5), (16.0, 28.5)],
    &[(16.0, 34.0), (41.0, 34.0), (41.0, 41.0), (16.0, 41.0)],
];

pub static PROFILE: Profile = Profile {
    product: "Einst",
    theme: "Einst",
    ui_family: "Einst Sans",
    icon_family: "Einst-Icons",
    logo_mark: LOGO_MARK,
    ui: Resources { env: "EINST_UI", bundle_dir: "Einst", dev_dirs: &[concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/ui")] },
    factory: Resources { env: "EINST_RESOURCES", bundle_dir: "Factory", dev_dirs: &[concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/v1/System")] },
    save: EINST,
};

/// The extensions Einst's Open dialog offers besides projects: v1 parts
/// open as projects (into engine 0).
pub const OPEN_ALSO: &[&str] =
    &[dreist_formats::profile::LEGACY.kit, dreist_formats::profile::LEGACY.pad, dreist_formats::profile::LEGACY.engine, dreist_formats::profile::LEGACY.pattern, dreist_formats::profile::LEGACY.bank];

/// v1's effect menu: its 32 types in v1's order and categories.
pub fn effect_menu(category: usize) -> Vec<usize> {
    einst_formats::v1::effect_menu(category).filter_map(dreist_formats::params::effect_type_index).collect()
}

/// v1's behaviour where it differs from the reference: the sampler stops a
/// threshold-started recording after 1 s of silence (`silence > sr`); the envelope's end
/// rules (ev_dc13a646) and the forward loop without crossfade, wrapping at
/// the loop end minus 3 frames (ev_29992d9a).
pub const RULES: dreist_dsp::rules::Rules = dreist_dsp::rules::Rules { sampler_auto_stop_seconds: 1.0, envelope_v1_ends: true, loop_v1: true };

/// Install Einst's profile, effect nodes, v1's fast 2^x, rules and voice
/// parameter model.
pub fn install() {
    dreist_formats::profile::install(&PROFILE);
    dreist_dsp::fx::install_factory(dreist_dsp::fx::NodeFactory { create: einst_dsp::create, internal_ports: einst_dsp::internal_ports });
    dreist_dsp::fx::install_exp2(einst_dsp::fast_exp2);
    dreist_dsp::rules::install(RULES);
    dreist_dsp::model::install(&einst_dsp::voice_model::V1Voice);
}
