//! The screens golden and repaint checks under the reference profile.

use super::*;
use crate::screens::{click, editor};

#[test]
fn screens_match_fingerprints() {
    crate::app::test_setup();
    crate::screens::check_screens("screens");
}

/// Edits repaint only what they change (`App::invalidate_changes`): after
/// each, the incrementally repainted frame equals a full render.
#[test]
fn partial_repaints_match_full_renders() {
    crate::app::test_setup();
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
        dreist_gui::Rect::new(0, 0, 1135, 640),
        dreist_gui::Rect::new(100, 200, 300, 150),
        dreist_gui::Rect::new(430, 300, 200, 100),
        dreist_gui::Rect::new(5, 500, 500, 140),
        dreist_gui::Rect::new(700, 150, 435, 300),
    ] {
        ed.app.invalidate_rect(r);
        check(&mut ed, &format!("repainting {r:?} unchanged"));
    }
    // A layer knob moved by host automation.
    let e = ed.model.engine;
    let cutoff = dreist_engine::binding::Message::new(dreist_engine::binding::id::SET_LAYER_PARAMETER, &[e as i32, 0, 0], &[0.0, 0.2, 0.0]);
    ed.automation(&[cutoff]);
    check(&mut ed, "automation");
    // A front-panel toggle clicked.
    click(&mut ed, "Transport/MetroButton", dreist_gui::app::MouseButton::Left);
    check(&mut ed, "click");
    // Recorded steps from the engine.
    let w =
        dreist_engine::StepWrite { engine: e, pattern: ed.model.current_pattern(e), track: 0, step: 3, data: dreist_engine::pattern::StepData { on: true, ..dreist_engine::pattern::StepData::off() } };
    ed.step_writes(&[w]);
    check(&mut ed, "step writes");
    // Another LCD page.
    ed.model.editor_tab = crate::front::SONG_PAGE;
    ed.sync();
    check(&mut ed, "tab");
}

/// Each page tab shows its own page on the LCD (and only that one), and
/// the browser column's tabs switch between the browser and the mapping.
#[test]
fn page_tabs_switch_the_lcd_page() {
    crate::app::test_setup();
    let ed = Editor::new(dreist_gui::skin::default_theme(), Model::new(crate::project::new_document()));
    let mut ed = ed.expect("editor");
    let shown = |ed: &Editor, panel: &str| -> Vec<usize> {
        let p = ed.app.root.find(panel).expect("tab panel");
        p.children.iter().filter(|c| c.visible).map(|c| c.id as usize).collect()
    };
    let tabs = ed.app.root.find("MainTabButtons").expect("page tabs").children.len();
    assert_eq!(tabs, crate::front::PAGES.len());
    for tab in 0..tabs {
        let g = ed.app.root.path_of("MainTabButtons").unwrap();
        let (x, y) = ed.app.origin(&[g.clone(), vec![tab]].concat());
        let c = ed.app.ctrl(&[g, vec![tab]].concat()).unwrap().clone();
        let (cx, cy) = (x as f32 + c.rect.w as f32 / 2.0, y as f32 + c.rect.h as f32 / 2.0);
        for ev in ed.app.mouse_move(cx, cy) {
            ed.handle(&ev);
        }
        for ev in ed.app.mouse_down(dreist_gui::app::MouseButton::Left) {
            ed.handle(&ev);
        }
        for ev in ed.app.mouse_up(dreist_gui::app::MouseButton::Left) {
            ed.handle(&ev);
        }
        assert_eq!(ed.model.editor_tab, tab, "tab {tab} ({})", crate::front::PAGES[tab].0);
        assert_eq!(shown(&ed, "MainTabs"), vec![tab], "page {tab} alone on the LCD");
        let g = ed.app.root.find("MainTabButtons").unwrap();
        assert!(g.children.iter().enumerate().all(|(i, b)| b.on == (i == tab)), "only tab {tab} lit");
    }
    for (tab, _) in [(crate::front::MAPPING_PAGE, "mapping"), (crate::front::BROWSER_PAGE, "browser")] {
        ed.model.utility_tab = tab;
        ed.sync();
        assert_eq!(shown(&ed, "BrowserTabs"), vec![tab]);
    }
    // The window keeps v1's size.
    assert_eq!(ed.app.size(), crate::front::WINDOW);
}
