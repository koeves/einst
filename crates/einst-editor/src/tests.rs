use super::*;
use crate::editor::knobs::KNOB_EDIT;

/// A file of the installed factory content (`DREIST_CONTENT` overrides
/// its location).
fn demo(rel: &str) -> std::path::PathBuf {
    dreist_formats::legacy::demo(rel)
}

/// An editor on a factory project; `None` when the content or the skin is
/// not installed.
fn demo_editor(rel: &str) -> Option<Editor> {
    let proj = dreist_formats::Project::load(&demo(rel)).ok()?;
    Editor::new(dreist_gui::skin::default_theme(), Model::new(proj.raw)).ok()
}

/// The song page's blackout: its Activate button turns song mode on.
fn song_mode_on(ed: &mut Editor) {
    let p = ed.app.root.path_of("PowerButton").expect("song blackout");
    ed.handle(&UiEvent::Clicked { path: p, name: "PowerButton".into(), id: 0, on: false });
    let b = ed.app.root.find("BlackoutPanel").unwrap();
    assert!(!b.visible, "song mode on hides the blackout");
}

/// The path of `name` in front-panel section `section` (names like
/// RestartButton and RecButton recur on the pages).
fn front_path(ed: &Editor, section: &str, name: &str) -> Vec<usize> {
    crate::front::path_in(&ed.app.root, section, name).unwrap_or_else(|| panic!("no {section}/{name}"))
}

#[test]
fn pad_labels_show_names() {
    crate::app::test_setup();
    let p = demo("Presets/Hollow.geist");
    let Ok(proj) = dreist_formats::Project::load(&p) else { return };
    let Ok(ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(proj.raw)) else { return };
    let name = ed.model.pad_name(ed.model.engine, 0).unwrap();
    assert!(name.starts_with("P1 "), "{name}");
    let m = ed.app.root.find("EditorPads").unwrap();
    assert_eq!(m.children.len(), 16);
    assert!(m.children.iter().all(|p| p.rect.right() <= m.rect.w && p.rect.bottom() <= m.rect.h), "pads overflow the panel");
    // Pad 1 is bottom left (v1's bottom row first).
    assert!(m.children[0].rect.y > m.children[12].rect.y && m.children[0].rect.x < m.children[1].rect.x);
    let label = m.children[0].find("NameLabel").unwrap();
    assert_eq!(label.text, name);
    assert!(label.rect.w > 0 && label.rect.h > 0, "{:?}", label.rect);
}

#[test]
fn grid_multi_tool_draws_erases_and_chains() {
    crate::app::test_setup();
    use dreist_gui::app::MouseButton;
    let Some(mut ed) = demo_editor("Presets/Hollow.geist") else { return };
    let gp = ed.app.root.path_of("PatternGrid").unwrap();
    let (gx, gy) = ed.app.origin(&gp);
    let e = ed.model.engine;
    let pat = ed.model.current_pattern(e);
    let lay = ed.pattern.as_ref().unwrap().layout(&ed.app, &ed.model).unwrap();
    // v1 lists pad 16 at the top: track t is on row `lay.row(t)`.
    let row_of = |t: usize| lay.row(t).expect("track on view");
    let at = |step: usize, track: usize| (gx as f32 + lay.step_x(step) + lay.step_w / 2.0, gy as f32 + lay.row_y(row_of(track)) + lay.row_h / 2.0);
    let send = |ed: &mut Editor, evs: Vec<UiEvent>| {
        for ev in evs {
            ed.handle(&ev);
        }
    };
    // Clear track 0, then a left click turns step 0 on; another left
    // click leaves it on (no toggle); a right click turns it off.
    for st in 0..16 {
        ed.handle(&UiEvent::Mouse { path: gp.clone(), overload: "PatternGrid".into(), kind: dreist_gui::app::MouseKind::RightDown, x: lay.step_x(st) + 1.0, y: lay.row_y(row_of(0)) + 1.0 });
        ed.handle(&UiEvent::Mouse { path: gp.clone(), overload: "PatternGrid".into(), kind: dreist_gui::app::MouseKind::Up, x: lay.step_x(st) + 1.0, y: lay.row_y(row_of(0)) + 1.0 });
    }
    let on = |ed: &Editor, k: usize| ed.model.pattern(e, pat).tracks[0].steps[k].on;
    assert!(!on(&ed, 0));
    let (x, y) = at(0, 0);
    {
        let evs = ed.app.mouse_move(x, y);
        send(&mut ed, evs);
    }
    {
        let evs = ed.app.mouse_down(MouseButton::Left);
        send(&mut ed, evs);
    }
    {
        let evs = ed.app.mouse_up(MouseButton::Left);
        send(&mut ed, evs);
    }
    assert!(on(&ed, 0));
    {
        let evs = ed.app.mouse_down(MouseButton::Left);
        send(&mut ed, evs);
    }
    {
        let evs = ed.app.mouse_up(MouseButton::Left);
        send(&mut ed, evs);
    }
    assert!(on(&ed, 0), "a left click does not toggle off");
    {
        let evs = ed.app.mouse_down(MouseButton::Right);
        send(&mut ed, evs);
    }
    {
        let evs = ed.app.mouse_up(MouseButton::Right);
        send(&mut ed, evs);
    }
    assert!(!on(&ed, 0));
    // A gated pad: dragging from step 2 to step 5 makes one tied chain.
    let l0 = ed.model.pad_mut(e, 0).child_mut_or_insert("Layer", "Layer0");
    l0.set("PlaybackMode", "Gated");
    if l0.attr("AudioFileName").is_none_or(|f| f.is_empty()) {
        l0.set("AudioFileName", "x.wav");
    }
    assert!(!ed.model.pad_one_shot(e, 0));
    let (x2, y2) = at(2, 0);
    let (x5, _) = at(5, 0);
    {
        let evs = ed.app.mouse_move(x2, y2);
        send(&mut ed, evs);
    }
    {
        let evs = ed.app.mouse_down(MouseButton::Left);
        send(&mut ed, evs);
    }
    {
        let evs = ed.app.mouse_move(x5, y2);
        send(&mut ed, evs);
    }

    {
        let evs = ed.app.mouse_up(MouseButton::Left);
        send(&mut ed, evs);
    }
    let t = &ed.model.pattern(e, pat).tracks[0];
    assert!((2..=5).all(|k| t.steps[k].on), "{:?}", (0..8).map(|k| t.steps[k].on).collect::<Vec<_>>());
    assert!(!t.steps[2].tied() && (3..=5).all(|k| t.steps[k].tied()));
    // The hover cursor over the chain's end.
    let c = ed.pattern.as_ref().unwrap().hover_cursor(&ed.app, &ed.model, &gp, "PatternGrid", lay.step_x(5) + 2.0, lay.row_y(row_of(0)) + 2.0);
    assert_eq!(c, Some(dreist_gui::cursor::Cursor::image("ResizeRightThin")));
}

#[test]
fn pad_mode_buttons() {
    crate::app::test_setup();
    let Some(mut ed) = demo_editor("Presets/Hollow.geist") else { return };
    let pp = ed.app.root.path_of("PadMode").unwrap();
    // The second button: Chromatic.
    let (x, y) = ed.app.origin(&[pp.clone(), vec![1]].concat());
    let click = |ed: &mut Editor, x: f32, y: f32| {
        for ev in ed.app.mouse_move(x, y) {
            ed.handle(&ev);
        }
        for ev in ed.app.mouse_down(dreist_gui::app::MouseButton::Left) {
            ed.handle(&ev);
        }
        for ev in ed.app.mouse_up(dreist_gui::app::MouseButton::Left) {
            ed.handle(&ev);
        }
    };
    click(&mut ed, x as f32 + 10.0, y as f32 + 10.0);
    let cmds = ed.take_commands();
    assert!(cmds.iter().any(|c| matches!(c, Command::SetPadMode { mode: dreist_engine::live::PadMode::Chromatic, .. })), "{cmds:?}");
    assert_eq!(ed.model.pad_mode(ed.model.engine), dreist_engine::live::PadMode::Chromatic);
    let g = ed.app.ctrl(&pp).unwrap();
    assert!(g.children.iter().enumerate().all(|(i, b)| b.on == (i == 1)), "only Chromatic lit");
    // The pads show semitones from the selected pad.
    let pads = ed.app.root.find("EditorPads").unwrap();
    assert!(pads.children[0].find("NameLabel").unwrap().text.ends_with(" st"));
}

#[test]
fn loading_a_sample_rebuilds_with_it() {
    crate::app::test_setup();
    let content = browser::content_dir();
    let p = content.join(dreist_formats::legacy::DEMO_FACTORY).join("Presets/Hollow.geist");
    let Ok(proj) = dreist_formats::Project::load(&p) else { return };
    let Some(hit) =
        std::fs::read_dir(content.join(dreist_formats::legacy::DEMO_FACTORY).join("Hits/Kicks")).ok().and_then(|rd| rd.flatten().map(|e| e.path()).find(|p| p.extension().is_some_and(|e| e == "wav")))
    else {
        return;
    };
    let mut model = Model::new(proj.raw);
    model.samples = Some(dreist_formats::model::samples_dir(&p));
    model.engine = 0;
    model.pad[0] = 5;
    model.layer[0] = 0;
    browser::load_sample_into_doc(&mut model, &hit);
    assert_eq!(model.layer_sample(0, 5, 0).as_deref(), Some(hit.as_path()));
    let mut cache = dreist_engine::audio::SampleCache::default();
    let g = dreist_engine::Dreist::from_document(&model.doc, model.samples.as_ref().unwrap(), 44100.0, dreist_engine::RenderOptions::default(), &mut cache).unwrap();
    assert!(g.missing_samples.is_empty(), "{:?}", g.missing_samples);
    assert!(model.layer(0, 5, 0).valid);
}

#[test]
fn typing_a_tempo() {
    crate::app::test_setup();
    let p = browser::content_dir().join(dreist_formats::legacy::DEMO_FACTORY).join("Presets/Hollow.geist");
    let Ok(proj) = dreist_formats::Project::load(&p) else { return };
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(proj.raw)) else { return };
    let bp = ed.app.root.path_of("BPMTextBox").unwrap();
    let (x, y) = ed.app.origin(&bp);
    ed.app.mouse_move(x as f32 + 5.0, y as f32 + 5.0);
    // v1: a press drags the tempo (the whole number at the left) ...
    ed.take_commands();
    for ev in ed.app.mouse_down(dreist_gui::app::MouseButton::Left) {
        ed.handle(&ev);
    }
    assert!(!ed.app.is_editing());
    for ev in ed.app.mouse_move(x as f32 + 5.0, y as f32 - 25.0) {
        ed.handle(&ev);
    }
    for ev in ed.app.mouse_up(dreist_gui::app::MouseButton::Left) {
        ed.handle(&ev);
    }
    let set = ed.take_commands().into_iter().any(|c| matches!(c, Command::Message(m) if m.kind == dreist_engine::binding::id::SET_TEMPO));
    assert!(set, "dragging sets the tempo");
    // ... and a double click types it.
    ed.app.mouse_move(x as f32 + 5.0, y as f32 + 5.0);
    for _ in 0..2 {
        for ev in ed.app.mouse_down(dreist_gui::app::MouseButton::Left) {
            ed.handle(&ev);
        }
        if ed.app.is_editing() {
            break;
        }
        for ev in ed.app.mouse_up(dreist_gui::app::MouseButton::Left) {
            ed.handle(&ev);
        }
    }
    assert!(ed.app.is_editing());
    for _ in 0..8 {
        ed.app.key_named("Backspace");
    }
    assert!(ed.app.key_text("98"));
    let (evs, used) = ed.app.key_named("Enter");
    assert!(used);
    for ev in &evs {
        ed.handle(ev);
    }
    let cmds = ed.take_commands();
    assert!(cmds.iter().any(|c| matches!(c, Command::Message(m) if m.kind == dreist_engine::binding::id::SET_TEMPO && (m.value[0] - 98.0).abs() < 1e-3)), "{cmds:?}");
    assert!((ed.model.tempo() - 98.0).abs() < 1e-6);
}

#[test]
fn undo_restores_the_document() {
    crate::app::test_setup();
    let p = browser::content_dir().join(dreist_formats::legacy::DEMO_FACTORY).join("Presets/Hollow.geist");
    let Ok(proj) = dreist_formats::Project::load(&p) else { return };
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(proj.raw)) else { return };
    let orig = ed.model.doc.clone();
    ed.handle(&UiEvent::TextCommitted { path: vec![], name: "PatternNameBox".into(), text: "Renamed".into() });
    assert_ne!(ed.model.doc, orig);
    let undo = ed.app.root.path_of("UndoButton").unwrap();
    ed.handle(&UiEvent::Clicked { path: undo, name: "UndoButton".into(), id: 0, on: false });
    assert_eq!(ed.model.doc, orig);
    assert!(ed.take_commands().iter().any(|c| matches!(c, Command::Rebuild { .. })));
    let redo = ed.app.root.path_of("RedoButton").unwrap();
    ed.handle(&UiEvent::Clicked { path: redo, name: "RedoButton".into(), id: 0, on: false });
    assert_ne!(ed.model.doc, orig);
}

#[test]
fn sampler_source_list_offers_external_and_the_outputs() {
    crate::app::test_setup();
    use dreist_engine::binding::id;
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    ed.model.editor_tab = crate::front::SAMPLER_PAGE;
    ed.sync();
    let sl = sampler_path(&ed, "SourceList");
    // A new document records the master (the constructor's default).
    assert_eq!(ed.app.ctrl(&sl).unwrap().text, "Master");
    ed.handle(&UiEvent::Clicked { path: sl.clone(), name: "SourceList".into(), id: 0, on: false });
    let items = ed.app.menu.as_ref().map(|m| m.items.clone()).unwrap();
    assert_eq!(items.len(), 34);
    assert_eq!((items[0].as_str(), items[1].as_str(), items[33].as_str()), ("External", "Master", "Sub output 32"));
    ed.app.menu = None;
    ed.handle(&UiEvent::MenuSelected { anchor: sl.clone(), name: "SourceList".into(), index: 0 });
    let src: Vec<f32> = ed.take_commands().into_iter().filter_map(|c| if let Command::Message(m) = c { (m.kind == id::SAMPLER_RECORD_SOURCE).then_some(m.value[0]) } else { None }).collect();
    assert_eq!(src, vec![38.0]);
    ed.sync();
    assert_eq!(ed.app.ctrl(&sl).unwrap().text, "External");
}

#[test]
fn editor_only_bindings_learn_and_act() {
    crate::app::test_setup();
    use dreist_engine::binding::Message;
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    // Learning the browser's Auto load makes an editor-only binding (0x15c).
    let ap = ed.app.root.path_of("InteractionModeButton").expect("auto load button");
    let ctrl = ed.app.ctrl(&ap).unwrap().clone();
    let t = ed.editor_binding_target(&ap, &ctrl).expect("editor binding");
    assert_eq!((t.proto.kind, t.proto.editor, t.name.as_str(), t.group.as_str()), (0x15c, true, "Auto Mode", "Browser"));
    let node = midimap::add_binding(&mut ed.model.doc, true, 0, 20, &t.proto, 0, &t.group, &t.name, t.kind);
    assert_eq!(node.attr("EditorOnly"), Some("true"));
    // The binding's message sets the toggle (value ≥ 0.5) and acts.
    let msg = |kind: u32, v: f32| {
        let mut m = Message::new(kind, &[], &[v]);
        m.editor = true;
        m
    };
    ed.binding_messages(&[msg(0x15c, 1.0)]);
    assert!(ed.app.ctrl(&ap).unwrap().on);
    ed.binding_messages(&[msg(0x15c, 0.0)]);
    assert!(!ed.app.ctrl(&ap).unwrap().on);
    // Panels: Song tab.
    ed.binding_messages(&[msg(0x148, 1.0)]);
    assert_eq!(ed.model.editor_tab, 2);
    // Pattern length from the value: ⌊0.5·1023 + 1⌋ = 512.
    ed.binding_messages(&[msg(0x146, 1.0)]);
    ed.take_commands();
    ed.binding_messages(&[msg(0x1b2, 0.5)]);
    let cmds = ed.take_commands();
    let steps: Vec<f32> = cmds.iter().filter_map(|c| if let Command::Message(m) = c { (m.kind == dreist_engine::binding::id::SET_PATTERN_STEP_COUNT).then_some(m.value[0]) } else { None }).collect();
    assert_eq!(steps, vec![512.0]);
    // The longer pattern comes with a rebuild (the engine does not
    // allocate step storage on the audio thread), last.
    assert!(matches!(cmds.last(), Some(Command::Rebuild { .. })), "rebuild after growing the pattern");
    ed.binding_messages(&[msg(0x1b2, 0.0)]);
    assert!(!ed.take_commands().iter().any(|c| matches!(c, Command::Rebuild { .. })), "no rebuild when it shrinks");
    // Undo through its binding.
    ed.model.doc.set("Probe", 1);
    let before = ed.model.doc.clone();
    ed.history.undo.push_back(("Edit".into(), project::new_document()));
    ed.binding_messages(&[msg(0x18d, 1.0)]);
    assert_ne!(ed.model.doc, before);
}

#[test]
fn pattern_zoom_and_scroll() {
    crate::app::test_setup();
    let Some(mut ed) = demo_editor("Presets/Hollow.geist") else { return };
    let click = |ed: &mut Editor, name: &str| {
        let bp = ed.app.root.path_of(name).unwrap();
        let (x, y) = ed.app.origin(&bp);
        for ev in ed.app.mouse_move(x as f32 + 3.0, y as f32 + 3.0) {
            ed.handle(&ev);
        }
        for ev in ed.app.mouse_down(dreist_gui::app::MouseButton::Left) {
            ed.handle(&ev);
        }
        for ev in ed.app.mouse_up(dreist_gui::app::MouseButton::Left) {
            ed.handle(&ev);
        }
    };
    click(&mut ed, "ZoomInH");
    let e = ed.model.engine;
    assert_eq!(ed.model.pattern_view[e].h_zoom, 2);
    let hb = ed.app.root.find("HScrollBar").unwrap();
    assert!(hb.attr_f32("realPage", 1.0) < 1.0, "zoomed in: the bar has a thumb");
    click(&mut ed, "NudgeNext");
    let cmds = ed.take_commands();
    assert!(cmds.iter().any(|c| matches!(c, Command::Message(m) if m.kind == dreist_engine::binding::id::NUDGE_PATTERN_FORWARD)), "{cmds:?}");
    // Follow playback cursor: a playing step past the view scrolls to it.
    let pat = ed.model.pattern(e, ed.model.current_pattern(e));
    let lay = ed.pattern.as_ref().unwrap().layout(&ed.app, &ed.model).unwrap();
    let last = pat.steps - 1;
    assert!((last as usize) >= lay.first_step + lay.visible_steps);
    let mut steps = [None; 8];
    steps[e] = Some(last);
    ed.set_playback(true, steps, None);
    assert_eq!(ed.model.pattern_view[e].h_scroll, 0, "follow is off by default");
    ed.model.follow_playback = true;
    // v1: the view moves just far enough, the step becoming the last shown.
    steps[e] = Some(last - 1);
    ed.set_playback(true, steps, None);
    let scrolled = ed.model.pattern_view[e].h_scroll;
    assert_eq!(scrolled, last as usize - lay.visible_steps);
    steps[e] = Some(last);
    ed.set_playback(true, steps, None);
    assert_eq!(ed.model.pattern_view[e].h_scroll, scrolled + 1, "one more step");
}

#[test]
fn graph_lane_click_sets_the_step_value() {
    crate::app::test_setup();
    let Some(mut ed) = demo_editor("Presets/Hollow.geist") else { return };
    // The single global graph (toolbar View).
    if let Some(pp) = ed.pattern.as_mut() {
        pp.multi = false;
    }
    ed.sync();
    let gp = ed.app.root.path_of("PatternGraph").unwrap();
    let (x, y) = ed.app.origin(&gp);
    let h = ed.app.ctrl(&gp).unwrap().rect.h as f32;
    for ev in ed.app.mouse_move(x as f32 + 5.0, y as f32 + h * 0.75) {
        ed.handle(&ev);
    }
    for ev in ed.app.mouse_down(dreist_gui::app::MouseButton::Left) {
        ed.handle(&ev);
    }
    for ev in ed.app.mouse_up(dreist_gui::app::MouseButton::Left) {
        ed.handle(&ev);
    }
    let e = ed.model.engine;
    let pat = ed.model.pattern(e, ed.model.current_pattern(e));
    let v = pat.tracks[ed.model.pad[e]].steps[0].lanes[0];
    assert!((v - 0.25).abs() < 0.05, "velocity {v}");
}

#[test]
fn midi_learn_binds_the_next_cc() {
    crate::app::test_setup();
    let Some(mut ed) = demo_editor("Presets/Hollow.geist") else { return };
    let click = |ed: &mut Editor, path: &[usize]| {
        let (x, y) = ed.app.origin(path);
        for ev in ed.app.mouse_move(x as f32 + 3.0, y as f32 + 3.0) {
            ed.handle(&ev);
        }
        for ev in ed.app.mouse_down(dreist_gui::app::MouseButton::Left) {
            ed.handle(&ev);
        }
        for ev in ed.app.mouse_up(dreist_gui::app::MouseButton::Left) {
            ed.handle(&ev);
        }
    };
    let lb = front_path(&ed, "Transport", "LearnButton");
    click(&mut ed, &lb);
    assert!(ed.learn.active);
    // Target: the master gain knob.
    let knob = front_path(&ed, "MasterControls", "MasterSlider");
    let before = ed.model.doc.clone();
    click(&mut ed, &knob);
    let _ = ed.take_commands();
    assert!(ed.learn.target.is_some(), "a learn target");
    assert_eq!(ed.model.doc, before, "learn mode does not operate the control");
    ed.midi_cc(1, 0, 7);
    let rows = crate::midimap::rows(&ed.model.doc);
    assert!(rows.iter().any(|r| r.locale() == "MIDI Ch 1 CC 7"), "{rows:?}");
    let cmds = ed.take_commands();
    assert!(cmds.iter().any(|c| matches!(c, Command::AddBinding(_))));
}

#[test]
fn song_recording_and_retro() {
    crate::app::test_setup();
    use dreist_engine::songrec::SongEvent;
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    let song = |ed: &Editor| ed.model.doc.children.iter().find(|c| c.class == "SongSequencer").map(dreist_engine::song::Song::from_node).unwrap();
    // Recording: a part lands in the song when its pattern stops.
    ed.song_events(&[
        SongEvent::State(2),
        SongEvent::Started { track: 1, pattern: 2, time: 4.0, recording: true, phase: 0.0 },
        SongEvent::Stopped { track: 1, pattern: 2, time: 12.0, recording: true },
    ]);
    let s = song(&ed);
    assert_eq!(s.tracks[1].len(), 1);
    assert_eq!((s.tracks[1][0].start, s.tracks[1][0].length, s.tracks[1][0].pattern), (4.0, 8.0, 2));
    assert!(!ed.history.undo.is_empty(), "recording leaves an undo point");
    // Not recording: the part goes to the retro buffer; Retro puts it
    // where it was played.
    ed.song_events(&[
        SongEvent::State(0),
        SongEvent::Started { track: 0, pattern: 0, time: 2.0, recording: false, phase: 0.0 },
        SongEvent::Stopped { track: 0, pattern: 0, time: 6.0, recording: false },
    ]);
    assert_eq!(song(&ed).tracks[0].len(), 0);
    // The Retro button grabs the song retro buffer while the song is on.
    if let Some(seq) = ed.model.doc.children.iter_mut().find(|c| c.class == "SongSequencer") {
        seq.set("SongActive", "true");
    }
    let rb = ed.app.root.path_of("RetroButton").unwrap();
    ed.handle(&UiEvent::Clicked { path: rb, name: "RetroButton".into(), id: 0, on: false });
    let s = song(&ed);
    assert_eq!(s.tracks[0].len(), 1);
    // v1: where it was played.
    assert!((s.tracks[0][0].start - 2.0).abs() < 1e-9, "{:?}", s.tracks[0]);
}

#[test]
fn graph_rec_knob_writes_the_current_step() {
    crate::app::test_setup();
    let Some(mut ed) = demo_editor("Presets/Hollow.geist") else { return };
    let e = ed.model.engine;
    ed.model.pattern_view[e].lane = 0;
    let knob = front_path(&ed, "PadModePanel", "InstoSlider");
    let rec = front_path(&ed, "Transport", "RecButton");
    // Not recording: nothing is written.
    ed.model.playing = true;
    ed.model.steps[e] = Some(3);
    let pat = ed.model.current_pattern(e);
    let track = ed.model.pad[e];
    let before = ed.model.pattern(e, pat).tracks[track].steps[3].lanes[0];
    ed.handle(&UiEvent::ValueChanged { path: knob.clone(), name: "InstoSlider".into(), id: 0, value: 0.25 });
    assert_eq!(ed.model.pattern(e, pat).tracks[track].steps[3].lanes[0], before);
    ed.app.ctrl_mut(&rec).unwrap().on = true;
    ed.handle(&UiEvent::ValueChanged { path: knob, name: "InstoSlider".into(), id: 0, value: 0.25 });
    assert!((ed.model.pattern(e, pat).tracks[track].steps[3].lanes[0] - 0.25).abs() < 1e-6);
}

#[test]
fn scene_copy_paste_and_activity() {
    crate::app::test_setup();
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    ed.model.editor_tab = front::SCENE_PAGE;
    ed.sync();
    let used = |ed: &Editor, n: usize| {
        dreist_engine::scene::SceneManager::from_node(ed.model.doc.children.iter().find(|c| c.class == "SceneManager").unwrap_or(&dreist_formats::Node::new("SceneManager", None))).scenes[n].used
    };
    let m = ed.app.root.path_of("ScenesPanel").unwrap();
    let cell = |ed: &Editor, k: usize| [m.clone(), ed.app.ctrl(&m).unwrap().path_of(&format!("Scene{k}")).unwrap()].concat();
    // Snapshot scene 0 (its slot's button), copy it, paste into scene 2.
    let snap = [cell(&ed, 0), ed.app.ctrl(&cell(&ed, 0)).unwrap().path_of("SnapshotButton").unwrap()].concat();
    ed.handle(&UiEvent::Clicked { path: snap, name: "SnapshotButton".into(), id: 0, on: false });
    assert!(used(&ed, 0));
    let ev = |ed: &Editor, kind, k| UiEvent::Mouse { path: cell(ed, k), overload: "EditorScene".into(), kind, x: 5.0, y: 5.0 };
    ed.handle(&ev(&ed, dreist_gui::app::MouseKind::RightDown, 0));
    ed.handle(&UiEvent::MenuSelected { anchor: cell(&ed, 0), name: String::new(), index: 1 });
    ed.handle(&ev(&ed, dreist_gui::app::MouseKind::RightDown, 2));
    ed.handle(&UiEvent::MenuSelected { anchor: cell(&ed, 2), name: String::new(), index: 2 });
    assert!(used(&ed, 2));
    // Cut clears the source.
    ed.handle(&ev(&ed, dreist_gui::app::MouseKind::RightDown, 0));
    ed.handle(&UiEvent::MenuSelected { anchor: cell(&ed, 0), name: String::new(), index: 0 });
    assert!(!used(&ed, 0));
    // The trigger area shows the slot's state: used, cued, playing.
    let state = |ed: &Editor, k: usize| ed.app.ctrl(&cell(ed, k)).unwrap().find("TriggerImageView").unwrap().attr("sceneState").unwrap().to_string();
    assert_eq!(state(&ed, 2), "used");
    assert_eq!(state(&ed, 1), "empty");
    ed.scene_events(&[dreist_engine::scene::SceneEvent::Cued(1)]);
    assert_eq!(state(&ed, 1), "cued");
    ed.scene_events(&[dreist_engine::scene::SceneEvent::Activated(1)]);
    assert_eq!(state(&ed, 1), "active");
    // A press triggers, the release lets go.
    ed.take_commands();
    ed.handle(&ev(&ed, dreist_gui::app::MouseKind::Down, 3));
    ed.handle(&ev(&ed, dreist_gui::app::MouseKind::Up, 3));
    let m: Vec<_> = ed.take_commands().into_iter().filter_map(|c| if let Command::Message(m) = c { Some(m) } else { None }).collect();
    assert!(m.iter().any(|m| m.kind == dreist_engine::binding::id::SCENE_TRIGGER && m.value[..2] == [3.0, 1.0]));
    assert!(m.iter().any(|m| m.kind == dreist_engine::binding::id::SCENE_TRIGGER && m.value[..2] == [3.0, 0.0]));
}

#[test]
fn aux_pre_post_button_sets_pre_fade() {
    crate::app::test_setup();
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    fn all(c: &Ctrl, path: &mut Vec<usize>, out: &mut Vec<Vec<usize>>) {
        if !c.visible {
            return;
        }
        if c.name == "PrePostButton" {
            out.push(path.clone());
        }
        for (i, k) in c.children.iter().enumerate() {
            path.push(i);
            all(k, path, out);
            path.pop();
        }
    }
    ed.model.editor_tab = front::GLOBAL_MIXER_PAGE;
    ed.sync();
    let mut found = Vec::new();
    all(&ed.app.root, &mut Vec::new(), &mut found);
    let Some(g) = found.first().cloned() else { panic!("an aux strip with PrePostButton") };
    // The Pre / Post radio pair.
    let (pre_b, post_b) = ([g.clone(), vec![0]].concat(), [g.clone(), vec![1]].concat());
    assert_eq!(ed.app.ctrl(&pre_b).unwrap().name, "PreButton");
    ed.handle(&UiEvent::Clicked { path: pre_b.clone(), name: "PreButton".into(), id: 0, on: true });
    let pre = ed.model.doc.children.iter().filter(|c| c.class == "GlobalAux").any(|c| c.attr("IsPreFade") == Some("true"));
    assert!(pre, "a GlobalAux marked pre-fade");
    ed.sync();
    assert!(ed.app.ctrl(&pre_b).unwrap().on);
    assert!(!ed.app.ctrl(&post_b).unwrap().on);
    ed.handle(&UiEvent::Clicked { path: post_b.clone(), name: "PostButton".into(), id: 0, on: true });
    ed.sync();
    assert!(ed.app.ctrl(&post_b).unwrap().on);
}

#[test]
fn mixer_strip_press_selects_the_channel_and_shows_its_chain() {
    crate::app::test_setup();
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    let strip = |ed: &Editor, kind: mixer::Kind, ch: &str| {
        let m = ed.mixers.iter().find(|m| m.kind == kind).unwrap();
        let page = ed.app.ctrl(&m.page).unwrap();
        let i = page.children.iter().position(|c| c.overload == "MixerStrip" && c.attr("channel") == Some(ch)).unwrap_or_else(|| panic!("{ch} strip"));
        [m.page.clone(), vec![i]].concat()
    };
    let press = |ed: &mut Editor, p: &[usize]| ed.handle(&UiEvent::Mouse { path: p.to_vec(), overload: "MixerStrip".into(), kind: dreist_gui::app::MouseKind::Down, x: 5.0, y: 5.0 });
    let target = |ed: &Editor, kind: mixer::Kind| ed.mixers.iter().find(|m| m.kind == kind).unwrap().fx_target(&ed.model);
    // Global mixer: aux 1 shows its chain until another bus is picked.
    ed.model.editor_tab = front::GLOBAL_MIXER_PAGE;
    ed.sync();
    assert_eq!(target(&ed, mixer::Kind::Global), fxchain::Target::Aux(0));
    let aux = strip(&ed, mixer::Kind::Global, "Aux(1)");
    press(&mut ed, &aux);
    assert_eq!(target(&ed, mixer::Kind::Global), fxchain::Target::Aux(1));
    assert_eq!(ed.app.ctrl(&aux).unwrap().style, "AuxChannelBackgroundSelected");
    // The glue moves over the selected strip.
    let glue = ed.app.ctrl(&ed.mixers.iter().find(|m| m.kind == mixer::Kind::Global).unwrap().page).unwrap().find("LargeGlue").unwrap().clone();
    assert!(glue.visible);
    assert_eq!(glue.rect.x, ed.app.ctrl(&aux).unwrap().rect.x);
    let master = strip(&ed, mixer::Kind::Global, "Master");
    press(&mut ed, &master);
    assert_eq!(target(&ed, mixer::Kind::Global), fxchain::Target::Master);
    assert_eq!(ed.app.ctrl(&master).unwrap().style, "MasterChannelBackgroundSelected");
    assert_eq!(ed.app.ctrl(&aux).unwrap().style, "AuxChannelBackgroundEven");
    // Engine mixer: a press on engine 3 selects engine 3.
    ed.model.editor_tab = front::ENGINE_MIXER_PAGE;
    ed.sync();
    let e3 = strip(&ed, mixer::Kind::Engine, "Engine(2)");
    press(&mut ed, &e3);
    assert_eq!(ed.model.engine, 2);
    assert_eq!(target(&ed, mixer::Kind::Engine), fxchain::Target::Engine(2));
    assert_eq!(ed.app.ctrl(&e3).unwrap().style, "EngineChannelBackgroundSelected");
    // Layer mixer: a press on layer 4 makes it the current layer.
    ed.model.editor_tab = front::LAYER_MIXER_PAGE;
    ed.sync();
    let l4 = strip(&ed, mixer::Kind::Layer, "Layer(3)");
    press(&mut ed, &l4);
    assert_eq!(ed.model.layer[2], 3);
    assert_eq!(target(&ed, mixer::Kind::Layer), fxchain::Target::Layer(2, ed.model.pad[2], 3));
    // Pad mixer: a press on pad 5 selects it.
    ed.model.editor_tab = front::PAD_MIXER_PAGE;
    ed.sync();
    let p5 = strip(&ed, mixer::Kind::Pad, "Pad(4)");
    press(&mut ed, &p5);
    assert_eq!(ed.model.pad[2], 4);
    assert_eq!(ed.app.ctrl(&p5).unwrap().style, "PadChannelBackgroundSelected");
}

