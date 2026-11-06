//! Pattern editor mouse and menu behaviour as v1's.

use dreist_gui::app::{MouseKind, UiEvent};

use crate::pattern::Tool;
use crate::{Editor, Model};

fn editor() -> Option<Editor> {
    crate::app::test_setup();
    let mut ed = Editor::new(dreist_gui::skin::default_theme(), Model::new(crate::project::new_document())).ok()?;
    ed.model.editor_tab = 0;
    ed.sync();
    Some(ed)
}

/// Grid point of `step` on the row of track 3.
fn at(ed: &Editor, step: usize) -> (f32, f32) {
    let lay = ed.pattern.as_ref().unwrap().layout(&ed.app, &ed.model).unwrap();
    (lay.step_x(step) + 2.0, lay.row_y(lay.row(3).unwrap()) + 2.0)
}

fn mouse(ed: &mut Editor, kind: MouseKind, step: usize) {
    let gp = ed.app.root.path_of("PatternGrid").unwrap();
    let (x, y) = at(ed, step);
    ed.handle(&UiEvent::Mouse { path: gp, overload: "PatternGrid".into(), kind, x, y });
}

fn on(ed: &Editor, step: usize) -> bool {
    let e = ed.model.engine;
    ed.model.pattern(e, ed.model.current_pattern(e)).tracks[3].steps[step].on
}

fn velocity(ed: &Editor, step: usize) -> f32 {
    let e = ed.model.engine;
    ed.model.pattern(e, ed.model.current_pattern(e)).tracks[3].steps[step].lanes[0]
}

fn set_tool(ed: &mut Editor, t: Tool) {
    ed.pattern.as_mut().unwrap().tool = t;
}

fn click(ed: &mut Editor, step: usize) {
    mouse(ed, MouseKind::Down, step);
    mouse(ed, MouseKind::Up, step);
}

fn selection(ed: &Editor) -> Vec<(usize, usize)> {
    ed.pattern.as_ref().unwrap().selection.iter().copied().collect()
}

#[test]
fn right_presses_during_a_brush_cycle_insert_patterns() {
    let Some(mut ed) = editor() else { return };
    let templates = crate::patmenu::pattern_templates(&dreist_formats::paths::skin_dir());
    if templates.len() < 2 {
        return;
    }
    set_tool(&mut ed, Tool::Pen);
    mouse(&mut ed, MouseKind::Down, 0);
    mouse(&mut ed, MouseKind::Drag, 7);
    assert!((0..8).all(|s| on(&ed, s)), "the brush draws");
    for k in 0..2 {
        mouse(&mut ed, MouseKind::RightDown, 7);
        let bits = &templates[k].1;
        for s in 0..8 {
            assert_eq!(on(&ed, s), bits[s % bits.len()], "pattern {k}, step {s}");
        }
        // The right release keeps the left button's brush.
        mouse(&mut ed, MouseKind::Up, 7);
        assert!(ed.app.menu.is_none(), "no track menu while the left button is held");
    }
    // The brush has stopped.
    mouse(&mut ed, MouseKind::Drag, 9);
    assert!(!on(&ed, 9));
    mouse(&mut ed, MouseKind::Up, 9);
    // Without the left button the right one opens the pen's track menu.
    mouse(&mut ed, MouseKind::RightDown, 2);
    assert!(ed.app.menu.is_some());
}

#[test]
fn shift_selection_as_v1() {
    let Some(mut ed) = editor() else { return };
    set_tool(&mut ed, Tool::Pen);
    for s in [1, 3, 5] {
        click(&mut ed, s);
    }
    ed.app.mods.shift = true;
    // Shift-click on an event selects it alone.
    click(&mut ed, 1);
    assert_eq!(selection(&ed), vec![(3, 1)]);
    click(&mut ed, 3);
    assert_eq!(selection(&ed), vec![(3, 3)]);
    // Shift+cmd adds.
    ed.app.mods.cmd = true;
    click(&mut ed, 5);
    assert_eq!(selection(&ed), vec![(3, 3), (3, 5)]);
    // The pen's shift+cmd click does not deselect; the multi tool's
    // toggles.
    click(&mut ed, 5);
    assert_eq!(selection(&ed), vec![(3, 3), (3, 5)]);
    set_tool(&mut ed, Tool::Multi);
    click(&mut ed, 5);
    assert_eq!(selection(&ed), vec![(3, 3)]);
    click(&mut ed, 5);
    assert_eq!(selection(&ed), vec![(3, 3), (3, 5)]);
    ed.app.mods.cmd = false;
    set_tool(&mut ed, Tool::Pen);
    // A shift-drag from empty space draws a box.
    mouse(&mut ed, MouseKind::Down, 0);
    mouse(&mut ed, MouseKind::Drag, 4);
    mouse(&mut ed, MouseKind::Up, 4);
    assert_eq!(selection(&ed), vec![(3, 1), (3, 3)]);
    // A shift-click on empty space deselects.
    click(&mut ed, 2);
    assert!(selection(&ed).is_empty());
    ed.app.mods.shift = false;
}

