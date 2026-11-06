//! Keyboard accelerators and the standalone's menu bar.

use crate::*;

/// Menu bar tags beyond the accelerator ids.
const MENU_ABOUT: isize = 1000;

pub(crate) const MENU_AUDIO: isize = 1001;

pub(crate) const MENU_MIDI: isize = 1002;

impl Editor {
    /// [re:ed265e] (see `accel`): run the key's
    /// function. v1's pattern and song editors take no keys of their own
    /// (the pattern panel swallows them, [re:8a87e6]; the song panel only
    /// re-runs a drag, [re:313afa]).
    pub(crate) fn accelerator(&mut self, key: &str, mods: accel::Mods) -> bool {
        let o = &self.modals.options;
        let Some(e) = accel::lookup(key, mods, o.bool("RespondToKeyboard", true), o.bool("RespondToExtendedKeyboard", false), self.menu_bar) else { return false };
        self.run_action(e.action);
        true
    }

    pub(crate) fn run_action(&mut self, action: accel::Action) {
        match action {
            accel::Action::Nothing => {}
            accel::Action::Toggle(button) | accel::Action::Press(button) => {
                let Some(p) = self.app.root.path_of("Transport").and_then(|h| Some([h.clone(), (self.app.ctrl(&h)?).path_of(button)?].concat())) else { return };
                let Some(c) = self.app.ctrl_mut(&p) else { return };
                if matches!(action, accel::Action::Toggle(_)) {
                    c.on = !c.on;
                }
                let ev = UiEvent::Clicked { path: p, name: c.name.clone(), id: c.id, on: c.on };
                self.handle(&ev);
            }
            accel::Action::Editor(id) => self.accelerator_message(id),
            accel::Action::View(n) => {
                self.model.editor_tab = n;
                self.sync();
            }
        }
    }

    /// The standalone's menu bar ([re:ed265e]: the
    /// entry's function, whatever the keyboard options say).
    pub fn menu_bar_items(&self) -> (Vec<dreist_gui::menubar::Item>, Vec<(String, Vec<dreist_gui::menubar::Item>)>) {
        use dreist_gui::menubar::Item;
        fn item(id: u32) -> Item {
            let (title, _) = accel::entry(id).unwrap_or(("", accel::Action::Nothing));
            let k = accel::TABLE.iter().find(|e| e.id == id);
            Item {
                title: title.into(),
                key: k.map(|k| k.key.to_string()).unwrap_or_default(),
                shift: k.is_some_and(|k| k.mods.shift),
                cmd: k.is_some_and(|k| k.mods.cmd),
                alt: k.is_some_and(|k| k.mods.alt),
                ctrl: k.is_some_and(|k| k.mods.ctrl),
                tag: id as isize,
                submenu: Vec::new(),
            }
        }
        fn items(m: &[accel::Menu]) -> Vec<Item> {
            m.iter()
                .map(|e| match e {
                    accel::Menu::Item(id) => item(*id),
                    accel::Menu::Separator => Item::separator(),
                })
                .collect()
        }
        let app = vec![
            Item { title: format!("About {}", crate::product()), tag: MENU_ABOUT, ..Item::default() },
            Item::separator(),
            item(0x00),
            Item { title: "Audio Settings...".into(), tag: MENU_AUDIO, ..Item::default() },
            Item { title: "MIDI Settings...".into(), tag: MENU_MIDI, ..Item::default() },
        ];
        (app, accel::MENUS.iter().map(|(t, m)| (t.to_string(), items(m))).collect())
    }

    /// A menu bar item was chosen.
    pub fn menu_command(&mut self, tag: isize) {
        match tag {
            MENU_ABOUT => {
                self.main_action(&[], crate::editor::tools_menu::MainAction::About);
            }
            MENU_AUDIO => self.devices.show(devices::Panel::Audio),
            MENU_MIDI => self.devices.show(devices::Panel::Midi),
            t => {
                if let Some((_, a)) = u32::try_from(t).ok().and_then(accel::entry) {
                    self.run_action(a);
                }
            }
        }
        self.sync();
        self.app.dirty = true;
    }