#[test]
fn browser_loading_bar_follows_loader_progress() {
    crate::app::test_setup();
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    let Some(b) = ed.browser.as_mut() else { return };
    b.loading = Some((std::path::PathBuf::from("/x/Kick.wav"), 0.0));
    ed.load_progress(0.5);
    assert_eq!(ed.browser.as_ref().unwrap().loading.as_ref().map(|l| l.1), Some(0.5));
    ed.load_progress(1.5);
    assert!(ed.browser.as_ref().unwrap().loading.is_none());
}

#[test]
fn pattern_key_drag_out_writes_a_midi_file() {
    crate::app::test_setup();
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    let Some(k) = ed.app.root.path_of("Key2") else { return };
    ed.handle(&UiEvent::DragOut { from: k });
    let files = ed.app.drag_out.take().expect("files for the OS drag");
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].extension().and_then(|e| e.to_str()), Some("mid"));
    assert!(files[0].exists());
    let _ = std::fs::remove_file(&files[0]);
}

#[test]
fn mixer_output_menu_routes_the_channel() {
    crate::app::test_setup();
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    ed.model.editor_tab = front::PAD_MIXER_PAGE;
    ed.sync();
    // First visible OutputCombo with a strip channel.
    fn all(c: &Ctrl, path: &mut Vec<usize>, out: &mut Vec<Vec<usize>>) {
        if c.name == "OutputCombo" && c.visible {
            out.push(path.clone());
        }
        for (i, k) in c.children.iter().enumerate() {
            path.push(i);
            all(k, path, out);
            path.pop();
        }
    }
    let mut combos = Vec::new();
    all(&ed.app.root, &mut Vec::new(), &mut combos);
    let before = ed.model.doc.clone();
    for c in combos {
        ed.handle(&UiEvent::Clicked { path: c.clone(), name: "OutputCombo".into(), id: 0, on: false });
        ed.handle(&UiEvent::MenuSelected { anchor: c.clone(), name: "OutputCombo".into(), index: 3 });
        if ed.model.doc != before {
            break;
        }
    }
    fn find(n: &dreist_formats::Node) -> bool {
        n.attr("OutputLocation") == Some("Sub output 2") || n.children.iter().any(find)
    }
    assert!(find(&ed.model.doc), "a channel routed to Sub output 2");
    assert_eq!(super::mixer::output_items()[3], "Sub output 2");
}

#[test]
fn mixer_pad_strip_right_click_opens_the_pad_menu() {
    crate::app::test_setup();
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    ed.model.editor_tab = front::PAD_MIXER_PAGE;
    ed.sync();
    fn all(c: &Ctrl, path: &mut Vec<usize>, out: &mut Vec<Vec<usize>>) {
        if c.name == "OutputCombo" && c.visible {
            out.push(path.clone());
        }
        for (i, k) in c.children.iter().enumerate() {
            path.push(i);
            all(k, path, out);
            path.pop();
        }
    }
    let mut combos = Vec::new();
    all(&ed.app.root, &mut Vec::new(), &mut combos);
    let mut opened = false;
    for c in combos {
        let strip = c[..c.len() - 1].to_vec();
        if ed.mixers.iter().any(|m| m.pad_strip_at(&ed.app, &strip).is_some()) {
            ed.handle(&UiEvent::RightClicked { path: strip.clone(), name: String::new() });
            let items = ed.app.menu.as_ref().map(|m| m.items.clone()).unwrap_or_default();
            assert!(items.iter().any(|s| s.ends_with("Lock pad")), "{items:?}");
            opened = true;
            break;
        }
    }
    assert!(opened, "a pad strip");
}

#[test]
fn header_song_loop_restart_panic_and_gain() {
    crate::app::test_setup();
    use dreist_engine::binding::id;
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    let kinds = |ed: &mut Editor| -> Vec<(u32, f32)> { ed.take_commands().into_iter().filter_map(|c| if let Command::Message(m) = c { Some((m.kind, m.value[0])) } else { None }).collect() };
    let at = |ed: &Editor, n: &str| crate::front::path_in(&ed.app.root, "Transport", n).or_else(|| crate::front::path_in(&ed.app.root, "MasterControls", n)).unwrap();
    let ctrl = |ed: &Editor, n: &str| ed.app.ctrl(&at(ed, n)).unwrap().clone();
    // Song mode off: Restart and Loop disabled.
    assert!(!ctrl(&ed, "RestartButton").enabled);
    assert!(!ctrl(&ed, "LoopButton").enabled);
    let p = at(&ed, "SongButton");
    ed.handle(&UiEvent::Clicked { path: p, name: "SongButton".into(), id: 0, on: true });
    assert!(kinds(&mut ed).contains(&(id::SONG_SET_ACTIVE, 1.0)));
    assert!(ctrl(&ed, "SongButton").on);
    assert!(ctrl(&ed, "RestartButton").enabled);
    assert!(ctrl(&ed, "LoopButton").enabled);
    // A host loop disables Loop.
    ed.set_host_loop(true);
    assert!(!ctrl(&ed, "LoopButton").enabled);
    ed.set_host_loop(false);
    let p = at(&ed, "LoopButton");
    ed.handle(&UiEvent::Clicked { path: p, name: "LoopButton".into(), id: 0, on: true });
    assert!(kinds(&mut ed).contains(&(id::SONG_SET_INTERNAL_LOOP_ACTIVE, 1.0)));
    assert!(ctrl(&ed, "LoopButton").on);
    let p = at(&ed, "RestartButton");
    ed.handle(&UiEvent::Clicked { path: p, name: "RestartButton".into(), id: 0, on: false });
    assert!(kinds(&mut ed).iter().any(|k| k.0 == id::RETURN_TO_ZERO));
    // Panic: the message, play and record off.
    let rec = at(&ed, "RecButton");
    ed.app.ctrl_mut(&rec).unwrap().on = true;
    let p = at(&ed, "PanicButton");
    ed.handle(&UiEvent::Clicked { path: p, name: "PanicButton".into(), id: 0, on: false });
    assert!(kinds(&mut ed).iter().any(|k| k.0 == id::PANIC));
    assert!(!ctrl(&ed, "RecButton").on);
    // Master gain: drag, then a double click resets to 0 dB.
    let g = at(&ed, "MasterSlider");
    ed.handle(&UiEvent::ValueChanged { path: g.clone(), name: "MasterSlider".into(), id: 6000, value: 0.5 });
    assert!(kinds(&mut ed).contains(&(id::SET_MASTER_CHANNEL_GAIN, 0.5)));
    assert_eq!(ctrl(&ed, "MasterSlider").value, 0.5);
    ed.handle(&UiEvent::DoubleClicked { path: g, name: "MasterSlider".into() });
    assert!(kinds(&mut ed).contains(&(id::SET_MASTER_CHANNEL_GAIN, dreist_formats::params::UNITY_GAIN)));
    // Indicators: 10 ticks after a MIDI block or a stolen voice.
    let mut m = dreist_engine::Meters { midi_received: true, ..Default::default() };
    ed.update_meters(&m);
    assert!(ctrl(&ed, "MIDIIndi").on);
    assert!(!ctrl(&ed, "VoiceIndi").on);
    m.midi_received = false;
    for _ in 0..9 {
        ed.update_meters(&m);
    }
    assert!(ctrl(&ed, "MIDIIndi").on);
    ed.update_meters(&m);
    assert!(!ctrl(&ed, "MIDIIndi").on);
    // Learn: the tempo box binds tap tempo.
    let t = at(&ed, "BPMTextBox");
    let target = ed.learn_capture(&UiEvent::Pressed { path: t, name: "BPMTextBox".into() }).unwrap();
    assert_eq!((target.proto.kind, target.name.as_str(), target.group.as_str(), target.kind), (id::TAP_TEMPO, "Tap tempo", "Transport", "Gated"));
}

#[test]
fn undo_history_menu_steps_back() {
    crate::app::test_setup();
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    let original = ed.model.doc.clone();
    for k in 0..3 {
        let before = ed.model.doc.clone();
        ed.model.doc.set("Probe", k);
        // Mimic an edit event's snapshot.
        ed.history.undo.push_back((format!("Edit {k}"), before));
    }
    let h = ed.app.root.path_of("UndoHistory").unwrap();
    ed.handle(&UiEvent::Clicked { path: h.clone(), name: "UndoHistory".into(), id: 0, on: false });
    // Items: "Edit 2", divider, "Edit 1", "Edit 0": index 3 undoes all three.
    ed.handle(&UiEvent::MenuSelected { anchor: h, name: "UndoHistory".into(), index: 3 });
    assert_eq!(ed.model.doc, original);
    assert_eq!(ed.history.redo.len(), 3);
    assert_eq!(ed.history.redo.last().map(|r| r.0.as_str()), Some("Edit 0"));
}

#[test]
fn pattern_export_renders_the_engine_pattern() {
    crate::app::test_setup();
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    ed.patmenu.export = true;
    ed.handle(&UiEvent::Wheel { path: vec![], overload: String::new(), dy: 0.0 });
    let cmds = ed.take_commands();
    assert!(cmds.iter().any(|c| matches!(c, Command::Play(false))));
    let e = ed.model.engine;
    assert!(
        cmds.iter().any(
            |c| matches!(c, Command::ExportAudio { options, reveal: true, path, .. } if options.source == dreist_engine::export::Source::Engine(e) && path.extension().is_some_and(|x| x == "wav"))
        )
    );
}

#[test]
fn multitrack_automation_lanes() {
    crate::app::test_setup();
    let Some(mut ed) = demo_editor("Presets/Hollow.geist") else { return };
    let pp = ed.pattern.as_mut().expect("pattern panel");
    pp.multi = true;
    ed.sync();
    // Open track row 0's automation.
    let rows = ed.app.root.path_of("ControlsScrollView").unwrap();
    let row0 = ed.app.ctrl(&rows).unwrap().children.iter().position(|c| c.attr("trackRow") == Some("0")).unwrap();
    let ab = ed.app.ctrl(&[rows.clone(), vec![row0]].concat()).unwrap().children.iter().position(|c| c.name == "AutomateButton").unwrap();
    let ab = [rows.clone(), vec![row0, ab]].concat();
    ed.handle(&UiEvent::Clicked { path: ab, name: "AutomateButton".into(), id: 0, on: true });
    let autos = |ed: &Editor| ed.app.ctrl(&rows).unwrap().children.iter().filter(|c| c.attr("autoTrack").is_some()).count();
    assert_eq!(autos(&ed), 1, "one lane row");
    // Draw on the first lane row: track 0's velocity lane.
    let grid = ed.app.root.path_of("PatternGrid").unwrap();
    let (gx, gy) = ed.app.origin(&grid);
    let e = ed.model.engine;
    let first = ed.model.visible_pads().0;
    let lay = ed.pattern.as_ref().unwrap().layout_for(&ed.model, ed.app.ctrl(&grid).unwrap().rect.w, ed.app.ctrl(&grid).unwrap().rect.h);
    let g = lay.graphs[0];
    // Row 0 is the top one: the last pad.
    assert_eq!((g.track, g.lane), (first + 15, 0));
    let _ = (gx, gy);
    ed.handle(&UiEvent::Mouse { path: grid.clone(), overload: "PatternGrid".into(), kind: dreist_gui::app::MouseKind::Down, x: 2.0, y: g.y + g.h * 0.75 });
    ed.handle(&UiEvent::Mouse { path: grid, overload: "PatternGrid".into(), kind: dreist_gui::app::MouseKind::Up, x: 2.0, y: g.y + g.h * 0.75 });
    let pat = ed.model.pattern(e, ed.model.current_pattern(e));
    let v = pat.tracks[g.track].steps[lay.first_step].lanes[0];
    assert!((v - 0.25).abs() < 0.02, "velocity drawn {v}");
    if let Ok(out) = std::env::var("DREIST_MULTITRACK_SHOT") {
        ed.render(1.0).save_png(out).unwrap();
    }
}

#[test]
fn song_part_select_move_duplicate() {
    crate::app::test_setup();
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    ed.model.editor_tab = 2;
    ed.sync();
    song_mode_on(&mut ed);
    let body = ed.app.root.path_of("SongBody").unwrap();
    let (bx, by) = ed.app.origin(&body);
    let press = |ed: &mut Editor, x: f32, y: f32, drag_to: Option<f32>| {
        for ev in ed.app.mouse_move(bx as f32 + x, by as f32 + y) {
            ed.handle(&ev);
        }
        for ev in ed.app.mouse_down(dreist_gui::app::MouseButton::Left) {
            ed.handle(&ev);
        }
        if let Some(x2) = drag_to {
            for ev in ed.app.mouse_move(bx as f32 + x2, by as f32 + y) {
                ed.handle(&ev);
            }
        }
        for ev in ed.app.mouse_up(dreist_gui::app::MouseButton::Left) {
            ed.handle(&ev);
        }
    };
    // Insert a part on track 1 at bar 1, then drag it two bars right.
    press(&mut ed, 5.0, 30.0, None);
    let song = |ed: &Editor| ed.model.doc.children.iter().find(|c| c.class == "SongSequencer").map(dreist_engine::song::Song::from_node).unwrap();
    assert_eq!(song(&ed).tracks[0].len(), 1);
    let zoom = 16.0;
    press(&mut ed, 5.0, 30.0, Some(5.0 + 8.0 * zoom));
    assert!((song(&ed).tracks[0][0].start - 8.0).abs() < 1e-6, "{:?}", song(&ed).tracks[0]);
    let dup = ed.app.root.path_of("DuplicateButton").unwrap();
    let (dx, dy) = ed.app.origin(&dup);
    for ev in ed.app.mouse_move(dx as f32 + 3.0, dy as f32 + 3.0) {
        ed.handle(&ev);
    }
    for ev in ed.app.mouse_down(dreist_gui::app::MouseButton::Left) {
        ed.handle(&ev);
    }
    for ev in ed.app.mouse_up(dreist_gui::app::MouseButton::Left) {
        ed.handle(&ev);
    }
    assert_eq!(song(&ed).tracks[0].len(), 2);
}

#[test]
fn damaged_frames_match_full_renders() {
    crate::app::test_setup();
    let p = demo("Presets/Hollow.geist");
    let Ok(proj) = dreist_formats::Project::load(&p) else { return };
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(proj.raw)) else { return };
    ed.model.editor_tab = crate::front::SONG_PAGE;
    ed.sync();
    for scale in [1.0, 2.0] {
        let _ = ed.render(scale);
        for k in 0..4 {
            let mut m = dreist_engine::Meters { master: [0.3 + 0.1 * k as f32, 0.5], ..Default::default() };
            for e in 0..8 {
                m.engine[e] = [0.1 * e as f32 + 0.05 * k as f32, 0.2];
            }
            ed.update_meters(&m);
            assert!(!ed.app.dirty && ed.app.needs_redraw());
            let partial = ed.render_frame(scale).clone();
            let full = ed.render(scale);
            // Shapes crossing a damage edge go through clip masks:
            // only a few anti-aliased edge pixels may differ.
            let px = |d: &[u8]| d.chunks(4).map(|c| [c[0], c[1], c[2]]).collect::<Vec<_>>();
            let (a, b) = (px(partial.data()), px(full.data()));
            let off = a.iter().zip(&b).filter(|(x, y)| x.iter().zip(y.iter()).any(|(p, q)| p.abs_diff(*q) > 1)).count();
            assert!(off * 1000 <= a.len(), "scale {scale} tick {k}: {off} pixels differ");
        }
    }
    // Hover over controls, then over an open menu's items.
    let _ = ed.render(1.0);
    let check = |ed: &mut Editor, what: &str| {
        let partial = ed.render_frame(1.0).clone();
        let full = ed.render(1.0);
        let off = partial.data().chunks(4).zip(full.data().chunks(4)).filter(|(x, y)| x.iter().zip(y.iter()).take(3).any(|(p, q)| p.abs_diff(*q) > 1)).count();
        assert!(off * 1000 <= partial.data().len() / 4, "{what}: {off} pixels differ");
    };
    for (x, y) in [(620.0, 460.0), (760.0, 618.0), (1040.0, 452.0), (100.0, 600.0), (300.0, 140.0)] {
        let _ = ed.app.mouse_move(x, y);
        check(&mut ed, "hover");
    }
    if let Some(p) = ed.app.root.path_of("GridButton") {
        ed.app.open_menu(&p, vec!["One".into(), "Two".into(), "Three".into()], None);
        let _ = ed.render(1.0);
        let m = ed.app.menu.clone().unwrap();
        for i in 0..3 {
            let _ = ed.app.mouse_move(m.x + 5.0, m.y + 9.0 + i as f32 * 18.0);
            check(&mut ed, "menu hover");
        }
        ed.app.menu = None;
    }
    // Playheads on the song and pattern tabs.
    for tab in [2, 0] {
        ed.model.editor_tab = tab;
        ed.sync();
        let _ = ed.render(1.0);
        for k in 0..5u32 {
            ed.set_playback(true, [Some(k); 8], Some(k as f64 * 1.37));
            let partial = ed.render_frame(1.0).clone();
            let full = ed.render(1.0);
            let off = partial.data().chunks(4).zip(full.data().chunks(4)).filter(|(x, y)| x.iter().zip(y.iter()).take(3).any(|(p, q)| p.abs_diff(*q) > 1)).count();
            assert!(off * 1000 <= partial.data().len() / 4, "tab {tab} tick {k}: {off} pixels differ");
        }
    }
}

#[test]
fn keyboard_accelerators() {
    crate::app::test_setup();
    let Some(mut ed) = demo_editor("Presets/Hollow.geist") else { return };
    ed.modals.options.set("RespondToExtendedKeyboard", false);
    let key = |ed: &mut Editor, k: &str, cmd: bool, shift: bool| {
        ed.app.mods.cmd = cmd;
        ed.app.mods.shift = shift;
        for ev in ed.app.shortcut(k) {
            ed.handle(&ev);
        }
        ed.app.mods = Default::default();
    };
    let plays = |ed: &mut Editor| ed.outbox.drain(..).filter_map(|c| if let Command::Play(on) = c { Some(on) } else { None }).collect::<Vec<_>>();
    // A plugin without "extended keyboard": Space is not a shortcut.
    key(&mut ed, " ", false, false);
    assert!(plays(&mut ed).is_empty());
    // The standalone's menu bar: Space toggles Play.
    ed.menu_bar = true;
    key(&mut ed, " ", false, false);
    assert_eq!(plays(&mut ed), vec![true]);
    assert!(ed.model.playing);
    key(&mut ed, " ", false, false);
    assert_eq!(plays(&mut ed), vec![false]);
    // Number keys switch v1's pages; M toggles the metronome.
    key(&mut ed, "8", false, false);
    assert_eq!(ed.model.editor_tab, crate::front::GLOBAL_MIXER_PAGE);
    key(&mut ed, "1", false, false);
    assert_eq!(ed.model.editor_tab, crate::front::PATTERN_PAGE);
    key(&mut ed, "m", false, false);
    assert!(ed.outbox.iter().any(|c| matches!(c, Command::Message(m) if m.kind == dreist_engine::binding::id::TOGGLE_METRONOME && m.value[0] == 1.0)));
    // Menu bar items run regardless of the keyboard options.
    ed.menu_bar = false;
    ed.menu_command(0x0c);
    assert_eq!(plays(&mut ed), vec![true]);
    let (app, menus) = ed.menu_bar_items();
    assert!(app.iter().any(|i| i.title == "Audio Settings..."));
    assert_eq!(menus.iter().map(|m| m.0.as_str()).collect::<Vec<_>>(), ["File", "Edit", "Browser", "Transport", "View"]);
    assert_eq!(menus[0].1.iter().map(|i| i.title.as_str()).collect::<Vec<_>>(), ["Open Preset", "Open Engine", "Open Kit", "", "Save Preset", "Save Engine", "Save Kit"]);
    // Cmd+Z runs the header's Undo (nothing to undo: no change).
    let doc = ed.model.doc.clone();
    key(&mut ed, "z", true, false);
    assert_eq!(ed.model.doc, doc);
}

#[test]
fn pattern_select_copy_paste() {
    crate::app::test_setup();
    let Some(mut ed) = demo_editor("Presets/Hollow.geist") else { return };
    let e = ed.model.engine;
    let cur = ed.model.current_pattern(e);
    let on = |ed: &Editor| -> Vec<(usize, usize)> {
        let pat = ed.model.pattern(e, cur);
        (0..16).flat_map(|t| (0..pat.steps as usize).map(move |s| (t, s))).filter(|&(t, s)| pat.tracks[t].steps[s].on).collect()
    };
    let before = on(&ed);
    assert!(!before.is_empty());
    let key = |ed: &mut Editor, k: &str, cmd: bool| {
        ed.app.mods.cmd = cmd;
        for ev in ed.app.shortcut(k) {
            ed.handle(&ev);
        }
        ed.app.mods.cmd = false;
    };
    key(&mut ed, "a", true);
    key(&mut ed, "x", true);
    assert!(on(&ed).is_empty(), "cut removes the steps");
    ed.model.pad[e] = 0;
    key(&mut ed, "v", true);
    let after = on(&ed);
    // Pasted relative to the top track, so the same set when the
    // selection started at track 0 / step 0.
    let (t0, s0) = (before.iter().map(|x| x.0).min().unwrap(), before.iter().map(|x| x.1).min().unwrap());
    let expect: Vec<(usize, usize)> = before.iter().map(|&(t, s)| (t - t0, s - s0)).collect();
    assert_eq!(after, expect);
    key(&mut ed, "right", false);
    assert_eq!(on(&ed).len(), after.len());
}

#[test]
fn pattern_track_level_knob_sets_the_pad_gain() {
    crate::app::test_setup();
    let p = demo("Presets/Hollow.geist");
    let Ok(proj) = dreist_formats::Project::load(&p) else { return };
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(proj.raw)) else { return };
    let rows = ed.app.root.path_of("ControlsScrollView").unwrap();
    // The third track row's knob (pad 14: v1 lists pad 16 at the top).
    let mut found = Vec::new();
    fn walk(c: &dreist_gui::Ctrl, p: &mut Vec<usize>, out: &mut Vec<Vec<usize>>) {
        if c.name == "LevelSlider" {
            out.push(p.clone());
        }
        for (i, k) in c.children.iter().enumerate() {
            p.push(i);
            walk(k, p, out);
            p.pop();
        }
    }
    walk(ed.app.ctrl(&rows).unwrap(), &mut Vec::new(), &mut found);
    let knob = [rows.clone(), found[2].clone()].concat();
    ed.take_commands();
    ed.handle(&UiEvent::ValueChanged { path: knob, name: "LevelSlider".into(), id: 0, value: 0.4 });
    let e = ed.model.engine;
    let cmds = ed.take_commands();
    assert!(cmds.iter().any(|c| matches!(c, Command::Message(m) if m.kind == dreist_engine::binding::id::SET_PAD_PARAMETER && m.index[..2] == [e as i32, 13] && m.value[0] == 0.4)), "{cmds:?}");
    assert!((ed.model.pad_gain(e, 13) - 0.4).abs() < 1e-6);
}

#[test]
fn knob_double_click_resets() {
    crate::app::test_setup();
    use dreist_engine::binding::id;
    let Some(mut ed) = demo_editor("Presets/Hollow.geist") else { return };
    let dbl = |ed: &mut Editor, path: Vec<usize>, name: &str| {
        ed.take_commands();
        ed.handle(&UiEvent::DoubleClicked { path, name: name.into() });
        ed.take_commands().into_iter().filter_map(|c| if let Command::Message(m) = c { Some(m) } else { None }).collect::<Vec<_>>()
    };
    // The layer view's cutoff → 1.0 (the base value).
    ed.model.editor_tab = crate::front::PADS_PAGE;
    ed.model.source = 3;
    ed.sync();
    let cutoff = ed.app.root.path_of("CutoffKnob").unwrap();
    let m = dbl(&mut ed, cutoff, "CutoffKnob");
    let cut = dreist_formats::LayerParam::FilterCutoff as u8 as f32;
    assert!(m.iter().any(|m| m.kind == id::SET_LAYER_PARAMETER && m.value[..3] == [cut, 1.0, 0.0]), "{m:?}");
    assert_eq!(ed.model.source, 3);
    // Vel-Pitch → no velocity depth.
    let v2p = ed.app.root.path_of("V2P").unwrap();
    let m = dbl(&mut ed, v2p, "V2P");
    let tune = dreist_formats::LayerParam::CoarseTune as u8 as f32;
    assert!(m.iter().any(|m| m.kind == id::SET_LAYER_PARAMETER && m.value[..3] == [tune, 0.0, dreist_formats::TransmodSource::Velocity as u8 as f32]), "{m:?}");
    // A mixer strip's pan → 0.5 (once the mixer pages are wired).
    if !ed.mixers.is_empty() {
        ed.model.editor_tab = 4;
        ed.sync();
        let page = ed.mixers[0].page.clone();
        let base = ed.app.ctrl(&page).unwrap().clone();
        if let Some(pan) = base.path_of("PanKnob").map(|p| [page.clone(), p].concat()) {
            let m = dbl(&mut ed, pan, "PanKnob");
            assert!(m.iter().any(|m| m.value[0] == 0.5 || (m.kind == id::SET_LAYER_PARAMETER && m.value[1] == 0.5)), "{m:?}");
        }
    }
    // Off: a typed edit instead.
    let mut o = options::Options::load();
    o.set("DoubleClickResets", false);
    o.save();
    let cutoff = ed.app.root.path_of("CutoffKnob").unwrap();
    assert!(dbl(&mut ed, cutoff, "CutoffKnob").is_empty());
    assert!(ed.app.root.path_of(KNOB_EDIT).is_some());
    o.set("DoubleClickResets", true);
    o.save();
}

/// v1's knobs have no right-click menu.
#[test]
fn knob_right_click_has_no_menu() {
    crate::app::test_setup();
    let Some(mut ed) = demo_editor("Presets/Hollow.geist") else { return };
    let cutoff = ed.app.root.path_of("CutoffKnob").unwrap();
    ed.handle(&UiEvent::RightClicked { path: cutoff, name: "CutoffKnob".into() });
    assert!(ed.app.menu.is_none());
}

#[test]
fn output_selector_route_all_menu() {
    crate::app::test_setup();
    use dreist_engine::binding::id;
    let Some(mut ed) = demo_editor("Presets/Hollow.geist") else { return };
    ed.model.editor_tab = front::PAD_MIXER_PAGE;
    ed.sync();
    // The first visible OutputCombo of the pad mixer.
    fn find(c: &dreist_gui::Ctrl, path: &mut Vec<usize>) -> bool {
        for (i, k) in c.children.iter().enumerate() {
            path.push(i);
            if (k.visible && k.name == "OutputCombo") || (k.visible && find(k, path)) {
                return true;
            }
            path.pop();
        }
        false
    }
    let mut oc = Vec::new();
    assert!(find(&ed.app.root, &mut oc));
    ed.handle(&UiEvent::RightClicked { path: oc.clone(), name: "OutputCombo".into() });
    let items = ed.app.menu.as_ref().unwrap().items.clone();
    ed.take_commands();
    ed.handle(&UiEvent::MenuSelected { anchor: oc, name: "OutputCombo".into(), index: items.len() - 1 });
    let m: Vec<_> = ed.take_commands().into_iter().filter_map(|c| if let Command::Message(m) = c { Some(m) } else { None }).collect();
    let kinds: std::collections::BTreeSet<u32> = m.iter().map(|m| m.kind).collect();
    assert!(kinds.len() == 1 && [id::SET_PAD_OUTPUT, id::SET_LAYER_OUTPUT, id::SET_ENGINE_OUTPUT].contains(kinds.iter().next().unwrap()), "{items:?} {kinds:?}");
    assert!(m.iter().enumerate().all(|(i, m)| m.value[0] == (i + 2) as f32));
}

#[test]
fn engine_selector_menu() {
    crate::app::test_setup();
    let p = demo("Presets/Hollow.geist");
    let Ok(proj) = dreist_formats::Project::load(&p) else { return };
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(proj.raw)) else { return };
    let base = ed.header.selectors.clone().unwrap();
    let sel = |e: usize| {
        let mut p = [base.clone(), vec![e]].concat();
        let s = ed_ctrl_child(&ed, &p, "EngineSelectionButton");
        p.push(s);
        p
    };
    fn ed_ctrl_child(ed: &Editor, p: &[usize], n: &str) -> usize {
        ed.app.ctrl(p).unwrap().children.iter().position(|c| c.name == n).unwrap()
    }
    let s6 = sel(6);
    let s0 = sel(0);
    let pick = |ed: &mut Editor, at: &Vec<usize>, index: usize| {
        ed.handle(&UiEvent::RightClicked { path: at.clone(), name: "EngineSelectionButton".into() });
        let items = ed.app.menu.as_ref().unwrap().items.clone();
        ed.handle(&UiEvent::MenuSelected { anchor: at.clone(), name: "x".into(), index });
        items
    };
    let items = pick(&mut ed, &s6, 1);
    assert_eq!(items[..3], ["Cut", "Copy", &format!("{}Paste", dreist_gui::menu::DISABLED)]);
    let pads = |ed: &Editor, e: usize| ed.model.engine_node(e).map(|n| n.children.iter().filter(|c| c.class == "Pad").count()).unwrap_or(0);
    let pads6 = pads(&ed, 6);
    assert!(pads6 > 0);
    // Paste engine 7's copy over engine 1.
    pick(&mut ed, &s0, 2);
    assert_eq!(pads(&ed, 0), pads6);
    // Reset engine 1.
    pick(&mut ed, &s0, 7);
    assert!(ed.model.engine_node(0).is_none());
}

#[test]
fn effect_panel_cut_copy_paste() {
    crate::app::test_setup();
    let Some(mut ed) = demo_editor("Presets/Hollow.geist") else { return };
    fn find(c: &dreist_gui::Ctrl, path: &mut Vec<usize>, f: &dyn Fn(&dreist_gui::Ctrl) -> bool) -> bool {
        for (i, k) in c.children.iter().enumerate() {
            if !k.visible {
                continue;
            }
            path.push(i);
            if f(k) || find(k, path, f) {
                return true;
            }
            path.pop();
        }
        false
    }
    // Every mixer page shows the selected channel's chain on top.
    for tab in [front::LAYER_MIXER_PAGE, front::PAD_MIXER_PAGE, front::ENGINE_MIXER_PAGE, front::GLOBAL_MIXER_PAGE] {
        ed.model.editor_tab = tab;
        ed.sync();
        let mut panel = Vec::new();
        if !find(&ed.app.root, &mut panel, &|k| k.attr("fxSlot").is_some() && k.attr("fxKind").is_some_and(|v| v != "0")) {
            continue;
        }
        ed.handle(&UiEvent::RightClicked { path: panel.clone(), name: "x".into() });
        let items = ed.app.menu.as_ref().map(|m| m.items.clone()).unwrap_or_default();
        assert_eq!(items[..2], ["Cut".to_string(), "Copy".into()]);
        ed.handle(&UiEvent::MenuSelected { anchor: panel.clone(), name: "x".into(), index: 1 });
        ed.handle(&UiEvent::RightClicked { path: panel.clone(), name: "x".into() });
        let items = ed.app.menu.as_ref().unwrap().items.clone();
        assert_eq!(items[2], "Paste");
        ed.take_commands();
        ed.handle(&UiEvent::MenuSelected { anchor: panel, name: "x".into(), index: 2 });
        assert!(ed.take_commands().iter().any(|c| matches!(c, Command::Rebuild { .. })));
        return;
    }
    panic!("no effect panel found");
}

/// v1 has no per-track lengths: the grid's right edge does not set one.
#[test]
fn no_track_length_markers() {
    crate::app::test_setup();
    use dreist_gui::app::MouseKind;
    let Some(mut ed) = demo_editor("Presets/Hollow.geist") else { return };
    let grid = ed.app.root.path_of("PatternGrid").unwrap();
    let w = ed.app.ctrl(&grid).unwrap().rect.w as f32;
    let lay = ed.pattern.as_ref().unwrap().layout(&ed.app, &ed.model).unwrap();
    let mouse = |ed: &mut Editor, kind: MouseKind, x: f32, y: f32| ed.handle(&UiEvent::Mouse { path: grid.clone(), overload: "PatternGrid".into(), kind, x, y });
    ed.take_commands();
    mouse(&mut ed, MouseKind::Down, w - 5.0, lay.row_y(0) + 3.0);
    mouse(&mut ed, MouseKind::Drag, lay.step_x(4) + 2.0, lay.row_y(0) + 3.0);
    mouse(&mut ed, MouseKind::Up, lay.step_x(4) + 2.0, lay.row_y(0) + 3.0);
    assert!(!ed.take_commands().iter().any(|c| matches!(c, Command::Message(m) if m.kind == dreist_engine::binding::id::SET_TRACK_LENGTH)));
}

#[test]
fn automation_reaches_the_document() {
    crate::app::test_setup();
    use dreist_engine::binding::{Message, id};
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    let gain = dreist_formats::LayerParam::Gain as i32 as f32;
    ed.automation(&[Message::new(id::SET_LAYER_PARAMETER, &[0, 0, 0], &[gain, 0.25, 0.0])]);
    let l = ed.model.engine_node(0).and_then(|e| e.children.iter().find(|c| c.class == "Pad")).and_then(|p| p.children.iter().find(|c| c.class == "Layer"));
    assert_eq!(l.and_then(|l| l.f32("LayerGain")), Some(0.25));
    assert!(ed.history.undo.is_empty());
}

