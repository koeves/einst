//! The main menu on the master section's `GridButton` ("EINST▼"): the
//! original's Tools ([re:4139f8] [re:a00968]) and Help ([re:0316e2])
//! items and the transport options (`header`).

use crate::*;

/// The main menu's items, in order.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum MainAction {
    Preferences,
    About,
    Reset,
    SaveDefault,
    ClearDefault,
    FindMissing,
    Export,
    MidiConsole,
    TransportOptions,
    ResetScale,
    None,
}

impl Editor {
    fn main_items(&self) -> Vec<(String, MainAction)> {
        use MainAction::*;
        vec![
            ("Preferences".into(), Preferences),
            ("Transport options ▸".into(), TransportOptions),
            (String::new(), None),
            (format!("Reset {}", crate::product()), Reset),
            ("Save as default preset".into(), SaveDefault),
            ("Clear default preset".into(), ClearDefault),
            (String::new(), None),
            ("Find missing files...".into(), FindMissing),
            ("Export as audio...".into(), Export),
            ("Show MIDI console".into(), MidiConsole),
            (String::new(), None),
            ("Reset UI scale to 100%".into(), ResetScale),
            (String::new(), None),
            (format!("About {}", crate::product()), About),
        ]
    }

    /// The main menu (`GridButton`).
    pub(crate) fn tools_help_menu(&mut self, ev: &UiEvent) -> bool {
        match ev {
            UiEvent::Clicked { path, name, .. } if name == "GridButton" => {
                let items = self.main_items();
                self.main_menu = true;
                self.app.open_menu(path, items.into_iter().map(|(l, _)| l).collect(), None);
                false
            }
            UiEvent::MenuSelected { anchor, name, index } if name == "GridButton" && std::mem::take(&mut self.main_menu) => {
                let action = self.main_items().get(*index).map(|(_, a)| *a).unwrap_or(MainAction::None);
                self.main_action(anchor, action)
            }
            _ => false,
        }
    }

    /// Run a main menu item (`anchor`: the menu's button, for the
    /// transport options submenu). True when the document may have changed.
    pub(crate) fn main_action(&mut self, anchor: &[usize], action: MainAction) -> bool {
        let default = options::default_preset();
        {
            {
                match action {
                    MainAction::Preferences => self.modals.show(&mut self.app, modal::Kind::Preferences),
                    MainAction::About => self.modals.show(&mut self.app, modal::Kind::About),
                    MainAction::TransportOptions => {
                        self.header.open_options(&mut self.app, &self.model, anchor);
                        return false;
                    }
                    // Reset: the default preset if saved, else a new document.
                    MainAction::Reset => {
                        let loaded = default.as_ref().filter(|p| p.is_file()).and_then(|p| crate::app::read_project(p).map(|(raw, _)| (raw, p.clone())));
                        self.project_path = None;
                        match loaded {
                            Some((raw, p)) => self.open_document(raw, Some(dreist_formats::model::samples_dir(&p)), None),
                            None => self.open_document(project::new_document(), None, None),
                        }
                        self.outbox.push(self.model.rebuild_command());
                    }
                    MainAction::SaveDefault => {
                        if let Some(p) = default {
                            if let Some(d) = p.parent() {
                                let _ = std::fs::create_dir_all(d);
                            }
                            let _ = project::save(&self.model.doc, self.model.samples.as_deref(), &p);
                        }
                    }
                    MainAction::ClearDefault => {
                        if let Some(p) = default {
                            let _ = std::fs::remove_file(&p);
                            let _ = std::fs::remove_dir_all(dreist_formats::model::samples_dir(&p));
                        }
                    }
                    // [re:af38f1].
                    MainAction::FindMissing => self.filefind.show(filefind::missing_files(&self.model)),
                    // [re:c7a8f1].
                    MainAction::Export => self.modals.show(&mut self.app, modal::Kind::Export),
                    // The MIDI console (`MIDIDisplayPanel.skin`).
                    MainAction::MidiConsole => {
                        self.midiconsole.show();
                        return false;
                    }
                    // Lambda #8: scale 1 (when not already);
                    // LastEditorScaleFactor 100.
                    MainAction::ResetScale => {
                        if (self.app.ui_scale - 1.0).abs() > 1e-6 {
                            self.modals.options.set("LastEditorScaleFactor", 100);
                            if !cfg!(test) {
                                self.modals.options.save();
                            }
                            self.app.ui_scale = 1.0;
                            self.app.resize_request = true;
                        }
                    }
                    MainAction::None => return false,
                }
                true
            }
        }
    }
}
