//! Browser (`Browser.skin` with `Header.skin` and `FileList.skin`): source
//! list, path navigator and file list.
//!
//! - Source list ([re:f32dc0]):
//!   the categories of [re:f32dc0] (Content,
//!   Places, Drives, Favourites, Searches, Shortlists), 25 px rows in
//!   `BrowserSourceListItemStyle`.
//! - File list rows in `BrowserListItemStyle`.
//!
//! Loading (port): presets replace the project (`Command::LoadProject`);
//! kits replace the selected engine's pads and audio files the selected
//! layer's sample, as document edits followed by `Command::Rebuild`.

use std::path::{Path, PathBuf};

use dreist_formats::Node;
use dreist_gui::app::{App, MouseKind, UiEvent};
use dreist_gui::paint::{Painter, RectF};
use dreist_gui::render::Renderer;
use dreist_gui::style::{HAlign, VAlign};
use dreist_gui::{Colour, Ctrl, Rect, Skin};

use crate::command::Command;
use crate::model::Model;
use crate::{EditorError, err};

pub const CATEGORIES: [&str; 6] = ["Content", "Places", "Drives", "Favourites", "Searches", "Shortlists"];
pub(crate) const SOURCE_ROW: f32 = 25.0;
const FILE_ROW: f32 = 18.0;
/// The Preview play button's width (v1's 20 px icon).
const PREVIEW_ICON: f32 = 20.0;
/// Its red flash: 26 ticks of v1's 30 Hz timer.
const PREVIEW_FLASH: std::time::Duration = std::time::Duration::from_millis(867);

/// v1's list pane ([re:fa4043], [re:ed8e61]): a "Parent folder" button
/// across the top (20 px), the column header 2 px below it (18 px, the
/// view options button over the scroll bar's column at its right), then a
/// split of the folder's list over the current shortlist.
const PARENT_H: i32 = 20;
const HEADER_Y: i32 = 22;
const HEADER_H: i32 = 18;
const LIST_Y: i32 = HEADER_Y + HEADER_H;
const SLIDER_W: i32 = 11;
/// The split's bar (v1: 6 px; with no saved position it sits mid-way,
/// [re:d70824]).
const LIST_BAR: i32 = 6;
/// Type and Size columns ([re:9195ec]): 45 px each at the right, Size
/// rightmost; Name takes the rest.
const COLUMN_W: f32 = 45.0;

/// A file list column ([re:0feb7a]: clicking its header sorts by it,
/// clicking it again reverses the order).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Column {
    Name,
    Type,
    Size,
}

/// The Type column's text ([re:acb2eb]).
pub fn type_label(path: &Path, dir: bool) -> &'static str {
    use dreist_formats::profile::{DocKind, doc_kind};
    if dir {
        return "Folder";
    }
    let ext = path.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
    match ext.as_str() {
        "wav" => "Wave",
        "aif" => "AIF",
        "aiff" => "AIFF",
        "rx2" | "rex2" => "REX 2",
        "rex" | "rcy" | "rcso" => "Recycle",
        "mid" => "MIDI",
        e => match doc_kind(e) {
            Some(DocKind::Project) => "Preset",
            Some(DocKind::Kit) => "Kit",
            Some(DocKind::Pattern) => "Pattern",
            Some(DocKind::Pad) => "Pad",
            Some(DocKind::Bank) => "Bank",
            Some(DocKind::Engine) => "Engine",
            _ => match e {
                "g01" => "Guru G01",
                "g24" => "Guru G24",
                "kit" => "Guru kit",
                "gru" => "Guru song",
                _ => "Unknown",
            },
        },
    }
}

/// The Size column's text ([re:22e64a]): "--" for a folder.
pub fn size_label(path: &Path, dir: bool) -> String {
    if dir {
        return "--".into();
    }
    std::fs::metadata(path).map(|m| dreist_engine::memory::bytes_label(m.len())).unwrap_or_else(|_| "--".into())
}

/// Folders first, then by `col` (ascending or not); names break ties.
fn sort_entries(v: &mut [Entry], col: Column, asc: bool) {
    let size = |e: &Entry| if e.dir { 0 } else { std::fs::metadata(&e.path).map(|m| m.len()).unwrap_or(0) };
    v.sort_by(|a, b| {
        let by = match col {
            Column::Name => std::cmp::Ordering::Equal,
            Column::Type => type_label(&a.path, a.dir).cmp(type_label(&b.path, b.dir)),
            Column::Size => size(a).cmp(&size(b)),
        }
        .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        b.dir.cmp(&a.dir).then(if asc { by } else { by.reverse() })
    });
}

/// The view options menu's file types ([re:8b51c3]), each shown unless
/// unticked (option `Show<type>`, spaces as underscores, default on,
/// [re:b1c42d]).
pub const VIEW_TYPES: [&str; 13] = ["Audio", "Presets", "Engines", "Kits", "Pads", "Patterns", "Pattern Bank", "MIDI", "Guru Kit", "Guru Pattern", "Guru Pattern Bank", "Folders", "Sample Folders"];

fn view_option(t: usize) -> String {
    format!("Show{}", VIEW_TYPES[t].replace(' ', "_"))
}

/// The [`VIEW_TYPES`] entry a listed item belongs to; `None` for a file of
/// no type, which the original hides whenever any type is (that includes
/// Guru songs, [re:ca74b1]).
fn view_type(e: &Entry) -> Option<usize> {
    use dreist_formats::profile::{DocKind, doc_kind};
    if e.dir {
        return Some(if e.path.extension().is_some_and(|x| x.eq_ignore_ascii_case("samples")) { 12 } else { 11 });
    }
    let ext = e.path.extension().map(|x| x.to_string_lossy().to_lowercase()).unwrap_or_default();
    Some(match ext.as_str() {
        "wav" | "aif" | "aiff" | "rx2" | "rex" | "rex2" | "rcy" | "rcso" => 0,
        "mid" => 7,
        "kit" => 8,
        "g01" => 9,
        "g24" => 10,
        x => match doc_kind(x)? {
            DocKind::Project => 1,
            DocKind::Engine => 2,
            DocKind::Kit => 3,
            DocKind::Pad => 4,
            DocKind::Pattern => 5,
            DocKind::Bank => 6,
            _ => return None,
        },
    })
}

/// [re:ca74b1]: with every type shown the folder lists as is; otherwise
/// items of unticked types go, sample folders only while Folders is on.
fn visible(show: &[bool; 13], e: &Entry) -> bool {
    if show.iter().all(|s| *s) {
        return true;
    }
    match view_type(e) {
        Some(12) => show[11] && show[12],
        Some(t) => show[t],
        None => false,
    }
}

/// The current shortlist's menu ([re:200a75]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ShortAction {
    None,
    SelectAll,
    Remove,
    Crop,
    Clear,
    Export,
    DeleteSliceInfo,
}
/// The theme's content styles for the code-drawn lists: the file list
/// (`BrowserListStyle`), the source tree (`BrowserTree*`) and the path
/// navigator (`BrowserCrumbStyle`).
const ITEM_STYLE: &str = "BrowserListStyle";
const SOURCE_STYLE: &str = "BrowserTreeItemStyle";
const SOURCE_HEAD_STYLE: &str = "BrowserTreeHeadStyle";
/// The list's column header ([re:6f787e]: v1's header colours).
const HEADER_STYLE: &str = "BrowserListHeaderStyle";
const SOURCE_SELECTED_STYLE: &str = "BrowserTreeSelectedStyle";
const NAV_STYLE: &str = "BrowserCrumbStyle";

/// Listed files besides projects and kits of every family.
const EXTENSIONS: [&str; 13] = ["wav", "aif", "aiff", "patterntemplate", "rx2", "rex", "rcy", "rex2", "rcso", "gru", "kit", "g01", "g24"];

/// Whether the browser lists a file with extension `e`.
fn listed(e: &std::ffi::OsStr) -> bool {
    use dreist_formats::profile::{DocKind, doc_kind};
    let e = e.to_string_lossy();
    matches!(doc_kind(&e), Some(DocKind::Project | DocKind::Kit)) || EXTENSIONS.iter().any(|x| e.eq_ignore_ascii_case(x))
}

#[derive(Clone, Debug)]
struct Source {
    label: String,
    path: Option<PathBuf>,
    header: bool,
    /// Its category ([re:f32dc0]).
    cat: &'static str,
    /// Colour label index (0 = none).
    colour: usize,
    /// Its tree icon ([`crate::browsericons::draw`] kinds).
    icon: &'static str,
}

#[derive(Clone, Debug)]
pub struct Entry {
    name: String,
    path: PathBuf,
    dir: bool,
}

/// Back-button places kept.
const HISTORY_KEPT: usize = 100;

/// How long a preview click waits for the decode before handing it to the
/// next frames (a local one-shot decodes in a few milliseconds; a file
/// still to be downloaded must not stall the editor).
const PREVIEW_WAIT: std::time::Duration = std::time::Duration::from_millis(8);

pub struct Browser {
    page: Vec<usize>,
    sources: Vec<Source>,
    dir: Option<PathBuf>,
    entries: Vec<Entry>,
    selected: Option<usize>,
    /// [re:b86812]: the Preview play button pressed on this file, and when
    /// (its red triangle fades over 26 timer ticks).
    preview_flash: Option<(PathBuf, std::time::Instant)>,
    /// Multiple selection (`CtrlTable` cmd / shift clicks); empty when
    /// only `selected` is.
    multi: Vec<usize>,
    /// Row to open when the press is released (second click, folders).
    pending_open: Option<usize>,
    /// The file being loaded and its progress ([re:4513c6]
    ///  [re:4513c6], [re:176532]
    ///  [re:176532]).
    pub loading: Option<(std::path::PathBuf, f32)>,
    /// The listing is saved search results.
    saved: bool,
    /// A plain press on a row of a multiple selection: collapse to it on
    /// release unless it was dragged.
    collapse: Option<usize>,
    scroll: usize,
    history: Vec<PathBuf>,
    future: Vec<PathBuf>,
    pub(crate) auto_load: bool,
    /// `SliceModeButton`: audio files open the slicer instead of loading.
    slice_mode: bool,
    /// Audio file the slicer should open (taken by the editor).
    pub slice_request: Option<PathBuf>,
    /// An engine / pad / pattern / bank / MIDI file to load where the
    /// editor's selection is (v1's file click load, [re:4f89be]).
    pub load_request: Option<PathBuf>,
    /// The file press held alt.
    alt_click: bool,
    /// `PreviewButton`: audio files preview when clicked.
    preview_mode: bool,
    preview_volume: f32,
    /// The listing is search results for these terms.
    search: Option<String>,
    /// Searches made this session (Searches source category).
    searches: Vec<String>,
    favourites: Vec<PathBuf>,
    /// Shortlists (name, files) and the one listed.
    shortlists: Vec<(String, Vec<PathBuf>)>,
    shortlist: Option<String>,
    /// The open context menu's selection is files only (shortlist items
    /// enabled).
    shortlist_ok: bool,
    /// Right-clicked entry with the file list menu open.
    menu_entry: Option<usize>,
    /// Actions of the open file list menu.
    menu: Vec<FileAction>,
    /// Header Options menu open ([re:5eecea]): 1 the
    /// menu, 2 its Preview output submenu.
    header_menu: u8,
    /// Preview output pair (1-based).
    pub preview_output: usize,
    /// Decodes previews off the editor thread and keeps the recent ones.
    previews: dreist_engine::preview::PreviewLoader,
    /// "Pads track browser location" (option `PadsTrackBrowser`).
    pub pads_track: bool,
    /// Search box menu and the advanced search pane.
    pane: crate::search::SearchPane,
    /// The open source list item menu: the item (source row) and actions.
    source_menu: Option<(usize, Vec<SourceAction>)>,
    /// Source row being renamed ("Set name...").
    renaming: Option<usize>,
    /// "Export to folder": the shortlist and whether to add the folder to
    /// the favourites, taken by the editor (folder dialog).
    pub export_request: Option<(String, bool)>,
    /// The navigator's buttons as last drawn (hit tests).
    nav: std::cell::RefCell<Vec<NavButton>>,
    /// Pressed navigator button.
    nav_pressed: Option<usize>,
    /// The open navigator menu: its folder and actions.
    nav_menu: Option<(PathBuf, Vec<NavAction>)>,
    /// The Type and Size columns shown (options `FileHeaderShowStateType`
    /// / `FileHeaderShowStateSize`, off by default, [re:a81e4b]).
    show_type: bool,
    show_size: bool,
    /// The sort column and order ([re:0feb7a]).
    sort: (Column, bool),
    /// The current shortlist's list: selected rows, anchor, scroll.
    short_sel: Vec<usize>,
    short_anchor: Option<usize>,
    short_scroll: usize,
    /// The open shortlist menu's actions.
    short_menu: Vec<ShortAction>,
    /// The view options menu ([re:543135]): 1 open, 2 its Columns submenu,
    /// 3 its View submenu.
    view_menu: u8,
    /// [`VIEW_TYPES`] shown.
    show: [bool; 13],
}

/// [re:504f4e] /
/// [re:b164ed]: lock everything unless everything is locked, then unlock.
pub fn toggle_lock_all(model: &mut Model, class: &str, count: usize) {
    let e = model.engine;
    let name = |i: usize| format!("{class}{i}");
    let all = (0..count).all(|i| {
        model.engine_node(e).and_then(|n| n.children.iter().find(|c| c.class == class && c.name.as_deref() == Some(&name(i)))).is_some_and(|n| n.attr("LockState").is_some_and(|v| v.trim() == "true"))
    });
    let eng = model.engine_mut(e);
    for i in 0..count {
        let n = eng.child_mut_or_insert(class, &name(i));
        if all {
            n.attrs.retain(|(k, _)| k != "LockState");
        } else {
            n.set("LockState", "true");
        }
    }
}

/// Whether every pad / pattern of the engine is locked (button state).
pub fn all_locked(model: &Model, class: &str, count: usize) -> bool {
    let e = model.engine;
    (0..count).all(|i| {
        model
            .engine_node(e)
            .and_then(|n| n.children.iter().find(|c| c.class == class && c.name.as_deref() == Some(&format!("{class}{i}"))))
            .is_some_and(|n| n.attr("LockState").is_some_and(|v| v.trim() == "true"))
    })
}

const HEADER_MENU: [&str; 9] = ["New shortlist", "New shortlist from selection", "", "Preview output ▸", "", "(Un)Lock all pads", "(Un)Lock all patterns", "", "Pads track browser location"];

/// File list context menu items ([re:351450]
/// [re:351450]).
#[derive(Clone, Copy, Debug, PartialEq)]
enum FileAction {
    SelectAll,
    SaveSearch,
    DeleteSliceInfo,
    DeleteAllSliceInfo,
    ConfirmDelete { all: bool },
    NoSlice(bool),
    Reveal,
    RevealFolder,
    Favourite,
    FavouriteFolder,
    ShortlistCurrent,
    None,
}

