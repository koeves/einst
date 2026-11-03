//! Open the editor in a window (no audio): `editor [project.geist]`.

use einst_editor::{Editor, Model};

fn main() {
    let arg = std::env::args().nth(1);
    let doc = arg.as_ref().map(|p| geist_formats::Project::load(std::path::Path::new(p)).expect("project").raw).unwrap_or_else(|| geist_formats::Node::new("root", None));
    let mut model = Model::new(doc);
    model.samples = arg.as_ref().map(|p| geist_formats::model::samples_dir(std::path::Path::new(p)));
    model.name = arg.as_ref().and_then(|p| std::path::Path::new(p).file_stem()).map(|s| s.to_string_lossy().into_owned());
    let ed = Editor::new(geist_gui::skin::DEFAULT_THEME, model).expect("editor");
    einst_editor::window::open_blocking(ed, |cmds| {
        for c in cmds {
            println!("{c:?}")
        }
    });
}
