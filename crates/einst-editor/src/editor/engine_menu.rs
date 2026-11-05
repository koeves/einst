//! The engine selector and pad bank menus.

use crate::*;

/// Engine selector menu items ([re:b6961a]
/// [re:796ecd]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum EngineAction {
    Cut,
    Copy,
    Paste,
    UnMute,
    UnSolo,
    Reset,
    ResetAll,
    ConfirmResetAll,
    None,
}

impl Editor {
    /// Pad right-click menu ([re:816870]).
    /// The engine selectors' right-click menu ([re:b6961a]
    ///  [re:796ecd], items [re:45f3d6]..[re:11c0d7]; v1's own list
    /// [re:5a9b42]): Cut / Copy / Paste (0x97), Un-mute / Un-solo all pads / layers on this engine (0x8e / 0x8f,
    /// enabled while the engine's muted / soloed icon is lit), Reset engine
    /// (0x9b), Reset all engines (confirmed; 0x9b for every engine). The
    /// port edits the document and rebuilds.
    pub(crate) fn engine_menu(&mut self, ev: &UiEvent) -> bool {
        use dreist_engine::binding::id;
        match ev {
            UiEvent::RightClicked { path, .. } => {
                let Some(base) = self.header.selectors.clone() else { return false };
                let Some((e, _)) = header::selector_hit(&self.app, &base, path) else { return false };
                let e = e.min(model::ENGINES - 1);
                let icon = |ed: &Editor, n: &str| ed.app.ctrl(&[base.as_slice(), &[e]].concat()).and_then(|s| s.find(n)).is_some_and(|c| c.on);
                let items = vec![
                    ("Cut".into(), EngineAction::Cut),
                    ("Copy".into(), EngineAction::Copy),
                    (dreist_gui::menu::item("Paste", self.clipboard.engine.is_some()), EngineAction::Paste),
                    (String::new(), EngineAction::None),
                    (dreist_gui::menu::item("Un-mute all pads / layers on this engine", icon(self, "EngineMutedIcon")), EngineAction::UnMute),
                    (dreist_gui::menu::item("Un-solo all pads / layers on this engine", icon(self, "EngineSoloedIcon")), EngineAction::UnSolo),
                    (String::new(), EngineAction::None),
                    ("Reset engine".into(), EngineAction::Reset),
                    (String::new(), EngineAction::None),
                    ("Reset all engines".into(), EngineAction::ResetAll),
                ];
                let (labels, acts): (Vec<String>, Vec<EngineAction>) = items.into_iter().unzip();
                self.engine_menu = Some((path.clone(), e, acts));
                self.app.open_menu(path, labels, None);
                true
            }
            UiEvent::MenuSelected { anchor, index, .. } if self.engine_menu.as_ref().is_some_and(|(p, _, _)| p == anchor) => {
                let Some((path, e, acts)) = self.engine_menu.take() else { return false };
                let name = format!("Engine{e}");
                let rebuild = |ed: &mut Editor| ed.outbox.push(ed.model.rebuild_command());
                let reset = |ed: &mut Editor, k: usize| {
                    let n = format!("Engine{k}");
                    ed.model.doc.children.retain(|c| !(c.class == "Engine" && c.name.as_deref() == Some(&n)));
                };
                match acts.get(*index).copied().unwrap_or(EngineAction::None) {
                    EngineAction::Cut | EngineAction::Copy => {
                        let mut n = self.model.engine_node(e).cloned().unwrap_or_else(|| dreist_formats::Node::new("Engine", Some(&name)));
                        n.name = None;
                        self.clipboard.engine = Some(n);
                        if acts[*index] == EngineAction::Cut {
                            reset(self, e);
                            rebuild(self);
                        }
                    }
                    EngineAction::Paste => {
                        let Some(mut n) = self.clipboard.engine.clone() else { return true };
                        n.name = Some(name.clone());
                        match self.model.doc.children.iter_mut().find(|c| c.class == "Engine" && c.name.as_deref() == Some(&name)) {
                            Some(old) => *old = n,
                            None => self.model.doc.children.push(n),
                        }
                        rebuild(self);
                    }
                    EngineAction::UnMute | EngineAction::UnSolo => {
                        let solo = acts[*index] == EngineAction::UnSolo;
                        let (pk, lk) = if solo { (id::SOLO_PAD, id::SET_LAYER_SOLO) } else { (id::MUTE_PAD, id::SET_LAYER_MUTE) };
                        for p in 0..64 {
                            if self.model.pad_node(e, p).is_none() {
                                continue;
                            }
                            self.outbox.push(command::msg(pk, &[e as i32, p as i32], &[0.0]));
                            for l in 0..model::LAYERS {
                                if self.model.layer_node(e, p, l).is_some() {
                                    self.outbox.push(command::msg(lk, &[e as i32, p as i32, l as i32], &[0.0]));
                                }
                            }
                        }
                    }
                    EngineAction::Reset => {
                        reset(self, e);
                        rebuild(self);
                    }
                    EngineAction::ResetAll => {
                        let items = vec!["Are you sure you want to reset all engines? This action cannot be undone!".into(), String::new(), "Yes".into(), "No".into()];
                        self.engine_menu = Some((path.clone(), e, vec![EngineAction::None, EngineAction::None, EngineAction::ConfirmResetAll, EngineAction::None]));
                        self.app.open_menu(&path, items, None);
                    }
                    EngineAction::ConfirmResetAll => {
                        for k in 0..model::ENGINES {
                            reset(self, k);
                        }
                        rebuild(self);
                    }
                    EngineAction::None => {}
                }
                true
            }
            _ => false,
        }
    }