/// [re:e24e01]: a [re:b277e3] file
/// ("No slice") in a folder stops browser slicing there.
pub fn slicing_allowed(dir: &Path) -> bool {
    !dir.join(dreist_formats::legacy::NOSLICE_FILE).exists()
}

fn shortlists_path() -> Option<PathBuf> {
    crate::options::settings_dir().map(|d| d.join("Shortlists.txt"))
}

/// Shortlists file (the port's own; the original keeps `Shortlists.xml`):
/// `[name]` lines followed by file paths.
fn load_shortlists() -> Vec<(String, Vec<PathBuf>)> {
    parse_shortlists(&shortlists_path().and_then(|p| std::fs::read_to_string(p).ok()).unwrap_or_default())
}

fn parse_shortlists(text: &str) -> Vec<(String, Vec<PathBuf>)> {
    let mut v: Vec<(String, Vec<PathBuf>)> = Vec::new();
    for l in text.lines().map(str::trim).filter(|l| !l.is_empty()) {
        if let Some(n) = l.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            v.push((n.to_string(), Vec::new()));
        } else if let Some(last) = v.last_mut() {
            last.1.push(PathBuf::from(l));
        }
    }
    v
}

/// Saved search results ([re:565392]): one
/// `<term>[ n].searchcache` per search, the result paths one per line.
fn searches_dir() -> Option<PathBuf> {
    crate::options::settings_dir().map(|d| d.join("Searches"))
}

fn load_saved_searches() -> Vec<String> {
    let Some(d) = searches_dir() else { return Vec::new() };
    let mut v: Vec<PathBuf> = std::fs::read_dir(d).map(|rd| rd.flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|e| e == "searchcache")).collect()).unwrap_or_default();
    v.sort();
    v.into_iter().map(|p| format!("cache:{}", p.display())).collect()
}

/// [re:97abc7]: the first free
/// "<term>.searchcache" / "<term> <n>.searchcache".
fn save_search(term: &str, files: &[PathBuf]) -> Option<PathBuf> {
    let d = searches_dir()?;
    std::fs::create_dir_all(&d).ok()?;
    let safe: String = term.chars().map(|c| if c == '/' || c == ':' { '_' } else { c }).collect();
    let mut f = d.join(format!("{safe}.searchcache"));
    let mut n = 1;
    while f.exists() {
        f = d.join(format!("{safe} {n}.searchcache"));
        n += 1;
    }
    let text: String = files.iter().map(|p| format!("{}\n", p.display())).collect();
    dreist_formats::file::write_atomic(&f, text).ok()?;
    Some(f)
}

fn save_shortlists(v: &[(String, Vec<PathBuf>)]) {
    let Some(p) = shortlists_path() else { return };
    if let Some(d) = p.parent() {
        let _ = std::fs::create_dir_all(d);
    }
    let mut text = String::new();
    for (n, files) in v {
        text.push_str(&format!("[{n}]\n"));
        for f in files {
            text.push_str(&format!("{}\n", f.display()));
        }
    }
    let _ = dreist_formats::file::write_atomic(p, text);
}

/// Favourites file (the port's own; the original keeps
/// `<UserLocalSystem>/Favourites.xml`).
fn favourites_path() -> Option<PathBuf> {
    crate::options::settings_dir().map(|d| d.join("Favourites.txt"))
}

fn load_favourites() -> Vec<PathBuf> {
    favourites_path().and_then(|p| std::fs::read_to_string(p).ok()).map(|t| t.lines().filter(|l| !l.trim().is_empty()).map(PathBuf::from).collect()).unwrap_or_default()
}

fn save_favourites(v: &[PathBuf]) {
    let Some(p) = favourites_path() else { return };
    if let Some(d) = p.parent() {
        let _ = std::fs::create_dir_all(d);
    }
    let text: String = v.iter().map(|p| format!("{}\n", p.display())).collect();
    let _ = dreist_formats::file::write_atomic(p, text);
}

/// Source list item names and colour labels ([re:8d3031];
/// the original keeps them in its Favourites / Searches /
/// Shortlists XML): the port's `SourceLabels.txt`, one
/// `key<TAB>name<TAB>colour index` line per item (key = its path).
fn labels_path() -> Option<PathBuf> {
    crate::options::settings_dir().map(|d| d.join("SourceLabels.txt"))
}

fn load_labels() -> Vec<(String, String, usize)> {
    let text = labels_path().and_then(|p| std::fs::read_to_string(p).ok()).unwrap_or_default();
    text.lines()
        .filter_map(|l| {
            let mut it = l.split('\t');
            Some((it.next()?.to_string(), it.next().unwrap_or("").to_string(), it.next().and_then(|c| c.parse().ok()).unwrap_or(0)))
        })
        .collect()
}

/// An item's (name, colour index) labels.
fn label_of(labels: &[(String, String, usize)], path: &Path) -> (Option<String>, usize) {
    let key = path.to_string_lossy();
    labels.iter().find(|(k, _, _)| *k == key).map(|(_, n, c)| ((!n.is_empty()).then(|| n.clone()), *c)).unwrap_or((None, 0))
}

fn labels_get(path: &Path) -> (Option<String>, usize) {
    label_of(&load_labels(), path)
}

fn labels_set(path: &Path, name: Option<Option<&str>>, colour: Option<usize>) {
    let Some(f) = labels_path() else { return };
    let key = path.to_string_lossy().into_owned();
    let mut v = load_labels();
    let i = v.iter().position(|(k, _, _)| *k == key).unwrap_or_else(|| {
        v.push((key.clone(), String::new(), 0));
        v.len() - 1
    });
    if let Some(n) = name {
        v[i].1 = n.unwrap_or("").to_string();
    }
    if let Some(c) = colour {
        v[i].2 = c;
    }
    v.retain(|(_, n, c)| !n.is_empty() || *c != 0);
    if let Some(d) = f.parent() {
        let _ = std::fs::create_dir_all(d);
    }
    let text: String = v.iter().map(|(k, n, c)| format!("{k}\t{n}\t{c}\n")).collect();
    let _ = dreist_formats::file::write_atomic(f, text);
}

/// Content packs removed from the source list ([re:4ad7b9]
/// drops them from the content list; the folders
/// stay): `HiddenContent.txt`.
fn hidden_content() -> Vec<PathBuf> {
    crate::options::settings_dir().and_then(|d| std::fs::read_to_string(d.join("HiddenContent.txt")).ok()).map(|t| t.lines().filter(|l| !l.is_empty()).map(PathBuf::from).collect()).unwrap_or_default()
}

fn hide_content(p: &Path) {
    let Some(d) = crate::options::settings_dir() else { return };
    let mut v = hidden_content();
    v.push(p.to_path_buf());
    let _ = std::fs::create_dir_all(&d);
    let text: String = v.iter().map(|p| format!("{}\n", p.display())).collect();
    let _ = dreist_formats::file::write_atomic(d.join("HiddenContent.txt"), text);
}

/// A path navigator segment button ([re:76849c]
/// [re:76849c] / [re:c2aa99]).
#[derive(Clone, Debug, PartialEq)]
pub struct NavButton {
    pub x: f32,
    pub w: f32,
    pub label: String,
    /// Display name (its tooltip).
    pub display: String,
    /// Path index it navigates to: keeps that many
    /// + 1 components; -1 for the single non-path button.
    pub index: i32,
    pub last: bool,
}

/// Lay out the navigator for `parts` (path components, or one label for
/// search results / a shortlist when `path` is false) in `width`:
/// - Widths [re:491500]: each part text width + 10, the last + 20 when
///   slicing is disallowed there.
/// - Fitting (total < width − 15): every part left to right.
/// - Else a ".." button (text width; "Navigate to parent folder", index
///   first − 1) then the tail parts that fit in width − 10 − w(".."); the
///   last one's display name gains " (No slice)" when slicing is
///   disallowed (only here: the fitting layout never adds it).
pub fn nav_layout(measure: &mut dyn FnMut(&str) -> f32, parts: &[String], path: bool, no_slice: bool, width: f32) -> Vec<NavButton> {
    if !path || parts.is_empty() {
        let label = parts.first().cloned().unwrap_or_else(|| "/".into());
        let w = measure(&label) + 5.0;
        return vec![NavButton { x: 0.0, w, display: label.clone(), label, index: -1, last: true }];
    }
    let mut widths: Vec<f32> = parts.iter().map(|p| measure(p) + 10.0).collect();
    if no_slice {
        *widths.last_mut().unwrap() += 20.0;
    }
    let total: f32 = widths.iter().sum();
    let n = parts.len();
    let mut out = Vec::new();
    let mut x = 0.0;
    if total < width - 15.0 {
        for i in 0..n {
            out.push(NavButton { x, w: widths[i], label: parts[i].clone(), display: parts[i].clone(), index: i as i32, last: i + 1 == n });
            x += widths[i];
        }
        return out;
    }
    let dots = measure("..");
    let first = if n < 2 {
        n - 1
    } else {
        let mut avail = width - 10.0 - dots;
        let mut k = n;
        loop {
            avail -= widths[k - 1];
            if avail < 0.0 || k <= 2 {
                break if avail < 0.0 { k } else { k - 2 };
            }
            k -= 1;
        }
    };
    out.push(NavButton { x: 0.0, w: dots, label: "..".into(), display: "Navigate to parent folder".into(), index: first as i32 - 1, last: false });
    x = dots;
    for i in first..n {
        let last = i + 1 == n;
        let display = if last && no_slice { format!("{} (No slice)", parts[i]) } else { parts[i].clone() };
        out.push(NavButton { x, w: widths[i], label: parts[i].clone(), display, index: i as i32, last });
        x += widths[i];
    }
    out
}

/// Path navigator segment menu ([re:c2aa99]).
#[derive(Clone, Copy, Debug, PartialEq)]
enum NavAction {
    DeleteSliceInfo,
    ConfirmDelete,
    NoSlice(bool),
    CopyPath,
    Reveal,
    Favourite,
    None,
}

/// Source list item menu actions ([re:c80d01]
/// [re:c80d01]).
#[derive(Clone, Copy, Debug, PartialEq)]
enum SourceAction {
    Remove,
    ConfirmRemove,
    Reveal,
    SetName,
    ColourMenu,
    Colour(usize),
    Refresh,
    None,
}

pub const SOURCE_RENAME_EDIT: &str = "SourceNameEdit";

/// The content packs: `DREIST_CONTENT`, else `Content` in the documents
/// folder.
pub fn content_dir() -> PathBuf {
    std::env::var_os("DREIST_CONTENT").map(PathBuf::from).unwrap_or_else(|| crate::options::documents_dir().join("Content"))
}

/// v1's content, beside the current format's: the packs in v1's content
/// pack database (`ContentPacks.xml` in the `System` folder of v1's user
/// support folder, `~/Library/Application Support/<vendor>/<v1>`) and the
/// folders in the `Content` folder of v1's documents folder
/// (`~/Documents/<vendor>/<v1>`), as v1's path builders make them (static
/// reading of v1's engine; v1's installer chose the packs' folder, so the
/// database is the only record of it). `EINST_V1_CONTENT` replaces both
/// (one folder of packs). The names come from the legacy table: the
/// vendor folder of the current documents folder and the document root
/// element, which is v1's name.
pub fn v1_content_dirs() -> Vec<PathBuf> {
    if let Some(d) = std::env::var_os("EINST_V1_CONTENT") {
        return subdirs(Path::new(&d));
    }
    let home = dreist_formats::paths::home();
    let vendor = Path::new(dreist_formats::legacy::DOCUMENTS_DIR).parent().map(Path::to_path_buf).unwrap_or_default();
    let v1 = dreist_formats::legacy::ROOT_ELEMENT;
    let mut v = subdirs(&home.join(&vendor).join(v1).join("Content"));
    let db = home.join("Library/Application Support").join(vendor.file_name().unwrap_or_default()).join(v1).join("System/ContentPacks.xml");
    if let Ok(text) = std::fs::read_to_string(db) {
        for p in content_pack_paths(&text) {
            if !v.contains(&p) {
                v.push(p);
            }
        }
    }
    v
}

/// The pack folders a content pack database names: every quoted value or
/// element text that is an absolute path to an existing folder (the
/// database's element names are not known, only that it holds the packs'
/// paths).
fn content_pack_paths(xml: &str) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = Vec::new();
    for part in xml.split(['"', '<', '>']) {
        let t = part.trim().replace("&amp;", "&").replace("&apos;", "'");
        if t.starts_with('/') && Path::new(&t).is_dir() {
            let p = PathBuf::from(t.trim_end_matches('/'));
            if !v.contains(&p) && !hidden(&p) {
                v.push(p);
            }
        }
    }
    v
}

fn subdirs(p: &Path) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = std::fs::read_dir(p).map(|rd| rd.flatten().map(|e| e.path()).filter(|p| p.is_dir() && !hidden(p)).collect()).unwrap_or_default();
    v.sort();
    v
}

fn hidden(p: &Path) -> bool {
    p.file_name().map(|n| n.to_string_lossy().starts_with('.')).unwrap_or(true)
}

fn sources(favourites: &[PathBuf], searches: &[String], shortlists: &[(String, Vec<PathBuf>)]) -> Vec<Source> {
    sources_with(favourites, searches, shortlists, &v1_content_dirs())
}

/// [`sources`] with v1's content packs `v1`.
fn sources_with(favourites: &[PathBuf], searches: &[String], shortlists: &[(String, Vec<PathBuf>)], v1: &[PathBuf]) -> Vec<Source> {
    let mut v = Vec::new();
    let home = dreist_formats::paths::home();
    let (labels, hidden) = (load_labels(), hidden_content());
    for cat in CATEGORIES {
        v.push(Source { label: cat.into(), path: None, header: true, cat, colour: 0, icon: "" });
        let items: Vec<(String, PathBuf)> = match cat {
            // The current format's content packs, then v1's.
            "Content" => {
                let mut dirs = subdirs(&content_dir());
                for d in v1 {
                    if !dirs.contains(d) {
                        dirs.push(d.clone());
                    }
                }
                dirs.into_iter().map(|p| (p.file_name().unwrap().to_string_lossy().into_owned(), p)).collect()
            }
            "Places" => ["", "Desktop", "Documents", "Music"].iter().map(|d| (if d.is_empty() { "Home".to_string() } else { d.to_string() }, home.join(d))).filter(|(_, p)| p.is_dir()).collect(),
            "Drives" => subdirs(Path::new("/Volumes")).into_iter().map(|p| (p.file_name().unwrap().to_string_lossy().into_owned(), p)).collect(),
            "Favourites" => favourites.iter().map(|p| (p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(), p.clone())).collect(),
            // Searches carry the terms in a pseudo path.
            // (saved ones "cache:<file>").
            "Searches" => searches
                .iter()
                .map(|t| match t.strip_prefix("cache:") {
                    Some(f) => (Path::new(f).file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default(), PathBuf::from(t)),
                    None => (t.clone(), PathBuf::from(format!("search:{t}"))),
                })
                .collect(),
            "Shortlists" => shortlists.iter().map(|(n, _)| (n.clone(), PathBuf::from(format!("shortlist:{n}")))).collect(),
            _ => Vec::new(),
        };
        for (label, path) in items {
            if cat == "Content" && hidden.contains(&path) {
                continue;
            }
            let (name, colour) = label_of(&labels, &path);
            let icon = match cat {
                "Content" => "book",
                "Places" if path == home => "home",
                "Places" if path.file_name().is_some_and(|n| n == "Desktop") => "desktop",
                "Drives" => "drive",
                "Searches" => "search",
                "Shortlists" => "list",
                _ => "folder",
            };
            v.push(Source { label: name.unwrap_or(label), path: Some(path), header: false, cat, colour, icon });
        }
    }
    v
}

