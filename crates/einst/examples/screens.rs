//! Render Einst's editor to PNGs: `screens [out_dir]` (default
//! `docs/screens/m4`). Uses the demo project when installed, else a new
//! project; one image per LCD page, the mapping column, the dialogs
//! (About, preferences, export, audio and MIDI settings, MIDI console),
//! the effect panels authored for the newer types, the slicer and the
//! main menu.

use einst_editor::{Editor, Model};

/// The LCD pages' image names, in tab order.
const PAGES: [&str; 9] = ["pattern", "scene", "song", "pad-layers", "layer-mixer", "pad-mixer", "engine-mixer", "global-mixer", "sampler"];

fn editor() -> Editor {
    let mut ed = einst_editor::screens::editor().unwrap_or_else(|| {
        let model = Model::new(einst_editor::project::new_document());
        Editor::new(dreist_gui::skin::default_theme(), model).expect("editor")
    });
    ed.sync();
    ed
}

fn main() {
    einst::install();
    let out = std::path::PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| "docs/screens/m4".into()));
    std::fs::create_dir_all(&out).expect("out dir");
    let save = |ed: &mut Editor, name: &str| {
        let p = out.join(format!("{name}.png"));
        ed.render(1.0).save_png(&p).expect("png");
        println!("{}", p.display());
    };
    let mut ed = editor();
    save(&mut ed, "main");
    for (tab, name) in PAGES.iter().enumerate() {
        let mut ed = editor();
        ed.model.editor_tab = tab;
        ed.sync();
        save(&mut ed, name);
    }
    // The pad view of the Pad/Layers page (velocity split lanes under
    // their ruler): a press on the pad item.
    let mut ed = editor();
    ed.model.editor_tab = 3;
    ed.sync();
    if let Some(p) = ed.app.root.path_of("PadSelection") {
        let ev = dreist_gui::app::UiEvent::Mouse { path: p, overload: "PadSelectionButton".into(), kind: dreist_gui::app::MouseKind::Down, x: 2.0, y: 2.0 };
        ed.handle(&ev);
        // Velocity split mode: the lanes show.
        let (e, pad) = (ed.model.engine, ed.model.pad[ed.model.engine]);
        ed.model.engine_mut(e).child_mut_or_insert("Pad", &format!("Pad{pad}")).set("PadMode", dreist_dsp::voices::LAYER_MODE_NAMES[1]);
        ed.sync();
        save(&mut ed, "pad-view");
    }
    let mut ed = editor();
    ed.model.utility_tab = 1;
    ed.sync();
    save(&mut ed, "mapping");
    for modal in ["about", "preferences", "export", "audio", "midi", "console"] {
        let mut ed = editor();
        ed.show_modal(modal);
        std::thread::sleep(std::time::Duration::from_millis(600));
        ed.animate();
        save(&mut ed, modal);
    }
    let mut ed = editor();
    einst_editor::screens::newer_effects(&mut ed);
    save(&mut ed, "fx-newer-types");
    let mut ed = editor();
    if ed.open_slicer(&dreist_formats::legacy::demo("Hits/Cymbals/CY Cym Ar82 01 C2A St GB.wav")) {
        std::thread::sleep(std::time::Duration::from_millis(400));
        ed.animate();
        save(&mut ed, "slicer");
    }
    // The master section's main menu.
    let mut ed = editor();
    einst_editor::screens::click(&mut ed, "UtilityControls/GridButton", dreist_gui::app::MouseButton::Left);
    save(&mut ed, "main-menu");
}
