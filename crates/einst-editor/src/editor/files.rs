//! The file menu: projects and their parts, pattern MIDI files, Guru
//! swing and MIDI groove templates.

use crate::*;

/// A swing template import in progress.
pub(crate) struct SwingImport {
    guru: bool,
    files: std::collections::VecDeque<std::path::PathBuf>,
    question: Option<Question>,
}

enum Question {
    /// The file's template exists: overwrite it?
    Overwrite(std::path::PathBuf),
    /// A file failed: go on with the rest?
    Continue,
}

impl Editor {
    /// The master section's LOAD and SAVE menus ([re:be6692]
    /// [re:d52ca7]): LOAD has Load preset, engine, kit, pad, pattern bank,
    /// pattern, pattern from MIDI, mappings and Load Guru... ▸ (Preset, Kit,
    /// Pattern, Pattern Bank); SAVE has Save preset, current engine, kit,
    /// current pad, pattern bank, current pattern, current pattern as MIDI,
    /// mappings, Export as audio... and the check item Save samples with
    /// presets. Every save asks for the file.
    pub(crate) fn file_menu(&mut self, ev: &UiEvent) -> bool {
        use project::Part;
        let e = self.model.engine;
        let p = self.model.pad[e];
        let pat = self.model.current_pattern(e);
        match ev {
            UiEvent::Clicked { path, name, .. } if name == "LoadButton" => {
                let items = vec![
                    ("Load preset".to_string(), FileAction::OpenPreset),
                    ("Load engine".into(), FileAction::Open(Part::Engine(e))),
                    ("Load kit".into(), FileAction::Open(Part::Kit(e))),
                    ("Load pad".into(), FileAction::Open(Part::Pad(e, p))),
                    ("Load pattern bank".into(), FileAction::Open(Part::Bank(e))),
                    ("Load pattern".into(), FileAction::Open(Part::Pattern(e, pat))),
                    ("Load pattern from MIDI".into(), FileAction::OpenMidi),
                    ("Load mappings".into(), FileAction::Open(Part::Mappings)),
                    ("Load Guru... ▸".into(), FileAction::GuruMenu),
                ];
                self.show_file_menu(path, items);
                false
            }
            // (The sampler page has a SaveButton of its own.)
            UiEvent::Clicked { path, name, .. } if name == "SaveButton" && front::path_in(&self.app.root, "UtilityControls", "SaveButton").as_ref() == Some(path) => {
                let items = vec![
                    ("Save preset".to_string(), FileAction::SavePreset { as_new: true }),
                    ("Save current engine".into(), FileAction::Save { part: Part::Engine(e), as_new: true }),
                    ("Save kit".into(), FileAction::Save { part: Part::Kit(e), as_new: true }),
                    ("Save current pad".into(), FileAction::Save { part: Part::Pad(e, p), as_new: true }),
                    ("Save pattern bank".into(), FileAction::Save { part: Part::Bank(e), as_new: true }),
                    ("Save current pattern".into(), FileAction::Save { part: Part::Pattern(e, pat), as_new: true }),
                    ("Save current pattern as MIDI".into(), FileAction::SaveMidi),
                    ("Save mappings".into(), FileAction::Save { part: Part::Mappings, as_new: true }),
                    (String::new(), FileAction::None),
                    ("Export as audio...".into(), FileAction::ExportAudio),
                    (String::new(), FileAction::None),
                ];
                let mut items = items;
                // [re:d52ca7]: the check item for option
                // `SaveSamplesWithChunks` ([re:c2ab69] toggles it).
                let on = project::embed_samples();
                let label = if on { format!("{}Save samples with presets", dreist_gui::menu::CHECK) } else { "Save samples with presets".into() };
                items.push((label, FileAction::ToggleEmbedSamples));
                self.show_file_menu(path, items);
                false
            }
            UiEvent::MenuSelected { anchor, name, index } if name == "LoadButton" || name == "SaveButton" => {
                let Some(actions) = self.file_menu.take() else { return false };
                let action = actions.get(*index).cloned().unwrap_or(FileAction::None);
                self.file_action(anchor, action)
            }
            _ => false,
        }
    }