fn list(dir: &Path) -> Vec<Entry> {
    let mut v: Vec<Entry> = std::fs::read_dir(dir)
        .map(|rd| {
            rd.flatten()
                .map(|e| e.path())
                .filter(|p| !hidden(p))
                // Folders, `.samples` ones too (v1 lists them; its View
                // menu's Sample Folders hides them, [re:ca74b1]).
                .filter(|p| p.is_dir() || p.extension().is_some_and(listed))
                .map(|p| Entry { name: p.file_name().unwrap().to_string_lossy().into_owned(), dir: p.is_dir(), path: p })
                .collect()
        })
        .unwrap_or_default();
    v.sort_by(|a, b| b.dir.cmp(&a.dir).then(a.name.to_lowercase().cmp(&b.name.to_lowercase())));
    v
}

/// Make every `AudioFileName` under `n` absolute against `samples`.
pub(crate) fn absolutise(n: &mut Node, samples: &Path) {
    if let Some((_, v)) = n.attrs.iter_mut().find(|(k, _)| k == "AudioFileName") {
        let rel = v.replace('\\', "/");
        if !Path::new(&rel).is_absolute() {
            *v = dreist_formats::model::resolve_sample(samples, &rel).to_string_lossy().into_owned();
        }
    }
    for c in &mut n.children {
        absolutise(c, samples);
    }
}

/// Put `file` into the selected layer of the document.
pub fn load_sample_into_doc(model: &mut Model, file: &Path) {
    let e = model.engine;
    let (p, l) = (model.pad[e], model.layer[e]);
    load_sample_into_layer(model, e, p, l, file);
}

/// A sample's name: the file stem (a REX ".truncated" suffix ignored).
pub fn sample_stem(file: &Path) -> String {
    let (f, _) = dreist_engine::rex::strip_truncated(file);
    f.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default()
}

/// Point layer (`e`, `p`, `l`) at an audio file
/// ([re:3c4465]): nothing when the pad
/// or the layer is locked; the layer keeps its parameters unless the
/// `AllocatorClearLayersOnLoad` preference is on (then a fresh layer);
/// both envelope holds are set from the sample's length; the
/// pad is named "P<n> <file>" when its name is automatic (matches one of
/// its layers' files) or it has no samples, and it has at most one sample.
/// Returns whether the sample was loaded.
pub fn load_sample_into_layer(model: &mut Model, e: usize, p: usize, l: usize, file: &Path) -> bool {
    let locked = |n: Option<&Node>| n.is_some_and(Node::locked);
    if locked(model.pad_node(e, p)) || locked(model.layer_node(e, p, l)) {
        return false;
    }
    // [re:b277e3]: a REX file loads as
    // "<file>.truncated" (slices at their musical positions, see
    // `dreist_engine::rex`).
    let rex_path;
    let file = if dreist_engine::rex::is_rex(file) && !dreist_engine::rex::strip_truncated(file).1 {
        rex_path = PathBuf::from(format!("{}.truncated", file.display()));
        rex_path.as_path()
    } else {
        file
    };
    let stem = sample_stem(file);
    // Pad name rule (before the new sample counts).
    let valid: Vec<String> = (0..crate::model::LAYERS)
        .filter_map(|k| model.layer_node(e, p, k).and_then(|n| n.attr("AudioFileName")).filter(|f| !f.is_empty()).map(|f| sample_stem(Path::new(&f.replace('\\', "/")))))
        .collect();
    let pad_name = model.pad_name(e, p).unwrap_or_default();
    let auto = valid.iter().any(|f| pad_name == format!("P{} {f}", p + 1));
    let rename = (auto || valid.is_empty()) && valid.len() < 2;
    let reset = crate::options::Options::load().bool("AllocatorClearLayersOnLoad", false);
    let eng = model.doc.child_mut_or_insert("Engine", &format!("Engine{e}"));
    let pad = eng.child_mut_or_insert("Pad", &format!("Pad{p}"));
    let layer = pad.child_mut_or_insert("Layer", &format!("Layer{l}"));
    if reset {
        *layer = Node::new("Layer", Some(&format!("Layer{l}")));
    }
    layer.children.retain(|c| c.class != "AudioFileMarkers");
    layer.set("AudioFileName", file.to_string_lossy());
    layer.child_mut_or_insert("FixedString", "LayerName").set("string", &stem);
    if let Some((frames, sr)) = dreist_engine::audio::info(file) {
        let n = layer.clone();
        let get = |q: dreist_formats::params::LayerParam| n.f32(q.name()).unwrap_or(q.default_value()).clamp(0.0, 1.0);
        let bound = n.attr("PlaybackMode").map(dreist_dsp::voice::PlaybackMode::canonical) == Some("OneShotLengthBound");
        let h = dreist_engine::hold::reset_hold_time(&get, bound, frames as f64, frames as f64, sr, false);
        layer.set("AmpEnvHold", h).set("FreeEnvHold", h);
    }
    if rename {
        pad.child_mut_or_insert("FixedString", "PadName").set("string", format!("P{} {stem}", p + 1));
    }
    true
}

/// Replace the selected engine's pads with a kit's.
pub fn load_kit_into_doc(model: &mut Model, kit: &Path) -> bool {
    let before = (!crate::project::restore_outputs()).then(|| model.doc.clone());
    let ok = load_kit_inner(model, kit);
    if let (true, Some(old)) = (ok, before) {
        crate::project::keep_outputs(&old, &mut model.doc);
    }
    ok
}

fn load_kit_inner(model: &mut Model, kit: &Path) -> bool {
    let Some(mut root) = crate::app::read_document(kit) else { return false };
    absolutise(&mut root, &dreist_formats::model::samples_dir(kit));
    let e = model.engine;
    let eng = model.doc.child_mut_or_insert("Engine", &format!("Engine{e}"));
    // [re:118dc5]: locked pads stay.
    let locked = |c: &Node| c.class == "Pad" && c.locked();
    let keep: Vec<String> = eng.children.iter().filter(|c| locked(c)).filter_map(|c| c.name.clone()).collect();
    eng.children.retain(|c| c.class != "Pad" || locked(c));
    eng.children.extend(root.children.into_iter().filter(|c| c.class == "Pad" && !c.name.as_ref().is_some_and(|n| keep.contains(n))));
    if let Some(stem) = kit.file_stem() {
        eng.set("KitName", stem.to_string_lossy());
    }
    true
}

/// The browser column (v1 draws its lists in code): `Header.skin` at the
/// top, the advanced search pane under it (closed), the path navigator,
/// the source tree beside the file list, and `Footer.skin` at the bottom.
fn build_column(skin: &mut Skin, host: &mut Ctrl) -> Result<(), EditorError> {
    let (w, h) = (host.rect.w, host.rect.h);
    let panel = |name: &str, overload: &str, r: Rect, style: &str| Ctrl {
        class: "CtrlPanel".into(),
        name: name.into(),
        overload: overload.into(),
        style: style.into(),
        rect: r,
        visible: true,
        enabled: true,
        zorder: 1,
        ..Ctrl::default()
    };
    let mut header = crate::front::load(skin, "Browser/Header.skin")?;
    header.name = "BrowserHeader".into();
    let mut footer = crate::front::load(skin, "Browser/Footer.skin")?;
    footer.name = "Footer".into();
    let (hh, fh) = (header.rect.h, footer.rect.h);
    footer.rect.y = h - fh;
    host.children.push(header);
    host.children.push(panel("Search", "BrowserSearchPane", Rect::new(0, hh, w, 0), "InvisibleStyle"));
    const NAV: i32 = 20;
    host.children.push(panel("Navigator", "BrowserPathNavigator", Rect::new(1, hh + 2, w - 2, NAV), "InvisibleStyle"));
    let top = hh + NAV + 4;
    let body = h - top - fh - 2;
    let mut split = panel("SplitView", "", Rect::new(0, top, w, body), "InvisibleStyle");
    split.class = "CtrlSplitterPanel".into();
    for (k, v) in [
        // v1 (measured): a 4 px navy bar between the tree and the list,
        // at x 103 of the column.
        ("SplitterPanelStyle", "BrowserSplitStyle"),
        ("intMinSize1", "100"),
        ("intSplitPosition", "103"),
        ("intSplitterDim", "4"),
        ("optIsVertical", "false"),
        ("optDbClkCollapse1", "false"),
        ("optSnapCollapse1", "false"),
    ] {
        split.set_attr(k, v.into());
    }
    let mut sources = panel("SourceList", "BrowserSourceList", Rect::new(0, 0, 103, body), "InvisibleStyle");
    sources.display_name = "SplitterPanel0".into();
    let pw = w - 107;
    let mut pane = panel("Browser", "", Rect::new(107, 0, pw, body), "InvisibleStyle");
    pane.display_name = "SplitterPanel1".into();
    let button = |name: &str, r: Rect, style: &str, text: &str| Ctrl {
        class: "CtrlButton".into(),
        name: name.into(),
        style: style.into(),
        text: text.into(),
        rect: r,
        visible: true,
        enabled: true,
        zorder: 1,
        ..Ctrl::default()
    };
    pane.children.push(button("ParentFolderButton", Rect::new(0, 0, pw, PARENT_H), "BrowserParentNavigationButtonStyle", "Parent folder"));
    pane.children.push(panel("ListHeader", "BrowserListHeader", Rect::new(0, HEADER_Y, pw - SLIDER_W, HEADER_H), "InvisibleStyle"));
    pane.children.push(button("ViewOptionsButton", Rect::new(pw - SLIDER_W, HEADER_Y, SLIDER_W, HEADER_H), "BrowserHeaderViewOptionsButtonStyle", "▼"));
    let lh = body - LIST_Y;
    let half = lh / 2 - LIST_BAR / 2;
    let mut lists = panel("ListSplit", "", Rect::new(0, LIST_Y, pw, lh), "InvisibleStyle");
    lists.class = "CtrlSplitterPanel".into();
    for (k, v) in [("SplitterPanelStyle", "BrowserListSplitStyle".to_string()), ("intSplitPosition", half.to_string()), ("intSplitterDim", LIST_BAR.to_string()), ("optIsVertical", "true".into())] {
        lists.set_attr(k, v);
    }
    let mut files = panel("FileList", "BrowserFileList", Rect::new(0, 0, pw, half), "InvisibleStyle");
    files.display_name = "SplitterPanel0".into();
    let mut short = panel("ShortList", "BrowserShortList", Rect::new(0, half + LIST_BAR, pw, lh - half - LIST_BAR), "InvisibleStyle");
    short.display_name = "SplitterPanel1".into();
    lists.children.push(files);
    lists.children.push(short);
    pane.children.push(lists);
    split.children.push(sources);
    split.children.push(pane);
    host.children.push(split);
    host.children.push(footer);
    Ok(())
}

impl Browser {
    /// The footer's Auto load is on.
    pub fn auto_load(&self) -> bool {
        self.auto_load
    }

    /// The Hit / Loop mode button is in Loop (slice) mode.
    pub fn slice_mode(&self) -> bool {
        self.slice_mode
    }

    pub fn build(skin: &mut Skin, root: &mut Ctrl, page: &[usize]) -> Result<Browser, EditorError> {
        let mut host = &mut *root;
        for &i in page {
            host = &mut host.children[i];
        }
        build_column(skin, host)?;
        crate::search::SearchPane::mount(skin, host).map_err(err)?;
        // [re:ddb121]..[re:ddb121]: the Spinner shows only while an
        // asynchronous search runs (the port's search is synchronous).
        for panel in ["BrowserHeader", "Search"] {
            if let Some(c) = host.find_mut(panel).and_then(|p| p.find_mut("Spinner")) {
                c.visible = false;
                c.enabled = false;
            }
        }
        let start = subdirs(&content_dir()).into_iter().next();
        let mut b = Browser {
            page: page.to_vec(),
            sources: Vec::new(),
            dir: None,
            entries: Vec::new(),
            selected: None,
            multi: Vec::new(),
            saved: false,
            pending_open: None,
            loading: None,
            previews: Default::default(),
            collapse: None,
            scroll: 0,
            history: Vec::new(),
            future: Vec::new(),
            auto_load: false,
            slice_mode: false,
            slice_request: None,
            load_request: None,
            alt_click: false,
            preview_mode: false,
            preview_flash: None,
            preview_volume: 0.793_700_5,
            search: None,
            searches: Vec::new(),
            favourites: load_favourites(),
            shortlists: load_shortlists(),
            shortlist: None,
            shortlist_ok: false,
            menu_entry: None,
            menu: Vec::new(),
            header_menu: 0,
            pane: Default::default(),
            source_menu: None,
            renaming: None,
            export_request: None,
            nav: Default::default(),
            nav_pressed: None,
            nav_menu: None,
            preview_output: 1,
            pads_track: crate::options::Options::load().bool("PadsTrackBrowser", false),
            show_type: crate::options::Options::load().bool("FileHeaderShowStateType", false),
            show_size: crate::options::Options::load().bool("FileHeaderShowStateSize", false),
            sort: (Column::Name, true),
            short_sel: Vec::new(),
            short_anchor: None,
            short_scroll: 0,
            short_menu: Vec::new(),
            view_menu: 0,
            show: {
                let o = crate::options::Options::load();
                std::array::from_fn(|t| o.bool(&view_option(t), true))
            },
        };
        b.searches = load_saved_searches();
        b.sources = sources(&b.favourites, &b.searches, &b.shortlists);
        if let Some(d) = start {
            b.go(d, false);
        }
        Ok(b)
    }

    /// Pads track browser: show `dir` unless it is already shown.
    pub fn follow(&mut self, dir: PathBuf) {
        if self.dir.as_ref() != Some(&dir) {
            self.go(dir, true);
        }
    }

    /// [re:10d1a4].
    pub fn previous_location(&self) -> Option<&Path> {
        self.history.last().map(PathBuf::as_path)
    }

    /// [re:5e8f4c].
    pub fn next_location(&self) -> Option<&Path> {
        self.future.last().map(PathBuf::as_path)
    }

    /// Show a folder (pad menu "Open parent folder in browser").
    pub fn browse(&mut self, dir: PathBuf) {
        self.go(dir, true);
    }

