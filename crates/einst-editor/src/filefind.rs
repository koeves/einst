//! Missing files ([re:f7e56f], [re:37765e]): the audio files a project
//! refers to that could not be found while loading, in v1's "File Finder"
//! dialog: a list of their names and Close / Manual / Auto.
//!
//! - Auto ([re:513f9b]): choose a start folder ("Please choose a search
//!   location", from the last folder used), then [re:cdeafd] [re:3c8277]
//!   walks it recursively and matches files by name (a trailing
//!   ".truncated" extension is ignored); a match is loaded into every layer
//!   that asked for that file. Abort stops it.
//! - Manual ([re:1395ce]): "Please locate <file>" for each file in turn; a
//!   located file serves every request for the same path, one that does
//!   not load alerts "Failed to load audio file on path %s"; a cancel
//!   stops. All found closes the dialog.
//! - Close ([re:819096]) drops the requests.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use dreist_gui::app::{App, MouseKind, UiEvent};
use dreist_gui::paint::{Painter, RectF};
use dreist_gui::render::Renderer;
use dreist_gui::style::{HAlign, VAlign};
use dreist_gui::{Colour, Ctrl, Skin};

use crate::EditorError;
use crate::model::Model;

const ROW: f32 = 18.0;
const LIST_STYLE: &str = "DefaultTableViewStyle";
/// v1's list box style.
const LIST_STYLE_V1: &str = "FindFileListBoxStyle";
/// A missing file and the layers (engine, pad, layer) that use it.
#[derive(Clone, Debug, PartialEq)]
pub struct Missing {
    pub path: PathBuf,
    pub layers: Vec<(usize, usize, usize)>,
}

/// Layers whose sample file does not exist: one request per layer, as
/// the [re:2e303e] request list the table shows.
pub fn missing_files(model: &Model) -> Vec<Missing> {
    let mut out: Vec<Missing> = Vec::new();
    for e in 0..crate::model::ENGINES {
        for p in 0..64 {
            for l in 0..8 {
                let Some(f) =
                    model.layer_sample(e, p, l).or_else(|| model.layer_node(e, p, l).and_then(|n| n.attr("AudioFileName")).filter(|s| !s.is_empty()).map(|s| PathBuf::from(s.replace('\\', "/"))))
                else {
                    continue;
                };
                // A REX ".truncated" path names the file without it.
                if f.as_os_str().is_empty() || dreist_engine::rex::strip_truncated(&f).0.exists() {
                    continue;
                }
                out.push(Missing { path: f, layers: vec![(e, p, l)] });
            }
        }
    }
    out
}

/// File name used for matching (".truncated" dropped).
fn match_name(p: &Path) -> Option<String> {
    let p = if p.extension().is_some_and(|e| e.eq_ignore_ascii_case("truncated")) { p.with_extension("") } else { p.to_path_buf() };
    p.file_name().map(|n| n.to_string_lossy().into_owned())
}

/// A running [re:fb2fba]: what the worker reports.
#[derive(Debug, Default)]
pub struct JobState {
    /// Progress: 0..1, each file adding its share of its folder.
    pub progress: f32,
    /// The entry being looked at (shown in ScanLabel).
    pub current: String,
    pub abort: bool,
    pub done: bool,
    /// Per wanted name, the file found for it.
    pub found: Vec<Option<PathBuf>>,
}

/// [re:8fb33a]: the folder's entries in
/// reverse listing order; each gets `share` / count of the progress —
/// files add theirs, folders pass it on to their own entries. A file whose
/// name matches pending requests satisfies all of them; the search stops
/// when none is pending or on abort.
pub fn search_location(dir: &Path, share: f32, wanted: &[String], st: &std::sync::Mutex<JobState>) -> bool {
    let Ok(rd) = std::fs::read_dir(dir) else { return false };
    let mut entries: Vec<PathBuf> = rd.flatten().map(|e| e.path()).collect();
    entries.sort();
    entries.reverse();
    if entries.is_empty() {
        return false;
    }
    let part = share / entries.len() as f32;
    for p in entries {
        let Ok(mut g) = st.lock() else { return true };
        if g.abort {
            return true;
        }
        g.current = p.to_string_lossy().into_owned();
        if p.is_dir() {
            drop(g);
            if search_location(&p, part, wanted, st) {
                return true;
            }
            continue;
        }
        g.progress += part;
        if let Some(n) = p.file_name().map(|n| n.to_string_lossy().into_owned()) {
            for (i, w) in wanted.iter().enumerate() {
                if g.found[i].is_none() && *w == n {
                    g.found[i] = Some(p.clone());
                }
            }
        }
        if g.found.iter().all(Option::is_some) {
            return true;
        }
    }
    false
}