    /// Run a file menu item (`anchor`: the menu's button, for submenus).
    pub(crate) fn file_action(&mut self, anchor: &[usize], action: FileAction) -> bool {
        use project::Part;
        let e = self.model.engine;
        let pat = self.model.current_pattern(e);
        match action {
            FileAction::None => return false,
            FileAction::ToggleEmbedSamples => {
                let mut o = options::Options::load();
                let on = o.bool("SaveSamplesWithChunks", false);
                o.set("SaveSamplesWithChunks", !on);
                o.save();
                // v1 shows the menu again.
                let ev = UiEvent::Clicked { path: anchor.to_vec(), name: "SaveButton".into(), id: 0, on: false };
                self.file_menu(&ev);
                return false;
            }
            FileAction::GuruMenu => {
                let items = [("Preset", guru::Kind::Preset), ("Kit", guru::Kind::Kit), ("Pattern", guru::Kind::Pattern), ("Pattern Bank", guru::Kind::Bank)];
                let items = items.into_iter().map(|(l, k)| (l.to_string(), FileAction::OpenGuru(k))).collect();
                self.show_file_menu(anchor, items);
                return false;
            }
            FileAction::OpenGuru(kind) => {
                let Some(f) = self.dialog(Dialog::OpenGuru(kind)) else { return false };
                let part = match kind {
                    guru::Kind::Preset => {
                        self.outbox.push(Command::LoadProject(f));
                        return true;
                    }
                    guru::Kind::Kit => Part::Kit(e),
                    guru::Kind::Bank => Part::Bank(e),
                    guru::Kind::Pattern => Part::Pattern(e, pat),
                };
                if !self.open_part(part, &f) {
                    return false;
                }
            }
            FileAction::ExportAudio => {
                return self.main_action(anchor, crate::editor::tools_menu::MainAction::Export);
            }
            FileAction::OpenPreset => {
                if let Some(f) = self.dialog(Dialog::OpenProject) {
                    self.outbox.push(Command::LoadProject(f));
                }
            }
            FileAction::Open(part) => {
                let Some(f) = self.dialog(Dialog::OpenPart(part)) else { return false };
                if !self.open_part(part, &f) {
                    return false;
                }
            }
            FileAction::OpenMidi => {
                let Some(f) = self.dialog(Dialog::OpenMidi) else { return false };
                if !self.open_pattern_midi(&f) {
                    return false;
                }
            }
            FileAction::SaveMidi => {
                let Some(f) = self.dialog(Dialog::SaveMidi) else { return false };
                let f = if f.extension().is_none() { f.with_extension("mid") } else { f };
                self.save_pattern_midi(&f);
            }
            FileAction::SavePreset { as_new } => {
                let target = if as_new { None } else { self.project_path.clone() };
                let target = target.or_else(|| self.dialog(Dialog::SaveProject));
                if let Some(f) = target {
                    let f = if f.extension().is_none() { f.with_extension(dreist_formats::profile::get().save.project) } else { f };
                    if self.save_to(&f) && !cfg!(test) {
                        add_recent_preset(&f);
                    }
                }
            }
            FileAction::Save { part, as_new } => {
                let (ext, ..) = part.file_type();
                let key = (ext, e);
                let target = if as_new { None } else { self.part_paths.get(&key).cloned() };
                let Some(f) = target.or_else(|| self.dialog(Dialog::SavePart(part))) else { return false };
                let f = if f.extension().is_none() { f.with_extension(ext) } else { f };
                if project::save_part(&self.model.doc, self.model.samples.as_deref(), part, &f, project::embed_samples()).is_ok() {
                    self.part_paths.insert(key, f);
                }
            }
        }
        true
    }

    /// [re:536a37] → [re:3a91db]
    /// for the selected engine's current pattern. With option `AutoMIDICC`
    /// (default on) graphs that have data but no MIDI binding get one on a
    /// free CC of the engine's channel ([re:521ed5]: 1..127
    /// without bindings, skipping 7 and 10;
    /// [re:b99595]).
    pub fn save_pattern_midi(&mut self, path: &std::path::Path) -> bool {
        let cur = self.model.current_pattern(self.model.engine);
        self.save_pattern_midi_of(path, cur)
    }

