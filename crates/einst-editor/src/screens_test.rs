//! Pixel regression: fingerprints of the editor rendered in fixed states
//! (every editor and utility tab, a transmod source, the sampler pad mode,
//! the dialogs, a menu) so refactors are checked pixel for pixel
//! (`tests/golden/screens.txt`; `UPDATE_GOLDEN=1` rewrites it). Skipped
//! without the factory content.

use super::*;
use geist_testkit::{Golden, fnv_bytes};

fn editor() -> Option<Editor> {
    let p = std::path::PathBuf::from(std::env::var("HOME").ok()?).join("Documents/FXpansion/Geist2/Content/Geist2 Demo Factory/Presets/Hollow.geist");
    let doc = geist_formats::Project::load(&p).ok()?.raw;
    let mut model = Model::new(doc);
    model.samples = Some(geist_formats::model::samples_dir(&p));
    model.name = Some("Hollow".into());
    let mut ed = Editor::new(crate::DEFAULT_THEME, model).ok()?;
    ed.sync();
    Some(ed)
}

/// Click the first control named `name` (a `/`-separated path of names).
fn click(ed: &mut Editor, spec: &str, button: geist_gui::app::MouseButton) {
    let mut path = Vec::new();
    for part in spec.split('/') {
        let base = ed.app.ctrl(&path).expect("control").clone();
        path.extend(base.path_of(part).unwrap_or_else(|| panic!("no control {part}")));
    }
    let (x, y) = ed.app.origin(&path);
    let c = ed.app.ctrl(&path).unwrap();
    let (cx, cy) = (x as f32 + c.rect.w as f32 / 2.0, y as f32 + c.rect.h as f32 / 2.0);
    for ev in ed.app.mouse_move(cx, cy) {
        ed.handle(&ev);
    }
    for ev in ed.app.mouse_down(button) {
        ed.handle(&ev);
    }
    for ev in ed.app.mouse_up(button) {
        ed.handle(&ev);
    }
}

fn shot(ed: &mut Editor) -> u64 {
    fnv_bytes(ed.render(1.0).data())
}

#[test]
fn screens_match_fingerprints() {
    let Some(mut ed) = editor() else { return };
    let mut golden = Golden::open("screens");
    // Deterministic rendering first: the same state twice.
    let a = shot(&mut ed);
    assert_eq!(a, shot(&mut ed), "rendering is not deterministic");
    golden.check("main", a, &[]);
    for tab in 0..7 {
        let mut ed = editor().unwrap();
        ed.model.editor_tab = tab;
        ed.sync();
        golden.check(&format!("editor-tab-{tab}"), shot(&mut ed), &[]);
    }
    let mut ed = editor().unwrap();
    ed.model.utility_tab = 1;
    ed.sync();
    golden.check("transmod", shot(&mut ed), &[]);
    ed.model.source = 3;
    ed.sync();
    golden.check("transmod-source", shot(&mut ed), &[]);
    let mut ed = editor().unwrap();
    let e = ed.model.engine;
    ed.model.engine_mut(e).set("EnginePadMode", "Sampler");
    ed.sync();
    golden.check("sampler-pads", shot(&mut ed), &[]);
    for modal in ["about", "preferences", "export", "filefind", "metadata"] {
        let mut ed = editor().unwrap();
        ed.show_modal(modal);
        // Let the dialog slide in.
        std::thread::sleep(std::time::Duration::from_millis(600));
        ed.animate();
        golden.check(&format!("modal-{modal}"), shot(&mut ed), &[]);
    }
    let mut ed = editor().unwrap();
    click(&mut ed, "Header/OptionsButton", geist_gui::app::MouseButton::Left);
    golden.check("options-menu", shot(&mut ed), &[]);
    golden.finish();
}

/// Edits repaint only what they change (`App::invalidate_changes`): after
/// each, the incrementally repainted frame equals a full render.
#[test]
fn partial_repaints_match_full_renders() {
    let Some(mut ed) = editor() else { return };
    // Repainted regions match a full render up to anti-aliasing rounding
    // (paths are translated by the region's origin, a level or two, and
    // the odd edge pixel of a knob arc a little more); a missed repaint
    // leaves stale content, far off over many pixels.
    let check = |ed: &mut Editor, what: &str| {
        let partial = ed.render_frame(1.0).clone();
        let full = ed.render(1.0);
        let w = full.width() as usize;
        let diffs: Vec<(u8, usize)> =
            partial.data().chunks(4).zip(full.data().chunks(4)).enumerate().map(|(i, (a, b))| (a.iter().zip(b).map(|(x, y)| x.abs_diff(*y)).max().unwrap_or(0), i)).filter(|d| d.0 > 4).collect();
        let worst = diffs.iter().copied().max_by_key(|d| d.0).unwrap_or((0, 0));
        assert!(diffs.len() <= 4 && worst.0 <= 16, "{what}: stale pixels after a partial repaint ({} off, worst by {} at {}, {})", diffs.len(), worst.0, worst.1 % w, worst.1 / w);
    };
    let _ = ed.render_frame(1.0);
    check(&mut ed, "nothing changed");
    // Region repaints of an unchanged window.
    for r in [
        geist_gui::Rect::new(0, 0, 1400, 850),
        geist_gui::Rect::new(100, 200, 300, 150),
        geist_gui::Rect::new(390, 300, 200, 100),
        geist_gui::Rect::new(5, 600, 500, 200),
        geist_gui::Rect::new(700, 150, 600, 300),
    ] {
        ed.app.invalidate_rect(r);
        check(&mut ed, &format!("repainting {r:?} unchanged"));
    }
    // A layer knob moved by host automation.
    let e = ed.model.engine;
    let cutoff = geist_engine::binding::Message::new(geist_engine::binding::id::SET_LAYER_PARAMETER, &[e as i32, 0, 0], &[0.0, 0.2, 0.0]);
    ed.automation(&[cutoff]);
    check(&mut ed, "automation");
    // A header toggle clicked.
    click(&mut ed, "AllPads", geist_gui::app::MouseButton::Left);
    check(&mut ed, "click");
    // Recorded steps from the engine.
    let w =
        geist_engine::StepWrite { engine: e, pattern: ed.model.current_pattern(e), track: 0, step: 3, data: geist_engine::pattern::StepData { on: true, ..geist_engine::pattern::StepData::off() } };
    ed.step_writes(&[w]);
    check(&mut ed, "step writes");
    // Another editor tab.
    ed.model.editor_tab = 5;
    ed.sync();
    check(&mut ed, "tab");
}