pub struct FileFind {
    slide: crate::modal::Slide,
    overlay: Option<Vec<usize>>,
    pub open: bool,
    pub items: Vec<Missing>,
    pub selected: BTreeSet<usize>,
    scroll: usize,
    /// The editor runs the folder / file dialogs: (auto find, manual find index).
    pub request: Option<Request>,
    /// The Auto Find job.
    job: Option<std::sync::Arc<std::sync::Mutex<JobState>>>,
    /// [re:1395ce] / [re:513f9b]: the folder Manual and Auto start in (the
    /// last one chosen).
    pub last_dir: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Request {
    AutoFind,
    /// [re:1395ce]: locate each missing file in turn.
    Manual,
}

impl FileFind {
    pub fn build(skin: &mut Skin, root: &mut Ctrl) -> Result<FileFind, EditorError> {
        // [re:401f49] 0x10e: titled "File Finder".
        let overlay = crate::modal::overlay_titled(skin, root, "FileFindPanel.skin", "FileFindOverlay", "File Finder")?;
        if let Some(op) = &overlay {
            let mut o = &mut *root;
            for &i in op {
                o = &mut o.children[i];
            }
            if let Some(sp) = o.find_mut("ScanningPanel") {
                sp.visible = false;
            }
            if let Some(fl) = o.find_mut("FileList") {
                fl.overload = "FileFindList".into();
                fl.children.clear();
            }
        }
        Ok(FileFind { overlay, slide: Default::default(), open: false, items: Vec::new(), selected: BTreeSet::new(), scroll: 0, request: None, job: None, last_dir: None })
    }

    /// [re:af38f1] with the missing files.
    pub fn show(&mut self, items: Vec<Missing>) {
        self.open = !items.is_empty();
        self.items = items;
        self.selected.clear();
        self.scroll = 0;
    }

    pub fn sync(&mut self, app: &mut App) {
        // [re:89e6b8] / closing [re:48676b]: shown and removed at once,
        // centred ([re:24b9de]).
        let open = self.overlay.clone().map(|p| self.slide.sync(app, &p, self.open)).unwrap_or(false);
        if let Some(o) = self.overlay.clone().and_then(|p| app.ctrl_mut(&p)) {
            o.visible = open;
        }
    }

    fn mine(&self, path: &[usize]) -> bool {
        self.open && self.overlay.as_ref().is_some_and(|p| path.starts_with(p))
    }

    /// Point the layers of item `i` at `file` (the caller rebuilds).
    pub fn resolve(&mut self, model: &mut Model, i: usize, file: &Path) {
        if let Some(m) = self.items.get(i) {
            for &(e, p, l) in &m.layers {
                model.layer_mut(e, p, l).set("AudioFileName", file.to_string_lossy());
            }
        }
    }

    /// Drop resolved items.
    pub fn retain(&mut self, keep: impl Fn(usize) -> bool) {
        let items = std::mem::take(&mut self.items);
        self.items = items.into_iter().enumerate().filter(|(i, _)| keep(*i)).map(|(_, m)| m).collect();
        self.selected.clear();
        if self.items.is_empty() {
            self.open = false;
        }
    }

    fn panel_path(&self) -> Option<Vec<usize>> {
        self.overlay.as_ref().map(|o| [o.clone(), vec![0]].concat())
    }

    fn set_child(&self, app: &mut App, name: &str, f: impl FnOnce(&mut Ctrl)) {
        if let Some(c) = self.panel_path().and_then(|p| app.ctrl_mut(&p)).and_then(|c| c.find_mut(name)) {
            f(c);
        }
    }