    /// Pattern `cur` of the current engine as a MIDI file
    /// (also the pattern key's drag-out file).
    pub fn save_pattern_midi_of(&mut self, path: &std::path::Path, cur: usize) -> bool {
        use dreist_engine::binding::{Message, id};
        let e = self.model.engine;
        let pattern = self.model.pattern(e, cur);
        let eng = self.model.engine_node(e).cloned().unwrap_or_else(|| dreist_formats::Node::new("Engine", None));
        let swing = eng.children_of("SwingTemplate").next().map(dreist_engine::pattern::Swing::from_node).unwrap_or_default();
        let num = eng.u32("TempoNumerator").unwrap_or(1).max(1) as f64;
        let den = eng.u32("TempoDenominator").unwrap_or(1).max(1) as f64;
        let time = self.model.doc.children.iter().find(|c| c.class == "TimeInfo");
        let bpm = time.and_then(|t| t.f32("Tempo")).unwrap_or(120.0) as f64;
        // The project's time signature ([re:ee9d6e]).
        let time_sig = (time.and_then(|t| t.u32("TSN")).unwrap_or(4).max(1), time.and_then(|t| t.u32("TSD")).unwrap_or(4).max(1));
        let one_shot: Vec<bool> = (0..64)
            .map(|p| {
                !(0..model::LAYERS).any(|l| self.model.layer(e, p, l).valid && self.model.layer_node(e, p, l).and_then(|n| n.attr("PlaybackMode")).is_some_and(|m| matches!(m, "Gated" | "Looping")))
            })
            .collect();
        let auto_cc = options::Options::load().bool("AutoMIDICC", true);
        let x = dreist_engine::midifile::Export {
            pattern: &pattern,
            engine: e,
            bpm,
            time_sig,
            tempo_mult: num / den,
            swing: &swing,
            swing_depth: eng.f32("SwingTemplateDepth").unwrap_or(0.0) as f64,
            one_shot: &one_shot,
            auto_cc,
        };
        let mut added = Vec::new();
        let doc = &mut self.model.doc;
        let mut cc_for = |pad: usize, lane: usize, allocate: bool| -> Option<u8> {
            let rows = midimap::rows(doc);
            let bound = rows.iter().find_map(|r| {
                let m = r.proto.as_ref()?;
                let midimap::Key::Midi(_, cc) = r.key else { return None };
                (m.kind == id::SET_INSTO_MIDI_GRAPH_VALUE && m.index[0] == e as i32 && m.index[3] == cur as i32 && m.index[4] == pad as i32 && m.value[1] as usize == lane).then_some(cc as u8)
            });
            if bound.is_some() || !allocate {
                return bound;
            }
            let ch = e & 15;
            let used = |cc: usize| rows.iter().any(|r| r.key == midimap::Key::Midi(ch, cc));
            let cc = (1..128).find(|&cc| cc != 7 && cc != 10 && !used(cc))?;
            let proto = Message::new(id::SET_INSTO_MIDI_GRAPH_VALUE, &[e as i32, pad as i32, -1, cur as i32, pad as i32], &[0.0, lane as f32]);
            let name = dreist_formats::STEP_LANES.get(lane).copied().unwrap_or("");
            let node = midimap::add_binding(doc, true, ch, cc, &proto, 0, &format!("Engine {} | Pattern {} | Track {}", e + 1, cur + 1, pad + 1), name, "Continuous");
            added.push(node);
            Some(cc as u8)
        };
        let mut seed = 0x2545_f491u32;
        let mut rng = || {
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            (seed >> 8) as f32 / (1u32 << 24) as f32
        };
        let bytes = dreist_engine::midifile::export(&x, &mut cc_for, &mut rng);
        for b in added.iter().filter_map(dreist_engine::binding::NewBinding::from_node) {
            self.outbox.push(Command::AddBinding(Box::new(b)));
        }
        dreist_formats::file::write_atomic(path, bytes).is_ok()
    }

    /// [re:9b2c04] → [re:75cf76]
    /// into the selected engine's current pattern (not when it is locked),
    /// named after the file.
    pub fn open_pattern_midi(&mut self, path: &std::path::Path) -> bool {
        let cur = self.model.current_pattern(self.model.engine);
        self.open_pattern_midi_at(path, cur)
    }

