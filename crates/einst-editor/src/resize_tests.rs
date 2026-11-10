//! Live resizing in the v1 layout: its window is fixed (no anchors,
//! splitters or sizer) but for the browser column shown or hidden, so what
//! a live window goes through (host size reports on the way, a degenerate
//! 0 x 0 one included, the browser toggled back and forth, UI scales)
//! must leave the frame equal to an editor opened in the final state, and
//! every state keeps the panels inside their parents.

use crate::Editor;
use crate::front::{PAGES, WINDOW};
use crate::screens::editor;
use dreist_gui::{Ctrl, Rect};

/// v1's pad modes.
const PAD_MODES: [&str; 4] = ["Normal", "Chromatic", "Velocity", "MuteAndSolo"];

fn set_view(ed: &mut Editor, tab: usize, mode: &str) {
    ed.model.editor_tab = tab;
    let e = ed.model.engine;
    ed.model.engine_mut(e).set("EnginePadMode", mode);
    ed.sync();
    let _ = ed.render_frame(1.0);
}

/// A size report from the window (the host's view resized, a display
/// change), then a frame.
fn resize(ed: &mut Editor, w: i32, h: i32) {
    ed.app.resize(w, h);
    ed.app.dirty = true;
    let _ = ed.render_frame(1.0);
}

fn browser(ed: &mut Editor, hidden: bool) {
    ed.set_browser_hidden(hidden);
    let _ = ed.render_frame(1.0);
}

fn diff(a: &dreist_gui::tiny_skia::Pixmap, b: &dreist_gui::tiny_skia::Pixmap) -> (usize, u8, Rect) {
    assert_eq!((a.width(), a.height()), (b.width(), b.height()));
    let w = a.width() as i32;
    let (mut n, mut worst) = (0, 0u8);
    let (mut x0, mut y0, mut x1, mut y1) = (i32::MAX, i32::MAX, -1, -1);
    for (i, (p, q)) in a.data().chunks(4).zip(b.data().chunks(4)).enumerate() {
        let d = p.iter().zip(q).map(|(x, y)| x.abs_diff(*y)).max().unwrap_or(0);
        if d > 4 {
            n += 1;
            worst = worst.max(d);
            let (x, y) = (i as i32 % w, i as i32 / w);
            (x0, y0, x1, y1) = (x0.min(x), y0.min(y), x1.max(x), y1.max(y));
        }
    }
    (n, worst, Rect::new(x0, y0, x1 - x0 + 1, y1 - y0 + 1))
}

/// `DREIST_STAGE=dir` keeps the frames of a failure for review.
fn stage(name: &str, pm: &dreist_gui::tiny_skia::Pixmap) {
    if let Ok(d) = std::env::var("DREIST_STAGE") {
        let _ = std::fs::create_dir_all(&d);
        let _ = pm.save_png(std::path::Path::new(&d).join(format!("{name}.png")));
    }
}

/// After size reports, browser toggles and UI scale changes, the frame
/// equals that of an editor opened in the final state.
#[test]
fn live_resizes_match_a_fresh_editor() {
    crate::app::test_setup();
    for tab in 0..PAGES.len() {
        for mode in PAD_MODES {
            let Some(mut ed) = editor() else { return };
            set_view(&mut ed, tab, mode);
            for (w, h) in [(0, 0), (1, 1), (200, 120), (2000, 1200), WINDOW] {
                resize(&mut ed, w, h);
            }
            browser(&mut ed, true);
            let small = ed.app.size();
            resize(&mut ed, 0, 0);
            resize(&mut ed, small.0, small.1);
            browser(&mut ed, false);
            ed.app.ui_scale = 1.5;
            let _ = ed.render_frame(1.5);
            ed.app.ui_scale = 1.0;
            browser(&mut ed, true);
            browser(&mut ed, false);
            for hidden in [false, true] {
                browser(&mut ed, hidden);
                let live = ed.render(1.0);
                let mut fresh = editor().unwrap();
                fresh.set_browser_hidden(hidden);
                set_view(&mut fresh, tab, mode);
                let want = fresh.render(1.0);
                let (n, worst, at) = diff(&live, &want);
                if n > 4 {
                    stage(&format!("live-{tab}-{mode}-{hidden}"), &live);
                    stage(&format!("fresh-{tab}-{mode}-{hidden}"), &want);
                }
                assert!(n <= 4, "page {tab}, {mode}, browser hidden {hidden}: {n} pixels off (worst by {worst} in {at:?})");
            }
        }
    }
}

