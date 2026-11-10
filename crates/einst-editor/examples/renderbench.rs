//! Time full editor frames: `renderbench [project.geist] [scale] [frames]`.

use einst_editor::{Editor, Model};

fn main() {
    // Einst's profile and resources.
    einst::install();
    let args: Vec<String> = std::env::args().collect();
    let doc = args.get(1).filter(|s| !s.is_empty()).map(|p| dreist_formats::Project::load(std::path::Path::new(p)).expect("project").raw).unwrap_or_else(|| dreist_formats::Node::new("root", None));
    let scale: f32 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(2.0);
    if std::env::var("PLAYBENCH").is_ok() {
        playback(args.get(1).map(String::as_str).unwrap_or(""), scale);
        return;
    }
    let n: usize = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(20);
    // SYNCBENCH=1: only editor syncs (for profiling).
    if std::env::var("SYNCBENCH").is_ok() {
        let mut ed = Editor::new(dreist_gui::skin::default_theme(), Model::new(doc)).expect("editor");
        let t = std::time::Instant::now();
        for _ in 0..n {
            ed.sync();
        }
        println!("sync {:.2} ms", t.elapsed().as_secs_f64() * 1000.0 / n as f64);
        return;
    }
    let mut ed = Editor::new(dreist_gui::skin::default_theme(), Model::new(doc)).expect("editor");
    ed.sync();
    let _ = ed.render(scale);
    let t = std::time::Instant::now();
    for _ in 0..n {
        ed.app.dirty = true;
        let _ = ed.render(scale);
    }
    let per = t.elapsed().as_secs_f64() * 1000.0 / n as f64;
    println!("render {per:.1} ms/frame at scale {scale}");
    let t = std::time::Instant::now();
    for _ in 0..n {
        let _ = ed.take_commands();
        ed.animate();
    }
    println!("frame logic {:.2} ms", t.elapsed().as_secs_f64() * 1000.0 / n as f64);
    let t = std::time::Instant::now();
    for _ in 0..n {
        ed.sync();
    }
    println!("sync {:.2} ms", t.elapsed().as_secs_f64() * 1000.0 / n as f64);
    // Host automation moving a layer knob every frame (the plugin's
    // editor link: model update, sync, repaint of what changed).
    let _ = ed.render_frame(scale);
    let t = std::time::Instant::now();
    for i in 0..n {
        let e = ed.model.engine as i32;
        let m = dreist_engine::binding::Message::new(dreist_engine::binding::id::SET_LAYER_PARAMETER, &[e, 0, 0], &[0.0, (i % 50) as f32 / 50.0, 0.0]);
        ed.automation(&[m]);
        let _ = ed.render_frame(scale);
    }
    println!("automation frame {:.2} ms", t.elapsed().as_secs_f64() * 1000.0 / n as f64);
    // Mouse moves across the window.
    let t = std::time::Instant::now();
    let mut evs = 0;
    for i in 0..400 {
        let (x, y) = ((i * 37 % 1400) as f32, (i * 53 % 850) as f32);
        for ev in ed.app.mouse_move(x, y) {
            ed.handle(&ev);
            evs += 1;
        }
    }
    println!("mouse move {:.3} ms ({evs} events)", t.elapsed().as_secs_f64() * 1000.0 / 400.0);
    menu_hover(&mut ed, scale, n);
    clicks(&mut ed, scale, n);
}

/// One input event as the window delivers it: the editor's handling, the
/// tooltip query, the editor link's frame and the repaint. Returns the
/// time and whether the whole window was repainted.
fn deliver(ed: &mut Editor, scale: f32, events: Vec<dreist_gui::app::UiEvent>, t: std::time::Instant) -> (f64, bool) {
    for ev in events {
        ed.handle(&ev);
    }
    let path = ed.app.tooltip_target();
    ed.handle(&dreist_gui::app::UiEvent::ToolTip { path });
    let _ = ed.take_commands();
    ed.animate();
    ed.set_playback(false, [None; 8], None);
    ed.update_meters(&dreist_engine::Meters::default());
    let full = ed.app.dirty;
    let _ = ed.render_frame(scale);
    (t.elapsed().as_secs_f64() * 1000.0, full)
}

/// A press and release at (x, y): (press ms, release ms, full repaints).
fn click(ed: &mut Editor, scale: f32, x: f32, y: f32) -> (f64, f64, usize) {
    use dreist_gui::app::MouseButton;
    let t = std::time::Instant::now();
    let ev = ed.app.mouse_move(x, y);
    let _ = deliver(ed, scale, ev, t);
    let t = std::time::Instant::now();
    let ev = ed.app.mouse_down(MouseButton::Left);
    let (down, f0) = deliver(ed, scale, ev, t);
    let t = std::time::Instant::now();
    let ev = ed.app.mouse_up(MouseButton::Left);
    let (up, f1) = deliver(ed, scale, ev, t);
    (down, up, f0 as usize + f1 as usize)
}