#[test]
fn header_options_menu() {
    crate::app::test_setup();
    use dreist_engine::binding::id;
    let Some(mut ed) = demo_editor("Presets/Hollow.geist") else { return };
    // The main menu's "Transport options" submenu.
    let ob = front_path(&ed, "UtilityControls", "GridButton");
    let open = |ed: &mut Editor| {
        ed.handle(&UiEvent::Clicked { path: ob.clone(), name: "GridButton".into(), id: 0, on: false });
        assert!(ed.app.menu.as_ref().is_some_and(|m| m.items[8].starts_with("Transport options")));
        ed.handle(&UiEvent::MenuSelected { anchor: ob.clone(), name: "GridButton".into(), index: 8 });
        ed.app.menu.as_ref().map(|m| m.items.clone()).unwrap_or_default()
    };
    let pick = |ed: &mut Editor, index: usize| {
        ed.outbox.clear();
        ed.handle(&UiEvent::MenuSelected { anchor: ob.clone(), name: "GridButton".into(), index });
        ed.outbox.iter().filter_map(|c| if let Command::Message(m) = c { Some((m.kind, m.value[0], m.value[1])) } else { None }).collect::<Vec<_>>()
    };
    let items = open(&mut ed);
    assert_eq!(items.len(), 15, "{items:?}");
    assert_eq!(items[1], "✓ Pad selection follows MIDI");
    assert_eq!(items[4], "✓ No pre-roll");
    assert!(items[14].ends_with('▸'));
    assert_eq!(pick(&mut ed, 6), [(id::SET_PREROLL_MODE, 2.0, 0.0)]);
    assert_eq!(open(&mut ed)[6], "✓ 2 bars");
    assert_eq!(pick(&mut ed, 2), [(id::REGISTRY_FOLLOW_OPTIONS_CHANGED, 1.0, 1.0)]);
    // "Metronome only when recording" shares the pre-roll key.
    open(&mut ed);
    assert_eq!(pick(&mut ed, 9), [(id::SET_PREROLL_ONLY_ON_REC, 0.0, 0.0)]);
    open(&mut ed);
    assert_eq!(pick(&mut ed, 10), [(id::SET_METRO_ONLY_DURING_REC, 1.0, 0.0)]);
    assert!(open(&mut ed)[9].starts_with('✓'));
    assert!(!ed.model.follow_playback);
    pick(&mut ed, 0);
    assert!(ed.model.follow_playback);
    // The output sub menu.
    open(&mut ed);
    assert!(pick(&mut ed, 14).is_empty());
    let outs = ed.app.menu.as_ref().unwrap().items.clone();
    assert_eq!((outs.len(), outs[0].as_str(), outs[14].as_str()), (15, "✓ Master", "Sub output 14"));
    assert_eq!(pick(&mut ed, 3), [(id::SET_METRONOME_OUTPUT_LOCATION, 4.0, 0.0)]);
    let metro = |ed: &Editor, k: &str| ed.model.doc.children.iter().find(|c| c.class == "Metronome").and_then(|n| n.attr(k)).map(str::to_string);
    assert_eq!(metro(&ed, "OutputLocation").as_deref(), Some("Sub output 3"));
    // The volume slider and the metronome button write the document.
    open(&mut ed);
    ed.handle(&UiEvent::MenuSlider { anchor: ob.clone(), index: 13, value: 0.25 });
    assert_eq!(metro(&ed, "Gain").as_deref(), Some("0.25"));
    let mb = ed.app.root.path_of("MetroButton").unwrap();
    // v1: a right click on Metro opens the output menu.
    ed.handle(&UiEvent::RightClicked { path: mb.clone(), name: "MetroButton".into() });
    assert_eq!(ed.app.menu.as_ref().map(|m| m.items.len()), Some(15));
    ed.app.menu = None;
    ed.handle(&UiEvent::Clicked { path: mb, name: "MetroButton".into(), id: 0, on: true });
    assert_eq!(metro(&ed, "Mute").as_deref(), Some("false"));
    // Engine follow messages select.
    let m = |kind, index: &[i32]| dreist_engine::binding::Message { editor: true, ..dreist_engine::binding::Message::new(kind, index, &[]) };
    ed.binding_messages(&[m(id::UPDATE_ENGINE_SELECTION_CHANGED, &[3]), m(id::UPDATE_PAD_SELECTION_CHANGED, &[3, 9])]);
    assert_eq!((ed.model.engine, ed.model.pad[3]), (3, 9));
}

#[test]
fn graph_menu_inserts_a_ramp() {
    crate::app::test_setup();
    let p = demo("Presets/Hollow.geist");
    let Ok(proj) = dreist_formats::Project::load(&p) else { return };
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(proj.raw)) else { return };
    // The single global graph (toolbar View).
    if let Some(pp) = ed.pattern.as_mut() {
        pp.multi = false;
    }
    ed.sync();
    let ac = ed.app.root.path_of("SingleAutoControl").unwrap();
    let ob = {
        let c = ed.app.ctrl(&ac).unwrap();
        let mut p = ac.clone();
        p.extend(c.path_of("InsertButton").unwrap());
        p
    };
    let (x, y) = ed.app.origin(&ob);
    for ev in ed.app.mouse_move(x as f32 + 3.0, y as f32 + 3.0) {
        ed.handle(&ev);
    }
    for ev in ed.app.mouse_down(dreist_gui::app::MouseButton::Left) {
        ed.handle(&ev);
    }
    for ev in ed.app.mouse_up(dreist_gui::app::MouseButton::Left) {
        ed.handle(&ev);
    }
    ed.handle(&UiEvent::MenuSelected { anchor: ob.clone(), name: "InsertButton".into(), index: 1 });
    let e = ed.model.engine;
    let pat = ed.model.pattern(e, ed.model.current_pattern(e));
    let t = &pat.tracks[ed.model.pad[e]];
    assert!(t.steps[0].lanes[0] < 0.01 && t.steps[8].lanes[0] > 0.4, "ramp up on velocity");
}

#[test]
fn pattern_toolbar_duplicate_quantize_remix_copy() {
    crate::app::test_setup();
    let p = demo("Presets/Hollow.geist");
    let Ok(proj) = dreist_formats::Project::load(&p) else { return };
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(proj.raw)) else { return };
    let e = ed.model.engine;
    let cur = ed.model.current_pattern(e);
    // The toolbar's buttons (the pads and the header share names).
    let click = |ed: &mut Editor, n: &str| {
        let tb = ed.app.root.path_of("ToolBar").unwrap();
        let path = [tb.clone(), (ed.app.ctrl(&tb).unwrap()).path_of(n).unwrap()].concat();
        ed.handle(&UiEvent::Clicked { path, name: n.into(), id: 0, on: true });
    };
    let before = ed.model.pattern(e, cur);
    let n = before.steps as usize;
    click(&mut ed, "RepeatButton");
    let after = ed.model.pattern(e, cur);
    assert_eq!(after.steps as usize, (n * 2).min(1024));
    for t in 0..16 {
        for s in 0..n {
            assert_eq!(after.tracks[t].steps[s + n].on, before.tracks[t].steps[s].on, "track {t} step {s}");
        }
    }
    // Quantize resets every Shift lane.
    click(&mut ed, "QuantizeButton");
    let d = dreist_formats::params::step_lane_default(3);
    assert!(ed.model.pattern(e, cur).tracks.iter().all(|tr| tr.steps.iter().all(|st| (st.lanes[3] - d).abs() < 1e-4)));
    click(&mut ed, "MonoButton");
    assert_eq!(ed.model.pattern(e, cur).mono, !after.mono);
    // View mode: multi graph (the single global graph) and back to multi
    // track, restoring the saved option.
    // (The tools panel has a MultiButton too: the view's are in ModeGroup.)
    let view = |ed: &mut Editor, n: &str| {
        let g = ed.app.root.path_of("ModeGroup").unwrap();
        let path = [g.clone(), (ed.app.ctrl(&g).unwrap()).path_of(n).unwrap()].concat();
        ed.handle(&UiEvent::Clicked { path, name: n.into(), id: 0, on: true });
    };
    let multi = ed.pattern.as_ref().unwrap().multi;
    view(&mut ed, if multi { "SingleButton" } else { "MultiButton" });
    assert_eq!(ed.pattern.as_ref().unwrap().multi, !multi);
    view(&mut ed, if multi { "MultiButton" } else { "SingleButton" });
    assert_eq!(ed.pattern.as_ref().unwrap().multi, multi);
}

/// The path of `name` on the sampler page.
fn sampler_path(ed: &Editor, name: &str) -> Vec<usize> {
    let host = ed.app.root.path_of("SamplerPanel").expect("sampler page");
    [host.clone(), ed.app.ctrl(&host).unwrap().path_of(name).unwrap_or_else(|| panic!("no {name}"))].concat()
}

fn messages(ed: &mut Editor) -> Vec<dreist_engine::binding::Message> {
    ed.take_commands().into_iter().filter_map(|c| if let Command::Message(m) = c { Some(m) } else { None }).collect()
}

#[test]
fn sampler_takes_join_the_take_list_and_go_to_pads() {
    crate::app::test_setup();
    use dreist_engine::binding::id;
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    let tmp = std::env::temp_dir().join(format!("dreist-takes-{}", std::process::id()));
    ed.modals.options.set("SamplerRecordPath", tmp.to_string_lossy());
    ed.model.samples = Some(tmp.clone());
    ed.model.editor_tab = crate::front::SAMPLER_PAGE;
    ed.sync();
    let text = |ed: &Editor, n: &str| ed.app.ctrl(&sampler_path(ed, n)).unwrap().text.clone();
    assert_eq!(text(&ed, "RecordStatus"), "Idle");
    assert!(!ed.app.ctrl(&sampler_path(&ed, "SendToPadButton")).unwrap().enabled, "nothing to send");
    let tone: Vec<f32> = (0..4410).map(|i| (i as f32 * 0.05).sin() * 2.0).collect();
    assert_eq!(ed.sampler_take(&tone, &tone, 44100.0), Some(0));
    assert_eq!(ed.sampler_take(&tone[..2000], &tone[..2000], 44100.0), Some(1));
    assert_eq!((text(&ed, "TakeList").as_str(), text(&ed, "RecordStatus").as_str()), ("Take 2", "Idle (2 takes)"));
    // Take picks the current take.
    let tl = sampler_path(&ed, "TakeList");
    ed.handle(&UiEvent::Clicked { path: tl.clone(), name: "TakeList".into(), id: 0, on: false });
    assert_eq!(ed.app.menu.as_ref().map(|m| m.items.clone()).unwrap(), ["Take 1", "Take 2"]);
    ed.app.menu = None;
    ed.handle(&UiEvent::MenuSelected { anchor: tl, name: "TakeList".into(), index: 0 });
    assert_eq!(ed.sampler.current, 0);
    // → Pad: the take (saved as "<Name> <Take>.wav", peaks below 0 dBFS)
    // is the current pad's first layer.
    ed.take_commands();
    let send = sampler_path(&ed, "SendToPadButton");
    ed.handle(&UiEvent::Clicked { path: send, name: "SendToPadButton".into(), id: 0, on: false });
    let p = ed.model.pad[0];
    let file = ed.model.layer_sample(0, p, 0).expect("the take on the pad");
    assert!(file.starts_with(&tmp) && file.exists() && file.file_name().unwrap().to_string_lossy().starts_with("Sample Take 1"), "{file:?}");
    let d = dreist_engine::audio::SampleCache::default().get(&file).expect("readable wav");
    let peak = d.left.iter().fold(0.0f32, |m, v| m.max(v.abs()));
    assert!(peak <= 0.999 && peak > 0.9, "scaled: {peak}");
    assert!(ed.take_commands().iter().any(|c| matches!(c, Command::Rebuild { .. })));
    // → Pads: every take to the first layer of an empty pad.
    let all = sampler_path(&ed, "SendAllToPadsButton");
    ed.handle(&UiEvent::Clicked { path: all, name: "SendAllToPadsButton".into(), id: 0, on: false });
    let loaded = (0..16).filter(|&k| ed.model.layer_sample(0, k, 0).is_some()).count();
    assert_eq!(loaded, 3);
    // Clear drops the current take, Clear All the rest.
    let clear = sampler_path(&ed, "ClearButton");
    ed.handle(&UiEvent::Clicked { path: clear, name: "ClearButton".into(), id: 0, on: false });
    assert_eq!(ed.sampler.takes.len(), 1);
    assert!(messages(&mut ed).iter().any(|m| m.kind == id::SAMPLER_CLEAR_TAKE && m.value[0] == 0.0));
    let clear = sampler_path(&ed, "ClearAllButton");
    ed.handle(&UiEvent::Clicked { path: clear, name: "ClearAllButton".into(), id: 0, on: false });
    assert!(ed.sampler.takes.is_empty());
    assert!(messages(&mut ed).iter().any(|m| m.kind == id::SAMPLER_CLEAR_TAKES));
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn sampler_page_modes_arm_record_and_preview() {
    crate::app::test_setup();
    use dreist_engine::binding::id;
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    ed.model.editor_tab = crate::front::SAMPLER_PAGE;
    ed.sync();
    ed.take_commands();
    let click = |ed: &mut Editor, path: Vec<usize>, on: bool| ed.handle(&UiEvent::Clicked { path, name: "x".into(), id: 0, on });
    let child = |ed: &Editor, g: &str, i: usize| [sampler_path(ed, g), vec![i]].concat();
    let visible = |ed: &Editor, n: &str| ed.app.ctrl(&sampler_path(ed, n)).unwrap().visible;
    // Arm and Rec / Stop.
    let arm = sampler_path(&ed, "ArmButton");
    ed.handle(&UiEvent::Clicked { path: arm, name: "ArmButton".into(), id: 0, on: true });
    let rec = sampler_path(&ed, "RecButton");
    ed.handle(&UiEvent::Clicked { path: rec, name: "RecButton".into(), id: 0, on: true });
    let kinds: Vec<u32> = messages(&mut ed).iter().map(|m| m.kind).collect();
    assert_eq!(kinds, [id::SAMPLER_TOGGLE_ARMED, id::SAMPLER_TOGGLE_RECORD]);
    // Trigger mode: MIDI hides the threshold controls, Threshold shows them;
    // the record mode follows each start mode.
    let c = child(&ed, "TriggerModeButtonGroup", 1);
    click(&mut ed, c, true);
    let m = messages(&mut ed);
    assert_eq!((m[0].kind, m[0].value[0], m[1].kind), (id::SAMPLER_START_MODE, 1.0, id::SAMPLER_TIMED_MODE));
    assert!(!visible(&ed, "ThresholdPanel"));
    let c = child(&ed, "TriggerModeButtonGroup", 3);
    click(&mut ed, c, true);
    assert!(visible(&ed, "ThresholdPanel"));
    assert!(ed.app.ctrl(&child(&ed, "TriggerModeButtonGroup", 3)).unwrap().on);
    // Record mode: Looped (retake) shows the length; Free hides it.
    ed.take_commands();
    let c = child(&ed, "TimedModeButtonGroup", 2);
    click(&mut ed, c, true);
    let m = messages(&mut ed);
    assert_eq!((m[0].kind, m[0].value[0]), (id::SAMPLER_TIMED_MODE, 2.0));
    assert!(visible(&ed, "SyncPanel"));
    let c = child(&ed, "SyncButtonGroup", 1);
    click(&mut ed, c, true);
    assert!(visible(&ed, "SecondsPanel") && !visible(&ed, "BeatsPanel"));
    let c = child(&ed, "TimedModeButtonGroup", 0);
    click(&mut ed, c, true);
    assert!(!visible(&ed, "SyncPanel"));
    // Mono records the left channel.
    ed.take_commands();
    let mono = sampler_path(&ed, "MonoButton");
    ed.handle(&UiEvent::Clicked { path: mono, name: "MonoButton".into(), id: 0, on: true });
    assert!(messages(&mut ed).iter().any(|m| m.kind == id::SAMPLER_MONO && m.value[0] == 1.0));
    // Preview plays the take between its markers; Crop keeps that part.
    let tone: Vec<f32> = (0..1000).map(|i| (i as f32 * 0.05).sin() * 0.5).collect();
    ed.sampler_take(&tone, &tone, 44100.0);
    ed.sampler.takes[0].start = 100;
    ed.sampler.takes[0].end = 600;
    let pv = sampler_path(&ed, "PreviewButton");
    ed.handle(&UiEvent::Clicked { path: pv.clone(), name: "PreviewButton".into(), id: 0, on: true });
    assert!(ed.take_commands().iter().any(|c| matches!(c, Command::PreviewRegion { start: 100, end: 600, looping: false, .. })));
    ed.handle(&UiEvent::Clicked { path: pv, name: "PreviewButton".into(), id: 0, on: false });
    assert!(messages(&mut ed).iter().any(|m| m.kind == id::STOP_PREVIEW));
    let crop = sampler_path(&ed, "CropButton");
    ed.handle(&UiEvent::Clicked { path: crop, name: "CropButton".into(), id: 0, on: false });
    assert_eq!(ed.sampler.takes[0].frames(), 500);
    if let Ok(f) = std::env::var("DREIST_SAMPLER_SHOT") {
        ed.sampler_take(&[tone.clone(), tone.clone(), tone.clone()].concat(), &[], 44100.0);
        ed.sampler.takes[1].start = 500;
        ed.sampler.takes[1].end = 2500;
        ed.sync();
        ed.render(1.0).save_png(f).unwrap();
    }
    // The Export/Send menu.
    let sb = sampler_path(&ed, "SaveButton");
    ed.handle(&UiEvent::Clicked { path: sb.clone(), name: "SaveButton".into(), id: 0, on: false });
    let items = ed.app.menu.as_ref().map(|m| m.items.clone()).unwrap();
    assert_eq!((items[0].as_str(), items[2].as_str(), items.len()), ("Export take...", "Send take to current pad...", 8));
    ed.app.menu = None;
    ed.handle(&UiEvent::MenuSelected { anchor: sb, name: "SaveButton".into(), index: 7 });
    assert!(ed.sampler.auto_crop);
}

#[test]
fn engine_step_writes_reach_the_document() {
    crate::app::test_setup();
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(crate::project::new_document())) else { return };
    let mut data = dreist_engine::pattern::StepData::off();
    data.on = true;
    ed.step_writes(&[dreist_engine::StepWrite { engine: 0, pattern: 2, track: 5, step: 7, data }]);
    let st = ed
        .model
        .engine_node(0)
        .and_then(|n| n.children.iter().find(|c| c.class == "StepPattern" && c.name.as_deref() == Some("StepPattern2")))
        .and_then(|p| p.children.iter().find(|c| c.name.as_deref() == Some("Track5")))
        .and_then(|t| t.children.iter().find(|c| c.name.as_deref() == Some("S7")));
    assert_eq!(st.and_then(|n| n.attr("State")), Some("true"));
}

/// A layer row's Trigger plays that layer only; the pad item's the pad.
#[test]
fn layer_trigger_plays_the_layer() {
    crate::app::test_setup();
    let Some(mut ed) = demo_editor("Presets/Hollow.geist") else { return };
    ed.model.editor_tab = crate::front::PADS_PAGE;
    ed.sync();
    let (e, p) = (ed.model.engine, ed.model.pad[ed.model.engine]);
    let row = ed.app.root.path_of("LayerSelectionButton2").unwrap();
    let trig = |ed: &Editor, from: &[usize]| {
        let host = ed.app.ctrl(from).unwrap();
        let mut path = from.to_vec();
        path.extend(host.path_of("Trigger").unwrap());
        path
    };
    let t = trig(&ed, &row);
    ed.take_commands();
    ed.handle(&UiEvent::Pressed { path: t.clone(), name: "Trigger".into() });
    let cmds = ed.take_commands();
    assert!(matches!(cmds.as_slice(), [Command::LayerHit { engine, pad, layer: 2, velocity: 127 }] if *engine == e && *pad == p), "{cmds:?}");
    ed.handle(&UiEvent::Clicked { path: t, name: "Trigger".into(), id: 0, on: false });
    assert!(matches!(ed.take_commands().as_slice(), [Command::PadRelease { .. }]));
}

#[test]
fn v2a_knob_reset_and_edit() {
    crate::app::test_setup();
    let Some(mut ed) = demo_editor("Presets/Hollow.geist") else { return };
    // The pad view (the pad item of the selector).
    ed.model.editor_tab = crate::front::PADS_PAGE;
    ed.inspector.pad_view = true;
    ed.sync();
    let knob = ed.app.root.path_of("V2A").unwrap();
    let v2a = |cmds: &[Command]| {
        cmds.iter().find_map(|c| match c {
            Command::Message(m) if m.kind == dreist_engine::binding::id::SET_PAD_PARAMETER && m.value[1] == 2.0 => Some(m.value[0]),
            _ => None,
        })
    };
    ed.take_commands();
    ed.handle(&UiEvent::RightClicked { path: knob.clone(), name: "V2A".into() });
    assert_eq!(ed.app.menu.as_ref().unwrap().items, vec!["Reset".to_string(), "Edit".to_string()]);
    ed.handle(&UiEvent::MenuSelected { anchor: knob.clone(), name: "V2A".into(), index: 0 });
    assert_eq!(v2a(&ed.take_commands()), Some(1.0));
    ed.app.menu = None;
    ed.handle(&UiEvent::RightClicked { path: knob.clone(), name: "V2A".into() });
    ed.handle(&UiEvent::MenuSelected { anchor: knob.clone(), name: "V2A".into(), index: 1 });
    assert!(ed.app.is_editing());
    for _ in 0..5 {
        ed.app.key_named("Backspace");
    }
    ed.app.key_text("40");
    let (evs, _) = ed.app.key_named("Enter");
    for ev in evs {
        ed.handle(&ev);
    }
    let got = v2a(&ed.take_commands()).expect("edit posts 0x4e");
    assert!((got - 0.4).abs() < 1e-6, "{got}");
    assert!(ed.app.root.path_of("V2AValueEdit").is_none(), "edit box removed");
}

#[test]
fn file_menu_parts_save_and_open() {
    crate::app::test_setup();
    let p = demo("Presets/Hollow.geist");
    let Ok(proj) = dreist_formats::Project::load(&p) else { return };
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(proj.raw)) else { return };
    ed.model.samples = Some(dreist_formats::model::samples_dir(&p));
    let dir = std::env::temp_dir().join(format!("dreist-parts-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let d2 = dir.clone();
    ed.dialogs = Some(Box::new(move |d| match d {
        Dialog::SavePart(part) | Dialog::OpenPart(part) => Some(d2.join(format!("x.{}", part.file_type().0))),
        Dialog::SaveMidi | Dialog::OpenMidi => Some(d2.join("x.mid")),
        _ => None,
    }));
    // The master section's LOAD and SAVE menus.
    let (load, save) = (front_path(&ed, "UtilityControls", "LoadButton"), front_path(&ed, "UtilityControls", "SaveButton"));
    let menu = std::cell::Cell::new(false);
    let pick = |ed: &mut Editor, label: &str| {
        let items = ed.app.menu.as_ref().expect("file menu").items.clone();
        let i = items.iter().position(|s| s == label).unwrap_or_else(|| panic!("{label} in {items:?}"));
        let (b, n) = if menu.get() { (save.clone(), "SaveButton") } else { (load.clone(), "LoadButton") };
        ed.handle(&UiEvent::MenuSelected { anchor: b, name: n.into(), index: i });
    };
    let open_menu = |ed: &mut Editor, saving: bool| {
        menu.set(saving);
        let (b, n) = if saving { (save.clone(), "SaveButton") } else { (load.clone(), "LoadButton") };
        ed.handle(&UiEvent::Clicked { path: b, name: n.into(), id: 0, on: false });
    };
    let open = |ed: &mut Editor| open_menu(ed, false);
    let save_menu = |ed: &mut Editor| open_menu(ed, true);
    open(&mut ed);
    let items = ed.app.menu.as_ref().unwrap().items.clone();
    assert_eq!(
        items,
        ["Load preset", "Load engine", "Load kit", "Load pad", "Load pattern bank", "Load pattern", "Load pattern from MIDI", "Load mappings", "Load Guru... ▸"].map(String::from).to_vec()
    );
    ed.app.menu = None;
    save_menu(&mut ed);
    let items = ed.app.menu.as_ref().unwrap().items.clone();
    assert_eq!(items.len(), 9, "{items:?}");
    assert_eq!(items[6], "Save current pattern as MIDI");
    pick(&mut ed, "Save current engine");
    let text = std::fs::read_to_string(dir.join("x.einstengine")).expect("engine file");
    assert!(text.contains("<Engine name=\"Engine\""), "{}", &text[..200.min(text.len())]);
    assert!(dir.join("x.samples").is_dir(), "samples collected");
    // Save current pattern.
    ed.app.menu = None;
    save_menu(&mut ed);
    pick(&mut ed, "Save current pattern");
    assert!(std::fs::read_to_string(dir.join("x.einstpattern")).unwrap().contains("<StepPattern name=\"StepPattern\""));
    ed.app.menu = None;
    save_menu(&mut ed);
    pick(&mut ed, "Save current pattern as MIDI");
    let mid = std::fs::read(dir.join("x.mid")).expect("midi file");
    assert_eq!(&mid[..4], b"MThd");
    let ch = (ed.model.engine & 15) as u8;
    assert!(mid.windows(1).any(|w| w[0] == 0x90 | ch), "note ons on the engine's channel");
    // Load pattern from MIDI brings the same pattern back (steps on).
    let pat = ed.model.current_pattern(ed.model.engine);
    let on_before: usize = ed.model.pattern(ed.model.engine, pat).tracks.iter().map(|t| t.steps.iter().filter(|s| s.on).count()).sum();
    ed.app.menu = None;
    open(&mut ed);
    pick(&mut ed, "Load pattern from MIDI");
    let on_after: usize = ed.model.pattern(ed.model.engine, pat).tracks.iter().map(|t| t.steps.iter().filter(|s| s.on).count()).sum();
    assert!(on_after > 0 && on_after <= on_before, "{on_after} vs {on_before}");
    assert_eq!(ed.model.pattern_name(ed.model.engine, pat), "x");
    // Change the engine, then Load engine brings the file back.
    let e = ed.model.engine;
    let before = ed.model.engine_node(e).and_then(|n| n.children.iter().find(|c| c.class == "Pad").cloned());
    ed.model.engine_mut(e).children.retain(|c| c.class != "Pad");
    ed.take_commands();
    ed.app.menu = None;
    open(&mut ed);
    pick(&mut ed, "Load engine");
    let after = ed.model.engine_node(e).and_then(|n| n.children.iter().find(|c| c.class == "Pad").cloned());
    assert_eq!(after.map(|n| n.name), before.map(|n| n.name));
    assert!(ed.take_commands().iter().any(|c| matches!(c, Command::Rebuild { .. })));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn swing_menu_head_items_and_template_pick() {
    crate::app::test_setup();
    let Some(mut ed) = demo_editor("Presets/Hollow.geist") else { return };
    let Some(drop) = ed.app.root.path_of("SwingDrop") else { return };
    ed.handle(&UiEvent::Clicked { path: drop.clone(), name: "SwingDrop".into(), id: 0, on: false });
    let items = ed.app.menu.as_ref().expect("swing menu").items.clone();
    assert_eq!(&items[..3], &["Extract timing from current pattern".to_string(), "Import MIDI template".into(), "Import Guru swing/shuffle".into()]);
    if items.len() > 4 {
        ed.take_commands();
        ed.handle(&UiEvent::MenuSelected { anchor: drop.clone(), name: "SwingDrop".into(), index: 4 });
        assert!(ed.take_commands().iter().any(|c| matches!(c, Command::SwingTemplate(..))));
    }
    // The Swing knob's right-click adds Reset / Edit.
    ed.app.menu = None;
    if let Some(knob) = ed.app.root.path_of("SwingKnob") {
        ed.handle(&UiEvent::RightClicked { path: knob.clone(), name: "SwingKnob".into() });
        let items = ed.app.menu.as_ref().expect("knob menu").items.clone();
        assert_eq!(&items[items.len() - 2..], &["Reset".to_string(), "Edit".into()]);
        let anchor = ed.app.menu.as_ref().unwrap().anchor.clone();
        ed.take_commands();
        ed.handle(&UiEvent::MenuSelected { anchor, name: "SwingDrop".into(), index: items.len() - 2 });
        assert!(ed.take_commands().iter().any(|c| matches!(c, Command::Message(m) if m.kind == dreist_engine::binding::id::SET_ENGINE_SWING_TEMPLATE_DEPTH && m.value[0] == 0.0)));
    }
    // Import MIDI template turns a file's timing into a template.
    let dir = std::env::temp_dir().join(format!("dreist-swing-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let mid = dir.join("Groove Test.mid");
    assert!(ed.save_pattern_midi(&mid));
    if let Some(out) = ed.import_midi_template(&mid) {
        assert!(out.ends_with("Groove Test.swingtemplate"));
        let _ = std::fs::remove_file(&out);
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn engine_selector_level_overlay() {
    crate::app::test_setup();
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    let mut m = dreist_engine::Meters::default();
    m.engine[0] = [1.0, 1.0];
    ed.update_meters(&m);
    fn find(c: &dreist_gui::Ctrl) -> Option<&dreist_gui::Ctrl> {
        if c.attr("volEngine") == Some("0") {
            return Some(c);
        }
        c.children.iter().find_map(find)
    }
    // v1: the level curve of (L + R)/2 (0 dB → 0.8).
    let a = find(&ed.app.root).map(|c| c.value);
    assert!(a.is_some_and(|a| (a - 0.8).abs() < 1e-4), "{a:?}");
    if let Ok(out) = std::env::var("DREIST_HEADER_SHOT") {
        let mut m = dreist_engine::Meters::default();
        m.engine[0] = [1.0, 1.0];
        m.master = [0.8, 0.8];
        ed.update_meters(&m);
        ed.render(1.0).save_png(out).unwrap();
    }
    // Falls by at most 0.1 per tick.
    ed.update_meters(&dreist_engine::Meters::default());
    let a = find(&ed.app.root).map(|c| c.value).unwrap();
    assert!((a - 0.7).abs() < 1e-4, "{a}");
}

#[test]
fn layer_wave_zoom_bar() {
    crate::app::test_setup();
    let p = demo("Presets/Hollow.geist");
    let Ok(proj) = dreist_formats::Project::load(&p) else { return };
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(proj.raw)) else { return };
    ed.model.samples = Some(dreist_formats::model::samples_dir(&p));
    let e = ed.model.engine;
    let Some((pad, l)) = (0..64).flat_map(|p| (0..model::LAYERS).map(move |l| (p, l))).find(|&(p, l)| ed.model.layer_sample(e, p, l).is_some_and(|f| f.exists())) else { return };
    ed.model.pad[e] = pad;
    ed.model.layer[e] = l;
    ed.sync();
    assert_eq!(ed.inspector.view, (0.0, 1.0));
    fn tagged(c: &dreist_gui::Ctrl, name: &str, path: &mut Vec<usize>) -> bool {
        for (i, k) in c.children.iter().enumerate() {
            path.push(i);
            if (k.name == name && k.attr("layerZoom").is_some()) || tagged(k, name, path) {
                return true;
            }
            path.pop();
        }
        false
    }
    let mut zin = Vec::new();
    assert!(tagged(&ed.app.root, "ZoomInButton", &mut zin));
    if let Ok(out) = std::env::var("DREIST_WAVE_SHOT") {
        ed.render(1.0).save_png(out).unwrap();
    }
    ed.handle(&UiEvent::Clicked { path: zin, name: "ZoomInButton".into(), id: 0, on: false });
    assert!((ed.inspector.view.1 - 0.5).abs() < 1e-4, "{:?}", ed.inspector.view);
    let mut group = Vec::new();
    assert!(tagged(&ed.app.root, "AutoZoomGroup", &mut group));
    group.push(2);
    ed.handle(&UiEvent::Clicked { path: group.clone(), name: String::new(), id: 0, on: true });
    assert!((ed.inspector.view.1 - 0.01).abs() < 1e-4, "start-marker zoom: {:?}", ed.inspector.view);
    // Back to no auto zoom (the option is restored).
    *group.last_mut().unwrap() = 0;
    ed.handle(&UiEvent::Clicked { path: group, name: String::new(), id: 0, on: true });
}

#[test]
fn pad_menu_multi_selection() {
    crate::app::test_setup();
    let Some(mut ed) = demo_editor("Presets/Hollow.geist") else { return };
    let e = ed.model.engine;
    multipad::select(&mut ed.model, e, 1, false, false);
    multipad::select(&mut ed.model, e, 3, true, false);
    fn find(c: &dreist_gui::Ctrl, path: &mut Vec<usize>) -> bool {
        for (i, k) in c.children.iter().enumerate() {
            path.push(i);
            if (k.overload == "EditorPad" && k.attr("padIndex") == Some("1")) || find(k, path) {
                return true;
            }
            path.pop();
        }
        false
    }
    let mut pad1 = Vec::new();
    assert!(find(&ed.app.root, &mut pad1));
    let menu = |ed: &mut Editor, label: &str| {
        ed.handle(&UiEvent::Mouse { path: pad1.clone(), overload: "EditorPad".into(), kind: dreist_gui::app::MouseKind::RightDown, x: 5.0, y: 5.0 });
        let items = ed.app.menu.as_ref().expect("pad menu").items.clone();
        assert!(!items.iter().any(|s| s == "Cut" || s.starts_with("Rename")), "{items:?}");
        assert!(items.contains(&"(Clear pad)".to_string()), "{items:?}");
        let i = items.iter().position(|s| s.ends_with(label)).unwrap_or_else(|| panic!("{label} in {items:?}"));
        ed.handle(&UiEvent::MenuSelected { anchor: pad1.clone(), name: "EditorPad".into(), index: i });
    };
    menu(&mut ed, "Lock pad");
    assert!(padmenu::pad_locked(&ed.model, e, 1) && padmenu::pad_locked(&ed.model, e, 3));
    assert!(!padmenu::pad_locked(&ed.model, e, 2));
}

#[test]
fn layer_row_menu() {
    crate::app::test_setup();
    let p = demo("Presets/Hollow.geist");
    let Ok(proj) = dreist_formats::Project::load(&p) else { return };
    let mut model = Model::new(proj.raw);
    model.samples = Some(dreist_formats::model::samples_dir(&p));
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), model) else { return };
    fn find(c: &dreist_gui::Ctrl, path: &mut Vec<usize>) -> bool {
        for (i, k) in c.children.iter().enumerate() {
            path.push(i);
            if k.overload == "LayerRow" && k.attr("layer") == Some("0") {
                return true;
            }
            if find(k, path) {
                return true;
            }
            path.pop();
        }
        false
    }
    let mut row = Vec::new();
    assert!(find(&ed.app.root, &mut row), "layer row");
    let e = ed.model.engine;
    let pad = ed.model.pad[e];
    let menu = |ed: &mut Editor, label: &str| -> Vec<String> {
        ed.handle(&UiEvent::Mouse { path: row.clone(), overload: "LayerRow".into(), kind: dreist_gui::app::MouseKind::RightDown, x: 5.0, y: 5.0 });
        let items = ed.app.menu.as_ref().expect("layer menu").items.clone();
        let i = items.iter().position(|s| s.ends_with(label)).unwrap_or_else(|| panic!("{label} in {items:?}"));
        ed.handle(&UiEvent::MenuSelected { anchor: row.clone(), name: "LayerRow".into(), index: i });
        items
    };
    assert!(ed.model.layer(e, pad, 0).valid);
    let items = menu(&mut ed, "Copy");
    assert_eq!(items.iter().map(|s| s.trim_start_matches(|c: char| !c.is_ascii_alphanumeric())).collect::<Vec<_>>(), padmenu::LAYER_ITEMS.iter().map(|s| s.to_string()).collect::<Vec<_>>());
    ed.model.layer_mut(e, pad, 0).set("AmpEnvAttack", 0.4).set("LayerGain", 0.3);
    menu(&mut ed, "Reset envelopes");
    let n = ed.model.layer_node(e, pad, 0).unwrap();
    assert_eq!(n.f32("AmpEnvAttack"), Some(0.0));
    assert_eq!(n.f32("LayerGain"), Some(0.3));
    assert!(n.f32("AmpEnvHold").is_some_and(|h| (0.0..=1.0).contains(&h)));
    menu(&mut ed, "Reset layer");
    let n = ed.model.layer_node(e, pad, 0).unwrap();
    assert!((n.f32("LayerGain").unwrap() - 0.7937).abs() < 1e-3 && n.attr("AudioFileName").is_some());
    menu(&mut ed, "Cut");
    assert!(!ed.model.layer(e, pad, 0).valid);
    menu(&mut ed, "Paste");
    assert!(ed.model.layer(e, pad, 0).valid);
    menu(&mut ed, "Clear layer");
    assert!(ed.model.layer_node(e, pad, 0).is_some_and(|n| n.attrs.is_empty()));
    ed.outbox.clear();
    menu(&mut ed, "Tune loop to tempo");
    assert!(ed.outbox.iter().any(|c| matches!(c, crate::command::Command::Message(m) if m.kind == dreist_engine::binding::id::TUNE_LAYER)));
}

#[test]
fn pad_row_menu() {
    crate::app::test_setup();
    let p = demo("Presets/Hollow.geist");
    let Ok(proj) = dreist_formats::Project::load(&p) else { return };
    let mut model = Model::new(proj.raw);
    model.samples = Some(dreist_formats::model::samples_dir(&p));
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), model) else { return };
    let e = ed.model.engine;
    let pad = ed.model.pad[e];
    let mut clip = padmenu::Clipboard::default();
    let mut out = Vec::new();
    let items = padmenu::pad_row_items(&ed.model, &clip, e, pad);
    assert_eq!(items.len(), padmenu::PAD_ROW_ITEMS.len());
    ed.model.layer_mut(e, pad, 1).set("LayerGain", 0.2);
    assert!(padmenu::pad_row_action(&mut ed.model, &mut clip, e, pad, 6, &mut out));
    assert!((ed.model.layer_node(e, pad, 1).unwrap().f32("LayerGain").unwrap() - 0.7937).abs() < 1e-3);
    assert!(!padmenu::pad_row_action(&mut ed.model, &mut clip, e, pad, 4, &mut out));
    assert_eq!(crate::multipad::layer_selection(&ed.model, e, pad).1, 0xff);
    assert!(padmenu::pad_row_action(&mut ed.model, &mut clip, e, pad, 0, &mut out));
    assert!(clip.pad.is_some() && !ed.model.layer(e, pad, 0).valid);
    assert!(padmenu::pad_row_action(&mut ed.model, &mut clip, e, pad, 2, &mut out));
    assert!(ed.model.layer(e, pad, 0).valid);
}

#[test]
fn pad_menu_locks_and_swaps() {
    crate::app::test_setup();
    let p = demo("Presets/Hollow.geist");
    let Ok(proj) = dreist_formats::Project::load(&p) else { return };
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(proj.raw)) else { return };
    fn find(c: &dreist_gui::Ctrl, idx: &str, path: &mut Vec<usize>) -> bool {
        for (i, k) in c.children.iter().enumerate() {
            path.push(i);
            if k.overload == "EditorPad" && k.attr("padIndex") == Some(idx) {
                return true;
            }
            if find(k, idx, path) {
                return true;
            }
            path.pop();
        }
        false
    }
    let mut pad1 = Vec::new();
    assert!(find(&ed.app.root, "1", &mut pad1));
    let menu = |ed: &mut Editor, label: &str| {
        ed.handle(&UiEvent::Mouse { path: pad1.clone(), overload: "EditorPad".into(), kind: dreist_gui::app::MouseKind::RightDown, x: 5.0, y: 5.0 });
        let items = ed.app.menu.as_ref().expect("pad menu").items.clone();
        let i = items.iter().position(|s| s.ends_with(label)).unwrap_or_else(|| panic!("{label} in {items:?}"));
        ed.handle(&UiEvent::MenuSelected { anchor: pad1.clone(), name: "EditorPad".into(), index: i });
    };
    let e = ed.model.engine;
    menu(&mut ed, "Lock pad");
    assert!(padmenu::pad_locked(&ed.model, e, 1));
    // Locked pads survive kit loads.
    let before = ed.model.pad_node(e, 1).cloned();
    let kit = browser::content_dir().join(dreist_formats::legacy::DEMO_FACTORY);
    let kit = walk_find(&kit, "geistkit");
    if let Some(k) = kit {
        assert!(browser::load_kit_into_doc(&mut ed.model, &k));
        assert_eq!(ed.model.pad_node(e, 1).cloned(), before);
    }
    menu(&mut ed, "Unlock all pads");
    assert!(!padmenu::pad_locked(&ed.model, e, 1));
    let (a, b) = (ed.model.pad_name(e, 1), ed.model.pad_name(e, 2));
    menu(&mut ed, "Swap pad to ▸");
    ed.handle(&UiEvent::MenuSelected { anchor: pad1.clone(), name: "EditorPad".into(), index: 2 });
    assert_eq!((ed.model.pad_name(e, 2), ed.model.pad_name(e, 1)), (a, b));
    assert!(ed.take_commands().iter().any(|c| matches!(c, Command::Rebuild { .. })));
}