    fn go(&mut self, dir: PathBuf, record: bool) {
        // A shortlist becomes the current one; the folder stays.
        if let Some(n) = dir.to_str().and_then(|s| s.strip_prefix("shortlist:")) {
            self.show_shortlist(n.to_string());
            return;
        }
        if record {
            if let Some(d) = self.dir.take() {
                self.history.push(d);
                // A long browsing session keeps the recent places.
                if self.history.len() > HISTORY_KEPT {
                    self.history.remove(0);
                }
            }
            self.future.clear();
        }
        if let Some(t) = dir.to_str().and_then(|s| s.strip_prefix("search:")) {
            let t = t.to_string();
            self.run_search(&t);
            return;
        }
        if let Some(f) = dir.to_str().and_then(|s| s.strip_prefix("cache:")) {
            let text = std::fs::read_to_string(f).unwrap_or_default();
            self.entries = text
                .lines()
                .filter(|l| !l.is_empty())
                .map(PathBuf::from)
                .map(|p| Entry { name: p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(), dir: p.is_dir(), path: p })
                .collect();
            self.search = Path::new(f).file_stem().map(|s| s.to_string_lossy().into_owned());
            self.saved = true;
            self.selected = None;
            self.scroll = 0;
            self.multi.clear();
            return;
        }
        self.saved = false;
        // A favourite file: open its folder with it selected.
        let (dir, select) = if dir.is_file() { (dir.parent().map(Path::to_path_buf).unwrap_or(dir.clone()), Some(dir)) } else { (dir, None) };
        self.entries = list(&dir);
        let show = self.show;
        self.entries.retain(|e| visible(&show, e));
        sort_entries(&mut self.entries, self.sort.0, self.sort.1);
        self.dir = Some(dir);
        self.search = None;
        self.selected = select.and_then(|s| self.entries.iter().position(|e| e.path == s));
        self.scroll = 0;
        self.multi.clear();
    }

    /// [re:1f1658]; in v1 the chosen shortlist becomes the current one,
    /// listed under the folder's files ([re:dbf248]).
    fn show_shortlist(&mut self, name: String) {
        self.shortlist = Some(name);
        self.short_sel.clear();
        self.short_anchor = None;
        self.short_scroll = 0;
    }

    /// The current shortlist's files, in the header's sort order.
    fn short_entries(&self) -> Vec<Entry> {
        let files = self.shortlist.as_ref().and_then(|n| self.shortlists.iter().find(|(s, _)| s == n)).map(|(_, f)| f.clone()).unwrap_or_default();
        let mut v: Vec<Entry> = files.into_iter().map(|p| Entry { name: p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(), dir: p.is_dir(), path: p }).collect();
        sort_entries(&mut v, self.sort.0, self.sort.1);
        v
    }

    /// Sort by `col` ([re:0feb7a]): the same column again reverses the
    /// order. The selected file stays selected.
    fn sort_by(&mut self, col: Column) {
        self.sort = if self.sort.0 == col { (col, !self.sort.1) } else { (col, true) };
        let keep: Vec<PathBuf> = self.selection().into_iter().filter_map(|i| self.entries.get(i).map(|e| e.path.clone())).collect();
        let lead = self.selected.and_then(|i| self.entries.get(i).map(|e| e.path.clone()));
        sort_entries(&mut self.entries, self.sort.0, self.sort.1);
        self.selected = lead.and_then(|p| self.entries.iter().position(|e| e.path == p));
        self.multi = if keep.len() > 1 { self.entries.iter().enumerate().filter(|(_, e)| keep.contains(&e.path)).map(|(i, _)| i).collect() } else { Vec::new() };
        self.short_sel.clear();
        self.short_anchor = None;
    }

    /// The header's columns (x, width) for a list `w` wide: Name, then
    /// Type and Size when shown ([re:9195ec]).
    fn columns(&self, w: f32) -> Vec<(Column, f32, f32)> {
        let mut right = w;
        let mut v = Vec::new();
        if self.show_size {
            right -= COLUMN_W;
            v.push((Column::Size, right, COLUMN_W));
        }
        if self.show_type {
            right -= COLUMN_W;
            v.push((Column::Type, right, COLUMN_W));
        }
        v.push((Column::Name, 0.0, right));
        v.reverse();
        v
    }

    /// The navigator's parts: the folder's components, else one label
    /// ("Search Results", the shortlist's name).
    fn nav_parts(&self) -> (Vec<String>, bool) {
        if self.search.is_some() {
            return (vec!["Search Results".into()], false);
        }
        match &self.dir {
            Some(d) => (
                d.components()
                    .filter_map(|c| match c {
                        std::path::Component::Normal(s) => Some(s.to_string_lossy().into_owned()),
                        _ => None,
                    })
                    .collect(),
                true,
            ),
            None => (Vec::new(), false),
        }
    }

    /// The folder button `index` navigates to.
    fn nav_target(&self, index: i32) -> Option<PathBuf> {
        let d = self.dir.as_ref()?;
        let (parts, path) = self.nav_parts();
        if !path || index < 0 {
            return None;
        }
        let keep = (index as usize + 1).min(parts.len());
        let mut p = PathBuf::from("/");
        for c in &parts[..keep] {
            p.push(c);
        }
        let _ = d;
        Some(p)
    }

    fn nav_hit(&self, x: f32) -> Option<usize> {
        self.nav.borrow().iter().position(|b| x >= b.x && x < b.x + b.w)
    }

    /// The navigator button under `x` (tooltips).
    pub fn nav_button_at(&self, x: f32) -> Option<NavButton> {
        self.nav_hit(x).and_then(|i| self.nav.borrow().get(i).cloned())
    }

