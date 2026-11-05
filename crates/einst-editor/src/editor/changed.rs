//! Samples changed on disk ([re:22a658]): the layers' files are checked
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
    /// The open question: its anchor and the changed files.
    open: Option<(Vec<usize>, Vec<PathBuf>)>,
}

impl Watch {
    /// Check again at the next call.
    #[cfg(test)]
    pub(crate) fn reset_timer(&mut self) {
        self.last = None;
    }
}

/// The question's lines, as v1 words them.
pub(crate) fn question(changed: &[PathBuf]) -> Vec<String> {
    let name = |p: &PathBuf| p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let mut v = vec![format!("The following audio file(s) have been changed outside of {}:", crate::product())];
    if let Some(f) = changed.first() {
        v.push(name(f));
    }
    if changed.len() > 1 {
        v.push(format!("Plus {} others", changed.len() - 1));
    }
    v.extend(["Do you want to reload these files?".to_string(), String::new(), "Yes".into(), "No".into()]);
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
        let mut changed = Vec::new();
        for f in self.layer_files() {
            let Some(m) = std::fs::metadata(&f).ok().and_then(|m| m.modified().ok()) else { continue };
            match self.watch.stamps.insert(f.clone(), m) {
                Some(old) if m > old => changed.push(f),
                _ => {}
            }
        }
        if changed.is_empty() {
            return;
        }
        let anchor = Vec::new();
        self.app.open_menu(&anchor, question(&changed), None);
        self.watch.open = Some((anchor, changed));
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
        if self.watch.open.as_ref().is_none_or(|(a, _)| a != anchor) {
            return false;
        }
        let Some((_, files)) = self.watch.open.take() else { return false };
        let yes = question(&files).iter().position(|s| s == "Yes");
        if Some(*index) == yes {
            self.outbox.push(Command::ReloadSamples(files));
            self.outbox.push(self.model.rebuild_command());
        }
        true
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn question_lines() {
        let q = super::question(&["/a/x.wav".into(), "/a/y.wav".into(), "/a/z.wav".into()]);
        assert_eq!(q[1], "x.wav");
        assert_eq!(q[2], "Plus 2 others");
        assert_eq!(q.iter().filter(|s| *s == "Yes").count(), 1);
    }
}