    /// Import a MIDI file into pattern slot `cur` of the selected engine.
    pub(crate) fn open_pattern_midi_at(&mut self, path: &std::path::Path, cur: usize) -> bool {
        use dreist_engine::binding::id;
        let Ok(bytes) = std::fs::read(path) else { return false };
        let e = self.model.engine;
        let rows = midimap::rows(&self.model.doc);
        let cc_lanes = |cc: u8| -> Vec<(usize, usize)> {
            rows.iter()
                .filter(|r| r.key == midimap::Key::Midi(0, cc as usize))
                .filter_map(|r| r.proto.as_ref().filter(|m| m.kind == id::SET_INSTO_MIDI_GRAPH_VALUE).map(|m| (m.index[4].max(0) as usize, m.value[1].max(0.0) as usize)))
                .collect()
        };
        let name = path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        let Some(mut sp) = dreist_engine::midifile::import(&bytes, &name, &cc_lanes) else { return false };
        let eng = self.model.engine_mut(e);
        let slot = eng.child_mut_or_insert("StepPattern", &format!("StepPattern{cur}"));
        if slot.locked() {
            return false;
        }
        sp.name = Some(format!("StepPattern{cur}"));
        *slot = sp;
        self.outbox.push(self.model.rebuild_command());
        self.sync();
        true
    }

    /// Where an imported swing template goes:
    /// `<user swing folder>/<file stem>.swingtemplate`.
    fn swing_target(file: &std::path::Path) -> Option<std::path::PathBuf> {
        Some(swing::user_dir()?.join(format!("{}.swingtemplate", file.file_stem()?.to_string_lossy())))
    }

    /// [re:736dd8] into the user swing folder.
    pub fn import_midi_template(&mut self, file: &std::path::Path) -> Option<std::path::PathBuf> {
        let bytes = std::fs::read(file).ok()?;
        let name = file.file_stem()?.to_string_lossy().into_owned();
        let sp = dreist_engine::midifile::import(&bytes, &name, &|_| Vec::new())?;
        let pat = dreist_engine::pattern::PatternData::from_node(&sp);
        let t = swing::from_pattern(&pat, &name);
        let out = Self::swing_target(file)?;
        let _ = std::fs::create_dir_all(out.parent()?);
        swing::save(&out, &t).ok()?;
        Some(out)
    }

    /// A Guru 1 swing template converted to a grid swing template in the
    /// user swing folder; a failure alerts when it is the last file.
    pub fn import_guru_swing(&mut self, file: &std::path::Path, alert: bool) -> Option<std::path::PathBuf> {
        let t = match guru::convert_swing_text(file) {
            Ok(t) => t,
            Err(e) => {
                if let (true, Some(a)) = (alert, self.app.root.path_of("SwingDrop")) {
                    self.app.open_menu(&a, vec![e], None);
                }
                return None;
            }
        };
        let out = Self::swing_target(file)?;
        let _ = std::fs::create_dir_all(out.parent()?);
        swing::save(&out, &t).ok()?;
        Some(out)
    }

    /// The swing import ([re:44230b] MIDI, [re:bdda3c] Guru): each chosen
    /// file in turn; an existing template asks "A file named %s already
    /// exists in your swing template folder. Do you want to overwrite it?"
    /// (No skips it), and a failed file before the last asks "Do you want
    /// to continue trying to import the other files?".
    pub(crate) fn start_swing_import(&mut self, guru: bool, files: Vec<std::path::PathBuf>) {
        self.swing_import = Some(SwingImport { guru, files: files.into(), question: None });
        self.next_swing_import(None);
    }

    /// Carry on with the import: `answer` is the open question's (true
    /// Yes).
    pub(crate) fn next_swing_import(&mut self, answer: Option<bool>) {
        let Some(mut job) = self.swing_import.take() else { return };
        let mut convert_now = match (job.question.take(), answer) {
            (Some(Question::Overwrite(f)), Some(true)) => Some(f),
            (Some(Question::Continue), Some(false)) => return,
            _ => None,
        };
        loop {
            let file = match convert_now.take() {
                Some(f) => f,
                None => {
                    let Some(f) = job.files.pop_front() else { return };
                    if Self::swing_target(&f).is_some_and(|t| t.exists()) {
                        let name = f.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
                        let text = format!("A file named {name} already exists in your swing template folder. Do you want to overwrite it?");
                        job.question = Some(Question::Overwrite(f));
                        self.ask_swing_import(job, text);
                        return;
                    }
                    f
                }
            };
            let last = job.files.is_empty();
            let ok = if job.guru { self.import_guru_swing(&file, last).is_some() } else { self.import_midi_template(&file).is_some() };
            if !ok && !last {
                job.question = Some(Question::Continue);
                self.ask_swing_import(job, "Do you want to continue trying to import the other files?".into());
                return;
            }
        }
    }

