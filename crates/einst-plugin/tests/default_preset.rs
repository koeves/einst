//! A new instance with no host session and no project starts from the
//! default preset Tools › Save as default preset wrote. One test in its own
//! binary: it points the settings folder at a scratch one.

#[test]
fn new_instance_starts_from_the_saved_default_preset() {
    let dir = std::env::temp_dir().join(format!("einst-default-preset-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    // SAFETY: the only test in this binary, set before anything reads it.
    unsafe { std::env::set_var(einst_editor::app::SETTINGS_ENV, &dir) };
    einst::install();
    // What Tools › Save as default preset writes.
    let doc = dreist_formats::Node::parse(r#"<root version="3000" CurrentEngine="0"><TimeInfo Tempo="97" /></root>"#).unwrap();
    let preset = einst_editor::options::default_preset().unwrap();
    std::fs::create_dir_all(&dir).unwrap();
    einst_editor::project::save(&doc, None, &preset, false).unwrap();
    let loaded = einst_plugin::default_preset_engine(44_100.0, Default::default(), &mut Default::default());
    let _ = std::fs::remove_dir_all(&dir);
    let (g, samples) = loaded.unwrap_or_else(|| panic!("{} was not loaded", preset.display()));
    assert_eq!(g.tempo(), 97.0);
    assert_eq!(samples, dreist_formats::model::samples_dir(&preset));
}
