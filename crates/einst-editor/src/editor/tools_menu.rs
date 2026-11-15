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
    Performance,
    TransportOptions,
    ResetScale,
    /// Reset, confirmed.
    ConfirmReset,
    None,
}

/// Options key: the CPU and memory readout is on (off by default).
pub(crate) const PERF_OPTION: &str = "ShowPerformance";

impl Editor {
    /// The CPU and memory readout is on.
    pub fn perf_shown(&self) -> bool {
        self.modals.options.bool(PERF_OPTION, false)
    }

    /// Show `r` in the readout, adding or removing it to match the option
    /// (the app calls this about once a second).
    pub fn performance(&mut self, r: &dreist_gui::perf::Reading) {
        self.place_perf();
        if self.perf_shown() {
            dreist_gui::perf::update(&mut self.app, r);
        }
    }

    /// Add or remove the readout to match the option, in a strip taken
    /// from the bottom of the browser's list area (kept every frame through
    /// splitters and window sizes); hidden while that area is not shown.
    pub fn place_perf(&mut self) {
        let on = self.perf_shown();
        let list = self.browser.as_ref().map(|b| b.page().to_vec()).and_then(|page| Some([&page[..], &self.app.ctrl(&page)?.path_of("SplitView")?[..]].concat()));
        let strip = list.and_then(|p| dreist_gui::perf::reserve_below(&mut self.app, &p, on));
        dreist_gui::perf::show(&mut self.app, on);
        if on {
            dreist_gui::perf::place(&mut self.app, strip.unwrap_or_default());
        }
    }

    fn main_items(&self) -> Vec<(String, MainAction)> {
        use MainAction::*;
        // v1's Grid menu ([re:3695e4]): About, Preferences, Reset, the
        // default preset items (Clear only when there is one); then the
        // port's own items.
        let has_default = crate::options::default_preset().is_some_and(|p| p.is_file());
        vec![
            (format!("About {}", crate::product()), About),
            (String::new(), None),
            ("Preferences".into(), Preferences),
            (String::new(), None),
            (format!("Reset {}", crate::product()), Reset),
            ("Save as default preset".into(), SaveDefault),
            (dreist_gui::menu::item("Clear default preset", has_default), ClearDefault),
            (String::new(), None),
            ("Transport options ▸".into(), TransportOptions),
            ("Find missing files...".into(), FindMissing),
            ("Export as audio...".into(), Export),
            ("Show MIDI console".into(), MidiConsole),
            (if self.perf_shown() { "Hide CPU and memory" } else { "Show CPU and memory" }.into(), Performance),
            ("Reset UI scale to 100%".into(), ResetScale),
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
                // [re:0aaa8a]: Reset asks first.
                if action == MainAction::Reset {
                    let text = format!("Are you sure you want to reset {} to its default state? This action cannot be undone!", crate::product());
                    self.app.open_menu(anchor, vec![text, String::new(), "Yes".into(), "No".into()], None);
                    self.reset_confirm = true;
                    return false;
                }
                self.main_action(anchor, action)
            }
            UiEvent::MenuSelected { anchor, name, index } if name == "GridButton" && std::mem::take(&mut self.reset_confirm) => *index == 2 && self.main_action(anchor, MainAction::ConfirmReset),
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
                    MainAction::Reset | MainAction::ConfirmReset => {
                        let loaded = default.as_ref().filter(|p| p.is_file()).and_then(|p| crate::app::read_project(p).map(|(raw, _)| (raw, p.clone())));
                        self.project_path = None;
                        let (mut raw, samples) = match loaded {
                            Some((raw, p)) => (raw, Some(dreist_formats::model::samples_dir(&p))),
                            None => (project::new_document(), None),
                        };
                        // v1 loads the default preset as any state: locked
                        // bindings stay ([re:6421a6]).
                        if options::Options::load().bool("BindingsLocked", false) {
                            midimap::carry_bindings(&self.model.doc, &mut raw);
                        }
                        self.open_document(raw, samples, None);
                        self.outbox.push(self.model.rebuild_command());
                    }
                    MainAction::SaveDefault => {
                        if let Some(p) = default {
                            if let Some(d) = p.parent() {
                                let _ = std::fs::create_dir_all(d);
                            }
                            let _ = project::save(&self.model.doc, self.model.samples.as_deref(), &p, project::embed_samples());
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
                    // [re:0dfe90].
                    MainAction::Export => {
                        self.modals.export_outputs = dreist_engine::export::assigned_outputs(&self.model.doc);
                        self.modals.show(&mut self.app, modal::Kind::Export);
                    }
                    // The MIDI console (`MIDIDisplayPanel.skin`).
                    MainAction::MidiConsole => {
                        self.midiconsole.show();
                        return false;
                    }
                    // Lambda #8: scale 1 (when not already);
                    // LastEditorScaleFactor 100.
                    MainAction::Performance => {
                        let on = !self.perf_shown();
                        self.modals.options.set(PERF_OPTION, on);
                        if !cfg!(test) {
                            self.modals.options.save();
                        }
                        self.place_perf();
                        return false;
                    }
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
