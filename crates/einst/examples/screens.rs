//! Render Einst's editor to PNGs: `screens [out_dir]` (default
//! `docs/screens`). Uses the Geist2 demo project when installed, else a new
//! project; one image per editor tab, the transmod page and the About box.

use dreist_editor::{Editor, Model};

fn editor() -> Editor {
    let mut ed = dreist_editor::screens::editor().unwrap_or_else(|| {
        let model = Model::new(dreist_editor::project::new_document());
        Editor::new(dreist_gui::skin::default_theme(), model).expect("editor")
    });
    ed.sync();
    ed
}

fn main() {
    einst::install();
    let out = std::path::PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| "docs/screens".into()));
    std::fs::create_dir_all(&out).expect("out dir");
    let save = |ed: &mut Editor, name: &str| {
        let p = out.join(format!("{name}.png"));
        ed.render(1.0).save_png(&p).expect("png");
        println!("{}", p.display());
    };
    let mut ed = editor();
    save(&mut ed, "main");
    for tab in 0..7 {
        let mut ed = editor();
        ed.model.editor_tab = tab;
        ed.sync();
        save(&mut ed, &format!("editor-tab-{tab}"));
    }
    let mut ed = editor();
    ed.model.utility_tab = 1;
    ed.sync();
    save(&mut ed, "transmod");
    let mut ed = editor();
    ed.show_modal("about");
    std::thread::sleep(std::time::Duration::from_millis(600));
    ed.animate();
    save(&mut ed, "about");
}