/// Visible controls sticking out of their parent by more than a few
/// pixels (scrolled content aside).
fn outside(ed: &Editor) -> Vec<String> {
    fn walk(c: &Ctrl, path: &str, out: &mut Vec<String>) {
        if c.rect.w <= 0 || c.rect.h <= 0 {
            return;
        }
        for k in &c.children {
            if !k.visible {
                continue;
            }
            let r = k.rect;
            let p = format!("{path}/{}{}", k.name, if k.overload.is_empty() { String::new() } else { format!("[{}]", k.overload) });
            let scrolled = c.class.contains("Scroll") || c.class.contains("TableView");
            if !scrolled && r.w > 0 && r.h > 0 && (r.x < -3 || r.y < -3 || r.x + r.w > c.rect.w + 3 || r.y + r.h > c.rect.h + 3) {
                out.push(p.clone());
            }
            walk(k, &p, out);
        }
    }
    let mut out = Vec::new();
    walk(&ed.app.root, "", &mut out);
    out
}

/// Every page, pad mode and browser state: the window has v1's size (the
/// browser column's width less when hidden), nothing sticks out of its
/// parent beyond the skin's own overhangs, the pads keep to their grid.
#[test]
fn layout_holds_in_every_state() {
    crate::app::test_setup();
    let Some(mut ed) = editor() else { return };
    let mut known = std::collections::HashSet::new();
    for tab in 0..PAGES.len() {
        for mode in PAD_MODES {
            set_view(&mut ed, tab, mode);
            known.extend(outside(&ed));
        }
    }
    for hidden in [false, true] {
        let mut ed = editor().unwrap();
        browser(&mut ed, hidden);
        let (w, h) = ed.app.size();
        assert_eq!(h, WINDOW.1);
        assert!(if hidden { w < WINDOW.0 } else { w == WINDOW.0 }, "browser hidden {hidden}: window {w}");
        for tab in 0..PAGES.len() {
            for mode in PAD_MODES {
                set_view(&mut ed, tab, mode);
                let what = format!("page {tab}, {mode}, browser hidden {hidden}");
                let new: Vec<_> = outside(&ed).into_iter().filter(|p| !known.contains(p)).collect();
                assert!(new.is_empty(), "{what}: outside their parents: {new:#?}");
                let root = ed.app.root.rect;
                let back = ed.app.root.find("BackPanel").expect("back panel").rect;
                assert!(back.x + back.w <= root.w && back.y + back.h <= root.h, "{what}: back panel {back:?} in {root:?}");
                let m = ed.app.root.find("EditorPads").expect("pads");
                let pads: Vec<Rect> = m.children.iter().filter(|c| c.visible && c.name.starts_with("Pad")).map(|c| c.rect).collect();
                for (i, a) in pads.iter().enumerate() {
                    assert!(a.x >= 0 && a.y >= 0 && a.x + a.w <= m.rect.w && a.y + a.h <= m.rect.h, "{what}: pad {a:?} outside {:?}", m.rect);
                    for b in &pads[i + 1..] {
                        assert!(a.x + a.w <= b.x || b.x + b.w <= a.x || a.y + a.h <= b.y || b.y + b.h <= a.y, "{what}: pads {a:?} and {b:?} overlap");
                    }
                }
            }
        }
    }
}
