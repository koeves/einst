//! The Tools and Help menus.

use crate::*;

impl Editor {
    /// Tools (`HeaderBar::LoadSkin` lambda 0x2c0720) and Help (0x2c24a0)
    /// menus.
    pub(crate) fn tools_help_menu(&mut self, ev: &UiEvent) -> bool {
        const TOOLS: [&str; 9] = ["Preferences", "", "Reset Einst", "Save as default preset", "Clear default preset", "", "Find missing files...", "", "Export as audio..."];
        const HELP: [&str; 5] = ["About", "", "Reset UI size to default", "Reset UI scale to 100%", "Show MIDI Console"];
        match ev {
            UiEvent::Clicked { path, name, .. } if (name == "ToolButton" || name == "HelpButton") && !self.browser.as_ref().is_some_and(|b| path.starts_with(b.page())) => {
                let items = if name == "ToolButton" { &TOOLS[..] } else { &HELP[..] };
                self.app.open_menu(path, items.iter().map(|s| s.to_string()).collect(), None);
                false
            }
            UiEvent::MenuSelected { anchor, name, index } if name == "ToolButton" && !self.browser.as_ref().is_some_and(|b| anchor.starts_with(b.page())) => {
                let default = options::settings_dir().map(|d| d.join("DefaultPreset.geist"));
                match *index {
                    0 => self.modals.show(&mut self.app, modal::Kind::Preferences),
                    // Reset: the default preset if saved, else a new document.
                    2 => {
                        let loaded = default.as_ref().and_then(|p| crate::v1io::read_project(p).map(|(raw, _)| (raw, p.clone())));
                        self.project_path = None;
                        match loaded {
                            Some((raw, p)) => self.open_document(raw, Some(geist_formats::model::samples_dir(&p)), None),
                            None => self.open_document(project::new_document(), None, None),
                        }
                        self.outbox.push(self.model.rebuild_command());
                    }
                    3 => {
                        if let Some(p) = default {
                            if let Some(d) = p.parent() {
                                let _ = std::fs::create_dir_all(d);
                            }
                            let _ = project::save(&self.model.doc, self.model.samples.as_deref(), &p);
                        }
                    }
                    4 => {
                        if let Some(p) = default {
                            let _ = std::fs::remove_file(&p);
                            let _ = std::fs::remove_dir_all(geist_formats::model::samples_dir(&p));
                        }
                    }
                    // `FileFindPanel::Show`.
                    6 => self.filefind.show(filefind::missing_files(&self.model)),
                    // `ExportPanel::Show`.
                    8 => self.modals.show(&mut self.app, modal::Kind::Export),
                    _ => {}
                }
                true
            }
            UiEvent::MenuSelected { name, index, .. } if name == "HelpButton" => {
                match *index {
                    0 => self.modals.show(&mut self.app, modal::Kind::About),
                    // `HeaderBar::LoadSkin` $_23 lambda #7: the host resizes
                    // to `GetDefaultWindowSize`.
                    2 => {
                        let (w, h) = self.window_size.0;
                        self.app.resize(w, h);
                        self.app.resize_request = true;
                    }
                    // Lambda #8: scale 1 (when not already), the window at
                    // the root's size; LastEditorScaleFactor 100,
                    // LastEditorSize = that size.
                    3 => {
                        if (self.app.ui_scale - 1.0).abs() > 1e-6 {
                            self.modals.options.set("LastEditorScaleFactor", 100);
                            let (w, h) = self.app.size();
                            self.modals.options.set("LastEditorSize", format!("{w},{h}"));
                            if !cfg!(test) {
                                self.modals.options.save();
                            }
                            self.window_size.1 = (w, h);
                            self.app.ui_scale = 1.0;
                            self.app.resize_request = true;
                        }
                    }
                    4 => self.midiconsole.open = true,
                    _ => {}
                }
                true
            }
            _ => false,
        }
    }
}