#[test]
fn pen_velocity_drag_moves_the_selection() {
    let Some(mut ed) = editor() else { return };
    set_tool(&mut ed, Tool::Pen);
    for s in [1, 3, 5] {
        click(&mut ed, s);
    }
    ed.pattern.as_mut().unwrap().selection = [(3, 1), (3, 5)].into_iter().collect();
    let before = [velocity(&ed, 1), velocity(&ed, 3), velocity(&ed, 5)];
    let gp = ed.app.root.path_of("PatternGrid").unwrap();
    let (x, y) = at(&ed, 1);
    ed.handle(&UiEvent::Mouse { path: gp.clone(), overload: "PatternGrid".into(), kind: MouseKind::Down, x, y });
    ed.handle(&UiEvent::Mouse { path: gp.clone(), overload: "PatternGrid".into(), kind: MouseKind::Drag, x, y: y + 4.0 });
    ed.handle(&UiEvent::Mouse { path: gp, overload: "PatternGrid".into(), kind: MouseKind::Up, x, y: y + 4.0 });
    let d = velocity(&ed, 1) - before[0];
    assert!(d < -0.01, "the drag lowers the velocity");
    assert!((velocity(&ed, 5) - (before[2] + d)).abs() < 1e-4, "the other selected event follows");
    assert_eq!(velocity(&ed, 3), before[1], "unselected events stay");
    assert_eq!(selection(&ed), vec![(3, 1), (3, 5)], "the pen keeps the selection");
}

#[test]
fn horizontal_zoom_is_per_pattern() {
    let Some(mut ed) = editor() else { return };
    let e = ed.model.engine;
    let zoom = |ed: &mut Editor, name: &str| {
        let p = ed.app.root.path_of(name).unwrap();
        ed.handle(&UiEvent::Clicked { path: p, name: name.into(), id: 0, on: false });
    };
    zoom(&mut ed, "ZoomInH");
    zoom(&mut ed, "ZoomInH");
    let cur = ed.model.current_pattern(e);
    assert_eq!(ed.model.pattern_view[e].zoom_of(cur), 3, "one step per click");
    ed.model.engine_mut(e);
    crate::pattern::set_current_pattern(&mut ed.model, e, cur + 1);
    assert_eq!(ed.model.current_pattern(e), cur + 1);
    assert_eq!(ed.model.pattern_view[e].zoom_of(cur + 1), 1, "another pattern keeps its own");
    ed.sync();
    assert!(!ed.app.root.find("ZoomOutH").unwrap().enabled, "zoom out stops at 1");
    for _ in 0..30 {
        zoom(&mut ed, "ZoomInH");
    }
    assert_eq!(ed.model.pattern_view[e].zoom_of(cur + 1), crate::pattern::MAX_ZOOM);
    ed.sync();
    assert!(!ed.app.root.find("ZoomInH").unwrap().enabled);
    assert_eq!(ed.model.pattern_view[e].zoom_of(cur), 3);
}

#[test]
fn graph_type_menu_lists_v1_graph_types() {
    let Some(mut ed) = editor() else { return };
    let gc = ed.app.root.path_of("GraphCombo").unwrap();
    ed.handle(&UiEvent::Clicked { path: gc.clone(), name: "GraphCombo".into(), id: 0, on: false });
    let items = ed.app.menu.as_ref().map(|m| m.items.clone()).unwrap_or_default();
    assert_eq!(items.len(), 21 + 1 + 6, "{items:?}");
    assert_eq!(items[0], "Velocity");
    assert_eq!(items[5], "Coarse Pitch");
    assert!(!items.iter().any(|i| i.contains("Duration") || i.contains("Link")));
    assert_eq!(items[20], "Free env depth 2");
    assert!(items[22].contains("Pad FX 1"));
    // Free env depth 2.
    ed.handle(&UiEvent::MenuSelected { anchor: gc, name: "GraphCombo".into(), index: 20 });
    let e = ed.model.engine;
    assert_eq!(ed.model.pattern_view[e].lane, dreist_formats::params::V1_ROUTING_LANE + 3);
}