    /// [re:c2aa99]: the segment's folder menu.
    fn nav_items(&self, dir: &Path) -> Vec<(String, NavAction)> {
        let allowed = slicing_allowed(dir);
        let name = dir.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| "/".into());
        vec![
            ("Delete all slice information in this folder".into(), NavAction::DeleteSliceInfo),
            (if allowed { "Disallow slicing of files in this folder" } else { "Allow slicing of files in this folder" }.into(), NavAction::NoSlice(allowed)),
            (String::new(), NavAction::None),
            ("Copy path".into(), NavAction::CopyPath),
            ("Reveal in Finder".into(), NavAction::Reveal),
            (String::new(), NavAction::None),
            (format!("Add \"{name}\" to favourites"), NavAction::Favourite),
        ]
    }

    fn nav_action(&mut self, app: &mut App, anchor: &[usize], dir: PathBuf, a: NavAction) -> bool {
        match a {
            NavAction::DeleteSliceInfo => {
                let items = vec![
                    (dreist_gui::menu::item("Remove the slice information for all the files in this folder?", false), NavAction::None),
                    (dreist_gui::menu::item("This action cannot be undone!", false), NavAction::None),
                    (String::new(), NavAction::None),
                    ("Yes".to_string(), NavAction::ConfirmDelete),
                    ("No".into(), NavAction::None),
                ];
                let (labels, actions): (Vec<String>, Vec<NavAction>) = items.into_iter().unzip();
                self.nav_menu = Some((dir, actions));
                app.open_menu(anchor, labels, None);
                false
            }
            NavAction::ConfirmDelete => {
                for e in list(&dir).into_iter().filter(|e| !e.dir) {
                    let g = e.path.with_extension("gsa");
                    if g.exists() {
                        let _ = std::fs::remove_file(g);
                    }
                }
                true
            }
            NavAction::NoSlice(disallow) => {
                let f = dir.join(dreist_formats::legacy::NOSLICE_FILE);
                let _ = if disallow { std::fs::write(f, "No slice") } else { std::fs::remove_file(f) };
                true
            }
            NavAction::CopyPath => {
                if let Ok(mut c) = std::process::Command::new("pbcopy").stdin(std::process::Stdio::piped()).spawn() {
                    use std::io::Write;
                    if let Some(i) = c.stdin.as_mut() {
                        let _ = i.write_all(dir.to_string_lossy().as_bytes());
                    }
                    let _ = c.wait();
                }
                false
            }
            NavAction::Reveal => {
                let _ = std::process::Command::new("open").arg("-R").arg(&dir).spawn();
                false
            }
            NavAction::Favourite => {
                if !self.favourites.contains(&dir) {
                    self.favourites.push(dir);
                    save_favourites(&self.favourites);
                    self.sources = sources(&self.favourites, &self.searches, &self.shortlists);
                }
                true
            }
            NavAction::None => false,
        }
    }

    /// The item menu [re:c80d01] for source row `i` (category items only):
    /// Content — Remove, Reveal, colour; Places / Drives — Reveal;
    /// Favourites — Remove, Reveal, Set name, colour; Searches / Shortlists
    /// — Remove, Set name, colour, then Refresh search / Export to folder
    /// (and add as favourite).
    fn source_items(&self, i: usize) -> Vec<(String, SourceAction)> {
        let Some(src) = self.sources.get(i).filter(|s| !s.header) else { return Vec::new() };
        let cat = src.cat;
        let remove = !matches!(cat, "Places" | "Drives");
        let reveal = !matches!(cat, "Searches" | "Shortlists");
        let name = !matches!(cat, "Content" | "Places" | "Drives");
        let mut v = Vec::new();
        if remove {
            v.push((format!("Remove \"{}\"", src.label), SourceAction::Remove));
        }
        if reveal {
            v.push((format!("Reveal \"{}\" in Finder", src.label), SourceAction::Reveal));
        }
        if name || remove {
            v.push((String::new(), SourceAction::None));
        }
        if name {
            v.push(("Set name...".into(), SourceAction::SetName));
        }
        if remove {
            v.push(("Set colour label... ▸".into(), SourceAction::ColourMenu));
        }
        // v1 offers no shortlist export (a v2 addition).
        if cat == "Searches" {
            v.push((String::new(), SourceAction::None));
            v.push(("Refresh search".into(), SourceAction::Refresh));
        }
        v
    }

    fn source_action(&mut self, app: &mut App, anchor: &[usize], i: usize, a: SourceAction) -> bool {
        let Some(src) = self.sources.get(i).cloned() else { return false };
        let Some(path) = src.path.clone() else { return false };
        let open = |me: &mut Browser, app: &mut App, items: Vec<(String, SourceAction)>| {
            let (labels, actions): (Vec<String>, Vec<SourceAction>) = items.into_iter().unzip();
            me.source_menu = Some((i, actions));
            app.open_menu(anchor, labels, None);
        };
        match a {
            // Removing [re:4472d0] asks (Yes / No) first.
            SourceAction::Remove => {
                let q = format!("Are you sure you want to remove \"{}\". This action cannot be undone...", src.label);
                open(
                    self,
                    app,
                    vec![(dreist_gui::menu::item(q, false), SourceAction::None), (String::new(), SourceAction::None), ("Yes".into(), SourceAction::ConfirmRemove), ("No".into(), SourceAction::None)],
                );
                false
            }
            // [re:4ad7b9]: favourites
            // leave the list, saved searches and shortlists lose their
            // files, content packs leave the content list.
            SourceAction::ConfirmRemove => {
                let key = path.to_string_lossy().into_owned();
                match src.cat {
                    "Favourites" => {
                        self.favourites.retain(|f| *f != path);
                        save_favourites(&self.favourites);
                    }
                    "Searches" => {
                        if let Some(f) = key.strip_prefix("cache:") {
                            let _ = std::fs::remove_file(f);
                        }
                        let term = key.strip_prefix("search:").unwrap_or(&key).to_string();
                        self.searches.retain(|t| *t != term && *t != key);
                    }
                    "Shortlists" => {
                        let n = key.strip_prefix("shortlist:").unwrap_or(&key).to_string();
                        self.shortlists.retain(|(s, _)| *s != n);
                        save_shortlists(&self.shortlists);
                        if self.shortlist.as_deref() == Some(n.as_str()) {
                            self.shortlist = None;
                            self.entries.clear();
                        }
                    }
                    "Content" => hide_content(&path),
                    _ => return false,
                }
                labels_set(&path, Some(None), Some(0));
                self.sources = sources(&self.favourites, &self.searches, &self.shortlists);
                true
            }
            SourceAction::Reveal => {
                let _ = std::process::Command::new("open").arg("-R").arg(&path).spawn();
                false
            }
            // [re:a2fc7f]: an edit box over the row.
            SourceAction::SetName => {
                let Some(c) = app.ctrl(anchor).cloned() else { return false };
                let y = c.rect.y + (i as f32 * SOURCE_ROW) as i32 + 2;
                let rect = dreist_gui::ctrl::Rect::new(c.rect.x + 14, y, (c.rect.w - 18).max(40), SOURCE_ROW as i32 - 4);
                app.open_edit_box(&anchor[..anchor.len() - 1], SOURCE_RENAME_EDIT, rect, src.label.clone());
                self.renaming = Some(i);
                true
            }
            SourceAction::ColourMenu => {
                self.source_menu = Some((i, (0..=crate::padmenu::COLOURS.len()).map(SourceAction::Colour).collect()));
                app.open_menu(anchor, vec![crate::padmenu::colour_grid_item()], None);
                false
            }
            // Colour label [re:ede7fd] (cell 0 = none).
            SourceAction::Colour(k) => {
                labels_set(&path, None, Some(k));
                self.sources = sources(&self.favourites, &self.searches, &self.shortlists);
                true
            }
            // [re:0c9d4a]: run it again.
            SourceAction::Refresh => {
                let key = path.to_string_lossy().into_owned();
                match key.strip_prefix("search:") {
                    Some(t) => {
                        let t = t.to_string();
                        self.run_search(&t);
                    }
                    None => self.go(path, true),
                }
                true
            }
            SourceAction::None => false,
        }
    }

    /// [re:af0614]: the shortlist's
    /// files copied into `<dir>/<name>`, which joins the favourites when
    /// asked.
    pub fn export_shortlist(&mut self, name: &str, dir: &Path, favourite: bool) -> std::io::Result<PathBuf> {
        let to = dir.join(name);
        std::fs::create_dir_all(&to)?;
        let files = self.shortlists.iter().find(|(n, _)| n == name).map(|(_, f)| f.clone()).unwrap_or_default();
        for f in files {
            if let Some(n) = f.file_name() {
                if f.is_dir() {
                    continue;
                }
                std::fs::copy(&f, to.join(n))?;
            }
        }
        if favourite && !self.favourites.contains(&to) {
            self.favourites.push(to.clone());
            save_favourites(&self.favourites);
            self.sources = sources(&self.favourites, &self.searches, &self.shortlists);
        }
        Ok(to)
    }

    /// The rename commit ([re:a2fc7f] →
    /// [re:a2fc7f]): shortlists take the new name,
    /// other items show it.
    fn commit_source_name(&mut self, app: &mut App, path: &[usize], text: &str) {
        let parent = path[..path.len() - 1].to_vec();
        if let Some(c) = app.ctrl_mut(&parent) {
            c.children.retain(|k| k.name != SOURCE_RENAME_EDIT);
        }
        let Some(i) = self.renaming.take() else { return };
        let Some(src) = self.sources.get(i).cloned() else { return };
        let Some(p) = src.path else { return };
        let name = text.trim();
        if name.is_empty() {
            return;
        }
        if src.cat == "Shortlists" {
            let old = p.to_string_lossy().strip_prefix("shortlist:").unwrap_or_default().to_string();
            if !self.shortlists.iter().any(|(n, _)| n == name) {
                if let Some(s) = self.shortlists.iter_mut().find(|(n, _)| *n == old) {
                    s.0 = name.to_string();
                }
                save_shortlists(&self.shortlists);
                let colour = labels_get(&p).1;
                labels_set(&p, Some(None), Some(0));
                labels_set(&PathBuf::from(format!("shortlist:{name}")), None, Some(colour));
                if self.shortlist.as_deref() == Some(old.as_str()) {
                    self.shortlist = Some(name.to_string());
                }
            }
        } else {
            labels_set(&p, Some(Some(name)), None);
        }
        self.sources = sources(&self.favourites, &self.searches, &self.shortlists);
    }

    /// [re:7dd3b2] / [re:515934] / [re:5b43a5]: a drop on source list row
    /// `row`: files (no folders) on a shortlist join it; folders only on the
    /// Favourites category or one of its rows become favourites. True when
    /// something changed.
    pub fn drop_on_source(&mut self, row: usize, paths: &[PathBuf]) -> bool {
        let Some(src) = self.sources.get(row).cloned() else { return false };
        if paths.is_empty() {
            return false;
        }
        match src.cat {
            "Shortlists" if !src.header => {
                if paths.iter().any(|p| p.is_dir()) {
                    return false;
                }
                let name = src.path.as_ref().map(|p| p.to_string_lossy().trim_start_matches("shortlist:").to_string()).unwrap_or_default();
                let Some((_, files)) = self.shortlists.iter_mut().find(|(n, _)| *n == name) else { return false };
                for p in paths {
                    if !files.contains(p) {
                        files.push(p.clone());
                    }
                }
                save_shortlists(&self.shortlists);
                true
            }
            "Favourites" => {
                if !paths.iter().all(|p| p.is_dir()) {
                    return false;
                }
                for p in paths {
                    if !self.favourites.contains(p) {
                        self.favourites.push(p.clone());
                    }
                }
                save_favourites(&self.favourites);
                self.sources = sources(&self.favourites, &self.searches, &self.shortlists);
                true
            }
            _ => false,
        }
    }

    /// The source list rows as (category, header?).
    pub fn source_rows(&self) -> Vec<(&'static str, bool)> {
        self.sources.iter().map(|s| (s.cat, s.header)).collect()
    }

    /// Files dropped on the shortlist pane: added to the current shortlist
    /// (a new one when there is none). True when the shortlist changed.
    pub fn add_to_current_shortlist(&mut self, paths: Vec<PathBuf>) -> bool {
        if paths.is_empty() {
            return false;
        }
        let k = self.shortlist.as_ref().and_then(|n| self.shortlists.iter().position(|(s, _)| s == n));
        let Some(k) = k else {
            let k = self.new_shortlist(paths);
            self.shortlist = self.shortlists.get(k).map(|s| s.0.clone());
            return true;
        };
        let files = &mut self.shortlists[k].1;
        let before = files.len();
        for p in paths {
            if !files.contains(&p) {
                files.push(p);
            }
        }
        let changed = files.len() != before;
        save_shortlists(&self.shortlists);
        changed
    }

    /// [re:6fa118]: "Shortlist n".
    fn new_shortlist(&mut self, files: Vec<PathBuf>) -> usize {
        let n = (1..).find(|k| !self.shortlists.iter().any(|(s, _)| *s == format!("Shortlist {k}"))).unwrap_or(1);
        self.shortlists.push((format!("Shortlist {n}"), files));
        save_shortlists(&self.shortlists);
        self.sources = sources(&self.favourites, &self.searches, &self.shortlists);
        self.shortlists.len() - 1
    }

    /// The file list context menu for entry `i` ([re:351450]; v1's order
    /// [re:351450]): with two or
    /// more rows selected the reveal / shortlist / favourite items act on
    /// the whole selection.
    fn file_menu(&mut self, i: usize) -> Vec<(String, FileAction)> {
        use dreist_gui::menu::item;
        let entry = self.entries.get(i).cloned();
        let name = entry.as_ref().map(|e| e.name.clone()).unwrap_or_default();
        let sel: Vec<Entry> = self.selection().into_iter().filter_map(|k| self.entries.get(k).cloned()).collect();
        let many = sel.len() >= 2;
        let search = self.search.is_some();
        let listing = search;
        let files_only = !sel.is_empty() && sel.iter().all(|e| !e.dir);
        let folders_only = !sel.is_empty() && sel.iter().all(|e| e.dir);
        let mut v = vec![("Select all".to_string(), FileAction::SelectAll), (String::new(), FileAction::None)];
        let gsa = entry.as_ref().is_some_and(|e| e.path.with_extension("gsa").exists());
        v.push((item("Delete slice information", gsa), FileAction::DeleteSliceInfo));
        v.push(("Delete all slice information in this folder".into(), FileAction::DeleteAllSliceInfo));
        if let (Some(d), false) = (&self.dir, listing) {
            let allowed = slicing_allowed(d);
            v.push((if allowed { "Disallow slicing of files in this folder" } else { "Allow slicing of files in this folder" }.into(), FileAction::NoSlice(allowed)));
        }
        v.push((String::new(), FileAction::None));
        v.push((if many { "Reveal selected items in Finder".to_string() } else { format!("Reveal \"{name}\" in Finder") }, FileAction::Reveal));
        v.push((item("Reveal current folder in Finder", !listing), FileAction::RevealFolder));
        v.push((String::new(), FileAction::None));
        // v1 ([re:b9350b]): one item, to the current shortlist.
        let label = if many { "Shortlist selected items".to_string() } else { format!("Shortlist \"{name}\"") };
        self.shortlist_ok = files_only;
        v.push((item(label, files_only), FileAction::ShortlistCurrent));
        v.push((String::new(), FileAction::None));
        let fav = if many { "Add selected items to favourites".to_string() } else { format!("Add \"{name}\" to favourites") };
        v.push((item(fav, folders_only), FileAction::Favourite));
        if !listing {
            v.push(("Add current folder to favourites".into(), FileAction::FavouriteFolder));
        } else if search && !self.saved {
            v.push(("Save search results".into(), FileAction::SaveSearch));
        }
        v
    }

    /// The menu's target rows: the selection.
    fn menu_targets(&self) -> Vec<Entry> {
        self.selection().into_iter().filter_map(|k| self.entries.get(k).cloned()).collect()
    }

    fn file_action(&mut self, app: &mut App, anchor: &[usize], a: FileAction, _model: &mut Model, _out: &mut Vec<Command>) -> bool {
        let open = |items: Vec<(String, FileAction)>, app: &mut App, me: &mut Browser| {
            let (labels, actions): (Vec<String>, Vec<FileAction>) = items.into_iter().unzip();
            me.menu = actions;
            app.open_menu(anchor, labels, None);
        };
        match a {
            FileAction::SelectAll => {
                self.multi = (0..self.entries.len()).collect();
                if self.multi.len() < 2 {
                    self.selected = self.multi.pop();
                }
                true
            }
            // [re:a3462e] → [re:4473cd]
            // [re:4473cd]: the saved search joins the Searches category.
            FileAction::SaveSearch => {
                let files: Vec<PathBuf> = self.entries.iter().map(|e| e.path.clone()).collect();
                let term = self.search.clone().unwrap_or_default();
                if let Some(f) = save_search(&term, &files) {
                    self.searches.push(format!("cache:{}", f.display()));
                    self.sources = sources(&self.favourites, &self.searches, &self.shortlists);
                }
                true
            }
            // Deleting the slice analysis [re:18f4de] asks first.
            FileAction::DeleteSliceInfo | FileAction::DeleteAllSliceInfo => {
                let all = a == FileAction::DeleteAllSliceInfo;
                let q = if all { "Remove the slice information for all the files in this folder?" } else { "Remove the slice information for all the selected files?" };
                open(
                    vec![
                        (dreist_gui::menu::item(q, false), FileAction::None),
                        (dreist_gui::menu::item("This action cannot be undone!", false), FileAction::None),
                        (String::new(), FileAction::None),
                        ("Yes".into(), FileAction::ConfirmDelete { all }),
                        ("No".into(), FileAction::None),
                    ],
                    app,
                    self,
                );
                false
            }
            // [re:18f4de]: every selected row's `.gsa`.
            FileAction::ConfirmDelete { all } => {
                let files: Vec<PathBuf> = if all { self.entries.iter().filter(|e| !e.dir).map(|e| e.path.clone()).collect() } else { self.menu_targets().into_iter().map(|e| e.path).collect() };
                for f in files {
                    let g = f.with_extension("gsa");
                    if g.exists() {
                        let _ = std::fs::remove_file(g);
                    }
                }
                true
            }
            FileAction::NoSlice(disallow) => {
                if let Some(d) = &self.dir {
                    let f = d.join(dreist_formats::legacy::NOSLICE_FILE);
                    let _ = if disallow { std::fs::write(f, "No slice") } else { std::fs::remove_file(f) };
                }
                true
            }
            FileAction::Reveal | FileAction::RevealFolder => {
                let ts: Vec<PathBuf> = if a == FileAction::Reveal { self.menu_targets().into_iter().map(|e| e.path).collect() } else { self.dir.clone().into_iter().collect() };
                if !ts.is_empty() {
                    let _ = std::process::Command::new("open").arg("-R").args(ts).spawn();
                }
                false
            }
            FileAction::Favourite | FileAction::FavouriteFolder => {
                let ts: Vec<PathBuf> = if a == FileAction::Favourite { self.menu_targets().into_iter().filter(|e| e.dir).map(|e| e.path).collect() } else { self.dir.clone().into_iter().collect() };
                for t in ts {
                    if !self.favourites.contains(&t) {
                        self.favourites.push(t);
                    }
                }
                save_favourites(&self.favourites);
                self.sources = sources(&self.favourites, &self.searches, &self.shortlists);
                true
            }
            FileAction::ShortlistCurrent => {
                let paths: Vec<PathBuf> = self.menu_targets().into_iter().map(|e| e.path).collect();
                self.add_to_current_shortlist(paths)
            }
            FileAction::None => false,
        }
    }

    /// Quick search ([re:b9443a]): one
    /// "File name Contains <text>" option over the current folder (sub
    /// folders and favourites as the search box menu sets them).
    fn run_search(&mut self, terms: &str) {
        self.entries = self.scan(&[crate::search::SearchOption::name_contains(terms)], false);
        self.search = Some(terms.to_string());
        self.saved = false;
        if !self.searches.iter().any(|s| s == terms) {
            self.searches.push(terms.to_string());
            self.sources = sources(&self.favourites, &self.searches, &self.shortlists);
        }
        self.selected = None;
        self.scroll = 0;
        self.multi.clear();
    }

    /// Search locations ([re:a88f89]): the last folder
    /// shown, plus the favourites when "Search within favourites" is on.
    fn scan(&self, opts: &[crate::search::SearchOption], match_all: bool) -> Vec<Entry> {
        let deep = self.pane.sub_folders;
        let mut locs = vec![(self.dir.clone().unwrap_or_else(content_dir), deep)];
        if self.pane.favourites {
            locs.extend(self.favourites.iter().filter(|f| f.is_dir()).map(|f| (f.clone(), deep)));
        }
        let mut v: Vec<Entry> =
            crate::search::scan(&locs, opts, match_all).into_iter().map(|(p, dir)| Entry { name: p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(), path: p, dir }).collect();
        v.sort_by(|a, b| b.dir.cmp(&a.dir).then(a.name.to_lowercase().cmp(&b.name.to_lowercase())));
        v
    }

    /// Advanced search pane "Search".
    fn run_advanced(&mut self, opts: &[crate::search::SearchOption], match_all: bool) {
        self.entries = self.scan(opts, match_all);
        self.search = Some("Advanced search".into());
        self.saved = false;
        self.selected = None;
        self.scroll = 0;
        self.multi.clear();
    }

    /// Preview an audio file ([re:aeb0c0]): at once when it is cached or
    /// decodes quickly, else from [`Browser::poll_preview`] once decoded.
    fn preview(&mut self, path: &Path, out: &mut Vec<Command>) {
        if let Some(d) = self.previews.request(path, PREVIEW_WAIT) {
            out.push(Command::Preview(Some(d)));
        }
    }

    /// A preview whose file finished decoding (called every frame);
    /// whether the play button's flash still fades (repaint).
    pub fn poll_preview(&mut self, out: &mut Vec<Command>) -> bool {
        if let Some(d) = self.previews.take_ready() {
            out.push(Command::Preview(Some(d)));
        }
        if self.preview_flash.as_ref().is_some_and(|(_, t)| t.elapsed() > PREVIEW_FLASH) {
            self.preview_flash = None;
            return true;
        }
        self.preview_flash.is_some()
    }

    /// [re:b86812]: the selected row's Preview play button (an audio file,
    /// Auto play off): its left edge in the row.
    fn preview_button_x(&self, en: &Entry, selected: bool, name_w: f32) -> Option<f32> {
        let ext = en.path.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
        (selected && !en.dir && !self.preview_mode && crate::search::is_audio(&ext)).then_some(name_w - PREVIEW_ICON - 2.0)
    }

    /// Whether a press on this entry previews it (Preview on, Auto load
    /// off, not a file the slicer takes; [re:aeb0c0]).
    fn previews_on_press(&self, en: &Entry) -> bool {
        let ext = en.path.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
        !en.dir && self.preview_mode && !self.auto_load && matches!(ext.as_str(), "wav" | "aif" | "aiff") && !(self.slice_mode && en.path.parent().is_none_or(slicing_allowed))
    }

    #[cfg(test)]
    pub fn set_preview_mode(&mut self, on: bool) {
        self.preview_mode = on;
    }

    #[cfg(test)]
    pub fn pane_sub_folders(&self) -> bool {
        self.pane.sub_folders
    }

    #[cfg(test)]
    pub fn test_shortlist(&mut self, files: Vec<PathBuf>) -> String {
        let k = self.new_shortlist(files);
        self.shortlists[k].0.clone()
    }

    #[cfg(test)]
    pub fn source_row(&self, label: &str) -> Option<(usize, usize)> {
        self.sources.iter().position(|s| s.label == label).map(|i| (i, self.sources[i].colour))
    }

    #[cfg(test)]
    pub fn current_dir(&self) -> Option<&Path> {
        self.dir.as_deref()
    }

    #[cfg(test)]
    pub fn test_short_names(&self) -> Vec<String> {
        self.short_entries().into_iter().map(|e| e.name).collect()
    }

    #[cfg(test)]
    pub fn test_columns(&self) -> (bool, bool) {
        (self.show_type, self.show_size)
    }

    #[cfg(test)]
    pub fn entry_paths(&self) -> Vec<PathBuf> {
        self.entries.iter().map(|e| e.path.clone()).collect()
    }

    /// Selected rows ([re:e4d468]).
    pub fn selection(&self) -> Vec<usize> {
        if self.multi.is_empty() { self.selected.into_iter().collect() } else { self.multi.clone() }
    }

    /// [re:fd547d]: the selected entries'
    /// paths. A drag ends any pending collapse of the selection.
    pub fn drag_items(&self) -> Vec<PathBuf> {
        self.selection().into_iter().filter_map(|i| self.entries.get(i)).map(|e| e.path.clone()).collect()
    }

    /// The press became a drag: keep the multiple selection.
    pub fn drag_started(&mut self) {
        self.collapse = None;
        self.pending_open = None;
    }

    /// Open or close the advanced search pane.
    pub fn toggle_search_pane(&mut self, app: &mut App) {
        let open = !self.pane.is_open();
        self.pane.set_open(app, &self.page, open);
    }

    pub fn page(&self) -> &[usize] {
        &self.page
    }

    pub fn sync(&mut self, app: &mut App, model: &Model) {
        // [re:381995]: the list pane's button.
        let previous = self.search.is_some();
        if let Some(b) = app.root.path_of("ParentFolderButton").filter(|p| p.starts_with(&self.page)).and_then(|p| app.ctrl_mut(&p)) {
            b.text = if previous { "Previous location" } else { "Parent folder" }.into();
        }
        self.pane.sync(app, &self.page);
        let (lp, lt) = (all_locked(model, "Pad", 64), all_locked(model, "StepPattern", 24));
        if let Some(page) = app.ctrl_mut(&self.page) {
            if let Some(b) = page.find_mut("LockPadsButton") {
                b.on = lp;
            }
            if let Some(b) = page.find_mut("LockPatternsButton") {
                b.on = lt;
            }
        }
        let auto = self.auto_load;
        let Some(page) = app.ctrl_mut(&self.page) else { return };
        if let Some(b) = page.find_mut("InteractionModeButton") {
            b.on = auto;
        }
        if let Some(b) = page.find_mut("HitLoopModeButton") {
            b.on = self.slice_mode;
        }
        if let Some(b) = page.find_mut("PreviewButton") {
            b.on = self.preview_mode;
        }
        if let Some(b) = page.find_mut("PreviewVolumeKnob") {
            b.value = self.preview_volume;
        }
    }

    fn open(&mut self, i: usize, model: &mut Model, out: &mut Vec<Command>) -> bool {
        let Some(en) = self.entries.get(i).cloned() else { return false };
        self.open_entry(en, model, out)
    }

    /// Open a file or folder of either list (see [`Browser::open`]).
    fn open_entry(&mut self, en: Entry, model: &mut Model, out: &mut Vec<Command>) -> bool {
        if en.dir {
            self.go(en.path, true);
            return true;
        }
        let ext = en.path.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
        self.loading = Some((en.path.clone(), 0.0));
        // Guru files: preset → a new document, kit → the
        // current engine, pattern → the current slot, bank → all slots.
        if let Some(k) = crate::guru::kind(&en.path) {
            let e = model.engine;
            let r = match k {
                crate::guru::Kind::Preset => {
                    out.push(Command::LoadProject(en.path));
                    return true;
                }
                crate::guru::Kind::Kit => crate::guru::import_kit(&mut model.doc, e, &en.path),
                crate::guru::Kind::Bank => crate::guru::import_bank(&mut model.doc, e, &en.path),
                crate::guru::Kind::Pattern => {
                    let slot = model.current_pattern(e);
                    crate::guru::import_pattern(&mut model.doc, e, slot, &en.path)
                }
            };
            if r.is_ok() {
                out.push(model.rebuild_command());
            }
            return r.is_ok();
        }
        let kind = dreist_formats::profile::doc_kind(&ext);
        match ext.as_str() {
            _ if kind == Some(dreist_formats::profile::DocKind::Project) => out.push(Command::LoadProject(en.path)),
            _ if kind == Some(dreist_formats::profile::DocKind::Kit) => {
                if load_kit_into_doc(model, &en.path) {
                    out.push(model.rebuild_command());
                }
            }
            "wav" | "aif" | "aiff" if self.slice_mode && en.path.parent().is_none_or(slicing_allowed) => self.slice_request = Some(en.path),
            e if crate::search::is_audio(e) && self.preview_mode && !self.auto_load => self.preview(&en.path, out),
            e if crate::search::is_audio(e) => {
                // v1: with alt, the pad's first layer without a sample.
                let (ed, p) = (model.engine, model.pad[model.engine]);
                match self.alt_click.then(|| (0..crate::model::LAYERS).find(|&l| !model.layer(ed, p, l).valid)) {
                    Some(Some(l)) => {
                        load_sample_into_layer(model, ed, p, l, &en.path);
                    }
                    Some(None) => return false,
                    None => load_sample_into_doc(model, &en.path),
                }
                out.push(model.rebuild_command());
            }
            // v1 loads every supported part on a click ([re:4f89be]): engines,
            // pads, patterns, banks and MIDI patterns where the selection is.
            _ if kind.is_some() || matches!(ext.as_str(), "mid" | "midi") => self.load_request = Some(en.path),
            _ => return false,
        }
        true
    }

    pub fn handle(&mut self, app: &mut App, model: &mut Model, ev: &UiEvent, out: &mut Vec<Command>) -> bool {
        if let Some(a) = self.pane.handle(app, &self.page, ev) {
            match a {
                crate::search::PaneAction::Handled => {}
                crate::search::PaneAction::Quick(t) => self.run_search(&t),
                crate::search::PaneAction::Run(o, m) => self.run_advanced(&o, m),
            }
            return true;
        }
        match ev {
            UiEvent::TextCommitted { path, name, text } if name == "SearchBox" && path.starts_with(&self.page) => {
                let t = text.trim().to_string();
                if t.is_empty() {
                    if let Some(d) = self.dir.clone() {
                        self.go(d, false);
                    }
                } else {
                    self.pane.add_recent(&t);
                    self.run_search(&t);
                }
                true
            }
            UiEvent::ValueChanged { path, name, value, .. } if name == "PreviewVolumeKnob" && path.starts_with(&self.page) => {
                self.preview_volume = *value;
                out.push(crate::command::msg(dreist_engine::binding::id::SET_PREVIEW_VOLUME, &[], &[*value]));
                true
            }
            // Path navigator buttons: a click navigates
            // ([re:2876c2] → [re:2aae14]),
            // a right press opens the folder menu.
            UiEvent::Mouse { overload, kind: MouseKind::Down, x, .. } if overload == "BrowserPathNavigator" => {
                self.nav_pressed = self.nav_hit(*x);
                false
            }
            UiEvent::Mouse { overload, kind: MouseKind::Up, x, .. } if overload == "BrowserPathNavigator" => {
                let Some(i) = self.nav_pressed.take().filter(|&i| self.nav_hit(*x) == Some(i)) else { return false };
                let index = self.nav.borrow()[i].index;
                match self.nav_target(index) {
                    Some(p) => {
                        self.go(p, true);
                        true
                    }
                    None => false,
                }
            }
            UiEvent::Mouse { path, overload, kind: MouseKind::RightDown, x, .. } if overload == "BrowserPathNavigator" => {
                let Some(i) = self.nav_hit(*x) else { return false };
                let index = self.nav.borrow()[i].index;
                let Some(dir) = self.nav_target(index) else { return false };
                let (labels, actions): (Vec<String>, Vec<NavAction>) = self.nav_items(&dir).into_iter().unzip();
                self.nav_menu = Some((dir, actions));
                app.open_menu(path, labels, None);
                true
            }
            UiEvent::MenuSelected { anchor, index, .. } if self.nav_menu.is_some() && anchor.starts_with(&self.page) => {
                let Some((dir, actions)) = self.nav_menu.take() else { return false };
                let a = actions.get(*index).copied().unwrap_or(NavAction::None);
                let anchor = anchor.clone();
                self.nav_action(app, &anchor, dir, a)
            }
            // Source list item menu ([re:c80d01]).
            UiEvent::Mouse { path, overload, kind: MouseKind::RightDown, y, .. } if overload == "BrowserSourceList" => {
                let i = (*y / SOURCE_ROW) as usize;
                let items = self.source_items(i);
                if items.is_empty() {
                    return false;
                }
                let (labels, actions): (Vec<String>, Vec<SourceAction>) = items.into_iter().unzip();
                self.source_menu = Some((i, actions));
                app.open_menu(path, labels, None);
                true
            }
            UiEvent::MenuSelected { anchor, index, .. } if self.source_menu.is_some() && anchor.starts_with(&self.page) => {
                let Some((i, actions)) = self.source_menu.take() else { return false };
                let a = actions.get(*index).copied().unwrap_or(SourceAction::None);
                let anchor = anchor.clone();
                self.source_action(app, &anchor, i, a)
            }
            UiEvent::TextCommitted { path, name, text } if name == SOURCE_RENAME_EDIT => {
                let (path, text) = (path.clone(), text.clone());
                self.commit_source_name(app, &path, &text);
                true
            }
            // File list context menu.
            UiEvent::Mouse { path, overload, kind: MouseKind::RightDown, y, .. } if overload == "BrowserFileList" => {
                if self.dir.is_none() && self.entries.is_empty() {
                    return false;
                }
                let i = self.scroll + (*y / FILE_ROW) as usize;
                // A row outside the selection becomes the only selection.
                if i < self.entries.len() && !self.selection().contains(&i) {
                    self.selected = Some(i);
                    self.multi.clear();
                }
                self.menu_entry = Some(i);
                let (labels, actions): (Vec<String>, Vec<FileAction>) = self.file_menu(i).into_iter().unzip();
                self.menu = actions;
                app.open_menu(path, labels, None);
                true
            }
            UiEvent::MenuSelected { anchor, index, .. } if anchor.starts_with(&self.page) && !self.menu.is_empty() => {
                let a = std::mem::take(&mut self.menu).get(*index).copied().unwrap_or(FileAction::None);
                let anchor = anchor.clone();
                let r = self.file_action(app, &anchor, a, model, out);
                if self.menu.is_empty() {
                    self.menu_entry = None;
                }
                r
            }
            UiEvent::Wheel { overload, dy, .. } if overload == "BrowserFileList" => {
                self.scroll_by(-(dy.round() as i32) * 3);
                true
            }
            UiEvent::Mouse { path, overload, kind: MouseKind::Up, x, y } if overload == "BrowserFileList" => {
                let inside = app.ctrl(path).is_some_and(|c| *x >= 0.0 && *y >= 0.0 && *x < c.rect.w as f32 && *y < c.rect.h as f32);
                if let Some(i) = self.pending_open.take().filter(|_| inside) {
                    return self.open(i, model, out);
                }
                if let Some(i) = self.collapse.take() {
                    self.multi.clear();
                    self.selected = Some(i);
                    return true;
                }
                false
            }
            UiEvent::Mouse { overload, kind: MouseKind::Down, y, .. } if overload == "BrowserSourceList" => {
                let i = (*y / SOURCE_ROW) as usize;
                if let Some(Source { path: Some(p), .. }) = self.sources.get(i).cloned() {
                    self.go(p, true);
                    return true;
                }
                false
            }
            UiEvent::Mouse { path, overload, kind: MouseKind::Down, x, y } if overload == "BrowserFileList" => {
                let i = self.scroll + (*y / FILE_ROW) as usize;
                if i >= self.entries.len() {
                    return false;
                }
                // [re:b86812]: a press on the selected row's play button
                // previews the file.
                let w = app.ctrl(path).map(|c| c.rect.w as f32).unwrap_or(0.0);
                let name_w = self.columns(w - SLIDER_W as f32).first().map(|c| c.2).unwrap_or(w);
                let yr = *y - (i - self.scroll) as f32 * FILE_ROW;
                if let Some(ix) = self.preview_button_x(&self.entries[i], self.selected == Some(i) && self.multi.is_empty(), name_w) {
                    if (ix..ix + PREVIEW_ICON).contains(x) && yr > 1.0 && yr < FILE_ROW {
                        let f = self.entries[i].path.clone();
                        self.preview(&f, out);
                        self.preview_flash = Some((f, std::time::Instant::now()));
                        return true;
                    }
                }
                // cmd toggles the row, shift extends from the anchor.
                if app.mods.cmd || app.mods.shift {
                    let mut set = self.selection();
                    if app.mods.shift {
                        let a = self.selected.unwrap_or(i);
                        set = (a.min(i)..=a.max(i)).collect();
                    } else if let Some(k) = set.iter().position(|&s| s == i) {
                        set.remove(k);
                    } else {
                        set.push(i);
                        set.sort_unstable();
                    }
                    if !app.mods.shift {
                        self.selected = if set.contains(&i) { Some(i) } else { set.first().copied() };
                    }
                    self.multi = if set.len() > 1 { set } else { Vec::new() };
                    return true;
                }
                if self.multi.len() > 1 && self.multi.contains(&i) {
                    self.collapse = Some(i);
                    return true;
                }
                self.multi.clear();
                self.alt_click = app.mods.alt;
                let again = self.selected == Some(i);
                self.selected = Some(i);
                // A second click / a folder opens on release, unless the
                // press turned into a drag.
                if again || self.entries[i].dir {
                    self.pending_open = Some(i);
                } else if (self.auto_load || self.previews_on_press(&self.entries[i])) && !is_preset(&self.entries[i].path) {
                    // v1 ([re:8eb19f] / [re:c33e1a]): presets load on a double
                    // click only.
                    // Selecting a file loads (Auto load) or previews it
                    // (Preview) on the press.
                    self.open(i, model, out);
                }
                true
            }
            // The view options menu and its Columns submenu ([re:130c7c]):
            // a column's check item shows or hides it (saved as an option).
            UiEvent::MenuSelected { anchor, index, .. } if self.view_menu == 1 => {
                self.view_menu = 0;
                let check = |on: bool, t: &str| if on { format!("✓ {t}") } else { t.to_string() };
                match *index {
                    0 => {
                        let mut items: Vec<String> = VIEW_TYPES.iter().zip(self.show).map(|(t, on)| check(on, t)).collect();
                        items.push(String::new());
                        items.push("View all".into());
                        self.view_menu = 3;
                        app.open_menu(anchor, items, None);
                    }
                    1 => {
                        self.view_menu = 2;
                        app.open_menu(anchor, vec![check(self.show_type, "Type"), check(self.show_size, "Size")], None);
                    }
                    _ => {}
                }
                false
            }
            // [re:130c7c]: a type toggles, View all shows every one; the
            // folder lists again.
            UiEvent::MenuSelected { index, .. } if self.view_menu == 3 => {
                self.view_menu = 0;
                match *index {
                    t @ 0..=12 => self.show[t] = !self.show[t],
                    14 => self.show = [true; 13],
                    _ => return false,
                }
                let mut o = crate::options::Options::load();
                for t in 0..13 {
                    o.set(&view_option(t), self.show[t]);
                }
                o.save();
                if let Some(d) = self.dir.clone().filter(|_| self.search.is_none()) {
                    self.go(d, false);
                }
                true
            }
            UiEvent::MenuSelected { index, .. } if self.view_menu == 2 => {
                self.view_menu = 0;
                let (key, on) = match *index {
                    0 => {
                        self.show_type = !self.show_type;
                        ("FileHeaderShowStateType", self.show_type)
                    }
                    1 => {
                        self.show_size = !self.show_size;
                        ("FileHeaderShowStateSize", self.show_size)
                    }
                    _ => return false,
                };
                let mut o = crate::options::Options::load();
                o.set(key, on);
                o.save();
                true
            }
            // The column header: sort by the clicked column ([re:0feb7a]).
            UiEvent::Mouse { path, overload, kind: MouseKind::Down, x, .. } if overload == "BrowserListHeader" => {
                let w = app.ctrl(path).map(|c| c.rect.w as f32).unwrap_or(0.0);
                if let Some((col, ..)) = self.columns(w).into_iter().find(|(_, cx, cw)| *x >= *cx && *x < cx + cw) {
                    self.sort_by(col);
                }
                true
            }
            // The current shortlist ([re:dbf248]): click selects (cmd
            // toggles, shift extends), a second click opens; Auto load /
            // Preview act on the press as in the folder's list.
            UiEvent::Mouse { overload, kind: MouseKind::Down, y, .. } if overload == "BrowserShortList" => {
                let list = self.short_entries();
                let i = self.short_scroll + (*y / FILE_ROW).max(0.0) as usize;
                if i >= list.len() {
                    self.short_sel.clear();
                    return true;
                }
                if app.mods.cmd {
                    if let Some(k) = self.short_sel.iter().position(|&s| s == i) {
                        self.short_sel.remove(k);
                    } else {
                        self.short_sel.push(i);
                    }
                    self.short_anchor = Some(i);
                    return true;
                }
                if app.mods.shift {
                    let a = self.short_anchor.unwrap_or(i);
                    self.short_sel = (a.min(i)..=a.max(i)).collect();
                    return true;
                }
                let again = self.short_sel == [i];
                self.short_sel = vec![i];
                self.short_anchor = Some(i);
                if again || list[i].dir || self.auto_load || self.previews_on_press(&list[i]) {
                    return self.open_entry(list[i].clone(), model, out);
                }
                true
            }
            UiEvent::Wheel { overload, dy, .. } if overload == "BrowserShortList" => {
                let max = self.short_entries().len().saturating_sub(1) as i32;
                self.short_scroll = (self.short_scroll as i32 - dy.round() as i32 * 3).clamp(0, max) as usize;
                true
            }
            // [re:200a75]: the shortlist's menu.
            UiEvent::Mouse { path, overload, kind: MouseKind::RightDown, y, .. } if overload == "BrowserShortList" => {
                use dreist_gui::menu::item;
                let list = self.short_entries();
                let i = self.short_scroll + (*y / FILE_ROW).max(0.0) as usize;
                if i < list.len() && !self.short_sel.contains(&i) {
                    self.short_sel = vec![i];
                    self.short_anchor = Some(i);
                }
                let (any, items) = (!self.short_sel.is_empty(), !list.is_empty());
                let mut v = vec![
                    (item("Select all", items), ShortAction::SelectAll),
                    (String::new(), ShortAction::None),
                    (item("Remove selected", any), ShortAction::Remove),
                    (item("Crop to selected", any), ShortAction::Crop),
                    (item("Clear shortlist", items), ShortAction::Clear),
                    (String::new(), ShortAction::None),
                    // The port keeps every shortlist saved (no separate
                    // shortlist file to write).
                    (item("Save as shortlist file", false), ShortAction::None),
                    (item("Collect files and export as favourite", items), ShortAction::Export),
                ];
                if self.short_sel.iter().filter_map(|&k| list.get(k)).any(|e| e.path.with_extension("gsa").exists()) {
                    v.push((String::new(), ShortAction::None));
                    v.push(("Delete slice information".into(), ShortAction::DeleteSliceInfo));
                }
                let (labels, actions): (Vec<String>, Vec<ShortAction>) = v.into_iter().unzip();
                self.short_menu = actions;
                app.open_menu(path, labels, None);
                true
            }
            UiEvent::MenuSelected { index, .. } if !self.short_menu.is_empty() => {
                let a = std::mem::take(&mut self.short_menu).get(*index).copied().unwrap_or(ShortAction::None);
                self.short_action(a)
            }
            // Header Options ([re:5eecea] [re:ea2f28]).
            UiEvent::MenuSelected { index, .. } if self.header_menu == 2 => {
                self.header_menu = 0;
                self.preview_output = (*index + 1).min(16);
                out.push(crate::command::msg(dreist_engine::binding::id::SET_PREVIEW_OUTPUT, &[], &[self.preview_output as f32]));
                true
            }
            UiEvent::MenuSelected { index, .. } if self.header_menu == 1 => {
                self.header_menu = 0;
                match *index {
                    0 => {
                        self.new_shortlist(Vec::new());
                    }
                    1 => {
                        let files = self.drag_items();
                        self.new_shortlist(files);
                    }
                    3 => {
                        // Output pairs, as v1 lists them ([re:3f4d0d]): Master,
                        // Sub output 1..15.
                        if let Some(p) = app.root.path_of("ToolButton").filter(|p| p.starts_with(&self.page)) {
                            let items = crate::mixer::preview_output_items();
                            self.header_menu = 2;
                            app.open_menu(&p, items, self.preview_output.checked_sub(1));
                        }
                        return false;
                    }
                    5 => toggle_lock_all(model, "Pad", crate::model::V1_PADS),
                    6 => toggle_lock_all(model, "StepPattern", 24),
                    8 => {
                        self.pads_track = !self.pads_track;
                        let mut o = crate::options::Options::load();
                        o.set("PadsTrackBrowser", self.pads_track);
                        o.save();
                    }
                    _ => {}
                }
                true
            }
            UiEvent::Clicked { path, name, .. } if path.starts_with(&self.page) => match name.as_str() {
                "ToolButton" => {
                    let mut items: Vec<String> = HEADER_MENU.iter().map(|s| s.to_string()).collect();
                    if self.pads_track {
                        items[8] = format!("✓ {}", items[8]);
                    }
                    self.header_menu = 1;
                    app.open_menu(path, items, None);
                    false
                }
                "LockPadsButton" => {
                    toggle_lock_all(model, "Pad", crate::model::V1_PADS);
                    true
                }
                "LockPatternsButton" => {
                    toggle_lock_all(model, "StepPattern", 24);
                    true
                }
                // [re:381995]: "Previous location" while search results are
                // listed, else the folder's parent.
                "ParentFolderButton" => {
                    if self.search.is_some() {
                        if let Some(p) = self.history.pop() {
                            self.go(p, false);
                        }
                    } else if let Some(p) = self.dir.as_ref().and_then(|d| d.parent()).map(Path::to_path_buf) {
                        self.go(p, true);
                    }
                    true
                }
                // [re:543135]: Columns ▸ Type / Size.
                "ViewOptionsButton" => {
                    self.view_menu = 1;
                    app.open_menu(path, vec!["View ▸".into(), "Columns ▸".into()], None);
                    false
                }
                "ParentButton" => {
                    if let Some(p) = self.dir.as_ref().and_then(|d| d.parent()).map(Path::to_path_buf) {
                        self.go(p, true);
                    }
                    true
                }
                "EnterButton" => self.selected.is_some_and(|i| self.open(i, model, out)),
                "RefreshButton" => {
                    if let Some(d) = self.dir.clone() {
                        self.go(d, false);
                    }
                    true
                }
                "PrevLocnButton" => {
                    if let Some(p) = self.history.pop() {
                        if let Some(d) = self.dir.take() {
                            self.future.push(d);
                        }
                        self.go(p, false);
                    }
                    true
                }
                "NextLocnButton" => {
                    if let Some(p) = self.future.pop() {
                        if let Some(d) = self.dir.take() {
                            self.history.push(d);
                        }
                        self.go(p, false);
                    }
                    true
                }
                "NextFileButton" | "PrevFileButton" => {
                    let n = self.entries.len();
                    if n > 0 {
                        let cur = self.selected.unwrap_or(0);
                        let i = if name == "NextFileButton" { (cur + 1).min(n - 1) } else { cur.saturating_sub(1) };
                        self.selected = Some(i);
                        if (self.auto_load || self.preview_mode) && !self.entries[i].dir {
                            self.open(i, model, out);
                        }
                    }
                    true
                }
                "PreviewButton" => {
                    self.preview_mode = !self.preview_mode;
                    true
                }
                "StopButton" => {
                    self.previews.cancel();
                    out.push(Command::Preview(None));
                    true
                }
                "InteractionModeButton" => {
                    self.auto_load = !self.auto_load;
                    true
                }
                "HitLoopModeButton" => {
                    self.slice_mode = !self.slice_mode;
                    true
                }
                _ => false,
            },
            _ => false,
        }
    }

    pub fn draw(&self, r: &Renderer, p: &mut Painter, c: &Ctrl) -> bool {
        let s = r.styles;
        let (w, h) = (c.rect.w as f32, c.rect.h as f32);
        match c.overload.as_str() {
            "BrowserSourceList" => {
                p.fill_rect(RectF::new(0.0, 0.0, w, h), crate::front::colour_or(s, "BrowserTreeBackStyle", "colFill", Colour::TRANSPARENT));
                // v1's column is inset: a shadow fading over 10 px from the
                // left edge (measured #6E -> #C0).
                for i in 0..10 {
                    let a = 0.42 * (1.0 - i as f32 / 10.0).powf(1.6);
                    p.fill_rect(RectF::new(i as f32, 0.0, 1.0, h), Colour::rgba(0, 0, 20, (a * 255.0) as u8));
                }
                let font = s.font(SOURCE_STYLE);
                let text = s.colour(SOURCE_STYLE, "colText", Colour::BLACK);
                let sel = crate::front::colour_or(s, SOURCE_SELECTED_STYLE, "colFill", Colour::rgba(232, 204, 91, 255));
                let sel_text = s.colour(SOURCE_SELECTED_STYLE, "colText", text);
                for (i, src) in self.sources.iter().enumerate() {
                    let y = i as f32 * SOURCE_ROW;
                    if y > h {
                        break;
                    }
                    let current = src.path.is_some() && src.path == self.dir;
                    // [re:f32dc0]: a colour label
                    // fills the row.
                    if let Some(c) = src.colour.checked_sub(1).and_then(|k| crate::padmenu::COLOURS.get(k)).and_then(|c| u32::from_str_radix(&c[1..], 16).ok()) {
                        p.fill_rect(RectF::new(0.0, y, w, SOURCE_ROW), Colour::rgba((c >> 16) as u8, (c >> 8) as u8, c as u8, 255));
                    }
                    if current {
                        p.fill_rect(RectF::new(0.0, y, w, SOURCE_ROW), sel);
                    }
                    let col = if current { sel_text } else { text };
                    // v1's section heads (measured, ui-v1-style "browser
                    // tree"): upper case at x 14, after a down triangle
                    // (x 4-12) when the section has entries.
                    if src.header {
                        if self.sources.get(i + 1).is_some_and(|n| !n.header) {
                            let cy = y + SOURCE_ROW / 2.0;
                            p.fill_polygon(&[(4.0, cy - 3.0), (12.0, cy - 3.0), (8.0, cy + 2.0)], col);
                        }
                        p.text(RectF::new(14.0, y, w - 18.0, SOURCE_ROW), &src.label.to_uppercase(), col, &s.font(SOURCE_HEAD_STYLE), HAlign::Left, VAlign::Center);
                        continue;
                    }
                    let (x, f) = (34.0, &font);
                    if !src.icon.is_empty() {
                        let st = if current { crate::browsericons::SELECTED_STYLE } else { crate::browsericons::STYLE };
                        crate::browsericons::draw(r, p, st, RectF::new(13.0, y + (SOURCE_ROW - 18.0) / 2.0, 18.0, 18.0), src.icon, "");
                    }
                    p.text(RectF::new(x, y, w - x - 4.0, SOURCE_ROW), &src.label, col, f, HAlign::Left, VAlign::Center);
                }
                true
            }
            "BrowserFileList" => {
                let selected = |k: usize| if self.multi.is_empty() { self.selected == Some(k) } else { self.multi.contains(&k) };
                self.draw_rows(r, p, w, h, &self.entries, self.scroll, &selected);
                true
            }
            "BrowserShortList" => {
                let list = self.short_entries();
                self.draw_rows(r, p, w, h, &list, self.short_scroll, &|k| self.short_sel.contains(&k));
                true
            }
            // [re:6f787e]: the header: its frame, the column titles (Name
            // at the left, Type and Size centred), dividers after Name and
            // Type, and the sort order's arrow in the sorted column.
            "BrowserListHeader" => {
                let st = HEADER_STYLE;
                let back = crate::front::colour_or(s, st, "colFill", Colour::TRANSPARENT);
                let edge = s.colour(st, "colBorder", Colour::BLACK);
                let text = s.colour(st, "colText", Colour::BLACK);
                let font = s.font(st);
                p.fill_rect(RectF::new(1.0, 0.0, w - 2.0, h - 1.0), back);
                p.stroke_rect(RectF::new(1.5, 0.5, w - 3.0, h - 2.0), edge, 1.0);
                for (col, x, cw) in self.columns(w) {
                    let title = match col {
                        Column::Name => "Name",
                        Column::Type => "Type",
                        Column::Size => "Size",
                    };
                    if col != Column::Name {
                        p.line(x + 0.5, 0.0, x + 0.5, h - 1.0, edge, 1.0);
                    }
                    let (tx, align) = if col == Column::Name { (x + 4.0, HAlign::Left) } else { (x, HAlign::Center) };
                    p.text(RectF::new(tx, 0.0, cw - 12.0 - (tx - x), h - 1.0), title, text, &font, align, VAlign::Center);
                    if self.sort.0 == col {
                        let (ax, cy) = (x + cw - 10.0, (h - 1.0) / 2.0);
                        let tri = if self.sort.1 { [(ax, cy - 2.0), (ax + 7.0, cy - 2.0), (ax + 3.5, cy + 2.0)] } else { [(ax, cy + 2.0), (ax + 7.0, cy + 2.0), (ax + 3.5, cy - 2.0)] };
                        p.fill_polygon(&tri, text);
                    }
                }
                true
            }
            "BrowserPathNavigator" => {
                // [re:0565d9]: text at (2, 0,
                // w − 4, h); a chevron (w − 10, 0) → (w, h/2) → (w − 10, h)
                // after every button but the last, which gets a right edge
                // line and, for "(No slice)", a 10 point star at
                // (w − 10, h/2) (radii 0.4h / 0.15h) in the text colour.
                let st = NAV_STYLE;
                let font = s.font(st);
                let text = s.colour(st, "colText", Colour::BLACK);
                let border = s.colour(st, "colBorder", Colour::BLACK);
                let (parts, is_path) = self.nav_parts();
                let no_slice = is_path && self.dir.as_deref().is_some_and(|d| !slicing_allowed(d));
                let fonts = &mut *p.fonts;
                let buttons = nav_layout(&mut |t: &str| fonts.measure(&font, t).0, &parts, is_path, no_slice, w);
                for b in &buttons {
                    let x0 = b.x;
                    if b.last {
                        p.line(x0 + b.w + 0.5, 0.5, x0 + b.w + 0.5, h + 0.5, border, 1.0);
                        if b.display.contains("(No slice)") {
                            let (cx, cy) = (x0 + b.w - 10.0, h * 0.5);
                            let pts: Vec<(f32, f32)> = (0..10)
                                .map(|k| {
                                    let r = if k % 2 == 0 { 0.4 * h } else { 0.15 * h };
                                    let a = std::f32::consts::PI * (k as f32 / 5.0) - std::f32::consts::FRAC_PI_2;
                                    (cx + r * a.cos(), cy + r * a.sin())
                                })
                                .collect();
                            p.fill_polygon(&pts, text);
                        }
                    } else {
                        p.line(x0 + b.w - 10.0, 0.0, x0 + b.w, h * 0.5, border, 1.0);
                        p.line(x0 + b.w - 10.0, h, x0 + b.w, h * 0.5, border, 1.0);
                    }
                    p.text(RectF::new(x0 + 2.0, 0.0, (b.w - 4.0).max(0.0), h), &b.label, text, &font, HAlign::Left, VAlign::Center);
                }
                *self.nav.borrow_mut() = buttons;
                true
            }
            _ => false,
        }
    }

    /// `e_notifyFileLoadProgress` 0x143: the loading row's fraction; 1.5
    /// ends it. True when it changed.
    pub fn load_progress(&mut self, f: f32) -> bool {
        let Some((_, p)) = self.loading.as_mut() else { return false };
        if (f - 1.5).abs() <= 1e-6 {
            self.loading = None;
            return true;
        }
        let changed = (*p - f).abs() > 1e-6;
        *p = f;
        changed
    }

    /// Mouse wheel / scroll support for the file list.
    /// Rows of a list (the folder's or the shortlist's) from `scroll`:
    /// alternate shading, selection, the loading bar, icon and name, and
    /// the Type and Size columns when shown, after a divider each
    /// ([re:22e64a]).
    #[allow(clippy::too_many_arguments)]
    fn draw_rows(&self, r: &Renderer, p: &mut Painter, w: f32, h: f32, entries: &[Entry], scroll: usize, selected: &dyn Fn(usize) -> bool) {
        let s = r.styles;
        let font = s.font(ITEM_STYLE);
        let text = s.colour(ITEM_STYLE, "colText", Colour::BLACK);
        let sel = crate::front::colour_or(s, ITEM_STYLE, "colSelection", Colour::rgba(232, 204, 91, 255));
        let sel_text = s.colour(ITEM_STYLE, "colSelectedText", text);
        let divider = crate::front::colour_or(s, ITEM_STYLE, "colBorder", text).with_alpha(0.5);
        let (row, alt) = (crate::front::colour_or(s, ITEM_STYLE, "colRow", Colour::TRANSPARENT), crate::front::colour_or(s, ITEM_STYLE, "colRowAlt", Colour::TRANSPARENT));
        p.fill_rect(RectF::new(0.0, 0.0, w, h), crate::front::colour_or(s, ITEM_STYLE, "colFill", Colour::TRANSPARENT));
        // Columns line up with the header, which leaves the scroll bar's
        // width free at the right.
        let cols = self.columns(w - SLIDER_W as f32);
        let name_w = cols.first().map(|c| c.2).unwrap_or(w);
        let rows = (h / FILE_ROW) as usize + 1;
        for (k, en) in entries.iter().enumerate().skip(scroll).take(rows) {
            let y = (k - scroll) as f32 * FILE_ROW;
            let on = selected(k);
            p.fill_rect(RectF::new(0.0, y, w, FILE_ROW), if k % 2 == 0 { row } else { alt });
            if on {
                p.fill_rect(RectF::new(0.0, y, w, FILE_ROW), sel);
            }
            // The loading bar ([re:ef4a8d]): colAltFill,
            // (2, 2, (w − 4)·p, h − 2) while 0 < p ≤ 1.
            if let Some((_, pr)) = self.loading.as_ref().filter(|(lp, pr)| *lp == en.path && *pr > 0.0 && *pr <= 1.0) {
                p.fill_rect(RectF::new(2.0, y + 2.0, (w - 4.0) * pr, FILE_ROW - 2.0), sel.with_alpha(0.6));
            }
            let col = if on { sel_text } else { text };
            // The folder icon, or the file's type badge.
            let st = if on { crate::browsericons::SELECTED_STYLE } else { crate::browsericons::STYLE };
            let icon = RectF::new(3.0, y + 1.0, 16.0, FILE_ROW - 2.0);
            if en.dir {
                crate::browsericons::draw(r, p, st, icon, "folder", "");
            } else {
                let ext = en.path.extension().map(|e| e.to_string_lossy().into_owned()).unwrap_or_default();
                crate::browsericons::draw(r, p, st, icon, "file", &crate::browsericons::badge(&ext));
                // [re:4e8f2d]: a file in a folder that disallows slicing
                // carries the star over its icon.
                if crate::search::is_audio(&ext.to_lowercase()) && en.path.parent().is_some_and(|d| !slicing_allowed(d)) {
                    let (cx, cy, ro, ri) = (icon.x + icon.w - 4.0, icon.y + 4.0, 4.0f32, 1.7f32);
                    let pts: Vec<(f32, f32)> = (0..10)
                        .map(|k| {
                            let a = std::f32::consts::PI * (k as f32 / 5.0) - std::f32::consts::FRAC_PI_2;
                            let rr = if k % 2 == 0 { ro } else { ri };
                            (cx + rr * a.cos(), cy + rr * a.sin())
                        })
                        .collect();
                    p.fill_polygon(&pts, Colour::rgba(232, 180, 40, 255));
                }
            }
            p.text(RectF::new(22.0, y, name_w - 24.0, FILE_ROW), &en.name, col, &font, HAlign::Left, VAlign::Center);
            // [re:b86812]: the play button at the name column's right, and
            // after a press a red triangle over it fading out.
            if let Some(ix) = self.preview_button_x(en, on, name_w) {
                p.fill_polygon(&[(ix + 5.0, y + 4.0), (ix + 12.0, y + 9.0), (ix + 5.0, y + 14.0)], col);
                if let Some((_, t)) = self.preview_flash.as_ref().filter(|(f, _)| *f == en.path) {
                    let ticks = (t.elapsed().as_secs_f32() * 30.0) as u32;
                    let a = 255u32.saturating_sub(10 * ticks) as f32 / 255.0;
                    p.fill_polygon(&[(ix + 5.0, y + 4.0), (ix + 12.0, y + 9.0), (ix + 5.0, y + 14.0)], Colour::rgba(255, 0, 0, 255).with_alpha(a));
                }
            }
            for &(c, x, cw) in cols.iter().skip(1) {
                p.line(x + 0.5, y, x + 0.5, y + FILE_ROW, divider, 1.0);
                let t = if c == Column::Type { type_label(&en.path, en.dir).to_string() } else { size_label(&en.path, en.dir) };
                p.text(RectF::new(x + 3.0, y, cw - 4.0, FILE_ROW), &t, col, &font, HAlign::Left, VAlign::Center);
            }
        }
    }

    /// Apply a shortlist menu action to the current shortlist.
    fn short_action(&mut self, a: ShortAction) -> bool {
        let Some(name) = self.shortlist.clone() else { return false };
        let list = self.short_entries();
        let sel: Vec<PathBuf> = self.short_sel.iter().filter_map(|&k| list.get(k)).map(|e| e.path.clone()).collect();
        let edit = |me: &mut Browser, f: &dyn Fn(&mut Vec<PathBuf>)| {
            if let Some((_, files)) = me.shortlists.iter_mut().find(|(n, _)| *n == name) {
                f(files);
            }
            save_shortlists(&me.shortlists);
            me.short_sel.clear();
            me.short_anchor = None;
            me.short_scroll = 0;
        };
        match a {
            ShortAction::SelectAll => self.short_sel = (0..list.len()).collect(),
            ShortAction::Remove => edit(self, &|f| f.retain(|p| !sel.contains(p))),
            ShortAction::Crop => edit(self, &|f| f.retain(|p| sel.contains(p))),
            ShortAction::Clear => edit(self, &|f| f.clear()),
            ShortAction::Export => self.export_request = Some((name, true)),
            ShortAction::DeleteSliceInfo => {
                for p in &sel {
                    let _ = std::fs::remove_file(p.with_extension("gsa"));
                }
            }
            ShortAction::None => return false,
        }
        true
    }

    pub fn scroll_by(&mut self, rows: i32) {
        let max = self.entries.len().saturating_sub(1);
        self.scroll = (self.scroll as i32 + rows).clamp(0, max as i32) as usize;
    }
}