/// Clicks: the main menu opened and closed again by a click beside it, a
/// knob pressed and released, the first visible plain buttons toggled
/// twice (on and back off).
fn clicks(ed: &mut Editor, scale: f32, n: usize) {
    let centre = |ed: &Editor, p: &[usize]| {
        let r = ed.app.abs_rect(p);
        (r.x as f32 + r.w as f32 / 2.0, r.y as f32 + r.h as f32 / 2.0)
    };
    fn report(ed: &mut Editor, n: usize, what: &str, f: &mut dyn FnMut(&mut Editor) -> (f64, f64, usize)) {
        let (mut d, mut u, mut full) = (0.0, 0.0, 0);
        let k = n.clamp(1, 10);
        for _ in 0..k {
            let (a, b, c) = f(ed);
            d += a;
            u += b;
            full += c;
        }
        println!("click {what}: press {:.2} ms, release {:.2} ms (sums per round), {full} full repaints in {k} rounds", d / k as f64, u / k as f64);
    }
    let Some(menu) = ed.app.root.path_of("GridButton") else { return };
    let (mx, my) = centre(ed, &menu);
    report(ed, n, "menu open + close", &mut |ed| {
        let a = click(ed, scale, mx, my);
        let b = click(ed, scale, 5.0, 5.0);
        (a.0 + b.0, a.1 + b.1, a.2 + b.2)
    });
    fn find(c: &dreist_gui::Ctrl, class: &str, path: &mut Vec<usize>, out: &mut Vec<Vec<usize>>) {
        if !c.visible || !c.enabled {
            return;
        }
        if c.class == class && !c.passive {
            out.push(path.clone());
        }
        for (i, k) in c.children.iter().enumerate() {
            path.push(i);
            find(k, class, path, out);
            path.pop();
        }
    }
    let mut knobs = Vec::new();
    find(&ed.app.root, "CtrlKnob", &mut Vec::new(), &mut knobs);
    if let Some(k) = knobs.first() {
        let (x, y) = centre(ed, k);
        report(ed, n, "knob", &mut |ed| click(ed, scale, x, y));
    }
    for name in ["MetroButton", "MuteButton"] {
        let Some(p) = ed.app.root.path_of(name).filter(|p| ed.app.shown(p)) else { continue };
        let (x, y) = centre(ed, &p);
        report(ed, n, name, &mut |ed| {
            let a = click(ed, scale, x, y);
            let b = click(ed, scale, x, y);
            (a.0 + b.0, a.1 + b.1, a.2 + b.2)
        });
    }
}

/// The main menu open, the pointer moving from item to item as the window
/// drives it: the move and its tooltip query, the editor link's frame
/// (animations, transport, meters) and the repaint.
fn menu_hover(ed: &mut Editor, scale: f32, n: usize) {
    use dreist_gui::app::UiEvent;
    let Some(p) = ed.app.root.path_of("GridButton") else { return };
    ed.handle(&UiEvent::Clicked { path: p, name: "GridButton".into(), id: 0, on: false });
    let Some(m) = ed.app.menu.clone() else { return };
    let rows: Vec<f32> = (0..m.items.len()).filter(|&i| !m.items[i].is_empty()).map(|i| m.y + i as f32 * 20.0 + 10.0).collect();
    let _ = ed.render_frame(scale);
    let (mut full, mut ms) = (0, Vec::new());
    for k in 0..n.max(rows.len()) {
        let t = std::time::Instant::now();
        for ev in ed.app.mouse_move(m.x + m.w / 2.0, rows[k % rows.len()]) {
            ed.handle(&ev);
        }
        let path = ed.app.tooltip_target();
        ed.handle(&UiEvent::ToolTip { path });
        let _ = ed.take_commands();
        ed.animate();
        ed.set_playback(false, [None; 8], None);
        ed.update_meters(&dreist_engine::Meters::default());
        full += ed.app.dirty as usize;
        let _ = ed.render_frame(scale);
        ms.push(t.elapsed().as_secs_f64() * 1000.0);
    }
    ms.sort_by(f64::total_cmp);
    let mean = ms.iter().sum::<f64>() / ms.len() as f64;
    println!("menu hover {mean:.2} ms (max {:.2}), {full} full repaints of {}", ms[ms.len() - 1], ms.len());
    ed.app.menu = None;
}

/// Playback frames as the plugin's editor link drives them: `PLAYBENCH=1`.
#[allow(dead_code)]
pub fn playback(path: &str, scale: f32) {
    let p = std::path::Path::new(path);
    let doc = dreist_formats::Project::load(p).expect("project").raw;
    let mut ed = Editor::new(dreist_gui::skin::default_theme(), Model::new(doc.clone())).expect("editor");
    if let Some(t) = std::env::var("TAB").ok().and_then(|t| t.parse().ok()) {
        ed.model.editor_tab = t;
    }
    ed.sync();
    let mut cache = dreist_engine::audio::SampleCache::default();
    let mut g = dreist_engine::Dreist::from_document(&doc, &dreist_formats::model::samples_dir(p), 44100.0, dreist_engine::RenderOptions::default(), &mut cache).expect("engine");
    g.start();
    let (mut l, mut r) = (vec![0.0f32; 735], vec![0.0f32; 735]);
    let _ = ed.render_frame(scale);
    let (mut full, mut partial, mut idle) = (0, 0, 0);
    let t = std::time::Instant::now();
    let frames = 300;
    for _ in 0..frames {
        g.process(&mut l, &mut r);
        let steps: [Option<u32>; 8] = std::array::from_fn(|e| g.current_step(e));
        ed.set_playback(g.is_playing(), steps, g.is_playing().then(|| g.song_position()));
        ed.update_meters(&g.meters);
        g.meters.clear();
        ed.animate();
        if ed.app.dirty {
            full += 1;
        } else if ed.app.needs_redraw() {
            partial += 1;
        } else {
            idle += 1;
        }
        let _ = ed.render_frame(scale);
    }
    let ms = t.elapsed().as_secs_f64() * 1000.0 / frames as f64;
    println!("playback: {ms:.1} ms/frame (engine included); {full} full, {partial} partial, {idle} idle of {frames}");
}