    /// Auto Find [re:513f9b]: HoldingPanel hidden, ScanningPanel shown,
    /// ScanLabel cleared and the [re:fb2fba] searches `root` on a worker.
    pub fn start_auto_find(&mut self, app: &mut App, root: &Path) {
        let wanted: Vec<String> = self.items.iter().map(|m| match_name(&m.path).unwrap_or_default()).collect();
        let st = std::sync::Arc::new(std::sync::Mutex::new(JobState { found: vec![None; wanted.len()], ..JobState::default() }));
        let worker = st.clone();
        let root = root.to_path_buf();
        let job = move || {
            search_location(&root, 1.0, &wanted, &worker);
            if let Ok(mut g) = worker.lock() {
                g.done = true;
            }
        };
        std::thread::spawn(job);
        self.set_child(app, "HoldingPanel", |c| c.visible = false);
        self.set_child(app, "ScanningPanel", |c| c.visible = true);
        self.set_child(app, "ScanLabel", |c| c.text.clear());
        self.job = Some(st);
    }

    /// [re:02d115]: ScanLabel follows the job; when it ends the found
    /// files are applied (true: rebuild), the holding panel comes back
    /// with the list, which closes the panel when nothing is left.
    pub fn tick(&mut self, app: &mut App, model: &mut Model) -> bool {
        let Some(st) = self.job.clone() else { return false };
        let (current, done, found) = {
            let Ok(g) = st.lock() else { return false };
            (g.current.clone(), g.done, g.found.clone())
        };
        self.set_child(app, "ScanLabel", |c| c.text = current);
        app.dirty = true;
        if !done {
            return false;
        }
        self.job = None;
        let mut hit = BTreeSet::new();
        for (i, f) in found.iter().enumerate() {
            if let Some(f) = f {
                self.resolve(model, i, f);
                hit.insert(i);
            }
        }
        self.retain(|i| !hit.contains(&i));
        self.set_child(app, "HoldingPanel", |c| c.visible = true);
        self.set_child(app, "ScanningPanel", |c| c.visible = false);
        !hit.is_empty()
    }

    /// Whether an Auto Find job runs.
    pub fn scanning(&self) -> bool {
        self.job.is_some()
    }

    /// Find Selected for item `i` located at `file`, plus the others whose
    /// names exist next to it.
    /// [re:1395ce]: the located `file` serves item `i` and every other
    /// request for the same path; a file that does not load drops item
    /// `i` (Err with the alert text).
    pub fn manual_find(&mut self, model: &mut Model, i: usize, file: &Path) -> Result<(), String> {
        let Some(want) = self.items.get(i).map(|m| m.path.clone()) else { return Ok(()) };
        let (real, _) = dreist_engine::rex::strip_truncated(file);
        let loads = dreist_engine::audio::load(&real).is_ok() || dreist_engine::rex::load(&real).is_some();
        if !loads {
            self.retain(|k| k != i);
            return Err(format!("Failed to load audio file on path {}", file.display()));
        }
        let same: Vec<usize> = (0..self.items.len()).filter(|&k| self.items[k].path == want).collect();
        for &k in &same {
            self.resolve(model, k, file);
        }
        self.retain(|k| !same.contains(&k));
        Ok(())
    }

    /// The folder Manual starts in: the last one a file was located in.
    pub fn last_dir(&self) -> Option<PathBuf> {
        self.last_dir.clone()
    }