/// The right-click menu of the `EditorPad` with `padIndex` `idx`: picks
/// the item ending with `label`; returns the pad's path.
fn pad_menu_pick(ed: &mut Editor, idx: usize, label: &str) -> Vec<usize> {
    fn find(c: &dreist_gui::Ctrl, idx: &str, path: &mut Vec<usize>) -> bool {
        for (i, k) in c.children.iter().enumerate() {
            path.push(i);
            if (k.overload == "EditorPad" && k.attr("padIndex") == Some(idx)) || find(k, idx, path) {
                return true;
            }
            path.pop();
        }
        false
    }
    let mut path = Vec::new();
    assert!(find(&ed.app.root, &idx.to_string(), &mut path));
    ed.handle(&UiEvent::Mouse { path: path.clone(), overload: "EditorPad".into(), kind: dreist_gui::app::MouseKind::RightDown, x: 5.0, y: 5.0 });
    let items = ed.app.menu.as_ref().expect("pad menu").items.clone();
    let i = items.iter().position(|s| s.ends_with(label)).unwrap_or_else(|| panic!("{label} in {items:?}"));
    ed.handle(&UiEvent::MenuSelected { anchor: path.clone(), name: "EditorPad".into(), index: i });
    path
}

/// The grid label of pad `p` (pad grid children are pads 0..16 in order).
fn pad_grid_label(ed: &Editor, p: usize) -> String {
    ed.app.root.find("EditorPads").unwrap().children[p].find("NameLabel").unwrap().text.clone()
}

/// Clear pad resets the whole pad, name included ([re:8719e8]); the
/// default name shows as the pad number ([re:1cc6fe]).
#[test]
fn clear_pad_resets_the_pad_and_its_name() {
    crate::app::test_setup();
    let Some(mut ed) = demo_editor("Presets/Hollow.geist") else { return };
    let e = ed.model.engine;
    assert!(ed.model.layer(e, 0, 0).valid);
    ed.model.set_pad_name(e, 0, "Kick");
    ed.model.pad_mut(e, 0).set("V2A", "0.25").set("Gain", "0.5").set("PadMode", "Random");
    pad_menu_pick(&mut ed, 0, "Clear pad");
    let pad = ed.model.pad_node(e, 0).unwrap();
    assert_eq!(ed.model.pad_name(e, 0), None);
    assert!(pad.children.is_empty(), "{pad:?}");
    assert_eq!(pad.attrs, vec![("V2A".to_string(), "0.25".to_string())]);
    assert!(!ed.model.layer(e, 0, 0).valid);
    assert_eq!(pad_grid_label(&ed, 0), "1");
    // A stored default name also shows as the number.
    ed.model.set_pad_name(e, 0, "Pad 1");
    ed.model.set_pad_name(e, 1, "Pad 1");
    pad_menu_pick(&mut ed, 2, "Lock pad");
    assert_eq!((pad_grid_label(&ed, 0), pad_grid_label(&ed, 1)), ("1".to_string(), "Pad 1".to_string()));
}

/// Clear all pads resets every unlocked pad and skips locked ones
/// ([re:8719e8]).
#[test]
fn clear_all_pads_resets_unlocked_pads() {
    crate::app::test_setup();
    let Some(mut ed) = demo_editor("Presets/Hollow.geist") else { return };
    let e = ed.model.engine;
    ed.model.set_pad_name(e, 1, "Snare");
    pad_menu_pick(&mut ed, 1, "Lock pad");
    let locked = ed.model.pad_node(e, 1).cloned();
    pad_menu_pick(&mut ed, 0, "Clear all pads");
    assert_eq!(ed.model.pad_node(e, 1).cloned(), locked);
    for p in (0..16).filter(|&p| p != 1) {
        assert_eq!(ed.model.pad_name(e, p), None, "pad {p}");
        assert!(!ed.model.layer(e, p, 0).valid, "pad {p}");
        assert_eq!(pad_grid_label(&ed, p), (p + 1).to_string());
    }
}

/// Move pad copies the pad and resets the source, name included
/// ([re:8719e8]).
#[test]
fn move_pad_resets_the_source() {
    crate::app::test_setup();
    let Some(mut ed) = demo_editor("Presets/Hollow.geist") else { return };
    let e = ed.model.engine;
    ed.model.set_pad_name(e, 0, "Kick");
    let src = ed.model.pad_node(e, 0).cloned().unwrap();
    let path = pad_menu_pick(&mut ed, 0, "Move pad to ▸");
    ed.handle(&UiEvent::MenuSelected { anchor: path, name: "EditorPad".into(), index: 5 });
    let mut moved = ed.model.pad_node(e, 5).cloned().unwrap();
    moved.name = src.name.clone();
    assert_eq!(moved, src);
    assert_eq!(ed.model.pad_name(e, 5).as_deref(), Some("Kick"));
    assert_eq!(ed.model.pad_name(e, 0), None);
    assert!(!ed.model.layer(e, 0, 0).valid);
    assert_eq!(pad_grid_label(&ed, 0), "1");
}

fn walk_find(dir: &std::path::Path, ext: &str) -> Option<std::path::PathBuf> {
    for e in std::fs::read_dir(dir).ok()?.flatten() {
        let p = e.path();
        if p.is_dir() {
            if let Some(f) = walk_find(&p, ext) {
                return Some(f);
            }
        } else if p.extension().is_some_and(|x| x == ext) {
            return Some(p);
        }
    }
    None
}

#[test]
fn pattern_and_track_menus() {
    crate::app::test_setup();
    let p = demo("Presets/Hollow.geist");
    let Ok(proj) = dreist_formats::Project::load(&p) else { return };
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(proj.raw)) else { return };
    let e = ed.model.engine;
    let sel = ed.app.root.path_of("PatternSelectors").unwrap();
    let key0 = {
        let c = ed.app.ctrl(&sel).unwrap();
        let mut p = sel.clone();
        p.extend(c.path_of("Key0").unwrap());
        p
    };
    let pick = |ed: &mut Editor, anchor: &Vec<usize>, label: &str| {
        let items = ed.app.menu.as_ref().expect("menu").items.clone();
        let i = items.iter().position(|s| s.starts_with(label)).unwrap_or_else(|| panic!("{label} in {items:?}"));
        ed.handle(&UiEvent::MenuSelected { anchor: anchor.clone(), name: String::new(), index: i });
    };
    ed.handle(&UiEvent::RightClicked { path: key0.clone(), name: "Key0".into() });
    pick(&mut ed, &key0, "Lock pattern");
    assert!(ed.model.pattern_node(e, 0).and_then(|n| n.attr("LockState")).is_some());
    let used0 = ed.model.pattern_used(e, 0);
    let empty = (1..24).find(|&q| !ed.model.pattern_used(e, q)).unwrap();
    ed.handle(&UiEvent::RightClicked { path: key0.clone(), name: "Key0".into() });
    pick(&mut ed, &key0, "Copy to");
    ed.handle(&UiEvent::MenuSelected { anchor: key0.clone(), name: String::new(), index: empty });
    assert_eq!(ed.model.pattern_used(e, empty), used0);
    // Track name area of the first row.
    let rows = ed.app.root.path_of("ControlsScrollView").unwrap();
    let row = ed.app.ctrl(&rows).unwrap();
    let t = row.children[0].attr("track").unwrap().parse::<usize>().unwrap();
    let mut sa = rows.clone();
    sa.push(0);
    sa.push(row.children[0].children.iter().position(|c| c.name == "SelectArea").unwrap());
    // v1's Cut is the pattern editor's: the selected steps go to its step
    // clipboard and are switched off.
    ed.pattern.as_mut().unwrap().selection = [(t, 2)].into_iter().collect();
    ed.take_commands();
    ed.handle(&UiEvent::Mouse { path: sa.clone(), overload: "TrackSelectArea".into(), kind: dreist_gui::app::MouseKind::RightDown, x: 2.0, y: 2.0 });
    pick(&mut ed, &sa, "Cut");
    assert!(ed.pattern.as_ref().unwrap().has_copied_steps());
    assert!(ed.pattern.as_ref().unwrap().selection.is_empty());
    assert!(!ed.take_commands().is_empty());
    ed.handle(&UiEvent::Mouse { path: sa.clone(), overload: "TrackSelectArea".into(), kind: dreist_gui::app::MouseKind::RightDown, x: 2.0, y: 2.0 });
    pick(&mut ed, &sa, "Lock track");
    assert!(padmenu::track_node(&ed.model, e, t).and_then(|n| n.attr("LockState")).is_some());
}

#[test]
fn song_resize_start_phase_band_quantize() {
    crate::app::test_setup();
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    ed.model.editor_tab = 2;
    ed.sync();
    song_mode_on(&mut ed);
    let body = ed.app.root.path_of("SongBody").unwrap();
    let (bx, by) = ed.app.origin(&body);
    let drag = |ed: &mut Editor, x: f32, y: f32, x2: f32, y2: f32, shift: bool| {
        ed.app.mods.shift = shift;
        for ev in ed.app.mouse_move(bx as f32 + x, by as f32 + y) {
            ed.handle(&ev);
        }
        for ev in ed.app.mouse_down(dreist_gui::app::MouseButton::Left) {
            ed.handle(&ev);
        }
        for ev in ed.app.mouse_move(bx as f32 + x2, by as f32 + y2) {
            ed.handle(&ev);
        }
        for ev in ed.app.mouse_up(dreist_gui::app::MouseButton::Left) {
            ed.handle(&ev);
        }
        ed.app.mods.shift = false;
    };
    let song = |ed: &Editor| ed.model.doc.children.iter().find(|c| c.class == "SongSequencer").map(dreist_engine::song::Song::from_node).unwrap();
    let rh = {
        let c = ed.app.ctrl(&body).unwrap();
        ((c.rect.h as f32 - 20.0) / 8.0).floor().max(12.0)
    };
    let mid = 20.0 + rh * 0.3;
    // Insert (one 4 QN pattern = 64 px at zoom 16).
    drag(&mut ed, 5.0, mid, 5.0, mid, false);
    assert_eq!(song(&ed).tracks[0].len(), 1);
    let snap = ed.song.as_ref().unwrap().snap_qn();
    // Right edge: two snap steps longer.
    drag(&mut ed, 63.0, mid, 63.0 + 32.0 * snap as f32, mid, false);
    let p = song(&ed).tracks[0][0].clone();
    assert!((p.length - (4.0 + 2.0 * snap)).abs() < 1e-6, "{p:?}");
    // Left edge: one step later, end fixed, phase follows.
    drag(&mut ed, 1.0, mid, 1.0 + 16.0 * snap as f32, mid, false);
    let p = song(&ed).tracks[0][0].clone();
    assert!((p.start - snap).abs() < 1e-6 && (p.end() - (4.0 + 2.0 * snap)).abs() < 1e-6, "{p:?}");
    assert!((p.phase - snap % 4.0).abs() < 1e-6, "{p:?}");
    // Bottom band: phase drag.
    let x = (p.start as f32 + 1.0) * 16.0;
    let low = 20.0 + rh - 3.0;
    drag(&mut ed, x, low, x + 16.0 * snap as f32, low, false);
    let q = song(&ed).tracks[0][0].clone();
    assert!((q.start - p.start).abs() < 1e-6 && (q.phase - (p.phase - snap).rem_euclid(4.0)).abs() < 1e-6, "{q:?}");
    // Shift band over empty space selects nothing new, over the part selects it.
    drag(&mut ed, 300.0, mid, 2.0, mid, true);
    assert_eq!(ed.song.as_ref().unwrap().selected.len(), 1);
}

