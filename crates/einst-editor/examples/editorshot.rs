//! Render the editor for a project to PNG: `editorshot [project.geist] [out.png] [scale] [editor_tab] [utility_tab] [source]`
//! (`GEIST_PAD_MODE=Sampler` sets the engine's pad mode, `GEIST_METER_SECS=t`
//! plays the project for t seconds into the meters, `GEIST_SLICE=file` opens
//! the slicer, `GEIST_MODAL=about|preferences|export|filefind|metadata` a dialog,
//! `GEIST_CLICK=Ctrl/Name` clicks a control, e.g. `Header/OptionsButton`;
//! `GEIST_RIGHT=1` right-clicks it instead, `GEIST_MENU=a|b` then picks the
//! open menu's items ending in a, then b (`#3` picks index 3),
//! `GEIST_HOVER=1` leaves the pointer there and shows its tooltip).

use einst_editor::{Editor, Model};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let doc = args.get(1).filter(|s| !s.is_empty()).map(|p| geist_formats::Project::load(std::path::Path::new(p)).expect("project").raw).unwrap_or_else(|| geist_formats::Node::new("root", None));
    let out = args.get(2).map(String::as_str).unwrap_or("editor.png");
    let scale: f32 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(1.0);
    let mut model = Model::new(doc);
    model.samples = args.get(1).filter(|s| !s.is_empty()).map(|p| geist_formats::model::samples_dir(std::path::Path::new(p)));
    model.name = args.get(1).and_then(|p| std::path::Path::new(p).file_stem()).map(|s| s.to_string_lossy().into_owned());
    let mut ed = Editor::new(geist_gui::skin::DEFAULT_THEME, model).expect("editor");
    if let Some(t) = args.get(4).and_then(|s| s.parse().ok()) {
        ed.model.editor_tab = t;
    }
    if let Some(t) = args.get(5).and_then(|s| s.parse().ok()) {
        ed.model.utility_tab = t;
    }
    if let Some(t) = args.get(6).and_then(|s| s.parse().ok()) {
        ed.model.source = t;
    }
    if let Ok(mode) = std::env::var("GEIST_PAD_MODE") {
        let e = ed.model.engine;
        ed.model.engine_mut(e).set("EnginePadMode", &mode);
    }
    ed.sync();
    if std::env::var("GEIST_LEARN").is_ok() {
        ed.set_learn(true);
    }
    if let Ok(m) = std::env::var("GEIST_MODAL") {
        ed.show_modal(&m);
        // Let the dialog slide in.
        std::thread::sleep(std::time::Duration::from_millis(600));
        ed.animate();
    }
    if let Ok(spec) = std::env::var("GEIST_CLICK") {
        // "Parent/Name": the first Name under the first Parent.
        let mut path = Vec::new();
        for part in spec.split('/') {
            let base = ed.app.ctrl(&path).expect("control").clone();
            let rel = base.path_of(part).unwrap_or_else(|| panic!("no control {part}"));
            path.extend(rel);
        }
        let (x, y) = ed.app.origin(&path);
        let c = ed.app.ctrl(&path).unwrap();
        let (cx, cy) = (x as f32 + c.rect.w as f32 / 2.0, y as f32 + c.rect.h as f32 / 2.0);
        for ev in ed.app.mouse_move(cx, cy) {
            ed.handle(&ev);
        }
        if std::env::var("GEIST_HOVER").is_err() {
            let b = if std::env::var("GEIST_RIGHT").is_ok() { geist_gui::app::MouseButton::Right } else { geist_gui::app::MouseButton::Left };
            for ev in ed.app.mouse_down(b) {
                ed.handle(&ev);
            }
            for ev in ed.app.mouse_up(b) {
                ed.handle(&ev);
            }
        }
        let path = ed.app.tooltip_target();
        ed.handle(&geist_gui::app::UiEvent::ToolTip { path });
        for pick in std::env::var("GEIST_MENU").unwrap_or_default().split('|').filter(|s| !s.is_empty()) {
            let m = ed.app.menu.clone().expect("open menu");
            let i = pick.strip_prefix('#').and_then(|n| n.parse().ok()).or_else(|| m.items.iter().position(|s| s.ends_with(pick))).unwrap_or_else(|| panic!("{pick} in {:?}", m.items));
            ed.handle(&geist_gui::app::UiEvent::MenuSelected { anchor: m.anchor.clone(), name: String::new(), index: i });
        }
    }
    if let Ok(f) = std::env::var("GEIST_SLICE") {
        ed.open_slicer(std::path::Path::new(&f));
    }
    // GEIST_METER_SECS=t: play the project for t seconds and feed the meters.
    if let (Some(path), Some(secs)) = (args.get(1).filter(|s| !s.is_empty()), std::env::var("GEIST_METER_SECS").ok().and_then(|v| v.parse::<f32>().ok())) {
        let mut g = geist_engine::Geist::load(std::path::Path::new(path), 44100.0, geist_engine::RenderOptions::default()).expect("engine");
        g.start();
        let (mut l, mut r) = (vec![0.0f32; 512], vec![0.0f32; 512]);
        for _ in 0..(secs * 44100.0 / 512.0) as usize {
            g.process(&mut l, &mut r);
            ed.update_meters(&g.meters);
            g.meters.clear();
        }
    }
    ed.render(scale).save_png(out).expect("png");
    println!("{out}");
}
