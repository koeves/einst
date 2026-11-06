//! Samples changed or removed on disk ([re:22a658]): the layers' files are checked
//! every two seconds; files modified since they were first seen are
//! listed in a question, and Yes reads them again.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime};

use dreist_gui::app::UiEvent;

use crate::Editor;
use crate::command::Command;

/// The watch state.
#[derive(Default)]
pub(crate) struct Watch {
    stamps: HashMap<PathBuf, SystemTime>,
    last: Option<Instant>,
    /// The open question: its anchor, the files and whether they were
    /// removed (else changed).
    open: Option<(Vec<usize>, Vec<PathBuf>, bool)>,
}

impl Watch {
    /// Check again at the next call.
    #[cfg(test)]
    pub(crate) fn reset_timer(&mut self) {
        self.last = None;
    }
}

/// The question's lines, as v1 words them (`removed`: the files are gone).
pub(crate) fn question(changed: &[PathBuf], removed: bool) -> Vec<String> {
    let name = |p: &PathBuf| p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let head = if removed {
        "The following audio file(s) have been removed from the original location".to_string()
    } else {
        format!("The following audio file(s) have been changed outside of {}:", crate::product())
    };
    let mut v = vec![head];
    if let Some(f) = changed.first() {
        v.push(name(f));
    }
    if changed.len() > 1 {
        v.push(format!("Plus {} others", changed.len() - 1));
    }
    let ask = if removed {
        "Do you wish to find these files? (Clicking no will cause the samples to be unloaded from memory and the undo history to be cleared)"
    } else {
        "Do you want to reload these files?"
    };
    v.extend([ask.to_string(), String::new(), "Yes".into(), "No".into()]);
    v
}

impl Editor {
    /// The layers' sample files.
    fn layer_files(&self) -> Vec<PathBuf> {
        let mut v: Vec<PathBuf> = Vec::new();
        for e in 0..crate::model::ENGINES {
            for p in 0..64 {
                for l in 0..crate::model::LAYERS {
                    if let Some(f) = self.model.layer_sample(e, p, l) {
                        if !v.contains(&f) {
                            v.push(f);
                        }
                    }
                }
            }
        }
        v
    }

    /// Check the files (at most every two seconds); asks when some changed.
    pub(crate) fn watch_samples(&mut self) {
        if self.watch.open.is_some() || self.watch.last.is_some_and(|t| t.elapsed() < Duration::from_secs(2)) {
            return;
        }
        self.watch.last = Some(Instant::now());
        let (mut changed, mut removed) = (Vec::new(), Vec::new());
        for f in self.layer_files() {
            let Some(m) = std::fs::metadata(&f).ok().and_then(|m| m.modified().ok()) else {
                // Seen before, gone now.
                if self.watch.stamps.remove(&f).is_some() {
                    removed.push(f);
                }
                continue;
            };
            match self.watch.stamps.insert(f.clone(), m) {
                Some(old) if m > old => changed.push(f),
                _ => {}
            }
        }
        let (files, gone) = if !removed.is_empty() { (removed, true) } else { (changed, false) };
        if files.is_empty() {
            return;
        }
        let anchor = Vec::new();
        self.app.open_menu(&anchor, question(&files, gone), None);
        self.watch.open = Some((anchor, files, gone));
    }

    /// A file the editor itself wrote: no question for it.
    pub(crate) fn sample_written(&mut self, f: &std::path::Path) {
        if let Some(m) = std::fs::metadata(f).ok().and_then(|m| m.modified().ok()) {
            self.watch.stamps.insert(f.to_path_buf(), m);
        }
    }

    /// The question's answer: Yes reloads the files and rebuilds.
    pub(crate) fn changed_samples_menu(&mut self, ev: &UiEvent) -> bool {
        let UiEvent::MenuSelected { anchor, index, .. } = ev else { return false };
        if self.watch.open.as_ref().is_none_or(|(a, _, _)| a != anchor) {
            return false;
        }
        let Some((_, files, gone)) = self.watch.open.take() else { return false };
        let q = question(&files, gone);
        let yes = Some(*index) == q.iter().position(|s| s == "Yes");
        let no = Some(*index) == q.iter().position(|s| s == "No");
        match (gone, yes, no) {
            (false, true, _) => {
                self.outbox.push(Command::ReloadSamples(files));
                self.outbox.push(self.model.rebuild_command());
            }
            // Removed files: Yes lists them in the missing files panel; No
            // unloads them from their layers.
            (true, true, _) => {
                let items: Vec<crate::filefind::Missing> = crate::filefind::missing_files(&self.model).into_iter().filter(|m| files.contains(&m.path)).collect();
                self.filefind.show(items);
                self.sync();
            }
            (true, _, true) => {
                for e in 0..crate::model::ENGINES {
                    for p in 0..64 {
                        for l in 0..crate::model::LAYERS {
                            if self.model.layer_sample(e, p, l).is_some_and(|f| files.contains(&f)) {
                                let n = self.model.layer_mut(e, p, l);
                                n.attrs.retain(|(k, _)| k != "AudioFileName");
                                n.children.retain(|c| c.class != "AudioFileMarkers");
                            }
                        }
                    }
                }
                self.history.clear();
                self.outbox.push(self.model.rebuild_command());
            }
            _ => {}
        }
        true
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn question_lines() {
        let q = super::question(&["/a/x.wav".into(), "/a/y.wav".into(), "/a/z.wav".into()], false);
        assert_eq!(q[1], "x.wav");
        assert_eq!(q[2], "Plus 2 others");
        assert_eq!(q.iter().filter(|s| *s == "Yes").count(), 1);
    }
}