    fn ask_swing_import(&mut self, job: SwingImport, text: String) {
        let anchor = self.app.root.path_of("SwingDrop").unwrap_or_default();
        self.app.open_menu(&anchor, vec![dreist_gui::menu::item(text, false), String::new(), "Yes".into(), "No".into()], None);
        self.swing_import = Some(job);
    }

    /// A pick in the import's question menu (Yes 2, No 3); dismissing the
    /// question ends the import.
    pub(crate) fn swing_import_event(&mut self, ev: &UiEvent) -> bool {
        if self.swing_import.as_ref().is_none_or(|j| j.question.is_none()) {
            return false;
        }
        match ev {
            UiEvent::MenuSelected { index, .. } => {
                self.next_swing_import(Some(*index == 2));
                true
            }
            _ if self.app.menu.is_none() => {
                self.swing_import = None;
                false
            }
            _ => false,
        }
    }

    pub(crate) fn show_file_menu(&mut self, anchor: &[usize], items: Vec<(String, FileAction)>) {
        let labels = items.iter().map(|(l, _)| l.clone()).collect();
        self.file_menu = Some(items.into_iter().map(|(_, a)| a).collect());
        self.app.open_menu(anchor, labels, None);
    }

    /// Load a part file into the document and rebuild the engine.
    pub(crate) fn open_part(&mut self, part: project::Part, file: &std::path::Path) -> bool {
        // Guru files ([re:dda000] → the Guru loaders).
        if let Some(k) = guru::kind(file) {
            let e = self.model.engine;
            let cur = self.model.current_pattern(e);
            let r = match (k, part) {
                (guru::Kind::Kit, _) => guru::import_kit(&mut self.model.doc, e, file),
                (guru::Kind::Bank, _) => guru::import_bank(&mut self.model.doc, e, file),
                (guru::Kind::Pattern, project::Part::Pattern(_, slot)) => guru::import_pattern(&mut self.model.doc, e, slot, file),
                (guru::Kind::Pattern, _) => guru::import_pattern(&mut self.model.doc, e, cur, file),
                (guru::Kind::Preset, _) => {
                    self.outbox.push(Command::LoadProject(file.to_path_buf()));
                    return true;
                }
            };
            if r.is_ok() {
                self.outbox.push(self.model.rebuild_command());
                self.sync();
            }
            return r.is_ok();
        }
        let ok = match part {
            project::Part::Kit(_) => browser::load_kit_into_doc(&mut self.model, file),
            _ => project::load_part(&mut self.model.doc, self.model.samples.as_deref(), part, file),
        };
        if ok {
            let (ext, ..) = part.file_type();
            self.part_paths.insert((ext, self.model.engine), file.to_path_buf());
            self.outbox.push(self.model.rebuild_command());
            self.sync();
        }
        ok
    }

    /// Save the document (collecting samples) and adopt the saved copy.
    pub fn save_to(&mut self, path: &std::path::Path) -> bool {
        match project::save(&self.model.doc, self.model.samples.as_deref(), path, project::embed_samples()) {
            Ok((doc, _errors)) => {
                self.model.doc = doc;
                self.model.samples = Some(dreist_formats::model::samples_dir(path));
                self.model.name = path.file_stem().map(|s| s.to_string_lossy().into_owned());
                self.project_path = Some(path.to_path_buf());
                self.outbox.push(Command::Saved(path.to_path_buf()));
                self.sync();
                true
            }
            Err(_) => false,
        }
    }

    /// Replace the document with a newly opened project.
    pub fn open_document(&mut self, doc: dreist_formats::Node, samples: Option<std::path::PathBuf>, name: Option<String>) {
        self.history.clear();
        self.set_document(doc);
        self.model.samples = samples;
        self.model.name = name;
        self.model.pad = [0; model::ENGINES];
        self.model.layer = [0; model::ENGINES];
        // The project's sampler settings become the remembered ones.
        let msgs = self.sampler.adopt_project(&mut self.model);
        for c in &msgs {
            self.model.apply(c);
        }
        self.outbox.extend(msgs);
        // [re:af38f1] when samples are missing.
        self.filefind.show(filefind::missing_files(&self.model));
        self.sync();
    }
}