    /// The editor messages the shortcuts post ([re:695294]).
    pub(crate) fn accelerator_message(&mut self, id: u32) {
        use project::Part;
        let e = self.model.engine;
        let menu = |ed: &mut Editor, name: &str, index: usize| ed.handle(&UiEvent::MenuSelected { anchor: Vec::new(), name: name.into(), index });
        let file = |ed: &mut Editor, action: FileAction| {
            ed.file_menu = Some(vec![action]);
            menu(ed, "LoadButton", 0);
        };
        match id {
            0x16a => {
                self.main_action(&[], crate::editor::tools_menu::MainAction::Preferences);
            }
            0x16c => file(self, FileAction::OpenPreset),
            0x16d => file(self, FileAction::Open(Part::Engine(e))),
            0x16e => file(self, FileAction::Open(Part::Kit(e))),
            0x16f => file(self, FileAction::Open(Part::Pad(e, self.model.pad[e]))),
            0x170 => file(self, FileAction::Open(Part::Bank(e))),
            0x171 => file(self, FileAction::Open(Part::Pattern(e, self.model.current_pattern(e)))),
            0x172 => file(self, FileAction::OpenMidi),
            0x173 => file(self, FileAction::Open(Part::Mappings)),
            0x175 => file(self, FileAction::Save { part: Part::Engine(e), as_new: false }),
            0x177 => file(self, FileAction::Save { part: Part::Engine(e), as_new: true }),
            0x17a => file(self, FileAction::Save { part: Part::Pad(e, self.model.pad[e]), as_new: true }),
            0x17b => file(self, FileAction::Save { part: Part::Bank(e), as_new: true }),
            0x17c => file(self, FileAction::Save { part: Part::Pattern(e, self.model.current_pattern(e)), as_new: true }),
            0x17d => file(self, FileAction::SaveMidi),
            0x17e => file(self, FileAction::Save { part: Part::Mappings, as_new: true }),
            0x17f => {
                self.main_action(&[], crate::editor::tools_menu::MainAction::Export);
            }
            0x174 => file(self, FileAction::SavePreset { as_new: false }),
            0x176 => file(self, FileAction::SavePreset { as_new: true }),
            0x178 => file(self, FileAction::Save { part: Part::Kit(e), as_new: false }),
            0x179 => file(self, FileAction::Save { part: Part::Kit(e), as_new: true }),
            // Reset UI size: the window has one size.
            0x152 => {}
            0x153 => {
                self.main_action(&[], crate::editor::tools_menu::MainAction::ResetScale);
            }
            // Header Retro (`e_grabRetroRecording`).
            0x23 => {
                if let Some(p) = self.app.root.path_of("Transport").and_then(|h| Some([h.clone(), (self.app.ctrl(&h)?).path_of("RetroButton")?].concat())) {
                    let ev = UiEvent::Clicked { path: p, name: "RetroButton".into(), id: 0, on: false };
                    self.handle(&ev);
                }
            }
            // [re:480aae]: Filter starts editing
            // the search box; Find is the box menu's "Advanced search..."
            // (its listeners are notified [re:80cb0a] → the pane toggles).
            0x160 | 0x161 => {
                self.binding_message(&dreist_engine::binding::Message::new(0x14e, &[], &[1.0]));
                let Some(b) = &mut self.browser else { return };
                if id == 0x160 {
                    let page = b.page().to_vec();
                    if let Some(q) = self.app.ctrl(&page).and_then(|c| c.path_of("SearchBox")) {
                        self.app.begin_edit(&[page, q].concat());
                    }
                } else {
                    b.toggle_search_pane(&mut self.app);
                }
            }
            _ => self.binding_message(&dreist_engine::binding::Message::new(id, &[], &[1.0])),
        }
    }
}