/// A preset (project) file: v1 loads these on a double click only.
fn is_preset(p: &Path) -> bool {
    p.extension().map(|e| e.to_string_lossy().to_lowercase()).and_then(|e| dreist_formats::profile::doc_kind(&e)) == Some(dreist_formats::profile::DocKind::Project)
}

#[cfg(test)]
mod search_tests {
    use super::*;

    #[test]
    fn path_navigator_layout() {
        crate::app::test_setup();
        // 6 px per character.
        let mut m = |t: &str| t.len() as f32 * 6.0;
        let parts: Vec<String> = ["Users", "me", "Kits"].iter().map(|s| s.to_string()).collect();
        let b = nav_layout(&mut m, &parts, true, false, 300.0);
        assert_eq!(b.iter().map(|b| (b.x, b.w, b.index, b.last)).collect::<Vec<_>>(), vec![(0.0, 40.0, 0, false), (40.0, 22.0, 1, false), (62.0, 34.0, 2, true)]);
        // Too narrow: ".." then the tail that fits in w − 10 − 12.
        let b = nav_layout(&mut m, &parts, true, true, 90.0);
        assert_eq!(b[0].label, "..");
        assert_eq!(b[0].index, 1);
        assert_eq!(b.last().map(|b| (b.label.as_str(), b.display.as_str())), Some(("Kits", "Kits (No slice)")));
        let b = nav_layout(&mut m, &["Search Results".into()], false, false, 300.0);
        assert_eq!((b.len(), b[0].w, b[0].index), (1, 89.0, -1));
    }

