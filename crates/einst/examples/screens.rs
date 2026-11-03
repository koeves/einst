//! Render Einst's editor to PNGs: `screens [out_dir]` (default
//! `docs/screens/m3a`). Uses the demo project when installed, else a new
//! project; one image per LCD page, the mapping column, the About and
//! preferences dialogs and the main menu.

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
    let out = std::path::PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| "docs/screens/m3a".into()));
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
    let mut ed = editor();
    ed.model.utility_tab = 1;
    ed.sync();
    save(&mut ed, "mapping");
    for modal in ["about", "preferences"] {
        let mut ed = editor();
        ed.show_modal(modal);
        std::thread::sleep(std::time::Duration::from_millis(600));
        ed.animate();
        save(&mut ed, modal);
    }
    // The master section's main menu.
    let mut ed = editor();
    einst_editor::screens::click(&mut ed, "UtilityControls/GridButton", dreist_gui::app::MouseButton::Left);
    save(&mut ed, "main-menu");
}