    /// [re:7941cb] / [re:0d824e]: a right click on the sampler's Preview
    /// volume knob lists the preview output pairs (the browser's preview
    /// output, one setting); a pick posts `e_setPreviewOutputLocation`.
    pub(crate) fn sampler_preview_menu(&mut self, ev: &UiEvent) -> bool {
        match ev {
            UiEvent::RightClicked { path, name } if name == "PreviewVolume" && self.sampler.owns(path) => {
                let cur = self.browser.as_ref().map_or(1, |b| b.preview_output);
                self.app.open_menu(path, crate::mixer::preview_output_items(), cur.checked_sub(1));
                self.preview_out_menu = Some(path.clone());
                true
            }
            UiEvent::MenuSelected { anchor, index, .. } if self.preview_out_menu.as_ref() == Some(anchor) => {
                self.preview_out_menu = None;
                let v = (*index + 1).min(16);
                if let Some(b) = self.browser.as_mut() {
                    b.preview_output = v;
                }
                self.outbox.push(crate::command::msg(dreist_engine::binding::id::SET_PREVIEW_OUTPUT, &[], &[v as f32]));
                true
            }
            _ => false,
        }
    }

    pub(crate) fn pad_menu(&mut self, ev: &UiEvent) -> bool {
        match ev {
            UiEvent::RightClicked { path, .. } | UiEvent::Mouse { path, kind: dreist_gui::app::MouseKind::RightDown, .. } if self.mixers.iter().any(|m| m.pad_strip_at(&self.app, path).is_some()) => {
                let Some(k) = self.mixers.iter().find_map(|m| m.pad_strip_at(&self.app, path)) else { return false };
                let path = path.clone();
                self.padmenu.open(&mut self.app, &path, &self.model, &self.clipboard, k);
                true
            }
            // The Pad / Layers selector's pad item: v1's pad item menu.
            UiEvent::Mouse { path, overload, kind: dreist_gui::app::MouseKind::RightDown, .. } if overload == "PadSelectionButton" => {
                let e = self.model.engine;
                let p = self.model.pad[e];
                let items = padmenu::pad_row_items(&self.model, &self.clipboard, e, p);
                self.pad_row_menu = Some((path.clone(), (e, p)));
                self.app.open_menu(path, items, None);
                true
            }
            UiEvent::MenuSelected { anchor, index, .. } if self.pad_row_menu.as_ref().is_some_and(|(a, _)| a == anchor) => {
                let Some((_, (e, p))) = self.pad_row_menu.take() else { return false };
                padmenu::pad_row_action(&mut self.model, &mut self.clipboard, e, p, *index, &mut self.outbox)
            }
            UiEvent::Mouse { path, overload, kind: dreist_gui::app::MouseKind::RightDown, .. } if overload == "LayerRow" => {
                let Some(l) = self.app.ctrl(path).and_then(|r| r.attr("layer")).and_then(|v| v.parse::<usize>().ok()) else { return false };
                let e = self.model.engine;
                let p = self.model.pad[e];
                let items = padmenu::layer_items(&self.model, &self.clipboard, e, p, l);
                self.layer_menu = Some((path.clone(), (e, p, l)));
                self.app.open_menu(path, items, None);
                true
            }
            UiEvent::MenuSelected { anchor, index, .. } if self.layer_menu.as_ref().is_some_and(|(a, _)| a == anchor) => {
                let Some((_, (e, p, l))) = self.layer_menu.take() else { return false };
                padmenu::layer_action(&mut self.model, &mut self.clipboard, e, p, l, *index, &mut self.outbox)
            }
            UiEvent::Mouse { path, overload, kind: dreist_gui::app::MouseKind::RightDown, .. } if overload == "EditorPad" => {
                let Some(idx) = self.app.ctrl(path).and_then(|c| c.attr("padIndex")).and_then(|v| v.parse::<usize>().ok()) else { return false };
                let path = path.clone();
                self.model.pad[self.model.engine] = idx;
                self.padmenu.open(&mut self.app, &path, &self.model, &self.clipboard, idx);
                true
            }
            UiEvent::MenuSelected { anchor, index, .. } if self.padmenu.anchor() == Some(anchor.as_slice()) => {
                let anchor = anchor.clone();
                let o = self.padmenu.select(&mut self.app, &anchor, &mut self.model, &mut self.clipboard, *index, &mut self.outbox);
                if let (Some(d), Some(b)) = (o.browse, self.browser.as_mut()) {
                    b.browse(d);
                    self.model.utility_tab = 0;
                }
                if let Some(l) = o.save {
                    let e = self.model.engine;
                    let p = self.model.pad[e];
                    let stem = self.model.layer_sample(e, p, l).and_then(|f| f.file_stem().map(|s| s.to_string_lossy().into_owned())).unwrap_or_default();
                    if let Some(mut dst) = self.dialog(Dialog::SaveSample(format!("{stem}.wav"))) {
                        // The save dialog forces the "wav" extension.
                        dst.set_extension("wav");
                        if let Err(msg) = padmenu::save_layer_sample(&self.model, e, p, l, &dst) {
                            let anchor = anchor.clone();
                            self.app.open_menu(&anchor, vec![msg], None);
                        }
                    }
                }
                o.changed
            }
            _ => false,
        }
    }
}