    #[test]
    fn content_lists_v1_packs_too() {
        crate::app::test_setup();
        let tmp = std::env::temp_dir().join(format!("einst-v1-content-{}", std::process::id()));
        let pack = tmp.join("Old Factory");
        std::fs::create_dir_all(&pack).unwrap();
        // A database naming the pack (and a file, which is no folder).
        let xml = format!("<root><ContentPack path=\"{}\" /><X>{}/none.txt</X></root>", pack.display(), tmp.display());
        assert_eq!(content_pack_paths(&xml), vec![pack.clone()]);
        let v = sources_with(&[], &[], &[], &subdirs(&tmp));
        let item = v.iter().find(|s| s.path.as_deref() == Some(pack.as_path())).expect("v1 pack listed");
        assert_eq!((item.cat, item.icon, item.label.as_str()), ("Content", "book", "Old Factory"));
        let _ = std::fs::remove_dir_all(&tmp);
    }

    /// [re:ca74b1]: the View menu's types; all on shows everything, else
    /// unticked types go, and sample folders follow Folders.
    #[test]
    fn view_filter_types() {
        crate::app::test_setup();
        let e = |n: &str, dir: bool| Entry { name: n.into(), path: PathBuf::from(format!("/x/{n}")), dir };
        let mut show = [true; 13];
        assert!(visible(&show, &e("a.txt", false)), "all on: as listed");
        show[0] = false;
        assert!(!visible(&show, &e("a.wav", false)) && visible(&show, &e("b.mid", false)));
        assert!(!visible(&show, &e("a.gru", false)), "no type: hidden while filtering");
        assert!(visible(&show, &e("Kick.samples", true)));
        show[11] = false;
        assert!(!visible(&show, &e("Kits", true)) && !visible(&show, &e("Kick.samples", true)));
        show[11] = true;
        show[12] = false;
        assert!(visible(&show, &e("Kits", true)) && !visible(&show, &e("Kick.samples", true)));
        assert_eq!(view_type(&e("a.einstkit", false)), Some(3));
        assert_eq!(view_option(6), "ShowPattern_Bank");
    }