#[test]
fn missing_files_auto_find() {
    crate::app::test_setup();
    let dir = std::env::temp_dir().join(format!("dreist-missing-{}", std::process::id()));
    std::fs::create_dir_all(dir.join("found")).unwrap();
    std::fs::write(dir.join("found/hat.wav"), b"").unwrap();
    let mut doc = project::new_document();
    {
        let eng = doc.child_mut_or_insert("Engine", "Engine0");
        let pad = eng.child_mut_or_insert("Pad", "Pad2");
        let l = pad.child_mut_or_insert("Layer", "Layer0");
        l.set("AudioFileName", "/nowhere/at/all/hat.wav");
    }
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    ed.open_document(doc, Some(dir.clone()), None);
    assert!(ed.filefind.open);
    assert_eq!(ed.filefind.items.len(), 1);
    let d2 = dir.clone();
    ed.dialogs = Some(Box::new(move |_| Some(d2.clone())));
    let auto = ed.app.root.path_of("AutoButton").unwrap();
    ed.handle(&UiEvent::Clicked { path: auto, name: "AutoButton".into(), id: 0, on: true });
    // The job runs on a worker; the scanning panel shows meanwhile.
    assert!(ed.filefind.scanning());
    for _ in 0..200 {
        ed.animate();
        if !ed.filefind.scanning() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert!(!ed.filefind.open);
    assert_eq!(ed.model.layer_node(0, 2, 0).and_then(|n| n.attr("AudioFileName")), Some(dir.join("found").join("hat.wav").to_string_lossy().as_ref()));
    assert!(ed.take_commands().iter().any(|c| matches!(c, Command::Rebuild { .. })));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn convert_timing_through_messages() {
    crate::app::test_setup();
    let p = demo("Presets/Hollow.geist");
    let Ok(proj) = dreist_formats::Project::load(&p) else { return };
    let mut m = Model::new(proj.raw);
    let e = m.engine;
    let cur = m.current_pattern(e);
    let want = m.pattern(e, cur).convert_timing(8, dreist_engine::pattern::resolution_index("1/8"));
    let mut out = Vec::new();
    convertor::convert(&m, cur, 8, dreist_engine::pattern::resolution_index("1/8"), &mut out);
    for c in &out {
        m.apply(c);
    }
    let got = m.pattern(e, cur);
    assert_eq!(got.steps, 8);
    for t in 0..16 {
        for s in 0..8 {
            assert_eq!(got.tracks[t].steps[s].on, want.tracks[t].steps[s].on, "t{t} s{s}");
        }
    }
    assert!(want.tracks.iter().any(|t| t.steps.iter().any(|s| s.on)));
}

#[test]
fn browser_search_box_menu_and_advanced_search() {
    crate::app::test_setup();
    use dreist_gui::app::UiEvent;
    let p = demo("Presets/Hollow.geist");
    let Ok(proj) = dreist_formats::Project::load(&p) else { return };
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(proj.raw)) else { return };
    let Some(page) = ed.browser.as_ref().map(|b| b.page().to_vec()) else { return };
    let find = |ed: &Editor, name: &str| (ed.app.ctrl(&page).unwrap()).path_of(name).map(|p| [page.clone(), p].concat()).unwrap();
    let sb = find(&ed, "SearchBox");
    let nav_y = ed.app.ctrl(&find(&ed, "Navigator")).unwrap().rect.y;
    // Search box menu → Advanced search... opens the pane.
    ed.handle(&UiEvent::Pressed { path: sb.clone(), name: "SearchBox".into() });
    ed.handle(&UiEvent::MenuSelected { anchor: sb.clone(), name: "SearchBox".into(), index: 2 });
    let search = find(&ed, "Search");
    assert!(ed.app.ctrl(&search).unwrap().visible);
    // 750 ms animation block.
    std::thread::sleep(std::time::Duration::from_millis(770));
    ed.animate();
    assert_eq!(ed.app.ctrl(&find(&ed, "Navigator")).unwrap().rect.y, nav_y + search::PANE_HEIGHT);
    assert_eq!(ed.app.ctrl(&search).unwrap().rect.h, search::PANE_HEIGHT);
    // Sub folders on.
    let was = ed.browser.as_ref().unwrap().pane_sub_folders();
    ed.handle(&UiEvent::Pressed { path: sb.clone(), name: "SearchBox".into() });
    ed.handle(&UiEvent::MenuSelected { anchor: sb.clone(), name: "SearchBox".into(), index: 0 });
    assert_ne!(ed.browser.as_ref().unwrap().pane_sub_folders(), was);
    if !ed.browser.as_ref().unwrap().pane_sub_folders() {
        ed.handle(&UiEvent::Pressed { path: sb.clone(), name: "SearchBox".into() });
        ed.handle(&UiEvent::MenuSelected { anchor: sb.clone(), name: "SearchBox".into(), index: 0 });
    }
    // Option 1: File name contains "hollow".
    let opt1 = find(&ed, "Option1");
    let term = (ed.app.ctrl(&opt1).unwrap()).path_of("SearchTermBox").map(|p| [opt1.clone(), p].concat()).unwrap();
    ed.handle(&UiEvent::TextCommitted { path: term, name: "SearchTermBox".into(), text: "hollow".into() });
    // Option 2: File type Folder, matched with "any".
    let opt2 = find(&ed, "Option2");
    let ty = (ed.app.ctrl(&opt2).unwrap()).path_of("SearchType").map(|p| [opt2.clone(), p].concat()).unwrap();
    ed.handle(&UiEvent::Clicked { path: ty.clone(), name: "SearchType".into(), id: 0, on: false });
    ed.handle(&UiEvent::MenuSelected { anchor: ty.clone(), name: "SearchType".into(), index: 3 });
    let sub = (ed.app.ctrl(&opt2).unwrap()).path_of("SubSearchType").map(|p| [opt2.clone(), p].concat()).unwrap();
    ed.handle(&UiEvent::Clicked { path: sub.clone(), name: "SubSearchType".into(), id: 0, on: false });
    ed.handle(&UiEvent::MenuSelected { anchor: sub.clone(), name: "SubSearchType".into(), index: 3 });
    assert_eq!(ed.app.ctrl(&sub).unwrap().text, "Folder");
    let go = find(&ed, "DoSearchButton");
    // Search from the content root.
    ed.browser.as_mut().unwrap().browse(browser::content_dir());
    ed.handle(&UiEvent::Clicked { path: go.clone(), name: "DoSearchButton".into(), id: 0, on: false });
    let hits = ed.browser.as_ref().unwrap().entry_paths();
    assert!(hits.iter().any(|p| p.file_name().is_some_and(|n| n == "Hollow.geist")));
    assert!(hits.iter().any(|p| p.is_dir()));
    // Match all: only folders named like "hollow".
    let all = find(&ed, "MatchAllButton");
    ed.handle(&UiEvent::Clicked { path: all, name: "MatchAllButton".into(), id: 0, on: true });
    ed.handle(&UiEvent::Clicked { path: go, name: "DoSearchButton".into(), id: 0, on: false });
    let hits = ed.browser.as_ref().unwrap().entry_paths();
    assert!(hits.iter().all(|p| p.is_dir()));
    // Recent items from a quick search.
    ed.handle(&UiEvent::TextCommitted { path: sb.clone(), name: "SearchBox".into(), text: "Blister".into() });
    assert!(ed.browser.as_ref().unwrap().entry_paths().iter().any(|p| p.file_name().is_some_and(|n| n == "Blister.geist")));
    ed.handle(&UiEvent::Pressed { path: sb.clone(), name: "SearchBox".into() });
    ed.handle(&UiEvent::MenuSelected { anchor: sb.clone(), name: "SearchBox".into(), index: 4 });
    ed.handle(&UiEvent::MenuSelected { anchor: sb.clone(), name: "SearchBox".into(), index: 0 });
    assert!(ed.browser.as_ref().unwrap().entry_paths().iter().any(|p| p.file_name().is_some_and(|n| n == "Blister.geist")));
    // Leave the saved option as it was.
    if !was {
        ed.handle(&UiEvent::Pressed { path: sb.clone(), name: "SearchBox".into() });
        ed.handle(&UiEvent::MenuSelected { anchor: sb, name: "SearchBox".into(), index: 0 });
    }
}

/// A graph stroke (press, drags across steps, release) is one undo
/// step, and undoing it restores every step.
#[test]
fn graph_stroke_is_one_undo_step() {
    crate::app::test_setup();
    use dreist_gui::app::MouseKind;
    let Some(mut ed) = demo_editor("Presets/Hollow.geist") else { return };
    ed.sync();
    let gp = ed.app.root.path_of("PatternGraph").unwrap();
    let h = ed.app.ctrl(&gp).unwrap().rect.h as f32;
    let lay = ed.pattern.as_ref().unwrap().layout(&ed.app, &ed.model).unwrap();
    let sx = |st: usize| (st - lay.first_step) as f32 * lay.step_w + lay.step_w / 2.0;
    let ev = |kind, st: usize, y: f32| UiEvent::Mouse { path: gp.clone(), overload: "PatternGraph".into(), kind, x: sx(st), y };
    let f = lay.first_step;
    let before = ed.model.doc.clone();
    let steps = ed.history.undo.len();
    ed.handle(&ev(MouseKind::Down, f, h * 0.9));
    for k in 1..6 {
        ed.handle(&ev(MouseKind::Drag, f + k, h * 0.1));
    }
    ed.handle(&ev(MouseKind::Up, f + 5, h * 0.1));
    assert_ne!(ed.model.doc, before, "the stroke edited the pattern");
    assert_eq!(ed.history.undo.len(), steps + 1);
    let undo = ed.app.root.path_of("UndoButton").unwrap();
    ed.handle(&UiEvent::Clicked { path: undo, name: "UndoButton".into(), id: 0, on: false });
    assert!(ed.model.doc == before, "one undo restores the whole stroke");
}

#[test]
fn graph_line_fine_erase_and_menu_from_step() {
    crate::app::test_setup();
    use dreist_gui::app::MouseKind;
    let Some(mut ed) = demo_editor("Presets/Hollow.geist") else { return };
    if let Some(pp) = ed.pattern.as_mut() {
        pp.multi = false;
    }
    ed.sync();
    let gp = ed.app.root.path_of("PatternGraph").unwrap();
    let h = ed.app.ctrl(&gp).unwrap().rect.h as f32;
    let lay = ed.pattern.as_ref().unwrap().layout(&ed.app, &ed.model).unwrap();
    let sx = |st: usize| (st - lay.first_step) as f32 * lay.step_w + lay.step_w / 2.0;
    let ev = |kind, st: usize, y: f32| UiEvent::Mouse { path: gp.clone(), overload: "PatternGraph".into(), kind, x: sx(st), y };
    let e = ed.model.engine;
    let vel = |ed: &Editor, st: usize| {
        let pat = ed.model.pattern(e, ed.model.current_pattern(e));
        pat.tracks[ed.model.pad[e]].steps[st].lanes[0]
    };
    let f = lay.first_step;
    // Option-drag: a line from 1.0 at the first step to 0.0 four steps on.
    ed.app.mods.alt = true;
    ed.handle(&ev(MouseKind::Down, f, 0.0));
    ed.handle(&ev(MouseKind::Drag, f + 4, h));
    ed.handle(&ev(MouseKind::Up, f + 4, h));
    ed.app.mods.alt = false;
    assert!((vel(&ed, f) - 1.0).abs() < 0.02 && (vel(&ed, f + 2) - 0.5).abs() < 0.02 && vel(&ed, f + 4) < 0.02, "line");
    // Shift: fine adjustment, 0.001 per pixel.
    ed.handle(&ev(MouseKind::Down, f + 6, h * 0.5));
    ed.app.mods.shift = true;
    ed.handle(&ev(MouseKind::Drag, f + 6, h * 0.5 - 100.0));
    ed.app.mods.shift = false;
    ed.handle(&ev(MouseKind::Up, f + 6, h * 0.5 - 100.0));
    assert!((vel(&ed, f + 6) - 0.6).abs() < 0.02, "fine {}", vel(&ed, f + 6));
    // Right-drag erases to the default.
    let def = dreist_formats::params::step_lane_default(0);
    ed.handle(&ev(MouseKind::RightDown, f, h));
    ed.handle(&ev(MouseKind::Drag, f + 2, h));
    ed.handle(&ev(MouseKind::Up, f + 2, h));
    assert!((vel(&ed, f + 1) - def).abs() < 1e-4);
    // Option-right: the menu, Insert Ramp Up from that step.
    ed.app.mods.alt = true;
    ed.handle(&ev(MouseKind::RightDown, f + 8, h));
    ed.app.mods.alt = false;
    ed.handle(&UiEvent::MenuSelected { anchor: gp.clone(), name: "PatternGraph".into(), index: 17 });
    ed.handle(&UiEvent::MenuSelected { anchor: gp.clone(), name: "PatternGraph".into(), index: 1 });
    assert!(vel(&ed, f + 8) < 0.01, "ramp starts at the clicked step");
    assert!((vel(&ed, f + 6) - 0.6).abs() < 0.02, "steps before it untouched");
}

#[test]
fn graph_edit_on_a_selected_step_offsets_the_selection() {
    crate::app::test_setup();
    use dreist_gui::app::MouseKind;
    let Some(mut ed) = demo_editor("Presets/Hollow.geist") else { return };
    if let Some(pp) = ed.pattern.as_mut() {
        pp.multi = false;
    }
    ed.sync();
    let gp = ed.app.root.path_of("PatternGraph").unwrap();
    let h = ed.app.ctrl(&gp).unwrap().rect.h as f32;
    let lay = ed.pattern.as_ref().unwrap().layout(&ed.app, &ed.model).unwrap();
    let sx = |st: usize| (st - lay.first_step) as f32 * lay.step_w + lay.step_w / 2.0;
    let e = ed.model.engine;
    let t = ed.model.pad[e];
    let other = (t + 1) % 16;
    let f = lay.first_step;
    let vel = |ed: &Editor, tr: usize, st: usize| ed.model.pattern(e, ed.model.current_pattern(e)).tracks[tr].steps[st].lanes[0];
    let before = [vel(&ed, t, f), vel(&ed, t, f + 1), vel(&ed, t, f + 2), vel(&ed, other, f)];
    ed.pattern.as_mut().unwrap().selection = [(t, f), (t, f + 2), (other, f)].into_iter().collect();
    ed.handle(&UiEvent::Mouse { path: gp.clone(), overload: "PatternGraph".into(), kind: MouseKind::Down, x: sx(f), y: h * 0.75 });
    ed.handle(&UiEvent::Mouse { path: gp, overload: "PatternGraph".into(), kind: MouseKind::Up, x: sx(f), y: h * 0.75 });
    let d = 0.25 - before[0];
    assert!((vel(&ed, t, f) - 0.25).abs() < 0.02);
    assert!((vel(&ed, t, f + 2) - (before[2] + d).clamp(0.0, 1.0)).abs() < 0.02, "selected step moved by the delta");
    assert!((vel(&ed, other, f) - (before[3] + d).clamp(0.0, 1.0)).abs() < 0.02, "selection on another track too");
    assert_eq!(vel(&ed, t, f + 1), before[1], "unselected step untouched");
}

#[test]
fn midi_learn_of_a_graph_lane() {
    crate::app::test_setup();
    let Some(mut ed) = demo_editor("Presets/Hollow.geist") else { return };
    if let Some(pp) = ed.pattern.as_mut() {
        pp.multi = false;
    }
    ed.sync();
    ed.learn.active = true;
    let ac = ed.app.root.path_of("SingleAutoControl").unwrap();
    let knob = [ac.clone(), (ed.app.ctrl(&ac).unwrap()).path_of("ScaleKnob").unwrap()].concat();
    ed.handle(&UiEvent::Pressed { path: knob, name: "ScaleKnob".into() });
    let t = ed.learn.target.clone().expect("target");
    assert_eq!(t.proto.kind, dreist_engine::binding::id::SET_INSTO_MIDI_GRAPH_VALUE);
    let e = ed.model.engine;
    assert_eq!(t.proto.index[4], ed.model.pad[e] as i32);
    assert_eq!(t.proto.value[1], ed.model.pattern_view[e].lane as f32);
    assert!(t.group.starts_with(&format!("Engine {} | Pattern", e + 1)), "{}", t.group);
    ed.midi_cc(1, 0, 20);
    let rows = crate::midimap::rows(&ed.model.doc);
    assert!(rows.iter().any(|r| r.locale() == "MIDI Ch 1 CC 20"), "{rows:?}");
}

#[test]
fn fx_chain_preset_menu_loads_and_steps() {
    crate::app::test_setup();
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    ed.model.editor_tab = front::GLOBAL_MIXER_PAGE;
    ed.sync();
    fn all(c: &Ctrl, name: &str, path: &mut Vec<usize>, out: &mut Vec<Vec<usize>>) {
        if c.name == name && c.visible {
            out.push(path.clone());
        }
        for (i, k) in c.children.iter().enumerate() {
            path.push(i);
            all(k, name, path, out);
            path.pop();
        }
    }
    let mut combos = Vec::new();
    all(&ed.app.root, "PresetCombo", &mut Vec::new(), &mut combos);
    let pc = combos.first().cloned().expect("chain PresetCombo on the global mixer");
    let res = dreist_formats::paths::skin_dir();
    if fxchain::chain_presets(&res).is_empty() {
        return;
    }
    let chain_of = |ed: &Editor| {
        fn find(n: &dreist_formats::Node) -> Option<dreist_formats::Node> {
            if n.class == "EffectChain" && fxchain::chain_name(n).is_some() {
                return Some(n.clone());
            }
            n.children.iter().find_map(find)
        }
        find(&ed.model.doc)
    };
    // Next with no current preset loads the first one.
    let mut nexts = Vec::new();
    all(&ed.app.root, "PresetNext", &mut Vec::new(), &mut nexts);
    ed.handle(&UiEvent::Clicked { path: nexts[0].clone(), name: "PresetNext".into(), id: 0, on: false });
    let first = chain_of(&ed).and_then(|c| fxchain::chain_name(&c)).expect("a preset loaded");
    ed.handle(&UiEvent::Clicked { path: nexts[0].clone(), name: "PresetNext".into(), id: 0, on: false });
    let second = chain_of(&ed).and_then(|c| fxchain::chain_name(&c)).unwrap();
    assert_ne!(first, second);
    // Clear chain from the menu.
    ed.handle(&UiEvent::Clicked { path: pc.clone(), name: "PresetCombo".into(), id: 0, on: false });
    ed.handle(&UiEvent::MenuSelected { anchor: pc.clone(), name: "PresetCombo".into(), index: 1 });
    assert!(chain_of(&ed).is_none(), "cleared");
    // The chain header's Active button: 0x60, and prev / next disable.
    assert!(ed.app.ctrl(&nexts[0]).unwrap().enabled);
    let host = &nexts[0][..nexts[0].len() - 1];
    let active = (0..ed.app.ctrl(host).unwrap().children.len()).map(|i| [host, &[i]].concat()).find(|p| ed.app.ctrl(p).unwrap().name == "ActiveButton").expect("chain ActiveButton beside PresetNext");
    ed.handle(&UiEvent::Clicked { path: active.clone(), name: "ActiveButton".into(), id: 0, on: false });
    let cmds = ed.take_commands();
    assert!(cmds.iter().any(|c| matches!(c, Command::Message(m) if m.kind == dreist_engine::binding::id::SET_FX_CHAIN_ACTIVE && m.value[0] == 0.0)), "{cmds:?}");
    ed.sync();
    assert!(!ed.app.ctrl(&nexts[0]).unwrap().enabled, "prev / next follow the chain's active state");
    assert!(!ed.app.ctrl(&active).unwrap().on);
}

#[test]
fn slicer_slides_over_the_lcd() {
    crate::app::test_setup();
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    let f = dreist_formats::legacy::demo("Hits/Cymbals/CY Cym Ar82 01 C2A St GB.wav");
    if !ed.open_slicer(&f) {
        return;
    }
    // The preview panel covers the LCD's page area.
    let ov = ed.app.root.path_of("SlicerOverlay").unwrap();
    let main = ed.app.root.path_of("MainTabs").unwrap();
    let (mx, my) = ed.app.origin(&main);
    let m = ed.app.ctrl(&main).unwrap().rect;
    assert_eq!(ed.app.ctrl(&ov).unwrap().rect, dreist_gui::Rect::new(mx, my, m.w, m.h));
    std::thread::sleep(std::time::Duration::from_millis(370));
    ed.animate();
    let panel = [ov.clone(), vec![0]].concat();
    assert_eq!(ed.app.ctrl(&panel).unwrap().rect.y, 0);
    assert!(ed.app.ctrl(&panel).unwrap().find("WaveDisplay").is_some_and(|w| w.overload == "EditorPreviewWaveEditor"));
    if let Ok(f) = std::env::var("DREIST_SHOT") {
        ed.render(1.0).save_png(f).unwrap();
    }
    // The other pages are off while slicing.
    let tabs = |ed: &Editor| ed.app.root.find("MainTabButtons").unwrap().children.iter().all(|b| b.enabled);
    assert!(!tabs(&ed));
    let at = |ed: &Editor, n: &str| [panel.clone(), ed.app.ctrl(&panel).unwrap().path_of(n).unwrap()].concat();
    // Divide: the division count replaces the sensitivity.
    let group = [at(&ed, "SliceModeButtonGroup"), vec![1]].concat();
    ed.handle(&UiEvent::Clicked { path: group, name: "CtrlButton".into(), id: 0, on: true });
    assert_eq!(ed.slicer.mode, 1);
    assert_eq!(ed.slicer.slices.len(), 16);
    let shown = |ed: &Editor, n: &str| ed.app.ctrl(&panel).unwrap().find(n).unwrap().visible;
    assert!(shown(&ed, "ChopDivisionBox") && !shown(&ed, "SensitivitySlider"));
    ed.handle(&UiEvent::TextCommitted { path: at(&ed, "ChopDivisionBox"), name: "ChopDivisionBox".into(), text: "8".into() });
    assert_eq!(ed.slicer.slices.len(), 8);
    // Without Auto load the Extract buttons send the slices to the pads.
    assert!(shown(&ed, "AudioButton"));
    ed.handle(&UiEvent::Clicked { path: at(&ed, "AudioButton"), name: "AudioButton".into(), id: 0, on: false });
    assert!(ed.model.layer(0, 1, 0).valid && !ed.model.layer(0, 8, 0).valid, "slices 1-8 on pads 1-8");
    // Cancel restores the document and the page tabs.
    ed.handle(&UiEvent::Clicked { path: at(&ed, "CancelButton"), name: "CancelButton".into(), id: 0, on: false });
    assert!(!ed.slicer.open && !ed.model.layer(0, 1, 0).valid);
    assert!(tabs(&ed));
}

#[test]
fn modulated_knob_centre_edits_the_base_and_ring_the_depth() {
    crate::app::test_setup();
    use dreist_gui::app::MouseButton;
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    ed.model.source = 5;
    ed.sync();
    let Some(k) = ed.app.root.path_of("VoicePan") else { return };
    assert_eq!(ed.app.ctrl(&k).unwrap().attr("editingDepth"), Some("true"));
    let (ox, oy) = ed.app.origin(&k);
    let r = ed.app.ctrl(&k).unwrap().rect;
    let drag = |ed: &mut Editor, x: f32, y: f32| {
        ed.take_commands();
        for e in ed.app.mouse_move(x, y) {
            ed.handle(&e);
        }
        for e in ed.app.mouse_down(MouseButton::Left) {
            ed.handle(&e);
        }
        for e in ed.app.mouse_move(x, y - 40.0) {
            ed.handle(&e);
        }
        for e in ed.app.mouse_up(MouseButton::Left) {
            ed.handle(&e);
        }
        ed.take_commands().into_iter().filter_map(|c| if let Command::Message(m) = c { Some(m) } else { None }).rfind(|m| m.kind == dreist_engine::binding::id::SET_LAYER_PARAMETER)
    };
    // Centre: the base (source 0).
    let m = drag(&mut ed, (ox + r.w / 2) as f32, (oy + r.h / 2) as f32).expect("base edit");
    assert_eq!(m.value[2], 0.0);
    // Corner (outside the centre circle): the depth for source 5.
    let m = drag(&mut ed, ox as f32 + 1.0, oy as f32 + 1.0).expect("depth edit");
    assert_eq!(m.value[2], 5.0);
}

#[test]
fn modulator_retrigger_flashes_its_source_button() {
    crate::app::test_setup();
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    let Some(f) = ed.app.root.path_of("FlashSrc5") else { return };
    assert_eq!(ed.app.ctrl(&f).unwrap().attr("alpha"), Some("0"));
    ed.modulators_triggered(1 << 5);
    ed.animate();
    let a: f32 = ed.app.ctrl(&f).unwrap().attr("alpha").unwrap().parse().unwrap();
    assert!(a > 0.9, "{a}");
    std::thread::sleep(std::time::Duration::from_millis(520));
    ed.animate();
    assert_eq!(ed.app.ctrl(&f).unwrap().attr_f32("alpha", 1.0), 0.0);
}

#[test]
fn dialogs_show_at_once_centred_and_titled() {
    crate::app::test_setup();
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    let Some(ov) = ed.app.root.path_of("PreferencesOverlay") else { return };
    let dialog = [ov.clone(), vec![0]].concat();
    ed.show_modal("preferences");
    assert!(ed.app.ctrl(&ov).unwrap().visible);
    let (w, h) = (ed.app.ctrl(&ov).unwrap().rect.w, ed.app.ctrl(&ov).unwrap().rect.h);
    let r = ed.app.ctrl(&dialog).unwrap().rect;
    assert_eq!((r.x, r.y), (w / 2 - r.w / 2, h / 2 - r.h / 2), "centred, no slide");
    // The title frame: 10 px wider, 35 px taller, the panel at (5, 29).
    let d = ed.app.ctrl(&dialog).unwrap();
    assert_eq!(d.style, "ModalPanelBackgroundStyle");
    let (panel, title) = (&d.children[0], &d.children[1]);
    assert_eq!((d.rect.w - panel.rect.w, d.rect.h - panel.rect.h), (10, 35));
    assert_eq!(title.text, format!("{} Preferences", crate::product()));
    // v1's preferences close with Finished (there is no Cancel), at once.
    let confirm = (ed.app.ctrl(&ov).unwrap()).path_of("ConfirmButton").map(|p| [ov.clone(), p].concat()).expect("confirm");
    ed.handle(&UiEvent::Clicked { path: confirm, name: "ConfirmButton".into(), id: 0, on: false });
    assert!(!ed.app.ctrl(&ov).unwrap().visible);
}

#[test]
fn marker_buttons_only_in_learn_mode() {
    crate::app::test_setup();
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    ed.sync();
    let Some(ps) = ed.app.root.path_of("PS") else { return };
    assert!(!ed.app.ctrl(&ps).unwrap().visible, "hidden outside learn");
    ed.set_learn(true);
    assert!(ed.app.ctrl(&ps).unwrap().visible);
    ed.handle(&UiEvent::Pressed { path: ps.clone(), name: "PS".into() });
    let t = ed.learn.target.clone().expect("learn target");
    assert_eq!(t.proto.kind, dreist_engine::binding::id::SET_LAYER_START_SAMPLE01);
    assert_eq!(t.name, "Play Start");
    assert!(t.group.starts_with("Engine 1 | Pad 1 | Layer 1"), "{}", t.group);
    ed.set_learn(false);
    assert!(!ed.app.ctrl(&ps).unwrap().visible);
}

#[test]
fn effect_chain_shows_four_of_six_slots_and_scrolls() {
    crate::app::test_setup();
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    ed.model.editor_tab = front::GLOBAL_MIXER_PAGE;
    ed.sync();
    let page = ed.mixers.iter().find(|m| m.kind == mixer::Kind::Global).unwrap().page.clone();
    let row = [page.clone(), ed.app.ctrl(&page).unwrap().path_of("ChainPanelScroll").expect("slot row")].concat();
    let view = row[..row.len() - 1].to_vec();
    let slots = ed.app.ctrl(&row).unwrap().children.len();
    assert_eq!(slots, fxchain::SLOTS);
    let slot_w = ed.app.ctrl(&row).unwrap().children[0].rect.w;
    assert_eq!(ed.app.ctrl(&view).unwrap().rect.w / slot_w, fxchain::VISIBLE_SLOTS as i32);
    assert!(ed.app.ctrl(&row).unwrap().children.iter().all(|s| s.find("EffectTypeCombo").is_some_and(|c| c.text == "No Effect")));
    // The wheel scrolls by a slot; the slot bar reveals the slot pressed.
    let slot0 = [row.clone(), vec![0]].concat();
    ed.handle(&UiEvent::Wheel { path: slot0, overload: "EditorEffectPanel".into(), dy: -1.0 });
    assert_eq!(ed.app.ctrl(&row).unwrap().rect.x, -slot_w);
    let bar = [page.clone(), ed.app.ctrl(&page).unwrap().path_of("ScrollBar").expect("slot bar")].concat();
    let w = ed.app.ctrl(&bar).unwrap().rect.w as f32;
    ed.handle(&UiEvent::Mouse { path: bar.clone(), overload: "EditorEffectPanelScrollbar".into(), kind: dreist_gui::app::MouseKind::Down, x: w - 2.0, y: 5.0 });
    assert_eq!(ed.app.ctrl(&row).unwrap().rect.x, -2 * slot_w, "slots 3–6 shown");
    ed.handle(&UiEvent::Mouse { path: bar, overload: "EditorEffectPanelScrollbar".into(), kind: dreist_gui::app::MouseKind::Down, x: 2.0, y: 5.0 });
    assert_eq!(ed.app.ctrl(&row).unwrap().rect.x, 0);
    // An empty slot's type menu offers the categories; a pick fills that slot.
    let combo = [row.clone(), vec![5], ed.app.ctrl(&[row.clone(), vec![5]].concat()).unwrap().path_of("EffectTypeCombo").unwrap()].concat();
    ed.handle(&UiEvent::Clicked { path: combo.clone(), name: "EffectTypeCombo".into(), id: 0, on: false });
    let items = ed.app.menu.as_ref().map(|m| m.items.clone()).unwrap_or_default();
    assert_eq!(items.len(), fxchain::CATEGORIES.len(), "{items:?}");
    ed.app.menu = None;
    ed.handle(&UiEvent::MenuSelected { anchor: combo.clone(), name: "EffectTypeCombo".into(), index: 4 });
    ed.app.menu = None;
    ed.handle(&UiEvent::MenuSelected { anchor: combo.clone(), name: "EffectTypeCombo".into(), index: 0 });
    let fx = fxchain::effects(&ed.model, fxchain::Target::Aux(0));
    assert_eq!(fx.iter().map(|(s, _, _)| *s).collect::<Vec<_>>(), vec![5], "the effect lands in slot 6");
    ed.sync();
    let kind = fx[0].1;
    assert_eq!(ed.app.ctrl(&combo).unwrap().text, dreist_formats::params::effect_label(kind));
    assert_eq!(ed.app.ctrl(&[row, vec![5]].concat()).unwrap().attr("fxKind"), Some(kind.to_string().as_str()));
}

/// v1's 64 scene slots: 16 on view, scrolled by page / row (and the view
/// control), the slots showing their scenes' key names; Clear All clears
/// all 64; Snapshot next takes the next free slot and scrolls to it.
#[test]
fn scene_page_scrolls_through_64_slots() {
    crate::app::test_setup();
    use dreist_engine::binding::id;
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    ed.model.editor_tab = front::SCENE_PAGE;
    ed.sync();
    let m = ed.app.root.path_of("ScenesPanel").unwrap();
    let slot = |ed: &Editor, k: usize| ed.app.ctrl(&m).unwrap().find(&format!("Scene{k}")).unwrap().clone();
    let key = |ed: &Editor, k: usize| slot(ed, k).find("SelectButton").unwrap().text.clone();
    assert_eq!(ed.app.ctrl(&m).unwrap().children.iter().filter(|c| c.overload == "EditorScene").count(), 16);
    // Scene0 is bottom left, Scene15 top right.
    assert!(slot(&ed, 0).rect.y > slot(&ed, 12).rect.y && slot(&ed, 0).rect.x < slot(&ed, 3).rect.x);
    assert_eq!((key(&ed, 0), key(&ed, 15)), ("C-2".to_string(), "D#-1".to_string()));
    let click = |ed: &mut Editor, name: &str| {
        let p = ed.app.root.path_of(name).unwrap();
        ed.handle(&UiEvent::Clicked { path: p, name: name.into(), id: 0, on: false });
    };
    let base = |ed: &Editor| ed.scenes.as_ref().unwrap().base();
    let enabled = |ed: &Editor, n: &str| ed.app.root.find(n).unwrap().enabled;
    assert!(!enabled(&ed, "PageDownButton") && enabled(&ed, "PageUpButton"));
    click(&mut ed, "RowUpButton");
    assert_eq!(base(&ed), 1);
    assert_eq!(key(&ed, 0), "E-2");
    click(&mut ed, "PageUpButton");
    click(&mut ed, "PageUpButton");
    click(&mut ed, "PageUpButton");
    assert_eq!(base(&ed), 12, "clamped at the last page");
    assert_eq!((key(&ed, 0), key(&ed, 15)), ("C2".to_string(), "D#3".to_string()));
    assert!(!enabled(&ed, "PageUpButton") && enabled(&ed, "RowDownButton"));
    // The slots act on their shown scenes.
    let s15 = slot(&ed, 15);
    assert_eq!(s15.attr("scene"), Some("63"));
    // The view control: a press on its bottom shows the first rows.
    let v = ed.app.root.path_of("ViewSlider").unwrap();
    let h = ed.app.ctrl(&v).unwrap().rect.h as f32;
    ed.handle(&UiEvent::Mouse { path: v.clone(), overload: "SceneViewSlider".into(), kind: dreist_gui::app::MouseKind::Down, x: 10.0, y: h - 4.0 });
    ed.handle(&UiEvent::Mouse { path: v, overload: "SceneViewSlider".into(), kind: dreist_gui::app::MouseKind::Up, x: 10.0, y: h - 4.0 });
    assert_eq!(base(&ed), 0);
    ed.take_commands();
    click(&mut ed, "ClearAllButton");
    let cleared = ed.take_commands().iter().filter(|c| matches!(c, Command::Message(m) if m.kind == id::SCENE_CLEAR)).count();
    assert_eq!(cleared, 64);
    // Snapshot next: the next free slot, revealed.
    let mgr = ed.model.doc.child_mut_or_insert("SceneManager", "Scenes");
    for n in 0..20 {
        mgr.child_mut_or_insert("Scene", &format!("Scene{n}")).set("used", "true");
    }
    click(&mut ed, "SnapshotToNextButton");
    assert!(ed.take_commands().iter().any(|c| matches!(c, Command::Message(m) if m.kind == id::SCENE_SNAPSHOT && m.value[0] == 20.0)));
    assert_eq!(base(&ed), 2, "scene 21 scrolled into view");
    // The name box names the selected scene; the slot shows it.
    let nb = ed.app.root.path_of("NameTextBox").unwrap();
    ed.handle(&UiEvent::TextCommitted { path: nb, name: "NameTextBox".into(), text: "Drop".into() });
    assert_eq!(ed.model.scene_name(20), "Drop");
    if let Ok(f) = std::env::var("DREIST_SCENE_SHOT") {
        ed.render(1.0).save_png(f).unwrap();
    }
}

#[test]
fn song_pattern_buttons_change_the_selection() {
    crate::app::test_setup();
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    ed.model.editor_tab = 2;
    ed.sync();
    song_mode_on(&mut ed);
    let body = ed.app.root.path_of("SongBody").unwrap();
    let (bx, by) = ed.app.origin(&body);
    let click = |ed: &mut Editor, x: f32, y: f32| {
        for ev in ed.app.mouse_move(x, y) {
            ed.handle(&ev);
        }
        for ev in ed.app.mouse_down(dreist_gui::app::MouseButton::Left) {
            ed.handle(&ev);
        }
        for ev in ed.app.mouse_up(dreist_gui::app::MouseButton::Left) {
            ed.handle(&ev);
        }
    };
    let song = |ed: &Editor| ed.model.doc.children.iter().find(|c| c.class == "SongSequencer").map(dreist_engine::song::Song::from_node).unwrap();
    let next = ed.app.root.path_of("NextPatternButton").unwrap();
    assert!(!ed.app.ctrl(&next).unwrap().enabled, "disabled without a selection");
    // Insert a part (selected), then step its pattern.
    click(&mut ed, bx as f32 + 5.0, by as f32 + 30.0);
    assert!(ed.app.ctrl(&next).unwrap().enabled);
    ed.handle(&UiEvent::Clicked { path: next.clone(), name: "NextPatternButton".into(), id: 0, on: false });
    assert_eq!(song(&ed).tracks[0][0].pattern, 1);
    let prev = ed.app.root.path_of("PrevPatternButton").unwrap();
    for _ in 0..3 {
        ed.handle(&UiEvent::Clicked { path: prev.clone(), name: "PrevPatternButton".into(), id: 0, on: false });
    }
    assert_eq!(song(&ed).tracks[0][0].pattern, 0, "clamped at 0");
    let sel = ed.app.root.path_of("SelectPatternButton").unwrap();
    ed.handle(&UiEvent::Clicked { path: sel.clone(), name: "SelectPatternButton".into(), id: 0, on: false });
    // v1's popup: 24 pattern keys, the part's pattern lit.
    let popup = ed.app.root.path_of("SongPatternPopup").expect("pattern popup");
    let key = |ed: &Editor, n: usize| [popup.clone(), ed.app.ctrl(&popup).unwrap().path_of(&format!("Key{n}")).unwrap()].concat();
    assert_eq!(ed.app.ctrl(&popup).unwrap().children.iter().filter(|c| c.name.starts_with("Key")).count(), 24);
    assert!(ed.app.ctrl(&key(&ed, 0)).unwrap().on);
    let k5 = key(&ed, 5);
    ed.handle(&UiEvent::Clicked { path: k5, name: "Key5".into(), id: 0, on: true });
    assert!(ed.app.root.path_of("SongPatternPopup").is_none(), "a pick closes the popup");
    assert_eq!(song(&ed).tracks[0][0].pattern, 5);
    // Click the part: its part buttons appear inside it.
    click(&mut ed, bx as f32 + 5.0, by as f32 + 30.0);
    ed.sync();
    let pn = ed.app.root.path_of("PartNextPatternButton").unwrap();
    assert!(ed.app.ctrl(&pn).unwrap().visible);
    ed.handle(&UiEvent::Clicked { path: pn, name: "PartNextPatternButton".into(), id: 0, on: false });
    assert_eq!(song(&ed).tracks[0][0].pattern, 6);
    // Erase: press / release post e_songSetEraseState.
    // The song panel's EraseButton (the pads panel has one too).
    let dup = ed.app.root.path_of("DuplicateButton").unwrap();
    let er = (1..dup.len()).rev().find_map(|n| (ed.app.ctrl(&dup[..n]).unwrap()).path_of("EraseButton").map(|p| [dup[..n].to_vec(), p].concat())).unwrap();
    let _ = ed.take_commands();
    ed.handle(&UiEvent::Pressed { path: er.clone(), name: "EraseButton".into() });
    ed.handle(&UiEvent::Clicked { path: er, name: "EraseButton".into(), id: 0, on: false });
    let cmds = ed.take_commands();
    let erase: Vec<f32> = cmds.iter().filter_map(|c| if let Command::Message(m) = c { (m.kind == dreist_engine::binding::id::SONG_SET_ERASE_STATE).then_some(m.value[0]) } else { None }).collect();
    assert_eq!(erase, vec![1.0, 0.0]);
}

#[test]
fn drag_and_drop_pads_patterns_and_browser_files() {
    crate::app::test_setup();
    use dreist_gui::app::{Mods, MouseKind};
    let p = demo("Presets/Hollow.geist");
    let Ok(proj) = dreist_formats::Project::load(&p) else { return };
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(proj.raw)) else { return };
    fn find(c: &dreist_gui::Ctrl, f: &dyn Fn(&dreist_gui::Ctrl) -> bool, path: &mut Vec<usize>) -> bool {
        for (i, k) in c.children.iter().enumerate() {
            path.push(i);
            if f(k) || find(k, f, path) {
                return true;
            }
            path.pop();
        }
        false
    }
    let by = |ed: &Editor, f: &dyn Fn(&dreist_gui::Ctrl) -> bool| {
        let mut v = Vec::new();
        assert!(find(&ed.app.root, f, &mut v));
        v
    };
    let pad = |ed: &Editor, i: usize| by(ed, &|k| k.overload == "EditorPad" && k.attr("padIndex") == Some(&i.to_string()));
    let e = ed.model.engine;
    let sample = |ed: &Editor, p: usize, l: usize| ed.model.layer_node(e, p, l).and_then(|n| n.attr("AudioFileName")).map(std::path::PathBuf::from);
    let name = |ed: &Editor, p: usize| ed.model.pad_node(e, p).map(|n| n.to_xml());
    let (n1, n2) = (name(&ed, 1), name(&ed, 2));
    // Plain drop swaps; the target becomes the selected pad.
    let (a, b) = (pad(&ed, 1), pad(&ed, 2));
    ed.handle(&UiEvent::Dropped { from: a.clone(), to: b.clone(), x: 0.0, y: 0.0 });
    assert_eq!(ed.model.pad[e], 2);
    assert_eq!(name(&ed, 1).map(|s| s.replace("Pad2", "Pad1")), n2.clone().map(|s| s.replace("Pad2", "Pad1")));
    assert_eq!(name(&ed, 2).map(|s| s.replace("Pad1", "Pad2")), n1.clone().map(|s| s.replace("Pad1", "Pad2")));
    // Alt copies 2 onto 3.
    ed.app.mods = Mods { alt: true, ..Mods::default() };
    let c = pad(&ed, 3);
    ed.handle(&UiEvent::Dropped { from: b.clone(), to: c.clone(), x: 0.0, y: 0.0 });
    assert_eq!(sample(&ed, 3, 0), sample(&ed, 2, 0));
    assert!(ed.history.undo.back().is_some_and(|u| u.0.starts_with("Copying pad 3 to pad 4")), "{:?}", ed.history.undo.back().map(|u| &u.0));
    ed.app.mods = Mods::default();
    // Pattern keys: swap.
    let key = |ed: &Editor, n: usize| by(ed, &|k| k.overload == "PatternSelector" && k.name == format!("Key{n}"));
    let pat = |ed: &Editor, n: usize| ed.model.pattern_node(e, n).map_or(0, |p| p.children.len());
    let (p0, p5) = (pat(&ed, 0), pat(&ed, 5));
    ed.handle(&UiEvent::Dropped { from: key(&ed, 0), to: key(&ed, 5), x: 0.0, y: 0.0 });
    assert_eq!((pat(&ed, 0), pat(&ed, 5)), (p5, p0));
    assert_eq!(ed.model.current_pattern(e), 5);
    // Browser: cmd-click two audio files, drop on pad 6 → pads 6 and 7.
    let dir = std::env::temp_dir().join(format!("dreist-dnd-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    for f in ["a.wav", "b.wav", "c.wav"] {
        let _ = std::fs::write(dir.join(f), b"");
    }
    ed.browser.as_mut().unwrap().browse(dir.clone());
    let list = by(&ed, &|k| k.overload == "BrowserFileList");
    let down = |ed: &mut Editor, row: f32| ed.handle(&UiEvent::Mouse { path: list.clone(), overload: "BrowserFileList".into(), kind: MouseKind::Down, x: 5.0, y: row * 18.0 + 2.0 });
    down(&mut ed, 0.0);
    ed.app.mods = Mods { cmd: true, ..Mods::default() };
    down(&mut ed, 2.0);
    ed.app.mods = Mods::default();
    assert_eq!(ed.browser.as_ref().unwrap().drag_items(), vec![dir.join("a.wav"), dir.join("c.wav")]);
    // A press on the selection keeps it for the drag.
    down(&mut ed, 2.0);
    ed.handle(&UiEvent::Dropped { from: list.clone(), to: pad(&ed, 6), x: 0.0, y: 0.0 });
    ed.handle(&UiEvent::Mouse { path: list.clone(), overload: "BrowserFileList".into(), kind: MouseKind::Up, x: -50.0, y: 0.0 });
    assert_eq!(sample(&ed, 6, 0), Some(dir.join("a.wav")));
    assert_eq!(sample(&ed, 7, 0), Some(dir.join("c.wav")));
    assert_eq!(ed.browser.as_ref().unwrap().drag_items().len(), 2);
    // One file with alt goes to the first free layer.
    down(&mut ed, 1.0);
    ed.handle(&UiEvent::Mouse { path: list.clone(), overload: "BrowserFileList".into(), kind: MouseKind::Up, x: 5.0, y: 20.0 });
    ed.app.mods = Mods { alt: true, ..Mods::default() };
    ed.handle(&UiEvent::Dropped { from: list.clone(), to: pad(&ed, 6), x: 0.0, y: 0.0 });
    ed.app.mods = Mods::default();
    assert_eq!(sample(&ed, 6, 1), Some(dir.join("b.wav")));
    // A locked pad takes nothing.
    padmenu::set_lock(ed.model.pad_mut(e, 8), true);
    ed.handle(&UiEvent::Dropped { from: list.clone(), to: pad(&ed, 8), x: 0.0, y: 0.0 });
    assert_eq!(sample(&ed, 8, 0).filter(|f| f.starts_with(&dir)), None);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn sample_load_sets_holds_names_pads_and_respects_locks() {
    crate::app::test_setup();
    let wav = demo("Hits/Cymbals/CY Cym Ar82 01 C2A St GB.wav");
    if !wav.exists() {
        return;
    }
    let (frames, sr) = dreist_engine::audio::info(&wav).expect("header info");
    let mut m = Model::new(project::new_document());
    assert!(browser::load_sample_into_layer(&mut m, 0, 3, 0, &wav));
    let n = m.layer_node(0, 3, 0).unwrap();
    let get = |q: dreist_formats::params::LayerParam| q.default_value().clamp(0.0, 1.0);
    let want = dreist_engine::hold::reset_hold_time(&get, false, frames as f64, frames as f64, sr, false);
    assert!((n.f32("AmpEnvHold").unwrap() - want).abs() < 1e-6);
    assert_eq!(n.f32("FreeEnvHold"), n.f32("AmpEnvHold"));
    assert_eq!(m.pad_name(0, 3).as_deref(), Some("P4 CY Cym Ar82 01 C2A St GB"));
    // A second sample keeps the automatic name only while one sample is
    // present: with two the name stays.
    assert!(browser::load_sample_into_layer(&mut m, 0, 3, 1, &wav));
    assert_eq!(m.pad_name(0, 3).as_deref(), Some("P4 CY Cym Ar82 01 C2A St GB"));
    // Locked pads take nothing.
    padmenu::set_lock(m.pad_mut(0, 5), true);
    assert!(!browser::load_sample_into_layer(&mut m, 0, 5, 0, &wav));
    assert!(!m.layer(0, 5, 0).valid);
}

#[test]
fn finder_files_drop_onto_pads() {
    crate::app::test_setup();
    let wav = demo("Hits/Cymbals/CY Cym Ar82 01 C2A St GB.wav");
    if !wav.exists() {
        return;
    }
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    fn pad(c: &Ctrl, idx: &str, path: &mut Vec<usize>) -> Option<Vec<usize>> {
        if c.overload == "EditorPad" && c.attr("padIndex") == Some(idx) {
            return Some(path.clone());
        }
        for (i, k) in c.children.iter().enumerate() {
            path.push(i);
            if let Some(p) = pad(k, idx, path) {
                return Some(p);
            }
            path.pop();
        }
        None
    }
    let Some(p2) = pad(&ed.app.root, "2", &mut Vec::new()) else { return };
    ed.handle(&UiEvent::Files { to: p2.clone(), files: vec![wav.clone()], x: 0.0, y: 0.0, dropped: false });
    assert_eq!(ed.app.cursor, dreist_gui::cursor::Cursor::image("DragAudio"));
    ed.handle(&UiEvent::Files { to: p2, files: vec![wav.clone()], x: 0.0, y: 0.0, dropped: true });
    let e = ed.model.engine;
    assert!(ed.model.layer(e, 2, 0).valid);
}

#[test]
fn dragging_files_over_the_pads_flashes_one_pad_per_file() {
    crate::app::test_setup();
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    let Some(m) = ed.pads_matrix() else { return };
    let mc = ed.app.ctrl(&m).unwrap().clone();
    let slot = mc.children.iter().position(|c| c.attr("padIndex") == Some("1")).unwrap();
    let mut to = m.clone();
    to.push(slot);
    let (ox, oy) = ed.app.origin(&to);
    let files: Vec<std::path::PathBuf> = ["a.wav", "b.wav", "c.wav"].iter().map(std::path::PathBuf::from).collect();
    ed.handle(&UiEvent::Files { to: to.clone(), files: files.clone(), x: ox as f32 + 5.0, y: oy as f32 + 5.0, dropped: false });
    ed.animate();
    let lit = |ed: &Editor| -> Vec<String> {
        let mc = ed.app.ctrl(&m).unwrap();
        mc.children.iter().filter(|c| c.attr("flashAlpha").is_some()).filter_map(|c| c.attr("padIndex").map(str::to_string)).collect()
    };
    let mut got = lit(&ed);
    got.sort();
    assert_eq!(got, vec!["1", "2", "3"]);
    // Leaving the pads ends the loops; the flashes then fade out.
    ed.handle(&UiEvent::Files { to: Vec::new(), files, x: 0.0, y: 0.0, dropped: false });
    std::thread::sleep(std::time::Duration::from_millis(250));
    ed.animate();
    assert!(lit(&ed).is_empty());
    // A click flashes the pad.
    let ev = UiEvent::Mouse { path: to.clone(), overload: "EditorPad".into(), kind: dreist_gui::app::MouseKind::Down, x: 5.0, y: 5.0 };
    ed.handle(&ev);
    ed.animate();
    assert_eq!(lit(&ed), vec!["1"]);
}

#[test]
fn dragging_a_pattern_key_highlights_the_target_key() {
    crate::app::test_setup();
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    fn key(c: &Ctrl, name: &str, path: &mut Vec<usize>) -> Option<Vec<usize>> {
        if c.overload == "PatternSelector" && c.name == name {
            return Some(path.clone());
        }
        for (i, k) in c.children.iter().enumerate() {
            path.push(i);
            if let Some(p) = key(k, name, path) {
                return Some(p);
            }
            path.pop();
        }
        None
    }
    let (Some(k0), Some(k1)) = (key(&ed.app.root, "Key0", &mut Vec::new()), key(&ed.app.root, "Key1", &mut Vec::new())) else { return };
    ed.handle(&UiEvent::DragOver { from: k0.clone(), to: k1.clone(), x: 0.0, y: 0.0 });
    assert_eq!(ed.app.ctrl(&k1).unwrap().attr("dragAlpha"), Some("0"));
    // One 1/30 s tick (+25), or more on a slow machine: wait up to 1 s for it.
    for _ in 0..25 {
        std::thread::sleep(std::time::Duration::from_millis(40));
        ed.animate();
        if ed.app.ctrl(&k1).unwrap().attr("dragAlpha") != Some("0") {
            break;
        }
    }
    assert!(matches!(ed.app.ctrl(&k1).unwrap().attr("dragAlpha"), Some("25" | "50" | "75" | "100")));
    // Back over the source: no highlight.
    ed.handle(&UiEvent::DragOver { from: k0.clone(), to: k0.clone(), x: 0.0, y: 0.0 });
    assert!(ed.app.ctrl(&k1).unwrap().attr("dragAlpha").is_none());
}

#[test]
fn utility_splitter_drags_and_collapses() {
    crate::app::test_setup();
    use dreist_gui::app::MouseButton;
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    let Some(sp) = ed.app.root.path_of("UtilitySplitter") else { return };
    let (ox, oy) = ed.app.origin(&sp);
    let c = ed.app.ctrl(&sp).unwrap().clone();
    let bar = dreist_gui::splitter::bar(&c);
    let start = dreist_gui::splitter::position(&c);
    let (bx, by) = ((ox + bar.x + bar.w / 2) as f32, (oy + bar.y + bar.h / 2) as f32);
    ed.app.mouse_move(bx, by);
    assert_eq!(ed.app.cursor, dreist_gui::cursor::Cursor::LeftRight);
    ed.app.mouse_down(MouseButton::Left);
    let evs = ed.app.mouse_move(bx + 50.0, by);
    assert!(matches!(evs.as_slice(), [UiEvent::Split { pos, end: false, .. }] if *pos == start + 50));
    let evs = ed.app.mouse_up(MouseButton::Left);
    for e in &evs {
        ed.handle(e);
    }
    let c = ed.app.ctrl(&sp).unwrap();
    let p0 = c.children.iter().find(|k| k.display_name == "SplitterPanel0").unwrap();
    assert_eq!(p0.rect.w, start + 50);
    // Clamped at intMaxSize0 = 600.
    std::thread::sleep(std::time::Duration::from_millis(450));
    let bx = bx + 50.0;
    ed.app.mouse_move(bx, by);
    ed.app.mouse_down(MouseButton::Left);
    ed.app.mouse_move(bx + 900.0, by);
    ed.app.mouse_up(MouseButton::Left);
    assert_eq!(dreist_gui::splitter::position(ed.app.ctrl(&sp).unwrap()), 600);
    // A double click collapses panel 0 (optDbClkCollapse1 false), animated.
    std::thread::sleep(std::time::Duration::from_millis(450));
    let bx = (ox + 600 + bar.w / 2) as f32;
    ed.app.mouse_move(bx, by);
    for _ in 0..2 {
        ed.app.mouse_down(MouseButton::Left);
        ed.app.mouse_up(MouseButton::Left);
    }
    std::thread::sleep(std::time::Duration::from_millis(300));
    ed.animate();
    assert_eq!(dreist_gui::splitter::position(ed.app.ctrl(&sp).unwrap()), 0);
}

#[test]
fn ui_scale_applies_at_once_and_no_restores_it() {
    crate::app::test_setup();
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    ed.modals.show(&mut ed.app, modal::Kind::Preferences);
    let Some(p) = ed.modals.prefs_path() else { return };
    let Some(ui) = ed.app.ctrl(&p).and_then(|c| c.path_of("UIScaling")).map(|q| [p.clone(), q].concat()) else { return };
    ed.handle(&UiEvent::MenuSelected { anchor: ui.clone(), name: "UIScaling".into(), index: 5 });
    assert!((ed.app.ui_scale - 1.5).abs() < 1e-6);
    std::thread::sleep(std::time::Duration::from_millis(2050));
    ed.animate();
    assert_eq!(ed.app.menu.as_ref().map(|m| m.items[0].clone()), Some(modal::KEEP_SCALE.to_string()));
    ed.app.menu = None;
    ed.handle(&UiEvent::MenuSelected { anchor: ui, name: "UIScaling".into(), index: 3 });
    assert!((ed.app.ui_scale - 1.0).abs() < 1e-6);
    assert_eq!(ed.modals.options.int("LastEditorScaleFactor", 0), 100);
}

#[test]
fn ui_style_applies_at_once() {
    crate::app::test_setup();
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    ed.modals.show(&mut ed.app, modal::Kind::Preferences);
    let Some(p) = ed.modals.prefs_path() else { return };
    let Some(ui) = ed.app.ctrl(&p).and_then(|c| c.path_of("UIStyle")).map(|q| [p.clone(), q].concat()) else { return };
    ed.handle(&UiEvent::Clicked { path: ui.clone(), name: "UIStyle".into(), id: 0, on: false });
    let items = ed.app.menu.as_ref().map(|m| m.items.clone()).unwrap_or_default();
    let i = items.iter().position(|t| t == "Ember").expect("style menu");
    let before = ed.app.skin.styles.colours.clone();
    ed.handle(&UiEvent::MenuSelected { anchor: ui, name: "UIStyle".into(), index: i });
    assert_eq!(ed.app.skin.theme, "Ember");
    assert_ne!(ed.app.skin.styles.colours, before);
    assert_eq!(ed.modals.options.string("StyleSheet", ""), "Ember");
}

#[test]
fn documents_path_reset_writes_at_once_and_alerts() {
    crate::app::test_setup();
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    ed.modals.show(&mut ed.app, modal::Kind::Preferences);
    let Some(p) = ed.modals.prefs_path() else { return };
    let find = |ed: &Editor, n: &str| ed.app.ctrl(&p).and_then(|c| c.path_of(n)).map(|q| [p.clone(), q].concat()).unwrap();
    // v1 ignores typing in the path box.
    let tb = find(&ed, "DocumentsPath");
    let before = ed.modals.options.string("LocalDocumentsPath", "<unset>");
    let dir = std::env::temp_dir().to_string_lossy().trim_end_matches('/').to_string();
    ed.handle(&UiEvent::TextCommitted { path: tb.clone(), name: "DocumentsPath".into(), text: dir });
    assert_eq!(ed.modals.options.string("LocalDocumentsPath", "<unset>"), before);
    // Reset asks; Yes writes the default and alerts every time.
    let rb = find(&ed, "ResetPathButton");
    for _ in 0..2 {
        ed.handle(&UiEvent::Clicked { path: rb.clone(), name: "ResetPathButton".into(), id: 0, on: false });
        assert!(ed.app.menu.as_ref().is_some_and(|m| m.items[0].starts_with("Are you sure you want to reset the Documents path to ")));
        ed.app.menu = None;
        ed.handle(&UiEvent::MenuSelected { anchor: rb.clone(), name: "ResetPathButton".into(), index: 2 });
        assert_eq!(ed.modals.options.string("LocalDocumentsPath", ""), modal::default_documents());
        assert_eq!(ed.app.menu.as_ref().map(|m| m.items[0].clone()), Some(modal::docs_restart()));
        ed.app.menu = None;
        ed.handle(&UiEvent::MenuSelected { anchor: rb.clone(), name: "ResetPathButton".into(), index: 0 });
    }
    // Browse hands the folder dialog to the editor.
    let bb = find(&ed, "BrowseButton");
    ed.handle(&UiEvent::Clicked { path: bb, name: "BrowseButton".into(), id: 0, on: false });
    assert!(!ed.modals.browse_requested, "taken by the editor (no dialog runner in tests)");
}

#[test]
fn export_panel_shows_progress_asks_and_closes() {
    crate::app::test_setup();
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    ed.modals.show(&mut ed.app, modal::Kind::Export);
    let job = command::ExportProgress::default();
    ed.modals.export_job = Some(job.clone());
    job.lock().unwrap().progress = 0.5;
    ed.animate();
    let ov = ed.app.root.path_of("ExportOverlay").unwrap();
    let bar = [ov.clone(), (ed.app.ctrl(&ov).unwrap()).path_of("ProgressBar").unwrap()].concat();
    let c = ed.app.ctrl(&bar).unwrap();
    assert!(c.visible && (c.value - 0.5).abs() < 1e-6);
    job.lock().unwrap().prompt = Some("clipped?".into());
    ed.animate();
    assert_eq!(ed.app.menu.as_ref().map(|m| m.items[0].clone()), Some("clipped?".into()));
    ed.app.menu = None;
    ed.handle(&UiEvent::MenuSelected { anchor: bar.clone(), name: "ProgressBar".into(), index: 2 });
    assert_eq!(job.lock().unwrap().answer, Some(true));
    job.lock().unwrap().result = Some(Ok(Vec::new()));
    ed.animate();
    assert!(ed.modals.open.is_none() && ed.modals.export_job.is_none());
}

#[test]
fn rex_loops_slice_with_their_own_points_and_load_truncated_on_pads() {
    crate::app::test_setup();
    let home = std::env::var("HOME").unwrap_or_default();
    let f = std::path::PathBuf::from(home).join(dreist_formats::legacy::LITE_FACTORY).join("Loops/Breaks/130 Whitekit 03 KS.rx2");
    if !f.exists() || dreist_engine::rex::load(&f).is_none() {
        return;
    }
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    assert!(ed.slicer.start(&ed.model, &f));
    assert_eq!(ed.slicer.slices.len(), 14);
    // On a pad the layer refers to the ".truncated" composite.
    assert!(browser::load_sample_into_layer(&mut ed.model, 0, 0, 0, &f));
    let name = ed.model.layer_node(0, 0, 0).and_then(|n| n.attr("AudioFileName")).unwrap().to_string();
    assert!(name.ends_with(".rx2.truncated"));
    assert_eq!(ed.model.pad_name(0, 0).as_deref(), Some("P1 130 Whitekit 03 KS"));
}

#[test]
fn layer_toggles_reach_the_document() {
    crate::app::test_setup();
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    for (name, attr, want) in [("PhaseButton", "PhasePosition", "-1"), ("RevButton", "IsReversed", "true"), ("TrackPitchButton", "EnvTracksPitch", "true")] {
        let b = ed.app.root.path_of(name).unwrap_or_else(|| panic!("{name}"));
        ed.handle(&UiEvent::Clicked { path: b.clone(), name: name.into(), id: 0, on: true });
        assert_eq!(ed.model.layer_node(0, 0, 0).and_then(|n| n.attr(attr)), Some(want), "{name}");
        assert!(ed.app.ctrl(&b).unwrap().on, "{name}");
    }
    assert!(ed.take_commands().iter().any(|c| matches!(c, Command::Message(m) if m.kind == dreist_engine::binding::id::SET_LAYER_PHASE_POSITION)));
}

/// The browser's file list (path) after browsing `dir`.
pub(crate) fn browser_list(ed: &mut Editor, dir: &std::path::Path) -> Option<Vec<usize>> {
    ed.browser.as_mut()?.browse(dir.to_path_buf());
    fn by(c: &Ctrl, path: &mut Vec<usize>) -> Option<Vec<usize>> {
        if c.overload == "BrowserFileList" {
            return Some(path.clone());
        }
        for (i, k) in c.children.iter().enumerate() {
            path.push(i);
            if let Some(p) = by(k, path) {
                return Some(p);
            }
            path.pop();
        }
        None
    }
    by(&ed.app.root, &mut Vec::new())
}

/// A preview test folder with two short WAVs (`a.wav` 300 frames, `b.wav` 200).
pub(crate) fn preview_files(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("einst-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for (f, n) in [("a.wav", 300), ("b.wav", 200)] {
        let v = vec![0.25f32; n];
        dreist_engine::write_wav(&dir.join(f), 44100, &v, &v).unwrap();
    }
    dir
}

#[test]
fn browser_preview_starts_on_the_press_and_reuses_decoded_files() {
    crate::app::test_setup();
    use dreist_gui::app::MouseKind;
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    let dir = preview_files("preview-press");
    let Some(list) = browser_list(&mut ed, &dir) else { return };
    ed.browser.as_mut().unwrap().set_preview_mode(true);
    let ev = |kind, row: f32| UiEvent::Mouse { path: list.clone(), overload: "BrowserFileList".into(), kind, x: 5.0, y: row * 18.0 + 2.0 };
    let preview = |cmds: Vec<Command>| {
        cmds.into_iter().find_map(|c| match c {
            Command::Preview(Some(d)) => Some(d),
            _ => None,
        })
    };
    // Selecting a file previews it on the press, without waiting for the
    // next frame's poll (a local file decodes within the wait).
    ed.handle(&ev(MouseKind::Down, 0.0));
    let a = preview(ed.take_commands()).expect("the press previews a.wav");
    assert_eq!(a.left.len(), 300);
    ed.handle(&ev(MouseKind::Up, 0.0));
    assert!(preview(ed.take_commands()).is_none(), "the release adds nothing");
    ed.handle(&ev(MouseKind::Down, 1.0));
    assert_eq!(preview(ed.take_commands()).expect("b.wav").left.len(), 200);
    // Back to a.wav: the decoded audio is reused.
    ed.handle(&ev(MouseKind::Down, 0.0));
    let again = preview(ed.take_commands()).expect("a.wav again");
    assert!(std::sync::Arc::ptr_eq(&a, &again));
    // Preview off: a press only selects.
    ed.browser.as_mut().unwrap().set_preview_mode(false);
    ed.handle(&ev(MouseKind::Down, 1.0));
    assert!(preview(ed.take_commands()).is_none());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn browser_menu_acts_on_the_selection() {
    crate::app::test_setup();
    use dreist_gui::app::{Mods, MouseKind};
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    let dir = std::env::temp_dir().join(format!("dreist-bmenu-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    for f in ["a.wav", "b.wav"] {
        let _ = std::fs::write(dir.join(f), b"");
    }
    let Some(b) = ed.browser.as_mut() else { return };
    b.browse(dir.clone());
    let Some(page) = ed.browser.as_ref().map(|b| b.page().to_vec()) else { return };
    let list = (ed.app.ctrl(&page).unwrap()).path_of("FileList").map(|p| [page.clone(), p].concat());
    fn by(c: &Ctrl, path: &mut Vec<usize>) -> Option<Vec<usize>> {
        if c.overload == "BrowserFileList" {
            return Some(path.clone());
        }
        for (i, k) in c.children.iter().enumerate() {
            path.push(i);
            if let Some(p) = by(k, path) {
                return Some(p);
            }
            path.pop();
        }
        None
    }
    let list = list.filter(|p| ed.app.ctrl(p).is_some_and(|c| c.overload == "BrowserFileList")).or_else(|| by(&ed.app.root, &mut Vec::new())).unwrap();
    let ev = |kind, row: f32| UiEvent::Mouse { path: list.clone(), overload: "BrowserFileList".into(), kind, x: 5.0, y: row * 18.0 + 2.0 };
    ed.handle(&ev(MouseKind::Down, 0.0));
    ed.app.mods = Mods { cmd: true, ..Mods::default() };
    ed.handle(&ev(MouseKind::Down, 1.0));
    ed.app.mods = Mods::default();
    ed.handle(&ev(MouseKind::RightDown, 1.0));
    let items = ed.app.menu.as_ref().unwrap().items.clone();
    assert!(items.iter().any(|t| t == "Reveal selected items in Finder"), "{items:?}");
    // v1: one item for the current shortlist, enabled for files.
    assert!(items.iter().any(|t| t == "Shortlist selected items"), "{items:?}");
    // Favourites take folders only: disabled for files.
    assert!(items.iter().any(|t| t == &dreist_gui::menu::item("Add selected items to favourites", false)), "{items:?}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn browser_source_list_item_menu() {
    crate::app::test_setup();
    use dreist_gui::app::MouseKind;
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    let dir = std::env::temp_dir().join(format!("dreist-srcmenu-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let f = dir.join("snare.wav");
    let _ = std::fs::write(&f, b"x");
    let Some(b) = ed.browser.as_mut() else { return };
    let name = b.test_shortlist(vec![f.clone()]);
    fn by(c: &Ctrl, path: &mut Vec<usize>) -> Option<Vec<usize>> {
        if c.overload == "BrowserSourceList" {
            return Some(path.clone());
        }
        for (i, k) in c.children.iter().enumerate() {
            path.push(i);
            if let Some(p) = by(k, path) {
                return Some(p);
            }
            path.pop();
        }
        None
    }
    let list = by(&ed.app.root, &mut Vec::new()).unwrap();
    let row = |ed: &Editor, label: &str| ed.browser.as_ref().unwrap().source_row(label);
    let menu = |ed: &mut Editor, r: usize, label: &str| {
        ed.handle(&UiEvent::Mouse { path: list.clone(), overload: "BrowserSourceList".into(), kind: MouseKind::RightDown, x: 5.0, y: r as f32 * 25.0 + 3.0 });
        let items = ed.app.menu.as_ref().expect("menu").items.clone();
        let i = items.iter().position(|s| s.ends_with(label)).unwrap_or_else(|| panic!("{label} in {items:?}"));
        ed.handle(&UiEvent::MenuSelected { anchor: list.clone(), name: String::new(), index: i });
        items
    };
    let (r, _) = row(&ed, &name).unwrap();
    let items = menu(&mut ed, r, "Set colour label... ▸");
    assert_eq!(items, vec![format!("Remove \"{name}\""), String::new(), "Set name...".into(), "Set colour label... ▸".into()]);
    ed.handle(&UiEvent::MenuSelected { anchor: list.clone(), name: String::new(), index: 5 });
    assert_eq!(row(&ed, &name).map(|r| r.1), Some(5));
    // Rename: the shortlist takes the name and keeps its colour.
    menu(&mut ed, r, "Set name...");
    let edit = ed.app.root.path_of(browser::SOURCE_RENAME_EDIT).expect("edit box");
    let new = format!("Drums {}", std::process::id());
    ed.handle(&UiEvent::TextCommitted { path: edit, name: browser::SOURCE_RENAME_EDIT.into(), text: new.clone() });
    assert_eq!(row(&ed, &new).map(|r| r.1), Some(5));
    // Export copies the files into <dir>/<name>.
    let out = ed.browser.as_mut().unwrap().export_shortlist(&new, &dir, false).unwrap();
    assert!(out.join("snare.wav").exists());
    // Remove asks first.
    let (r, _) = row(&ed, &new).unwrap();
    menu(&mut ed, r, &format!("Remove \"{new}\""));
    let items = ed.app.menu.as_ref().unwrap().items.clone();
    assert!(items[0].contains("Are you sure you want to remove"), "{items:?}");
    let yes = items.iter().position(|s| s == "Yes").unwrap();
    ed.handle(&UiEvent::MenuSelected { anchor: list.clone(), name: String::new(), index: yes });
    assert!(row(&ed, &new).is_none());
    // Places rows offer Reveal only.
    let (r, _) = row(&ed, "Home").unwrap();
    ed.handle(&UiEvent::Mouse { path: list.clone(), overload: "BrowserSourceList".into(), kind: MouseKind::RightDown, x: 5.0, y: r as f32 * 25.0 + 3.0 });
    assert_eq!(ed.app.menu.as_ref().unwrap().items, vec!["Reveal \"Home\" in Finder".to_string()]);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn tooltips_follow_the_pointer() {
    crate::app::test_setup();
    let Some(mut ed) = demo_editor("Presets/Hollow.geist") else { return };
    let hover = |ed: &mut Editor, name: &str| -> Option<String> {
        let path = ed.app.root.path_of(name).unwrap();
        let (x, y) = ed.app.origin(&path);
        let c = ed.app.ctrl(&path).unwrap();
        let (cx, cy) = (x as f32 + c.rect.w as f32 / 2.0, y as f32 + c.rect.h as f32 / 2.0);
        for ev in ed.app.mouse_move(cx, cy) {
            ed.handle(&ev);
        }
        let t = ed.app.tooltip_target();
        ed.handle(&UiEvent::ToolTip { path: t });
        ed.app.tooltip.as_ref().map(|t| t.text.clone())
    };
    // v1's DB15 / LongPercentage providers under the display names.
    assert_eq!(hover(&mut ed, "MasterSlider").map(|t| t.starts_with("Master gain: ") && t.ends_with("dB")), Some(true));
    assert_eq!(hover(&mut ed, "StepLengthCombo").as_deref(), Some("Length of time each step lasts"));
    assert_eq!(hover(&mut ed, "EngineSwingDepthKnob").map(|t| t.starts_with("Swing Template Depth: ") && t.ends_with('%')), Some(true));
    // Leaving the window hides it.
    ed.app.mouse_left();
    assert!(ed.app.tooltip.is_none());
}

#[test]
fn mouse_drag_between_pads_emits_a_drop() {
    crate::app::test_setup();
    use dreist_gui::app::MouseButton;
    let Some(mut ed) = demo_editor("Presets/Hollow.geist") else { return };
    fn find(c: &dreist_gui::Ctrl, idx: &str, path: &mut Vec<usize>) -> bool {
        for (i, k) in c.children.iter().enumerate() {
            path.push(i);
            if (k.overload == "EditorPad" && k.attr("padIndex") == Some(idx)) || find(k, idx, path) {
                return true;
            }
            path.pop();
        }
        false
    }
    let centre = |ed: &Editor, idx: &str| {
        let mut v = Vec::new();
        assert!(find(&ed.app.root, idx, &mut v));
        let (x, y) = ed.app.origin(&v);
        let r = ed.app.ctrl(&v).unwrap().rect;
        (v, x as f32 + r.w as f32 / 2.0, y as f32 + r.h as f32 / 2.0)
    };
    let (a, ax, ay) = centre(&ed, "1");
    let (b, bx, by) = centre(&ed, "2");
    ed.app.mouse_move(ax, ay);
    ed.app.mouse_down(MouseButton::Left);
    let over = ed.app.mouse_move(bx, by);
    let drag = over.iter().find(|e| matches!(e, UiEvent::DragOver { .. })).cloned().expect("drag over");
    ed.handle(&drag);
    assert_eq!(ed.app.cursor, dreist_gui::cursor::Cursor::image("DragPad"));
    let evs = ed.app.mouse_up(MouseButton::Left);
    assert_eq!(ed.app.cursor, dreist_gui::cursor::Cursor::Arrow);
    assert!(matches!(&evs[0], UiEvent::Dropped { from, to, .. } if *from == a && to.starts_with(&b)), "{evs:?}");
    // A click without travel is not a drop.
    ed.app.mouse_move(ax, ay);
    ed.app.mouse_down(MouseButton::Left);
    let evs = ed.app.mouse_up(MouseButton::Left);
    assert!(!evs.iter().any(|e| matches!(e, UiEvent::Dropped { .. })));
}

#[test]
fn song_body_hover_cursors() {
    crate::app::test_setup();
    let Some(mut ed) = demo_editor("Presets/Hollow.geist") else { return };
    fn find(c: &dreist_gui::Ctrl, path: &mut Vec<usize>) -> bool {
        for (i, k) in c.children.iter().enumerate() {
            path.push(i);
            if k.overload == "SongBody" || find(k, path) {
                return true;
            }
            path.pop();
        }
        false
    }
    let mut body = Vec::new();
    if !find(&ed.app.root, &mut body) {
        return;
    }
    let ev = UiEvent::Hover { path: body.clone(), overload: "SongBody".into(), x: 5000.0, y: 30.0 };
    ed.handle(&ev);
    assert_eq!(ed.app.cursor, dreist_gui::cursor::Cursor::image("Pen"));
    ed.app.mods.shift = true;
    ed.handle(&ev);
    assert_eq!(ed.app.cursor, dreist_gui::cursor::Cursor::image("Selection"));
}

#[test]
fn save_layer_sample_crops_to_markers() {
    crate::app::test_setup();
    let p = demo("Presets/Hollow.geist");
    let Ok(proj) = dreist_formats::Project::load(&p) else { return };
    let mut model = Model::new(proj.raw);
    model.samples = Some(dreist_formats::model::samples_dir(&p));
    let e = model.engine;
    let Some((pad, l)) = (0..64).flat_map(|p| (0..8).map(move |l| (p, l))).find(|&(p, l)| model.layer_sample(e, p, l).is_some_and(|f| f.exists())) else { return };
    let out = std::env::temp_dir().join(format!("dreist-save-{}.wav", std::process::id()));
    padmenu::save_layer_sample(&model, e, pad, l, &out).unwrap();
    let bytes = std::fs::read(&out).unwrap();
    let ch = u16::from_le_bytes([bytes[22], bytes[23]]) as u32;
    let data = u32::from_le_bytes([bytes[40], bytes[41], bytes[42], bytes[43]]);
    let mut cache = dreist_engine::audio::SampleCache::default();
    let src = cache.get(&model.layer_sample(e, pad, l).unwrap()).unwrap();
    let m = inspector::wave_markers(&model, e, pad, l, src.left.len() as u32);
    assert_eq!(data, (m.end.min(src.left.len() as u32) - m.start) * 3 * ch);
    let _ = std::fs::remove_file(&out);
}

#[test]
fn pads_strip_erase_repeat_and_timing() {
    crate::app::test_setup();
    let Some(mut ed) = demo_editor("Presets/Hollow.geist") else { return };
    let cp = ed.app.root.path_of("PadModePanel").unwrap();
    let child = |ed: &Editor, n: &str| {
        let c = ed.app.ctrl(&cp).unwrap();
        [cp.clone(), vec![c.children.iter().position(|k| k.name == n).unwrap()]].concat()
    };
    let er = child(&ed, "EraseButton");
    ed.handle(&UiEvent::Clicked { path: er.clone(), name: "EraseButton".into(), id: 0, on: true });
    let e = ed.model.engine;
    let st = |ed: &Editor, k: &str| ed.model.engine_node(e).and_then(|n| n.children.iter().find(|c| c.class == "EngineState")).and_then(|s| s.attr(k).map(str::to_string));
    assert_eq!(st(&ed, "NoteEraseEnabled").as_deref(), Some("true"));
    assert!(ed.app.ctrl(&er).unwrap().on);
    let rc = child(&ed, "RepeatCount");
    ed.handle(&UiEvent::Clicked { path: rc.clone(), name: "RepeatCount".into(), id: 0, on: false });
    let items = ed.app.menu.as_ref().unwrap().items.clone();
    assert_eq!(items[8], "✓ 1/16");
    assert_eq!(items.last().unwrap(), "✓ Sync to next division");
    let cmds = ed.take_commands();
    let _ = cmds;
    ed.handle(&UiEvent::MenuSelected { anchor: rc.clone(), name: "RepeatCount".into(), index: 10 });
    assert_eq!(ed.app.ctrl(&rc).unwrap().text, "1/32");
    let cmds = ed.take_commands();
    assert!(cmds.iter().any(|c| matches!(c, Command::Message(m) if m.kind == dreist_engine::binding::id::SET_NOTE_REPEAT_BASE && (m.value[0] * 13.0).round() as i32 == 10)));
    // Full level: the button and its knob (v1's column has room for
    // both).
    let fk = child(&ed, "FullLevelKnob");
    assert!(ed.app.ctrl(&fk).unwrap().visible);
    ed.outbox.clear();
    ed.handle(&UiEvent::ValueChanged { path: fk, name: "FullLevelKnob".into(), id: 0, value: 0.5 });
    let cmds = std::mem::take(&mut ed.outbox);
    assert!(cmds.iter().any(|c| matches!(c, Command::Message(m) if m.kind == dreist_engine::binding::id::SET_FULL_LEVEL_LEVEL && (m.value[0] - 0.5).abs() < 1e-6)));
    let fb = child(&ed, "FullLevelButton");
    ed.handle(&UiEvent::Clicked { path: fb, name: "FullLevelButton".into(), id: 0, on: true });
    let cmds = std::mem::take(&mut ed.outbox);
    assert!(cmds.iter().any(|c| matches!(c, Command::Message(m) if m.kind == dreist_engine::binding::id::SET_FULL_LEVEL_STATE && m.value[0] == 1.0)));
}

/// Scene names come from the document (the engine keeps none).
#[test]
fn scene_names_come_from_the_document() {
    crate::app::test_setup();
    let Some(mut ed) = demo_editor("Presets/Hollow.geist") else { return };
    assert_eq!(ed.model.scene_name(0), "Drums");
    ed.model.set_scene_name(5, "Fill");
    assert_eq!(ed.model.scene_name(5), "Fill");
    assert_eq!(ed.model.scene_name(95), "");
}

/// The running tempo (host / tap) reaches the document and the header,
/// without an undo step.
#[test]
fn engine_tempo_reaches_document_and_header() {
    crate::app::test_setup();
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    let undo = ed.history.undo.len();
    ed.engine_tempo(133.25);
    assert_eq!(ed.model.tempo(), 133.25);
    let bpm = ed.app.root.path_of("BPMTextBox").and_then(|p| ed.app.ctrl(&p)).map(|c| c.text.clone());
    assert_eq!(bpm.as_deref(), Some("133.25"));
    assert_eq!(ed.history.undo.len(), undo);
}

/// "nan" / "inf" typed into a value box are not numbers: nothing is sent
/// and the document keeps its value.
#[test]
fn typed_non_numbers_are_ignored() {
    crate::app::test_setup();
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    let Some(p) = ed.app.root.path_of("BPMTextBox") else { return };
    ed.take_commands();
    for text in ["nan", "inf", "-infinity", "NaN"] {
        ed.handle(&UiEvent::TextCommitted { path: p.clone(), name: "BPMTextBox".into(), text: text.into() });
        let cmds = ed.take_commands();
        assert!(cmds.is_empty(), "{text}: {cmds:?}");
        assert_eq!(ed.model.tempo(), 120.0, "{text}");
    }
    ed.handle(&UiEvent::TextCommitted { path: p.clone(), name: "BPMTextBox".into(), text: " 97.5 ".into() });
    assert_eq!(ed.model.tempo(), 97.5);
    assert_eq!(crate::conv::number("1e99"), None, "overflows to infinity");
}

/// The engine mixer's strips (`Channel0..7`) and their controls.
fn engine_strips(ed: &Editor) -> Vec<Vec<usize>> {
    let m = ed.mixers.iter().find(|m| m.kind == mixer::Kind::Engine).expect("engine mixer");
    let page = ed.app.ctrl(&m.page).unwrap();
    page.children.iter().enumerate().filter(|(_, c)| c.overload == "MixerStrip").map(|(i, _)| [m.page.clone(), vec![i]].concat()).collect()
}

fn strip_control(ed: &Editor, strip: &[usize], name: &str) -> Vec<usize> {
    [strip.to_vec(), ed.app.ctrl(strip).unwrap().path_of(name).unwrap_or_else(|| panic!("{name}"))].concat()
}

#[test]
fn engine_mixer_has_eight_strips_with_mix_send_and_setup_views() {
    crate::app::test_setup();
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    ed.model.editor_tab = front::ENGINE_MIXER_PAGE;
    ed.sync();
    let strips = engine_strips(&ed);
    assert_eq!(strips.len(), 8);
    for (k, s) in strips.iter().enumerate() {
        assert_eq!(ed.app.ctrl(s).unwrap().attr("channel"), Some(format!("Engine({k})").as_str()));
        assert_eq!(ed.app.ctrl(&strip_control(&ed, s, "NameLabel")).unwrap().text, ed.model.engine_name(k));
    }
    let shown = |ed: &Editor, name: &str| ed.app.ctrl(&strip_control(ed, &engine_strips(ed)[0], name)).unwrap().visible;
    let view = |ed: &mut Editor, button: usize| {
        let m = ed.mixers.iter().find(|m| m.kind == mixer::Kind::Engine).unwrap().page.clone();
        let g = [m.clone(), ed.app.ctrl(&m).unwrap().path_of("ModeGroup").unwrap()].concat();
        let b = [g, vec![button]].concat();
        let name = ed.app.ctrl(&b).unwrap().name.clone();
        ed.handle(&UiEvent::Clicked { path: b.clone(), name, id: 0, on: true });
        assert!(ed.app.ctrl(&b).unwrap().on);
    };
    // MIX: pan, fader, meters, output.
    for n in ["PanKnob", "GainSlider", "VUMeterLeft", "OutputCombo", "MuteButton"] {
        assert!(shown(&ed, n), "{n} in MIX");
    }
    assert!(!shown(&ed, "SendKnob0") && !shown(&ed, "EnginePitchKnob"));
    // SEND: the four sends.
    view(&mut ed, 1);
    assert!(shown(&ed, "SendKnob3") && shown(&ed, "SendLabel3") && !shown(&ed, "GainSlider") && !shown(&ed, "PanKnob"));
    // SETUP: pitch, swing template and depth, tempo modifier.
    view(&mut ed, 2);
    for n in ["EnginePitchKnob", "SwingKnob", "EngineSwingTemplateCombo", "EngineNumBox", "EngineDenomBox"] {
        assert!(shown(&ed, n), "{n} in SETUP");
    }
    assert!(!shown(&ed, "OutputCombo") && !shown(&ed, "SendKnob0") && !shown(&ed, "GainSlider"));
    assert!(shown(&ed, "MuteButton"), "M / S in every view");
    if let Ok(f) = std::env::var("DREIST_MIXER_SHOT") {
        ed.render(1.0).save_png(f).unwrap();
    }
}

#[test]
fn engine_strip_controls_post_engine_messages() {
    crate::app::test_setup();
    use dreist_engine::binding::id;
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    ed.model.editor_tab = front::ENGINE_MIXER_PAGE;
    ed.sync();
    let s = engine_strips(&ed)[3].clone();
    let msgs = |ed: &mut Editor, ev: UiEvent| {
        ed.take_commands();
        ed.handle(&ev);
        ed.take_commands().into_iter().filter_map(|c| if let Command::Message(m) = c { Some(m) } else { None }).collect::<Vec<_>>()
    };
    let value = |ed: &Editor, name: &str, v: f32| UiEvent::ValueChanged { path: strip_control(ed, &s, name), name: name.into(), id: 0, value: v };
    let ev = value(&ed, "GainSlider", 0.3);
    let m = msgs(&mut ed, ev);
    assert!(m.iter().any(|m| m.kind == id::SET_ENGINE_PARAMETER && m.index[0] == 3 && m.value[..2] == [0.3, dreist_formats::EngineParam::Gain as u8 as f32]), "{m:?}");
    let ev = value(&ed, "PanKnob", 0.25);
    let m = msgs(&mut ed, ev);
    assert!(m.iter().any(|m| m.kind == id::SET_ENGINE_PARAMETER && m.value[..2] == [0.25, dreist_formats::EngineParam::Pan as u8 as f32]), "{m:?}");
    let ev = value(&ed, "EnginePitchKnob", 0.75);
    let m = msgs(&mut ed, ev);
    assert!(m.iter().any(|m| m.kind == id::SET_ENGINE_PARAMETER && m.value[..2] == [0.75, dreist_formats::EngineParam::Pitch as u8 as f32]), "{m:?}");
    let ev = value(&ed, "SwingKnob", 0.4);
    let m = msgs(&mut ed, ev);
    assert!(m.iter().any(|m| m.kind == id::SET_ENGINE_SWING_TEMPLATE_DEPTH && m.index[0] == 3 && m.value[0] == 0.4), "{m:?}");
    let ev = value(&ed, "SendKnob2", 0.5);
    let m = msgs(&mut ed, ev);
    assert!(m.iter().any(|m| m.kind == id::SET_ENGINE_SEND_LEVEL && m.value[..2] == [2.0, 0.5]), "{m:?}");
    let mute = strip_control(&ed, &s, "MuteButton");
    let m = msgs(&mut ed, UiEvent::Clicked { path: mute, name: "MuteButton".into(), id: 0, on: true });
    assert!(m.iter().any(|m| m.kind == id::MUTE_ENGINE && m.index[0] == 3 && m.value[0] == 1.0), "{m:?}");
    let num = strip_control(&ed, &s, "EngineNumBox");
    let m = msgs(&mut ed, UiEvent::TextCommitted { path: num, name: "EngineNumBox".into(), text: "3".into() });
    assert!(m.iter().any(|m| m.kind == id::SET_ENGINE_TEMPO_NUM && m.value[0] == 3.0), "{m:?}");
    let den = strip_control(&ed, &s, "EngineDenomBox");
    let m = msgs(&mut ed, UiEvent::TextCommitted { path: den, name: "EngineDenomBox".into(), text: "40".into() });
    assert!(m.iter().any(|m| m.kind == id::SET_ENGINE_TEMPO_DENOM && m.value[0] == 16.0), "clamped to 16: {m:?}");
    ed.sync();
    assert_eq!(ed.model.engine_node(3).and_then(|n| n.u32("TempoNumerator")), Some(3));
    assert_eq!(ed.app.ctrl(&strip_control(&ed, &s, "EngineNumBox")).unwrap().text, "3");
    assert!(ed.app.ctrl(&strip_control(&ed, &s, "MuteButton")).unwrap().on);
}

/// The Pad / Layers selector: the pad item shows the pad view, a layer
/// row selects that layer and shows the layer view; rows show their
/// layer's name ("Empty" without a sample).
#[test]
fn pad_layers_selector_switches_views() {
    crate::app::test_setup();
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    ed.model.editor_tab = crate::front::PADS_PAGE;
    ed.sync();
    let shown = |ed: &Editor, n: &str| ed.app.root.find(n).is_some_and(|c| c.visible);
    assert!(shown(&ed, "LayerControls") && !shown(&ed, "PadControls"));
    let pad = ed.app.root.path_of("PadSelection").unwrap();
    ed.handle(&UiEvent::Mouse { path: pad, overload: "PadSelectionButton".into(), kind: dreist_gui::app::MouseKind::Down, x: 5.0, y: 5.0 });
    assert!(shown(&ed, "PadControls") && !shown(&ed, "LayerControls"));
    assert_eq!(ed.app.root.find("PadSelection").unwrap().style, "PadSelectionButtonPrimaryStyle");
    let row = ed.app.root.path_of("LayerSelectionButton2").unwrap();
    ed.handle(&UiEvent::Mouse { path: row.clone(), overload: "LayerRow".into(), kind: dreist_gui::app::MouseKind::Down, x: 5.0, y: 5.0 });
    let e = ed.model.engine;
    assert_eq!(ed.model.layer[e], 2);
    assert!(shown(&ed, "LayerControls") && !shown(&ed, "PadControls"));
    let r = ed.app.ctrl(&row).unwrap();
    assert_eq!(r.style, "SelectionButtonPrimaryStyle");
    assert_eq!(r.find("SampleName").unwrap().text, "Empty");
    // The connector sits beside the selected row.
    let glue = ed.app.root.find("LayerGlue").unwrap();
    assert_eq!(glue.rect.y, r.rect.y + 2);
    // Pad item: Row link → `e_setModifiers` [layers, pads, row].
    let rowlink = ed.app.root.path_of("AllGroups").unwrap();
    ed.take_commands();
    ed.handle(&UiEvent::Clicked { path: rowlink, name: "AllGroups".into(), id: 0, on: true });
    assert!(ed.model.all_rows);
    assert!(ed.take_commands().iter().any(|c| matches!(c, Command::Message(m) if m.kind == dreist_engine::binding::id::SET_MODIFIERS && m.value[2] == 1.0)));
}

/// The pad view's velocity split lanes: shown in velocity-split mode, a
/// drag of a layer's range posts its four split velocities.
#[test]
fn velocity_split_lane_edit() {
    crate::app::test_setup();
    use dreist_engine::binding::id;
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    ed.model.editor_tab = crate::front::PADS_PAGE;
    ed.inspector.pad_view = true;
    ed.sync();
    let lane = ed.app.root.path_of("Split0").unwrap();
    assert!(!ed.app.ctrl(&lane).unwrap().visible, "hidden outside velocity-split mode");
    ed.model.pad_mut(0, 0).set("PadMode", "Velocity split");
    ed.model.layer_mut(0, 0, 0).set("AudioFileName", "x.wav");
    ed.sync();
    let c = ed.app.ctrl(&lane).unwrap();
    assert!(c.visible && c.enabled);
    let mouse = |kind, x| UiEvent::Mouse { path: lane.clone(), overload: crate::velsplit::OVERLOAD.into(), kind, x, y: 20.0 };
    ed.take_commands();
    // Whole range: no handle under the press (mid lane), dragged left.
    ed.handle(&mouse(dreist_gui::app::MouseKind::Down, 250.0));
    ed.handle(&mouse(dreist_gui::app::MouseKind::Drag, 200.0));
    ed.handle(&mouse(dreist_gui::app::MouseKind::Up, 200.0));
    let kinds: Vec<u32> = ed.take_commands().into_iter().filter_map(|c| if let Command::Message(m) = c { Some(m.kind) } else { None }).collect();
    for k in [id::SET_LAYER_VELOCITY_SPLIT_MIN, id::SET_LAYER_VELOCITY_SPLIT_MIN_TURNOVER, id::SET_LAYER_VELOCITY_SPLIT_MAX_TURNOVER, id::SET_LAYER_VELOCITY_SPLIT_MAX] {
        assert!(kinds.contains(&k), "{kinds:?}");
    }
}

/// The layer view's setup and envelope controls post layer parameters:
/// knobs their value, Vel-Pitch the Velocity depth (2v − 1), an envelope
/// destination's depth that envelope's depth, and a new destination takes
/// the depth along.
#[test]
fn layer_view_controls_post_layer_parameters() {
    crate::app::test_setup();
    use dreist_engine::binding::id;
    use dreist_formats::{LayerParam, TransmodSource};
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    ed.model.editor_tab = crate::front::PADS_PAGE;
    ed.sync();
    let msgs = |ed: &mut Editor| ed.take_commands().into_iter().filter_map(|c| if let Command::Message(m) = c { Some(m) } else { None }).collect::<Vec<_>>();
    let turn = |ed: &mut Editor, name: &str, v: f32| {
        let p = ed.app.root.path_of(name).unwrap_or_else(|| panic!("{name}"));
        ed.take_commands();
        ed.handle(&UiEvent::ValueChanged { path: p, name: name.into(), id: 0, value: v });
    };
    let has = |m: &[dreist_engine::binding::Message], par: LayerParam, v: f32, src: TransmodSource| {
        m.iter().any(|m| m.kind == id::SET_LAYER_PARAMETER && m.value[0] == par as u8 as f32 && (m.value[1] - v).abs() < 1e-5 && m.value[2] == src as u8 as f32)
    };
    turn(&mut ed, "CutoffKnob", 0.3);
    assert!(has(&msgs(&mut ed), LayerParam::FilterCutoff, 0.3, TransmodSource::ActualParameter));
    turn(&mut ed, "V2P", 0.75);
    assert!(has(&msgs(&mut ed), LayerParam::CoarseTune, 0.5, TransmodSource::Velocity));
    assert!((ed.app.root.find("V2P").unwrap().value - 0.75).abs() < 1e-5, "the knob shows the depth");
    // The Free envelope's first destination is Cutoff by default.
    assert_eq!(ed.app.root.find("FreeEnvDest1Combo").unwrap().text, "Cutoff");
    turn(&mut ed, "FreeEnvDepth1Knob", 1.0);
    assert!(has(&msgs(&mut ed), LayerParam::FilterCutoff, 1.0, TransmodSource::FreeEnvelope));
    // Cutoff → Resonance moves the depth.
    let combo = ed.app.root.path_of("FreeEnvDest1Combo").unwrap();
    ed.handle(&UiEvent::Clicked { path: combo.clone(), name: "FreeEnvDest1Combo".into(), id: 0, on: false });
    assert_eq!(ed.app.menu.as_ref().unwrap().items[2], "Resonance");
    ed.take_commands();
    ed.handle(&UiEvent::MenuSelected { anchor: combo, name: "FreeEnvDest1Combo".into(), index: 2 });
    let m = msgs(&mut ed);
    assert!(has(&m, LayerParam::FilterCutoff, 0.0, TransmodSource::FreeEnvelope) && has(&m, LayerParam::FilterResonance, 1.0, TransmodSource::FreeEnvelope), "{m:?}");
    assert_eq!(ed.app.root.find("FreeEnvDest1Combo").unwrap().text, "Resonance");
    // Layer sends: `e_setLayerSendDepth` [aux, value].
    turn(&mut ed, "Send2Knob", 0.4);
    assert!(msgs(&mut ed).iter().any(|m| m.kind == id::SET_LAYER_SEND_DEPTH && m.value[0] == 1.0 && (m.value[1] - 0.4).abs() < 1e-5));
}

/// The song toolbar's tools: draw adds parts of the draw pattern (picked
/// in the popup beside the tool), select only selects (a press on empty
/// space starts a box), erase deletes the part under the pointer.
#[test]
fn song_tools_and_draw_pattern_popup() {
    crate::app::test_setup();
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    ed.model.editor_tab = front::SONG_PAGE;
    ed.sync();
    song_mode_on(&mut ed);
    let body = ed.app.root.path_of("SongBody").unwrap();
    let (bx, by) = ed.app.origin(&body);
    let click = |ed: &mut Editor, x: f32, y: f32| {
        for ev in ed.app.mouse_move(x, y) {
            ed.handle(&ev);
        }
        for ev in ed.app.mouse_down(dreist_gui::app::MouseButton::Left) {
            ed.handle(&ev);
        }
        for ev in ed.app.mouse_up(dreist_gui::app::MouseButton::Left) {
            ed.handle(&ev);
        }
    };
    let song = |ed: &Editor| ed.model.doc.children.iter().find(|c| c.class == "SongSequencer").map(dreist_engine::song::Song::from_node).unwrap();
    let group = ed.app.root.path_of("ToolButtonGroup").unwrap();
    let tool = |ed: &mut Editor, i: usize| {
        let b = [group.clone(), vec![i]].concat();
        ed.handle(&UiEvent::Clicked { path: b.clone(), name: "CtrlButton".into(), id: 0, on: true });
        assert!(ed.app.ctrl(&b).unwrap().on);
    };
    // Select: no new part on empty space.
    tool(&mut ed, 2);
    click(&mut ed, bx as f32 + 5.0, by as f32 + 30.0);
    assert!(song(&ed).tracks[0].is_empty());
    // Draw with pattern 4 (the popup beside the draw tool).
    tool(&mut ed, 0);
    let pen = ed.app.root.path_of("SetDrawPatternButton").unwrap();
    ed.handle(&UiEvent::Clicked { path: pen, name: "SetDrawPatternButton".into(), id: 0, on: false });
    let popup = ed.app.root.path_of("SongPatternPopup").expect("pattern popup");
    let k3 = [popup.clone(), ed.app.ctrl(&popup).unwrap().path_of("Key3").unwrap()].concat();
    ed.handle(&UiEvent::Clicked { path: k3, name: "Key3".into(), id: 0, on: true });
    click(&mut ed, bx as f32 + 5.0, by as f32 + 30.0);
    assert_eq!(song(&ed).tracks[0].len(), 1);
    assert_eq!(song(&ed).tracks[0][0].pattern, 3);
    // Erase: the part goes.
    tool(&mut ed, 1);
    click(&mut ed, bx as f32 + 5.0, by as f32 + 30.0);
    assert!(song(&ed).tracks[0].is_empty());
}

/// The pattern tools panel: the erase tool deletes with the left button,
/// the select tool selects instead of drawing, pen / multi draw.
#[test]
fn pattern_tools_panel_switches_the_left_button() {
    crate::app::test_setup();
    use dreist_gui::app::MouseKind;
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    ed.sync();
    let grid = ed.app.root.path_of("PatternGrid").unwrap();
    let lay = ed.pattern.as_ref().unwrap().layout(&ed.app, &ed.model).unwrap();
    let e = ed.model.engine;
    let tools = ed.app.root.path_of("ToolsPanel").unwrap();
    let group = [tools.clone(), ed.app.ctrl(&tools).unwrap().path_of("ButtonGroup").unwrap()].concat();
    let tool = |ed: &mut Editor, i: usize| {
        let b = [group.clone(), vec![i]].concat();
        let name = ed.app.ctrl(&b).unwrap().name.clone();
        ed.handle(&UiEvent::Clicked { path: b.clone(), name, id: 0, on: true });
        assert!(ed.app.ctrl(&b).unwrap().on);
    };
    let r = lay.row(0).unwrap();
    let press = |ed: &mut Editor, step: usize| {
        let (x, y) = (lay.step_x(step) + 2.0, lay.row_y(r) + 2.0);
        ed.handle(&UiEvent::Mouse { path: grid.clone(), overload: "PatternGrid".into(), kind: MouseKind::Down, x, y });
        ed.handle(&UiEvent::Mouse { path: grid.clone(), overload: "PatternGrid".into(), kind: MouseKind::Up, x, y });
    };
    let on = |ed: &Editor, k: usize| ed.model.pattern(e, ed.model.current_pattern(e)).tracks[0].steps[k].on;
    tool(&mut ed, 0);
    press(&mut ed, 2);
    assert!(on(&ed, 2), "pen draws");
    tool(&mut ed, 2);
    press(&mut ed, 5);
    assert!(!on(&ed, 5), "select does not draw");
    press(&mut ed, 2);
    assert!(on(&ed, 2) && ed.pattern.as_ref().unwrap().selection.contains(&(0, 2)), "select selects the event");
    tool(&mut ed, 1);
    press(&mut ed, 2);
    assert!(!on(&ed, 2), "erase deletes");
    tool(&mut ed, 3);
    press(&mut ed, 7);
    assert!(on(&ed, 7), "multi draws");
}

/// The graph lane's ÷2 / ×2 (manual §9:5) and the toolbar's Insert menu.
#[test]
fn graph_halve_double_and_toolbar_insert() {
    crate::app::test_setup();
    use dreist_engine::binding::id;
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    ed.sync();
    let e = ed.model.engine;
    let cur = ed.model.current_pattern(e);
    let n = ed.model.pattern(e, cur).steps as usize;
    let track = ed.model.pad[e];
    // A ramp on the shown lane.
    let lane = ed.model.pattern_view[e].lane;
    for st in 0..n {
        let m = dreist_engine::binding::Message::new(id::SET_STEP_GRAPH_STATE, &[e as i32, track as i32, -1, cur as i32, track as i32, st as i32], &[lane as f32, st as f32 / n as f32, 0.0]);
        ed.model.apply(&Command::Message(m));
    }
    let values = |ed: &mut Editor, name: &str| -> Vec<f32> {
        let p = ed.app.root.path_of("SingleAutoControl").and_then(|s| ed.app.ctrl(&s).unwrap().path_of(name).map(|k| [s, k].concat()));
        let Some(p) = p else { return Vec::new() };
        ed.take_commands();
        ed.handle(&UiEvent::Clicked { path: p, name: name.into(), id: 0, on: false });
        ed.take_commands().into_iter().filter_map(|c| if let Command::Message(m) = c { (m.kind == id::SET_STEP_GRAPH_STATE).then_some(m.value[1]) } else { None }).collect()
    };
    let half = values(&mut ed, "Div2Button");
    if !half.is_empty() {
        assert_eq!(half.len(), n);
        assert!((half[3] - 1.0 / n as f32).abs() < 1e-5, "÷2 stretches the first half: {half:?}");
        let double = values(&mut ed, "x2Button");
        // On the stretched lane: every second value, twice over.
        assert!((double[1] - half[2]).abs() < 1e-5 && (double[n / 2 + 1] - half[2]).abs() < 1e-5, "×2 compresses and repeats: {double:?}");
    }
    // Toolbar Insert: the pattern templates menu for the selected pad.
    let tb = ed.app.root.path_of("ToolBar").unwrap();
    let ins = [tb.clone(), ed.app.ctrl(&tb).unwrap().path_of("InsertButton").unwrap()].concat();
    ed.handle(&UiEvent::Clicked { path: ins.clone(), name: "InsertButton".into(), id: 0, on: false });
    if crate::patmenu::pattern_templates(&dreist_formats::paths::skin_dir()).is_empty() {
        return;
    }
    assert!(ed.app.menu.is_some(), "Insert opens the templates");
    ed.take_commands();
    ed.handle(&UiEvent::MenuSelected { anchor: ins, name: "InsertButton".into(), index: 0 });
    let steps = ed.take_commands().iter().filter(|c| matches!(c, Command::Message(m) if (m.kind == id::SET_STEP_ON || m.kind == id::SET_STEP_OFF) && m.index[1] == track as i32)).count();
    assert!(steps >= n, "the track filled from step 1");
}

/// A device host for the standalone's settings panels: two outputs (B
/// with its own rates), one input, two MIDI inputs and one output; counts
/// the audio restarts.
struct FakeDevices(std::sync::Arc<std::sync::atomic::AtomicUsize>);

impl devices::DeviceHost for FakeDevices {
    fn audio(&self, output: &str, _input: &str) -> devices::AudioSetup {
        let rates = if output == "B" { vec![48000, 96000] } else { vec![44100, 48000] };
        devices::AudioSetup { outputs: vec!["A".into(), "B".into()], inputs: vec!["Mic".into()], default_output: "A".into(), default_rate: rates[0], rates, out_channels: 4, in_channels: 2 }
    }
    fn midi(&self) -> devices::MidiSetup {
        devices::MidiSetup { inputs: vec!["Pads".into(), "Keys".into()], outputs: vec!["Synth".into()] }
    }
    fn restart(&self, _: &dreist_formats::Node, _: Option<&std::path::Path>) {
        self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }
}

/// A control of the panel at `overlay` by name.
fn panel_ctrl(ed: &Editor, overlay: &[usize], name: &str) -> Vec<usize> {
    [overlay.to_vec(), ed.app.ctrl(overlay).unwrap().path_of(name).unwrap_or_else(|| panic!("no {name}"))].concat()
}

fn click_named(ed: &mut Editor, path: &[usize], on: bool) {
    let name = ed.app.ctrl(path).unwrap().name.clone();
    ed.handle(&UiEvent::Clicked { path: path.to_vec(), name, id: 0, on });
}

/// The standalone's audio settings (the v1 set's authored dialog): it
/// opens from the menu, its popups list the host's devices, rates and
/// buffers, the channel tabs switch, Cancel puts the choices back and OK
/// after a change restarts audio.
#[test]
fn audio_settings_dialog_opens_and_works() {
    crate::app::test_setup();
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    assert!(ed.devices.available(devices::Panel::Audio), "the v1 set has the audio dialog");
    let restarts = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    ed.devices.host = Some(Box::new(FakeDevices(restarts.clone())));
    ed.menu_command(crate::editor::shortcuts::MENU_AUDIO);
    assert_eq!(ed.devices.open, Some(devices::Panel::Audio));
    let o = ed.devices.overlay_path(devices::Panel::Audio).unwrap();
    assert!(ed.app.ctrl(&o).unwrap().visible);
    // The Windows driver panel and the check box template stay hidden.
    for n in ["WinSettingPanel", "CheckBox"] {
        assert!(!ed.app.ctrl(&panel_ctrl(&ed, &o, n)).unwrap().visible, "{n}");
    }
    assert_eq!(ed.app.ctrl(&panel_ctrl(&ed, &o, "OutDevice")).unwrap().text, "A");
    assert_eq!(ed.app.ctrl(&panel_ctrl(&ed, &o, "SampleRate")).unwrap().text, "44100");
    // The output popup: pick B; its rate follows.
    let out_dev = panel_ctrl(&ed, &o, "OutDevice");
    click_named(&mut ed, &out_dev, false);
    assert_eq!(ed.app.menu.as_ref().map(|m| m.items.clone()).unwrap_or_default(), ["A", "B"]);
    ed.app.menu = None;
    ed.handle(&UiEvent::MenuSelected { anchor: out_dev.clone(), name: "OutDevice".into(), index: 1 });
    ed.sync();
    assert_eq!(ed.app.ctrl(&out_dev).unwrap().text, "B");
    assert_eq!(ed.app.ctrl(&panel_ctrl(&ed, &o, "SampleRate")).unwrap().text, "48000");
    // Buffer sizes.
    let buf = panel_ctrl(&ed, &o, "BufferSize");
    click_named(&mut ed, &buf, false);
    assert_eq!(ed.app.menu.as_ref().unwrap().items.len(), devices::BUFFER_SIZES.len());
    ed.app.menu = None;
    ed.handle(&UiEvent::MenuSelected { anchor: buf, name: "BufferSize".into(), index: 2 });
    assert_eq!(ed.devices.settings().buffer, 128);
    // INPUTS / OUTPUTS.
    let group = panel_ctrl(&ed, &o, "IOGroup");
    let outputs = [group.clone(), vec![1]].concat();
    click_named(&mut ed, &outputs, true);
    ed.sync();
    assert!(ed.app.ctrl(&outputs).unwrap().on && !ed.app.ctrl(&[group, vec![0]].concat()).unwrap().on);
    // Cancel: back to A, no restart.
    let cancel = panel_ctrl(&ed, &o, "Cancel");
    click_named(&mut ed, &cancel, false);
    assert_eq!(ed.devices.open, None);
    assert_eq!(ed.devices.settings().output, "");
    assert_eq!(restarts.load(std::sync::atomic::Ordering::SeqCst), 0);
    // Again with OK: the change restarts audio.
    ed.menu_command(crate::editor::shortcuts::MENU_AUDIO);
    ed.handle(&UiEvent::MenuSelected { anchor: out_dev, name: "OutDevice".into(), index: 1 });
    let ok = panel_ctrl(&ed, &o, "Close");
    assert_eq!(ed.app.ctrl(&ok).unwrap().text, "OK");
    click_named(&mut ed, &ok, false);
    assert_eq!(ed.devices.open, None);
    assert_eq!(restarts.load(std::sync::atomic::Ordering::SeqCst), 1);
    // Refresh re-reads the devices without closing.
    ed.menu_command(crate::editor::shortcuts::MENU_AUDIO);
    let refresh = panel_ctrl(&ed, &o, "Refresh");
    click_named(&mut ed, &refresh, false);
    assert_eq!(ed.devices.open, Some(devices::Panel::Audio));
}

/// The standalone's MIDI settings: both tables list the host's devices,
/// a click on a row's box picks that device (again: none), clock sync is
/// off in the port, Cancel restores.
#[test]
fn midi_settings_dialog_opens_and_works() {
    crate::app::test_setup();
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    assert!(ed.devices.available(devices::Panel::Midi));
    let restarts = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    ed.devices.host = Some(Box::new(FakeDevices(restarts.clone())));
    ed.menu_command(crate::editor::shortcuts::MENU_MIDI);
    assert_eq!(ed.devices.open, Some(devices::Panel::Midi));
    let o = ed.devices.overlay_path(devices::Panel::Midi).unwrap();
    let inputs = panel_ctrl(&ed, &o, "InputTable");
    assert_eq!(ed.app.ctrl(&inputs).unwrap().overload, "DeviceTable:InputTable");
    assert_eq!(ed.app.ctrl(&panel_ctrl(&ed, &o, "OutputTable")).unwrap().overload, "DeviceTable:OutputTable");
    assert!(!ed.app.ctrl(&panel_ctrl(&ed, &o, "Sync")).unwrap().enabled, "no clock sync in the port");
    // The second input's box (row 2 under the 20 px header).
    let down = |ed: &mut Editor, path: &[usize], y: f32| {
        ed.handle(&UiEvent::Mouse { path: path.to_vec(), overload: ed.app.ctrl(path).unwrap().overload.clone(), kind: dreist_gui::app::MouseKind::Down, x: 25.0, y });
    };
    down(&mut ed, &inputs, 20.0 + 18.0 + 5.0);
    assert_eq!(ed.devices.settings().midi_in, "Keys");
    down(&mut ed, &inputs, 20.0 + 18.0 + 5.0);
    assert_eq!(ed.devices.settings().midi_in, "", "a second click turns it off");
    let outputs = panel_ctrl(&ed, &o, "OutputTable");
    down(&mut ed, &outputs, 25.0);
    assert_eq!(ed.devices.settings().midi_out, "Synth");
    let cancel = panel_ctrl(&ed, &o, "Cancel");
    click_named(&mut ed, &cancel, false);
    assert_eq!((ed.devices.open, ed.devices.settings().midi_out.as_str()), (None, ""));
    assert_eq!(restarts.load(std::sync::atomic::Ordering::SeqCst), 0);
}

/// The MIDI console (main menu › Show MIDI console): incoming messages
/// as text, the filters and Clear.
#[test]
fn midi_console_opens_and_filters() {
    crate::app::test_setup();
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    ed.show_modal("console");
    assert!(ed.midiconsole.open, "the v1 set has the console");
    let o = ed.midiconsole.overlay_path().unwrap();
    assert!(ed.app.ctrl(&o).unwrap().visible);
    assert_eq!(ed.app.ctrl(&panel_ctrl(&ed, &o, "OutputView")).unwrap().overload, "MIDIConsoleView");
    ed.midi_in(&[[0x90, 60, 100], [0xb0, 7, 64]]);
    assert_eq!(ed.midiconsole.lines.len(), 2);
    // Notes off: notes are no longer listed.
    let notes = panel_ctrl(&ed, &o, "Notes");
    click_named(&mut ed, &notes, false);
    ed.midi_in(&[[0x90, 62, 100], [0xb0, 7, 65]]);
    assert_eq!(ed.midiconsole.lines.len(), 3);
    assert!(ed.midiconsole.lines[2].starts_with("C-Change"));
    let clear = panel_ctrl(&ed, &o, "Clear");
    click_named(&mut ed, &clear, false);
    assert!(ed.midiconsole.lines.is_empty());
}

/// The effect types v1 had no skin for (a newer format's projects can
/// hold them) get panels in v1's style: the slot shows a control per
/// parameter (a combo for an option parameter) and the controls set the
/// parameters.
#[test]
fn newer_effect_types_show_working_panels() {
    crate::app::test_setup();
    use dreist_engine::binding::id;
    let kinds = [6, 7, 10, 13, 14, 20, 21, 22, 26, 27, 37, 44, 45, 46, 47];
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    ed.model.editor_tab = front::ENGINE_MIXER_PAGE;
    for kind in kinds {
        // A project's engine chain holding the effect in slot 1.
        let e = ed.model.engine;
        let chain = ed.model.engine_mut(e).child_mut_or_insert("EffectChain", "EffectChain");
        chain.children.retain(|c| c.class != "Effect");
        let mut fx = dreist_formats::Node::new("Effect", Some("Effect0"));
        fx.set("EffectType", dreist_formats::params::EFFECT_TYPES[kind]).set("IsActive", "true");
        chain.children.push(fx);
        ed.sync();
        let page = ed.mixers.iter().find(|m| m.kind == mixer::Kind::Engine).unwrap().page.clone();
        let row = [page.clone(), ed.app.ctrl(&page).unwrap().path_of("ChainPanelScroll").expect("slot row")].concat();
        let slot = [row.clone(), vec![0]].concat();
        assert_eq!(ed.app.ctrl(&slot).unwrap().attr("fxKind"), Some(kind.to_string().as_str()), "kind {kind}");
        let names = dreist_formats::params::effect_param_names(kind);
        for (n, pname) in names.iter().enumerate() {
            let c = ed.app.ctrl(&slot).unwrap().find(&format!("Control{}", n + 1)).unwrap_or_else(|| panic!("kind {kind}: no control for {pname}")).clone();
            assert!(c.visible, "kind {kind}: {pname}");
            if dreist_formats::params::effect_option_strings(kind, n).is_some() {
                assert_eq!(c.class, "CtrlPopupButton", "kind {kind}: {pname} is an option");
            }
        }
        // The first knob sets its parameter.
        let (k, knob) = (0..names.len()).find_map(|n| ed.app.ctrl(&slot).unwrap().find(&format!("Control{}", n + 1)).filter(|c| c.class == "CtrlKnob").map(|c| (n, c.name.clone()))).unwrap();
        let path = [slot.clone(), ed.app.ctrl(&slot).unwrap().path_of(&knob).unwrap()].concat();
        ed.take_commands();
        ed.handle(&UiEvent::ValueChanged { path, name: knob, id: 0, value: 0.25 });
        let sent = ed.take_commands().into_iter().any(|c| matches!(c, Command::Message(m) if m.kind == id::SET_FX_PARAMETER && m.value[0] == k as f32 && (m.value[2] - 0.25).abs() < 1e-6));
        assert!(sent, "kind {kind}: the knob sets parameter {k}");
    }
}

/// A compressor's meter shows its gain reduction and, while the reading
/// does not change, rises by 0.01 a tick (v1).
#[test]
fn compressor_meter_follows_gain_reduction() {
    crate::app::test_setup();
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    ed.model.editor_tab = front::ENGINE_MIXER_PAGE;
    let e = ed.model.engine;
    let chain = ed.model.engine_mut(e).child_mut_or_insert("EffectChain", "EffectChain");
    chain.children.retain(|c| c.class != "Effect");
    let mut fx = dreist_formats::Node::new("Effect", Some("Effect0"));
    fx.set("EffectType", dreist_formats::params::EFFECT_TYPES[9]).set("IsActive", "true");
    chain.children.push(fx);
    ed.sync();
    let page = ed.mixers.iter().find(|m| m.kind == mixer::Kind::Engine).unwrap().page.clone();
    let row = [page.clone(), ed.app.ctrl(&page).unwrap().path_of("ChainPanelScroll").unwrap()].concat();
    let slot = [row, vec![0]].concat();
    let meter = |ed: &Editor| ed.app.ctrl(&slot).and_then(|c| c.find("DasMeter")).map(|c| c.value);
    assert!(meter(&ed).is_some(), "the bus compressor panel has its meter");
    let mut m = dreist_engine::Meters::default();
    m.fx_out.push(dreist_engine::FxOut { at: [e as i8, -1, -1], slot: 0, value: 0.5 });
    ed.update_meters(&m);
    assert_eq!(meter(&ed), Some(0.5));
    ed.update_meters(&m);
    assert!((meter(&ed).unwrap() - 0.51).abs() < 1e-6, "unchanged: rises");
    m.fx_out[0].value = 0.3;
    ed.update_meters(&m);
    assert_eq!(meter(&ed), Some(0.3));
}

/// The pen tool's right button opens the track's menu (as the track name
/// area does) instead of deleting the step.
#[test]
fn pen_tool_right_click_opens_the_track_menu() {
    crate::app::test_setup();
    use dreist_engine::binding::id;
    use dreist_gui::app::MouseKind;
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    ed.model.editor_tab = 0;
    ed.sync();
    let gp = ed.app.root.path_of("PatternGrid").unwrap();
    let lay = ed.pattern.as_ref().unwrap().layout(&ed.app, &ed.model).unwrap();
    let (x, y) = (lay.step_x(2) + 2.0, lay.row_y(lay.row(3).unwrap()) + 2.0);
    // A step on, with the multi tool's left button.
    ed.handle(&UiEvent::Mouse { path: gp.clone(), overload: "PatternGrid".into(), kind: MouseKind::Down, x, y });
    ed.handle(&UiEvent::Mouse { path: gp.clone(), overload: "PatternGrid".into(), kind: MouseKind::Up, x, y });
    ed.pattern.as_mut().unwrap().tool = crate::pattern::Tool::Pen;
    ed.take_commands();
    ed.handle(&UiEvent::Mouse { path: gp.clone(), overload: "PatternGrid".into(), kind: MouseKind::RightDown, x, y });
    let items = ed.app.menu.as_ref().map(|m| m.items.clone()).unwrap_or_default();
    assert_eq!(items.first().map(String::as_str), Some("Pen Tool"), "{items:?}");
    assert!(items.iter().any(|i| i == "Delete"), "the step under the mouse is on");
    assert!(items.iter().any(|i| i == "Clear track"));
    assert!(!ed.take_commands().iter().any(|c| matches!(c, Command::Message(m) if m.kind == id::SET_STEP_OFF)), "no delete");
    assert_eq!(ed.model.pad[ed.model.engine], 3, "the track's pad is selected");
    // The menu acts on that track: Clear track.
    let clear = items.iter().position(|i| i == "Clear track").unwrap();
    ed.app.menu = None;
    ed.handle(&UiEvent::MenuSelected { anchor: gp.clone(), name: "PatternGrid".into(), index: clear });
    let e = ed.model.engine;
    assert!(!ed.model.pattern(e, ed.model.current_pattern(e)).tracks[3].steps[2].on, "Clear track cleared it");
    // Quantize track timing: the track's Shift lane back to its default.
    ed.handle(&UiEvent::Mouse { path: gp.clone(), overload: "PatternGrid".into(), kind: MouseKind::RightDown, x, y });
    let q = ed.app.menu.as_ref().unwrap().items.iter().position(|i| i == "Quantize track timing").unwrap();
    ed.take_commands();
    ed.handle(&UiEvent::MenuSelected { anchor: gp.clone(), name: "PatternGrid".into(), index: q });
    assert!(ed.take_commands().iter().any(|c| matches!(c, Command::Message(m) if m.kind == id::CLEAR_TRACK_AUTOMATION && m.index[1] == 3)));
    // Erase Tool switches the tool.
    ed.handle(&UiEvent::Mouse { path: gp.clone(), overload: "PatternGrid".into(), kind: MouseKind::RightDown, x, y });
    ed.handle(&UiEvent::MenuSelected { anchor: gp, name: "PatternGrid".into(), index: 1 });
    assert_eq!(ed.pattern.as_ref().unwrap().tool, crate::pattern::Tool::Erase);
}

/// The slicer's zoom scrollbar (added under the wave): dragging the
/// thumb scrolls the zoomed view; it draws in the zoom slider style.
#[test]
fn slicer_zoom_scrollbar_scrolls() {
    crate::app::test_setup();
    use dreist_gui::app::MouseKind;
    let Some(mut ed) = demo_editor("Presets/Hollow.geist") else { return };
    if !ed.open_slicer(&dreist_formats::legacy::demo("Hits/Cymbals/CY Cym Ar82 01 C2A St GB.wav")) {
        return;
    }
    ed.sync();
    let bar = ed.app.root.path_of(crate::slicer::ZOOM_BAR).expect("zoom bar");
    let w = ed.app.ctrl(&bar).unwrap().rect.w as f32;
    let panel = bar[..bar.len() - 1].to_vec();
    let zoom_in = [panel.clone(), ed.app.ctrl(&panel).unwrap().path_of("ZoomInButton").unwrap()].concat();
    // Zoomed in twice: a quarter of the file on view, in its middle.
    let full = ed.slicer.view;
    for _ in 0..2 {
        ed.handle(&UiEvent::Clicked { path: zoom_in.clone(), name: "ZoomInButton".into(), id: 0, on: false });
    }
    let v = ed.slicer.view;
    assert!(v.1 - v.0 < (full.1 - full.0) / 3, "{v:?}");
    // Drag the thumb (from its middle) by a quarter of the track.
    let x0 = (v.0 + v.1) as f32 / 2.0 / full.1 as f32 * w;
    ed.handle(&UiEvent::Mouse { path: bar.clone(), overload: "WaveDisplayZoomSlider".into(), kind: MouseKind::Down, x: x0, y: 5.0 });
    ed.handle(&UiEvent::Mouse { path: bar.clone(), overload: "WaveDisplayZoomSlider".into(), kind: MouseKind::Drag, x: x0 + w * 0.25, y: 5.0 });
    ed.handle(&UiEvent::Mouse { path: bar, overload: "WaveDisplayZoomSlider".into(), kind: MouseKind::Up, x: x0 + w * 0.25, y: 5.0 });
    let moved = ed.slicer.view;
    assert!(moved.0 > v.0, "scrolled: {v:?} -> {moved:?}");
    assert_eq!(moved.1 - moved.0, v.1 - v.0, "same zoom");
}

/// The master display's memory readout shows the engine's sample memory
/// in the original's units ([re:91925c], [re:04a6c7]).
#[test]
fn memory_readout_shows_sample_memory() {
    crate::app::test_setup();
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    let text = |ed: &Editor| ed.app.root.find("MemoryUsageBox").map(|c| c.text.clone()).unwrap_or_default();
    ed.sync();
    assert_eq!(text(&ed), "", "nothing before the plugin reports");
    ed.model.memory = Some(20_656_947);
    ed.sync();
    assert_eq!(text(&ed), "19.7MB");
}

/// v1's list pane ([re:fa4043]): Parent folder goes up, the header sorts
/// by the clicked column and its Columns menu adds Size, and the chosen
/// shortlist is listed under the folder's files with its own menu
/// ([re:dbf248], [re:200a75]).
#[test]
fn browser_list_pane_header_and_shortlist() {
    crate::app::test_setup();
    use dreist_gui::app::MouseKind;
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    let root = std::env::temp_dir().join(format!("einst-pane-{}", std::process::id()));
    let dir = root.join("x");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("a.wav"), vec![0u8; 3000]).unwrap();
    std::fs::write(dir.join("b.wav"), vec![0u8; 100]).unwrap();
    let Some(page) = ed.browser.as_ref().map(|b| b.page().to_vec()) else { return };
    let at = |ed: &Editor, name: &str| -> Vec<usize> { [page.clone(), ed.app.ctrl(&page).unwrap().path_of(name).unwrap()].concat() };
    let names = |ed: &Editor| ed.browser.as_ref().unwrap().entry_paths().iter().map(|p| p.file_name().unwrap().to_string_lossy().into_owned()).collect::<Vec<_>>();
    ed.browser.as_mut().unwrap().browse(dir.clone());
    // Parent folder.
    let parent = at(&ed, "ParentFolderButton");
    ed.handle(&UiEvent::Clicked { path: parent, name: "ParentFolderButton".into(), id: 0, on: false });
    assert_eq!(names(&ed), vec!["x"]);
    ed.browser.as_mut().unwrap().browse(dir.clone());
    assert_eq!(names(&ed), vec!["a.wav", "b.wav"]);
    // Name sorts; again reverses.
    let header = at(&ed, "ListHeader");
    let hw = ed.app.ctrl(&header).unwrap().rect.w as f32;
    let click = |ed: &mut Editor, x: f32| ed.handle(&UiEvent::Mouse { path: header.clone(), overload: "BrowserListHeader".into(), kind: MouseKind::Down, x, y: 5.0 });
    click(&mut ed, 5.0);
    assert_eq!(names(&ed), vec!["b.wav", "a.wav"]);
    // Columns ▸ Size: shown, saved, and its header sorts by size.
    assert_eq!(ed.browser.as_ref().unwrap().test_columns(), (false, false));
    let view = at(&ed, "ViewOptionsButton");
    ed.handle(&UiEvent::Clicked { path: view.clone(), name: "ViewOptionsButton".into(), id: 0, on: false });
    ed.handle(&UiEvent::MenuSelected { anchor: view.clone(), name: "ViewOptionsButton".into(), index: 1 });
    ed.handle(&UiEvent::MenuSelected { anchor: view.clone(), name: "ViewOptionsButton".into(), index: 1 });
    assert_eq!(ed.browser.as_ref().unwrap().test_columns(), (false, true));
    assert!(crate::options::Options::load().bool("FileHeaderShowStateSize", false));
    click(&mut ed, hw - 10.0);
    assert_eq!(names(&ed), vec!["b.wav", "a.wav"], "smaller first");
    click(&mut ed, hw - 10.0);
    assert_eq!(names(&ed), vec!["a.wav", "b.wav"]);
    // View ▸ Audio off hides the audio files; View all brings them back
    // ([re:ca74b1]).
    let pick = |ed: &mut Editor, i: usize| {
        ed.handle(&UiEvent::Clicked { path: view.clone(), name: "ViewOptionsButton".into(), id: 0, on: false });
        ed.handle(&UiEvent::MenuSelected { anchor: view.clone(), name: "ViewOptionsButton".into(), index: 0 });
        ed.handle(&UiEvent::MenuSelected { anchor: view.clone(), name: "ViewOptionsButton".into(), index: i });
    };
    pick(&mut ed, 0);
    assert!(names(&ed).is_empty());
    assert!(!crate::options::Options::load().bool("ShowAudio", true));
    pick(&mut ed, 14);
    assert_eq!(names(&ed), vec!["a.wav", "b.wav"]);
    // The chosen shortlist lists below; the folder's files stay.
    let name = ed.browser.as_mut().unwrap().test_shortlist(vec![dir.join("b.wav"), dir.join("a.wav")]);
    ed.browser.as_mut().unwrap().browse(std::path::PathBuf::from(format!("shortlist:{name}")));
    assert_eq!(names(&ed), vec!["a.wav", "b.wav"]);
    assert_eq!(ed.browser.as_ref().unwrap().current_dir(), Some(dir.as_path()), "the folder stays");
    assert_eq!(ed.browser.as_ref().unwrap().test_short_names(), vec!["a.wav", "b.wav"]);
    // Its menu: Remove selected.
    let short = at(&ed, "ShortList");
    ed.handle(&UiEvent::Mouse { path: short.clone(), overload: "BrowserShortList".into(), kind: MouseKind::RightDown, x: 5.0, y: 2.0 });
    let items = ed.app.menu.as_ref().expect("shortlist menu").items.clone();
    assert_eq!(
        items.iter().filter(|t| !t.is_empty()).map(|t| t.trim_start_matches(dreist_gui::menu::DISABLED).to_string()).collect::<Vec<_>>()[..5],
        ["Select all", "Remove selected", "Crop to selected", "Clear shortlist", "Save as shortlist file"]
    );
    ed.handle(&UiEvent::MenuSelected { anchor: short, name: "ShortList".into(), index: 2 });
    assert_eq!(ed.browser.as_ref().unwrap().test_short_names(), vec!["b.wav"]);
    let _ = std::fs::remove_dir_all(&root);
}

/// A v1 layer's envelope rows are its four routings: they show the slots
/// and edit them in place (depth and destination per slot), keeping the
/// envelope depths in step and asking for a rebuild ([re:fad3cb]).
#[test]
fn envelope_rows_edit_v1_routing_slots() {
    crate::app::test_setup();
    use dreist_engine::binding::id;
    use dreist_formats::legacy::{routing_slots, set_routing_slot};
    use dreist_formats::{LayerParam, TransmodSource};
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    let (e, p, l) = (ed.model.engine, ed.model.pad[ed.model.engine], ed.model.layer[ed.model.engine]);
    for (r, slot) in [Some((1u8, 0.9f32)), Some((1, 0.7)), None, Some((0, 0.25))].into_iter().enumerate() {
        set_routing_slot(ed.model.layer_mut(e, p, l), r, slot);
    }
    ed.model.editor_tab = crate::front::PADS_PAGE;
    ed.sync();
    assert_eq!(ed.app.root.find("AmpEnvDest2Combo").unwrap().text, "Cutoff");
    assert_eq!(ed.app.root.find("FreeEnvDest2Combo").unwrap().text, "Pitch");
    assert!((ed.app.root.find("AmpEnvDepth2Knob").unwrap().value - 0.7).abs() < 1e-6);
    // Depth: the slot changes; the amp envelope's cutoff depth is the mean.
    let knob = ed.app.root.path_of("AmpEnvDepth2Knob").unwrap();
    ed.take_commands();
    ed.handle(&UiEvent::ValueChanged { path: knob, name: "AmpEnvDepth2Knob".into(), id: 0, value: 0.5 });
    let cmds = ed.take_commands();
    assert_eq!(routing_slots(ed.model.layer_node(e, p, l).unwrap()).unwrap()[1], Some((1, 0.5)));
    assert!(cmds.iter().any(|c| matches!(c, Command::Rebuild { .. })));
    assert!(cmds.iter().any(|c| matches!(c, Command::Message(m) if m.kind == id::SET_LAYER_PARAMETER && m.value[0] == LayerParam::FilterCutoff as u8 as f32 && (m.value[1] - 0.8).abs() < 1e-5 && m.value[2] == TransmodSource::AmpEnvelope as u8 as f32)));
    // Destination: slot 2 (free envelope, first row) to Drive, depth kept.
    let combo = ed.app.root.path_of("FreeEnvDest1Combo").unwrap();
    ed.handle(&UiEvent::Clicked { path: combo.clone(), name: "FreeEnvDest1Combo".into(), id: 0, on: false });
    ed.handle(&UiEvent::MenuSelected { anchor: combo, name: "FreeEnvDest1Combo".into(), index: 3 });
    assert_eq!(routing_slots(ed.model.layer_node(e, p, l).unwrap()).unwrap()[2], Some((3, 0.5)));
    ed.sync();
    assert_eq!(ed.app.root.find("FreeEnvDest1Combo").unwrap().text, "Drive");
}

/// v1's browser show / hide: the window shrinks to the main panel and
/// comes back.
#[test]
fn browser_show_hide_resizes_the_window() {
    crate::app::test_setup();
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    let b = ed.app.root.path_of("ShowHideButton").unwrap();
    assert!(ed.app.ctrl(&b).unwrap().visible);
    ed.handle(&UiEvent::Clicked { path: b.clone(), name: "ShowHideButton".into(), id: 100000, on: true });
    assert_eq!(ed.app.size(), (830, 640));
    assert!(ed.app.resize_request);
    let mp = ed.app.root.path_of("MovePanel").unwrap();
    assert_eq!(ed.app.ctrl(&mp).unwrap().rect.x, 1);
    assert!(!ed.app.ctrl(&ed.app.root.path_of("BrowsePanel").unwrap()).unwrap().visible);
    ed.handle(&UiEvent::Clicked { path: b, name: "ShowHideButton".into(), id: 100000, on: false });
    assert_eq!(ed.app.size(), (1135, 640));
    assert_eq!(ed.app.ctrl(&mp).unwrap().rect.x, 306);
}

/// v1's sampler preview output menu (right click on Preview volume).
#[test]
fn sampler_preview_output_menu() {
    crate::app::test_setup();
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    ed.model.editor_tab = crate::front::SAMPLER_PAGE;
    ed.sync();
    let Some(k) = ed.app.root.path_of("PreviewVolume") else { return };
    ed.handle(&UiEvent::RightClicked { path: k.clone(), name: "PreviewVolume".into() });
    let items = ed.app.menu.as_ref().expect("menu").items.clone();
    assert_eq!(items.len(), 16, "{items:?}");
    ed.take_commands();
    ed.handle(&UiEvent::MenuSelected { anchor: k, name: "PreviewVolume".into(), index: 3 });
    assert!(ed.take_commands().iter().any(|c| matches!(c, Command::Message(m) if m.kind == dreist_engine::binding::id::SET_PREVIEW_OUTPUT && m.value[0] == 4.0)));
}

/// v1's browser drop targets: files join the current shortlist; folders
/// dropped on Favourites become favourites.
#[test]
fn browser_shortlist_and_favourites_drops() {
    crate::app::test_setup();
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    let dir = std::env::temp_dir().join(format!("einst-drop-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let f = dir.join("a.wav");
    std::fs::write(&f, b"").unwrap();
    let Some(b) = ed.browser.as_mut() else { return };
    assert!(b.add_to_current_shortlist(vec![f.clone()]));
    assert!(!b.add_to_current_shortlist(vec![f.clone()]), "already there");
    let fav = b.source_rows().iter().position(|(cat, header)| *cat == "Favourites" && *header).unwrap();
    assert!(!b.drop_on_source(fav, std::slice::from_ref(&f)), "files are not favourites");
    assert!(b.drop_on_source(fav, std::slice::from_ref(&dir)));
    let _ = std::fs::remove_dir_all(&dir);
}

/// v1's sampler take dragged onto a pad: the take, saved, is that pad's
/// first layer.
#[test]
fn sampler_take_drags_onto_a_pad() {
    crate::app::test_setup();
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    let tmp = std::env::temp_dir().join(format!("dreist-takedrag-{}", std::process::id()));
    ed.modals.options.set("SamplerRecordPath", tmp.to_string_lossy());
    ed.model.samples = Some(tmp.clone());
    ed.model.editor_tab = crate::front::SAMPLER_PAGE;
    ed.sync();
    let tone: Vec<f32> = (0..4410).map(|i| (i as f32 * 0.05).sin() * 0.5).collect();
    assert_eq!(ed.sampler_take(&tone, &tone, 44100.0), Some(0));
    fn find(c: &dreist_gui::Ctrl, f: &dyn Fn(&dreist_gui::Ctrl) -> bool, path: &mut Vec<usize>) -> bool {
        for (i, k) in c.children.iter().enumerate() {
            path.push(i);
            if f(k) || find(k, f, path) {
                return true;
            }
            path.pop();
        }
        false
    }
    let (mut wave, mut pad) = (Vec::new(), Vec::new());
    assert!(find(&ed.app.root, &|c| c.overload == "SamplerWaveArea" && c.visible, &mut wave));
    assert!(find(&ed.app.root, &|c| c.overload == "EditorPad" && c.attr("padIndex") == Some("2"), &mut pad));
    ed.handle(&UiEvent::Dropped { from: wave, to: pad, x: 0.0, y: 0.0 });
    let file = ed.model.layer_sample(0, 2, 0).expect("the take on pad 3");
    assert!(file.starts_with(&tmp), "{file:?}");
    let _ = std::fs::remove_dir_all(&tmp);
}

/// v1 asks before a take overwrites a file layers already play; Yes
/// writes the take there.
#[test]
fn sampler_asks_before_overwriting_a_used_file() {
    crate::app::test_setup();
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    let tmp = std::env::temp_dir().join(format!("dreist-overwrite-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();
    ed.model.samples = Some(tmp.clone());
    ed.model.editor_tab = crate::front::SAMPLER_PAGE;
    ed.sync();
    let tone: Vec<f32> = (0..4410).map(|i| (i as f32 * 0.05).sin() * 0.5).collect();
    assert_eq!(ed.sampler_take(&tone, &tone, 44100.0), Some(0));
    let f = tmp.join("used.wav");
    std::fs::write(&f, b"").unwrap();
    crate::browser::load_sample_into_layer(&mut ed.model, 0, 0, 0, &f);
    assert!(crate::sampler::layers_use(&ed.model, &f));
    let anchor = ed.sampler.path_in(&ed.app, "SaveButton").unwrap();
    ed.sampler_overwrite = Some((anchor.clone(), crate::sampler::Request::ExportTake, f.clone()));
    ed.handle(&UiEvent::MenuSelected { anchor, name: "SaveButton".into(), index: 2 });
    assert!(std::fs::metadata(&f).unwrap().len() > 44, "the take was written");
    let _ = std::fs::remove_dir_all(&tmp);
}

/// v1 asks to reload samples changed outside the app; Yes reads them again.
#[test]
fn changed_samples_ask_to_reload() {
    crate::app::test_setup();
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    let tmp = std::env::temp_dir().join(format!("dreist-changed-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();
    ed.model.samples = Some(tmp.clone());
    let f = tmp.join("x.wav");
    std::fs::write(&f, b"1").unwrap();
    crate::browser::load_sample_into_layer(&mut ed.model, 0, 0, 0, &f);
    ed.watch_samples();
    assert!(ed.app.menu.is_none(), "first sight: no question");
    let later = std::time::SystemTime::now() + std::time::Duration::from_secs(5);
    std::fs::File::options().write(true).open(&f).unwrap().set_modified(later).unwrap();
    ed.watch.reset_timer();
    ed.watch_samples();
    let items = ed.app.menu.as_ref().expect("the question").items.clone();
    assert!(items[1].contains("x.wav"), "{items:?}");
    let yes = items.iter().position(|s| s == "Yes").unwrap();
    ed.handle(&UiEvent::MenuSelected { anchor: Vec::new(), name: String::new(), index: yes });
    assert!(ed.take_commands().iter().any(|c| matches!(c, Command::ReloadSamples(v) if v == &vec![f.clone()])));
    let _ = std::fs::remove_dir_all(&tmp);
}

/// v1 asks about samples removed from disk; No unloads them.
#[test]
fn removed_samples_ask_to_find_or_unload() {
    crate::app::test_setup();
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    let tmp = std::env::temp_dir().join(format!("dreist-removed-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();
    ed.model.samples = Some(tmp.clone());
    let f = tmp.join("gone.wav");
    std::fs::write(&f, b"1").unwrap();
    crate::browser::load_sample_into_layer(&mut ed.model, 0, 0, 0, &f);
    ed.watch_samples();
    std::fs::remove_file(&f).unwrap();
    ed.watch.reset_timer();
    ed.watch_samples();
    let items = ed.app.menu.as_ref().expect("the question").items.clone();
    assert!(items[0].contains("removed from the original location"), "{items:?}");
    let no = items.iter().position(|s| s == "No").unwrap();
    ed.handle(&UiEvent::MenuSelected { anchor: Vec::new(), name: String::new(), index: no });
    assert!(!ed.model.layer(0, 0, 0).valid, "unloaded");
    let _ = std::fs::remove_dir_all(&tmp);
}

/// v1 asks before Reset; No keeps the document.
#[test]
fn grid_reset_asks_first() {
    crate::app::test_setup();
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    let ob = front_path(&ed, "UtilityControls", "GridButton");
    ed.model.doc.set("Marker", "1");
    ed.handle(&UiEvent::Clicked { path: ob.clone(), name: "GridButton".into(), id: 0, on: false });
    let reset = ed.app.menu.as_ref().unwrap().items.iter().position(|s| s.starts_with("Reset ") && !s.contains("UI")).unwrap();
    ed.handle(&UiEvent::MenuSelected { anchor: ob.clone(), name: "GridButton".into(), index: reset });
    let q = ed.app.menu.as_ref().unwrap().items.clone();
    assert!(q[0].starts_with("Are you sure you want to reset"), "{q:?}");
    ed.handle(&UiEvent::MenuSelected { anchor: ob, name: "GridButton".into(), index: 3 });
    assert_eq!(ed.model.doc.attr("Marker"), Some("1"), "No keeps the document");
}

/// v1's browser clicks: presets only load on a double click; other parts
/// (here a pattern file) load on a click with Auto load.
#[test]
fn browser_click_loads_parts_and_presets_need_a_double_click() {
    crate::app::test_setup();
    use dreist_gui::app::MouseKind;
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    let dir = std::env::temp_dir().join(format!("dreist-bclick-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let preset = dir.join(format!("a.{}", dreist_formats::profile::get().save.project));
    let _ = std::fs::write(&preset, b"<x/>");
    let Some(b) = ed.browser.as_mut() else { return };
    b.auto_load = true;
    b.browse(dir.clone());
    fn by(c: &Ctrl, path: &mut Vec<usize>) -> Option<Vec<usize>> {
        if c.overload == "BrowserFileList" {
            return Some(path.clone());
        }
        for (i, k) in c.children.iter().enumerate() {
            path.push(i);
            if let Some(p) = by(k, path) {
                return Some(p);
            }
            path.pop();
        }
        None
    }
    let list = by(&ed.app.root, &mut Vec::new()).unwrap();
    ed.take_commands();
    let ev = |kind| UiEvent::Mouse { path: list.clone(), overload: "BrowserFileList".into(), kind, x: 5.0, y: 2.0 };
    ed.handle(&ev(MouseKind::Down));
    assert!(!ed.take_commands().iter().any(|c| matches!(c, Command::LoadProject(_))), "one click: no preset load");
    let _ = std::fs::remove_dir_all(&dir);
}

/// v1's link keys: alt edits all 16 pads, alt+shift the pad's row.
#[test]
fn link_keys_pick_pads() {
    crate::app::test_setup();
    let mut m = Model::new(project::new_document());
    assert_eq!(multipad::edit_pads(&m, 5), None);
    m.mod_alt = true;
    assert_eq!(multipad::edit_pads(&m, 5), Some(0xFFFF));
    m.mod_shift = true;
    assert_eq!(multipad::edit_pads(&m, 5), Some(0xF0));
    m.mod_alt = false;
    m.all_rows = true;
    assert_eq!(multipad::edit_pads(&m, 13), Some(0xF000));
}

/// v1: dragging a step selection moves it; alt copies it.
#[test]
fn step_selection_drags() {
    crate::app::test_setup();
    use dreist_engine::binding::id;
    use dreist_gui::app::MouseKind;
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    ed.model.editor_tab = 0;
    ed.sync();
    let gp = ed.app.root.path_of("PatternGrid").unwrap();
    let lay = ed.pattern.as_ref().unwrap().layout(&ed.app, &ed.model).unwrap();
    let at = |s: usize| (lay.step_x(s) + 2.0, lay.row_y(lay.row(3).unwrap()) + 2.0);
    let ev = |kind, (x, y): (f32, f32)| UiEvent::Mouse { path: gp.clone(), overload: "PatternGrid".into(), kind, x, y };
    // v1: the Pen always draws; the Multi tool drags a selection.
    ed.pattern.as_mut().unwrap().tool = crate::pattern::Tool::Multi;
    ed.handle(&ev(MouseKind::Down, at(2)));
    ed.handle(&ev(MouseKind::Up, at(2)));
    ed.pattern.as_mut().unwrap().selection = [(3, 2)].into_iter().collect();
    ed.take_commands();
    ed.handle(&ev(MouseKind::Down, at(2)));
    ed.handle(&ev(MouseKind::Drag, at(5)));
    ed.handle(&ev(MouseKind::Up, at(5)));
    let e = ed.model.engine;
    let pat = ed.model.pattern(e, ed.model.current_pattern(e));
    assert!(pat.tracks[3].steps[5].on && !pat.tracks[3].steps[2].on, "moved");
    assert!(ed.take_commands().iter().any(|c| matches!(c, Command::Message(m) if m.kind == id::SET_STEP_OFF)));
    // Alt copies.
    ed.app.mods.alt = true;
    ed.handle(&ev(MouseKind::Down, at(5)));
    ed.handle(&ev(MouseKind::Up, at(7)));
    ed.app.mods.alt = false;
    let pat = ed.model.pattern(e, ed.model.current_pattern(e));
    assert!(pat.tracks[3].steps[5].on && pat.tracks[3].steps[7].on, "copied");
}

/// With steps selected the nudge buttons move only those (v1); without, the
/// whole pattern.
#[test]
fn nudge_buttons_move_the_selection() {
    crate::app::test_setup();
    use dreist_engine::binding::id;
    let Some(mut ed) = demo_editor("Presets/Hollow.geist") else { return };
    ed.model.editor_tab = 0;
    ed.sync();
    let next = ed.app.root.path_of("NudgeNext").expect("nudge button");
    ed.take_commands();
    click_named(&mut ed, &next, false);
    let sent = |c: &[Command], k: u32| c.iter().any(|c| matches!(c, Command::Message(m) if m.kind == k));
    assert!(sent(&ed.take_commands(), id::NUDGE_PATTERN_FORWARD), "no selection: the pattern");
    ed.app.mods.cmd = true;
    for ev in ed.app.shortcut("a") {
        ed.handle(&ev);
    }
    ed.app.mods.cmd = false;
    assert!(ed.pattern.as_ref().unwrap().has_selection());
    ed.take_commands();
    click_named(&mut ed, &next, false);
    let c = ed.take_commands();
    assert!(!sent(&c, id::NUDGE_PATTERN_FORWARD) && !c.is_empty(), "only the selected steps move");
}

/// Showing the Mapping page turns MIDI learn on and leaving it turns it off
/// (v1's mapping panel sets the learn state with its own visibility).
#[test]
fn mapping_page_turns_learn_on() {
    crate::app::test_setup();
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    let bp = ed.app.root.path_of("BrowserTabButtons").expect("tab buttons");
    let n = ed.app.ctrl(&bp).unwrap().children.len();
    let tab = |ed: &Editor, page: usize| {
        (0..n)
            .map(|i| [bp.clone(), vec![i]].concat())
            .find(|p| ed.utility_tabs.as_ref().unwrap().clicked(&ed.app, &UiEvent::Clicked { path: p.clone(), name: String::new(), id: 0, on: false }) == Some(page))
            .expect("tab")
    };
    let (map, browse) = (tab(&ed, front::MAPPING_PAGE), tab(&ed, front::BROWSER_PAGE));
    ed.handle(&UiEvent::Clicked { path: map, name: String::new(), id: 0, on: true });
    assert!(ed.learn.active && ed.model.utility_tab == front::MAPPING_PAGE);
    ed.handle(&UiEvent::Clicked { path: browse, name: String::new(), id: 0, on: true });
    assert!(!ed.learn.active);
}

/// A drag dwelling on an engine selector selects that engine, and entering
/// the pads shows the Pad / Layers page (v1, 25 ticks).
#[test]
fn drag_dwell_selects_engine_and_shows_pads() {
    crate::app::test_setup();
    let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
    ed.model.editor_tab = front::PATTERN_PAGE;
    ed.sync();
    let from = ed.app.root.path_of("PatternSelectors").map(|p| [p, vec![0]].concat()).expect("pattern keys");
    let sel = [ed.header.selectors.clone().unwrap(), vec![3]].concat();
    ed.handle(&UiEvent::DragOver { from: from.clone(), to: sel, x: 0.0, y: 0.0 });
    ed.animate();
    assert_eq!(ed.model.engine, 0, "not yet");
    std::thread::sleep(std::time::Duration::from_millis(900));
    ed.animate();
    assert_eq!(ed.model.engine, 3);
    let pad = [ed.pads_matrix().unwrap(), vec![0]].concat();
    ed.handle(&UiEvent::DragOver { from, to: pad, x: 0.0, y: 0.0 });
    std::thread::sleep(std::time::Duration::from_millis(900));
    ed.animate();
    assert_eq!(ed.model.editor_tab, front::PADS_PAGE);
}

/// v1's automation Live mode: the knobs set the lane's track amounts
/// instead of rewriting the steps.
#[test]
fn automation_live_mode_sets_track_amounts() {
    crate::app::test_setup();
    let Some(mut ed) = demo_editor("Presets/Hollow.geist") else { return };
    if let Some(pp) = ed.pattern.as_mut() {
        pp.multi = false;
    }
    ed.sync();
    let ac = ed.app.root.path_of("SingleAutoControl").unwrap();
    let group = [ac.clone(), ed.app.ctrl(&ac).unwrap().path_of("ModeGroup").expect("mode switch")].concat();
    assert!(ed.app.ctrl(&group).unwrap().visible);
    let live = [group, vec![1]].concat();
    ed.handle(&UiEvent::Clicked { path: live, name: "CtrlButton".into(), id: 0, on: true });
    let knob = [ac.clone(), ed.app.ctrl(&ac).unwrap().path_of("ScaleKnob").unwrap()].concat();
    ed.take_commands();
    ed.handle(&UiEvent::ValueChanged { path: knob, name: "ScaleKnob".into(), id: 0, value: 0.8 });
    let e = ed.model.engine;
    let lane = dreist_formats::params::STEP_LANES[ed.model.pattern_view[e].lane];
    let t = crate::padmenu::track_node(&ed.model, e, ed.model.pad[e]).expect("track");
    assert_eq!(t.f32(&format!("{lane}Scal")), Some(0.8));
    assert!(!ed.take_commands().iter().any(|c| matches!(c, Command::Message(m) if m.kind == dreist_engine::binding::id::SET_STEP_GRAPH_STATE)), "no step rewrite");
}