    /// Returns (changed, rebuild needed).
    pub fn handle(&mut self, app: &mut App, ev: &UiEvent) -> (bool, bool) {
        if !self.open {
            return (false, false);
        }
        match ev {
            UiEvent::Mouse { path, overload, kind: MouseKind::Down, y, .. } if overload == "FileFindList" => {
                let _ = path;
                let i = self.scroll + (*y / ROW) as usize;
                if i < self.items.len() {
                    if app.mods.shift || app.mods.cmd {
                        if !self.selected.insert(i) {
                            self.selected.remove(&i);
                        }
                    } else {
                        self.selected.clear();
                        self.selected.insert(i);
                    }
                }
                (true, false)
            }
            UiEvent::Wheel { overload, dy, .. } if overload == "FileFindList" => {
                self.scroll = (self.scroll as i32 - dy.signum() as i32 * 3).clamp(0, self.items.len().saturating_sub(1) as i32) as usize;
                (true, false)
            }
            UiEvent::Clicked { path, name, .. } if self.mine(path) => match name.as_str() {
                "AutoButton" => {
                    self.request = Some(Request::AutoFind);
                    (true, false)
                }
                // [re:242e1b].
                "AbortButton" => {
                    if let Some(Ok(mut g)) = self.job.as_ref().map(|st| st.lock()) {
                        g.abort = true;
                    }
                    (true, false)
                }
                "ManualButton" => {
                    self.request = Some(Request::Manual);
                    (true, false)
                }
                // [re:819096]: Close drops the requests and closes.
                "IgnoreButton" => {
                    self.items.clear();
                    self.open = false;
                    (true, false)
                }
                _ => (true, false),
            },
            // Modal.
            UiEvent::Mouse { path, .. } | UiEvent::Wheel { path, .. } | UiEvent::Clicked { path, .. } | UiEvent::ValueChanged { path, .. } => (self.mine(path), false),
            _ => (false, false),
        }
    }

    /// [re:28fc2e]: v1's list box, one file name per request (the
    /// ".truncated" extension dropped).
    pub fn draw(&self, r: &Renderer, p: &mut Painter, c: &Ctrl) -> bool {
        if c.overload != "FileFindList" {
            return false;
        }
        let s = r.styles;
        let st = if s.raw(LIST_STYLE_V1, "colText").is_some() { LIST_STYLE_V1 } else { LIST_STYLE };
        let (w, h) = (c.rect.w as f32, c.rect.h as f32);
        let font = s.font(st);
        let text = s.colour(st, "colText", Colour::WHITE);
        let sel = s.colour(st, "colRowSelection", s.colour(st, "colAltFill", Colour::rgba(80, 120, 200, 255)));
        p.fill_rect(RectF::new(0.0, 0.0, w, h), s.colour(st, "colFill", Colour::rgba(30, 30, 30, 255)));
        let visible = (h / ROW).ceil() as usize;
        for (k, m) in self.items.iter().enumerate().skip(self.scroll).take(visible) {
            let y = (k - self.scroll) as f32 * ROW;
            if self.selected.contains(&k) {
                p.fill_rect(RectF::new(0.0, y, w, ROW), sel);
            }
            let name = match_name(&m.path).unwrap_or_default();
            let shown = p.fonts.shorten(&font, &name, w - 8.0, dreist_gui::style::Shorten::End);
            p.text(RectF::new(4.0, y, w - 8.0, ROW), &shown, text, &font, HAlign::Left, VAlign::Center);
        }
        if s.bool(st, "optDrawBorder", false) {
            p.stroke_rect(RectF::new(0.5, 0.5, w - 1.0, h - 1.0), s.colour(st, "colBorder", text), 1.0);
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_finds_by_name_and_ignores_truncated() {
        crate::app::test_setup();
        let d = std::env::temp_dir().join(format!("dreist-find-{}", std::process::id()));
        std::fs::create_dir_all(d.join("a/b")).unwrap();
        std::fs::write(d.join("a/b/kick.wav"), b"").unwrap();
        assert_eq!(match_name(Path::new("/x/kick.wav.truncated")).as_deref(), Some("kick.wav"));
        let st = std::sync::Mutex::new(JobState { found: vec![None; 2], ..JobState::default() });
        search_location(&d, 1.0, &["kick.wav".into(), "snare.wav".into()], &st);
        let g = st.lock().unwrap();
        assert_eq!(g.found[0].as_deref(), Some(d.join("a/b/kick.wav").as_path()));
        assert!(g.found[1].is_none());
        assert!((g.progress - 1.0).abs() < 1e-6);
        let _ = std::fs::remove_dir_all(&d);
    }
}