    /// [re:acb2eb], [re:22e64a]: the Type and Size columns' texts.
    #[test]
    fn type_and_size_labels() {
        crate::app::test_setup();
        let p = |n: &str| PathBuf::from(format!("/x/{n}"));
        assert_eq!(type_label(&p("Kits"), true), "Folder");
        for (n, t) in
            [("a.wav", "Wave"), ("a.AIF", "AIF"), ("a.aiff", "AIFF"), ("a.rx2", "REX 2"), ("a.rex", "Recycle"), ("a.mid", "MIDI"), ("a.g01", "Guru G01"), ("a.gru", "Guru song"), ("a.txt", "Unknown")]
        {
            assert_eq!(type_label(&p(n), false), t, "{n}");
        }
        assert_eq!(type_label(&p("a.einstkit"), false), "Kit");
        assert_eq!(size_label(&p("Kits"), true), "--");
        let f = std::env::temp_dir().join(format!("einst-size-{}.wav", std::process::id()));
        std::fs::write(&f, vec![0u8; 3000]).unwrap();
        assert_eq!(size_label(&f, false), "2.93KB");
        let _ = std::fs::remove_file(&f);
    }

    #[test]
    fn shortlists_parse() {
        crate::app::test_setup();
        let v = parse_shortlists("[Kicks]\n/a/k1.wav\n/a/k2.wav\n\n[Empty]\n");
        assert_eq!(v.len(), 2);
        assert_eq!(v[0].0, "Kicks");
        assert_eq!(v[0].1, vec![PathBuf::from("/a/k1.wav"), PathBuf::from("/a/k2.wav")]);
        assert!(v[1].1.is_empty());
    }

    #[test]
    fn noslice_file_blocks_slicing() {
        crate::app::test_setup();
        let d = std::env::temp_dir().join(format!("dreist-noslice-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        assert!(slicing_allowed(&d));
        std::fs::write(d.join(dreist_formats::legacy::NOSLICE_FILE), "No slice").unwrap();
        assert!(!slicing_allowed(&d));
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn lock_all_toggles_like_globally_lock() {
        crate::app::test_setup();
        let mut m = Model::new(crate::project::new_document());
        assert!(!all_locked(&m, "Pad", crate::model::V1_PADS));
        toggle_lock_all(&mut m, "Pad", crate::model::V1_PADS);
        assert!(all_locked(&m, "Pad", crate::model::V1_PADS));
        toggle_lock_all(&mut m, "Pad", crate::model::V1_PADS);
        assert!(!all_locked(&m, "Pad", crate::model::V1_PADS));
        // One unlocked pad: the toggle locks all.
        toggle_lock_all(&mut m, "StepPattern", 24);
        let e = m.engine;
        m.engine_mut(e).child_mut_or_insert("StepPattern", "StepPattern3").attrs.retain(|(k, _)| k != "LockState");
        toggle_lock_all(&mut m, "StepPattern", 24);
        assert!(all_locked(&m, "StepPattern", 24));
    }
}
