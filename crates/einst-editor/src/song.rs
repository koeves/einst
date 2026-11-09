//! Song tab (`SongPanel.skin`, [re:3829fc]): one row per
//! engine (`SongTrackHeaderPanel.skin` headers), a timeline and the parts.
//!
//! - Rows start 20 px down ([re:ef2dc3]: `20 + id·h`).
//! - Grid ([re:182a98]): bar lines in
//!   `colGridBar`, beat lines at α0.5, labels "bar.beat" (fine zoom) or the
//!   bar number, in `SongTimelineStyle`.
//! - Parts ([re:d5c964] /
//!   [re:b14d22]): rounded sections per pattern
//!   repeat in `colPart` ([re:d5c964] 0x20), border `colBorder`, the
//!   pattern name in `colText` at the top left.
//! - The blackout overlay (`SongPanelBlackoutPanelStyle`) covers the tracks
//!   while song mode is off.
//!
//! Edits post `e_songAddPart` 0x119 [track] [start, length, pattern],
//! `e_songDeletePart` 0x11a [track] [time] and `e_songSetActive` 0x117
//! (payloads are objects in the original; the port uses QN times).

use std::collections::BTreeSet;

use dreist_engine::binding::id;
use dreist_engine::song::{Song, SongPart, phase_mod, quantize_round};
use dreist_engine::songrec::{STOP_PART, SongEvent};
use dreist_gui::app::{App, MouseKind, UiEvent};
use dreist_gui::paint::{Painter, RectF};
use dreist_gui::render::Renderer;
use dreist_gui::style::{HAlign, Shorten, VAlign};
use dreist_gui::{Colour, Ctrl, Rect, Skin};

use crate::command::{Command, msg};
use crate::model::{ENGINES, Model};
use crate::{EditorError, err};

const TIMELINE_H: f32 = 20.0;
/// Editor tab page id of the song panel.
#[cfg(test)]
const SONG_TAB: usize = 2;
/// The theme's content styles for the code-drawn lanes and parts (the
/// v1 set's `SongLane*`, `SongPart*`, `Grid*` and `Timeline*` styles).
const PART_STYLE: &str = "SongPartBodyStyle";
const PART_SELECTED_STYLE: &str = "SongPartSelectedBodyStyle";
const FOOT_STYLE: &str = "SongPartFootStyle";
const RULER_STYLE: &str = "SongRulerStyle";
const BAR_STYLE: &str = "SongBarLineStyle";
/// The navy strip closing each lane (v1, measured: 8 px).
const FOOT_H: f32 = 8.0;

/// v1's selected part texture (`PartSelectedTexture`): a 2 px checker of
/// `c` over the face.
fn check(p: &mut Painter, r: RectF, c: Colour) {
    let (x0, y0) = (r.x.round() as i32, r.y.round() as i32);
    let (x1, y1) = ((r.x + r.w).round() as i32, (r.y + r.h).round() as i32);
    for y in (y0..y1).step_by(2) {
        let shift = if (y / 2) % 2 == 0 { 0 } else { 2 };
        for x in (x0 - x0.rem_euclid(4) + shift..x1).step_by(4) {
            let xa = x.max(x0);
            let xb = (x + 2).min(x1);
            if xb > xa {
                p.fill_rect(RectF::new(xa as f32, y as f32, (xb - xa) as f32, (2.min(y1 - y)) as f32), c);
            }
        }
    }
}

/// The lanes' horizontal scroll bar (`SongScrollStyle`; v1-song.png): a
/// light track (colFill) in a navy frame (colBorder), a navy thumb
/// (colAltFill) over the visible part (`realPage`) at the position.
fn draw_scroll(r: &Renderer, p: &mut Painter, c: &Ctrl) {
    let s = r.styles;
    let st = "SongScrollStyle";
    let (w, h) = (c.rect.w as f32, c.rect.h as f32);
    p.fill_rect(RectF::new(0.0, 0.0, w, h), s.colour(st, "colFill", Colour::rgba(215, 223, 228, 255)));
    p.stroke_rect(RectF::new(0.5, 0.5, w - 1.0, h - 1.0), s.colour(st, "colBorder", Colour::rgba(18, 25, 60, 255)), 1.0);
    let page = c.attr_f32("realPage", 1.0).clamp(0.0, 1.0);
    let tw = ((w - 6.0) * page).max(8.0).min(w - 6.0);
    let x = 3.0 + c.value.clamp(0.0, 1.0) * (w - 6.0 - tw);
    p.fill_rect(RectF::new(x.round(), 3.0, tw.round(), h - 6.0), s.colour(st, "colAltFill", Colour::rgba(44, 48, 77, 255)));
}

/// The song page's tools, in the toolbar's order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SongTool {
    /// Left adds parts of the draw pattern (and moves parts).
    Draw,
    /// Left deletes parts.
    Erase,
    /// Left selects, moves and resizes parts (no new parts).
    Select,
    /// Draw and select together.
    Multi,
}

const SONG_TOOLS: [SongTool; 4] = [SongTool::Draw, SongTool::Erase, SongTool::Select, SongTool::Multi];

/// v1's tool values (the tool buttons' ids): Select 0, Draw 1, Erase 2,
/// Multi 3 ([re:4e6b67]).
fn tool_id(t: SongTool) -> i64 {
    match t {
        SongTool::Select => 0,
        SongTool::Draw => 1,
        SongTool::Erase => 2,
        SongTool::Multi => 3,
    }
}

/// Option `SongLastTool` (default 1, Draw; [re:7fdc18] [re:9bf684]).
fn saved_tool() -> SongTool {
    match crate::options::Options::load().int("SongLastTool", 1) {
        0 => SongTool::Select,
        2 => SongTool::Erase,
        3 => SongTool::Multi,
        _ => SongTool::Draw,
    }
}
/// The pattern popup's name on the page.
const POPUP: &str = "SongPatternPopup";

/// [re:4e6b67] (left-button operation per tool [re:4e6b67]).
#[derive(Clone, Copy, Debug, PartialEq)]
enum OpKind {
    /// Pressed on a part (op 1): a move once the pointer has gone 3 px
    /// (op 4; Alt held copies, op 9) [re:00d39d].
    Move {
        moving: bool,
    },
    /// Rubber band over the saved selection (ops 2 / 3).
    Band,
    Start,
    Resize,
    Phase,
    /// A new part on `track` (op 10), extending with the pointer (op 11).
    Insert {
        track: usize,
        extending: bool,
    },
    /// Erase (op 12): every part the box from the press point touches.
    Erase,
}

struct Op {
    kind: OpKind,
    /// Grab position (QN, px).
    qn0: f64,
    x0: f32,
    y0: f32,
    /// Song and selection when the operation started.
    snapshot: Song,
    saved: BTreeSet<(usize, i64)>,
    /// The part the press was on (v1's operation part; none on empty space).
    part: Option<(usize, i64)>,
    /// The new part while drawing.
    new_part: Option<SongPart>,
}

/// What a left press starts ([re:4e6b67]: v1's operations 1, 2, 3, 5, 6,
/// 7, 10 and 12).
#[derive(Clone, Copy, Debug, PartialEq)]
enum LeftOp {
    /// 1: select the part; a drag moves (copies with Alt).
    Move,
    /// 2: rubber band, the selection cleared unless Shift.
    BandNew,
    /// 3: rubber band added to the selection.
    BandAdd,
    /// 5: the part's end.
    Resize,
    /// 6: the part's start.
    Start,
    /// 7: the phase anchor.
    Phase,
    /// 10: a new part of the draw pattern.
    Insert,
    /// 12: erase.
    Erase,
}

/// The part edge nearest a time ([re:473c89]): the part, whether the edge
/// is its end, the edge time, and whether the time is on the part's side
/// of that edge.
struct Closest {
    idx: usize,
    end: bool,
    edge: f64,
    inside: bool,
}

const ONE_TICK: f64 = 1.0 / 960.0;

/// A press in the track area: the track, the time, body px, the row
/// height, the part with the nearest edge and whether the press is on its
/// side of that edge, and Shift.
struct Press {
    track: usize,
    qn: f64,
    x: f32,
    y: f32,
    rh: f32,
    part: Option<(usize, i64)>,
    inside: bool,
    shift: bool,
}

fn closest_edge(parts: &[SongPart], t: f64) -> Option<Closest> {
    let mut best = 999999.0;
    let mut out = None;
    for (idx, p) in parts.iter().enumerate() {
        let end = quantize_round(p.start + p.length);
        if (p.start - t).abs() < best {
            best = (p.start - t).abs();
            out = Some(Closest { idx, end: false, edge: p.start, inside: p.start <= t });
        }
        let d = (end - ONE_TICK - t).abs();
        if d < best {
            best = d;
            out = Some(Closest { idx, end: true, edge: end, inside: t <= end - ONE_TICK });
        }
    }
    out
}

/// Add a part the way v1's tracks do ([re:e1fc6b]): parts it covers go,
/// parts it overlaps are cut at its edges (a part covering it on both
/// sides is split in two), the cut-off starts keeping their phase.
fn add_part_cut(model: &Model, song: &mut Song, t: usize, part: SongPart) {
    let Some(tr) = song.tracks.get_mut(t) else { return };
    let (s, e) = (part.start, quantize_round(part.start + part.length));
    let mut out = Vec::with_capacity(tr.len() + 2);
    for p in tr.drain(..) {
        let (ps, pe) = (p.start, quantize_round(p.start + p.length));
        if pe <= s || e < ps {
            out.push(p);
            continue;
        }
        let cycle = if p.pattern == STOP_PART { 1.0 } else { cycle_qn(model, t, p.pattern).max(1e-6) };
        let tail = |p: &SongPart| SongPart { start: e, length: quantize_round(pe - e), phase: phase_mod(e + p.phase - ps, cycle), ..p.clone() };
        if ps < s && e < pe {
            out.push(tail(&p));
            out.push(SongPart { length: quantize_round(s - ps), ..p });
        } else if ps < s || e < pe {
            if ps < s && s < pe {
                out.push(SongPart { length: quantize_round(s - ps), ..p });
            } else if ps < e && e < pe {
                out.push(tail(&p));
            } else {
                out.push(p);
            }
        }
    }
    out.push(part);
    out.retain(|p| p.length > 1e-9);
    out.sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap_or(std::cmp::Ordering::Equal));
    *tr = out;
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum MenuAction {
    Tool(usize),
    Cut,
    Copy,
    Paste,
    Delete,
    Duplicate,
    QuantizeStart,
    QuantizeBoth,
    QuantizeModeMenu,
    QuantizeMode(usize),
    SelectAll,
    SelectEmpty,
    ClearAll,
    EditPattern,
    RenamePattern,
    None,
}

/// `QuantizeMode` items ("SongLastQuantizeMode").
pub const QUANTIZE_MODES: [&str; 3] = ["Auto", "Maintain Start Phase", "Maintain Sync"];

pub struct SongPanel {
    page: Vec<usize>,
    /// `SongBody` (under `page`).
    named: dreist_gui::app::PathCache,
    /// Pixels per quarter note.
    zoom: f32,
    insert_pattern: usize,
    /// EraseButton held (0x123 `e_songSetEraseState`).
    erase: bool,
    /// Last clicked part (the part buttons' part).
    anchor: Option<(usize, i64)>,
    /// The pattern popup is open: for the selection (false) or the draw
    /// tool's pattern (true).
    pattern_menu: Option<bool>,
    /// The pattern popup (`SongPatternPopupPanel.skin`: 24 keys).
    popup_tpl: Option<Ctrl>,
    /// The left-button tool (the toolbar's tool group, manual §11:3).
    pub(crate) tool: SongTool,
    /// FollowButton: scroll to the playback position (option
    /// `SongLastFollowMode`, the song page's own; [re:cacf78] [re:1f0ef0]).
    pub(crate) follow_on: bool,
    /// First visible quarter note.
    scroll: f32,
    /// Selected parts (track, start in 1/960 QN).
    pub(crate) selected: BTreeSet<(usize, i64)>,
    op: Option<Op>,
    /// The song while an operation is in progress.
    working: Option<Song>,
    /// Band rectangle (px, body coordinates).
    band: Option<RectF>,
    /// Copied parts: (track, part with start relative to the earliest).
    clipboard: Vec<(usize, SongPart)>,
    quantize_mode: usize,
    /// Context menu actions and the right-click time.
    menu: Option<(Vec<MenuAction>, f64)>,
    /// The part under the pointer when the context menu opened.
    menu_part: Option<(usize, SongPart)>,
    /// The pattern being renamed in the song page's box (track, pattern).
    rename: Option<(usize, usize)>,
    /// Insert marker (QN): paste position.
    marker: f64,
    /// Timeline drag: operation 13 loop start, 14 loop end, 15 move loop,
    /// 16 marker; with the grab QN and the loop at the start.
    timeline: Option<(u8, f64, f64, f64)>,
    body_w: std::cell::Cell<f32>,
    /// Song recording ([re:7b9cde]: one part per track being
    /// recorded, with whether the song played when it started) and the
    /// retro buffer.
    pub rec_state: u8,
    recording: [Option<(SongPart, bool)>; ENGINES],
    retro: [Vec<(SongPart, bool)>; ENGINES],
}

fn key(start: f64) -> i64 {
    (start * 960.0).round() as i64
}

/// Selection sets from the song (by start key).
fn selected_parts(song: &Song, sel: &BTreeSet<(usize, i64)>) -> Vec<(usize, SongPart)> {
    song.tracks.iter().enumerate().flat_map(|(t, ps)| ps.iter().filter(move |p| sel.contains(&(t, key(p.start)))).map(move |p| (t, p.clone()))).collect()
}

/// Pattern length in QN divided by the engine tempo multiplier
/// (pattern length in QN / tempo multiplier).
fn cycle_qn(model: &Model, e: usize, p: usize) -> f64 {
    let mult = model.engine_node(e).map(|n| {
        let num = n.f32("TempoNumerator").unwrap_or(1.0).max(1.0) as f64;
        let den = n.f32("TempoDenominator").unwrap_or(1.0).max(1.0) as f64;
        num / den
    });
    pattern_qn(model, e, p) / mult.unwrap_or(1.0).max(1e-6)
}

fn song_of(model: &Model) -> Song {
    model.doc.children.iter().find(|c| c.class == "SongSequencer").map(Song::from_node).unwrap_or_else(|| Song::from_node(&dreist_formats::Node::new("SongSequencer", None)))
}

fn row_h(c: &Ctrl) -> f32 {
    ((c.rect.h as f32 - TIMELINE_H) / ENGINES as f32).floor().max(12.0)
}

/// Pattern length in quarter notes (steps at its resolution).
fn pattern_qn(model: &Model, e: usize, p: usize) -> f64 {
    let pat = model.pattern(e, p);
    pat.steps as f64 * 0.25 / dreist_engine::pattern::steps_per_16th(pat.resolution).max(1e-6)
}

impl SongPanel {
    pub fn build(skin: &mut Skin, root: &mut Ctrl, page: &[usize]) -> Result<SongPanel, EditorError> {
        let mut host = &mut *root;
        for &i in page {
            host = &mut host.children[i];
        }
        skin.mount(host, "SongPanel.skin").map_err(err)?;
        if let Some(sv) = host.find_mut("TrackScrollView") {
            let (w, h) = (sv.rect.w, sv.rect.h);
            sv.children.push(Ctrl {
                class: "CtrlPanel".into(),
                name: "SongBody".into(),
                overload: "SongBody".into(),
                style: "InvisibleStyle".into(),
                rect: Rect::new(0, 0, w, h),
                anchors: dreist_gui::anchors_fill(),
                visible: true,
                enabled: true,
                zorder: 1,
                ..Ctrl::default()
            });
        }
        let rel = skin.find("SongTrackHeaderPanel.skin").unwrap_or_else(|| "Song/SongTrackHeaderPanel.skin".into());
        let tpl = skin.load_skin(&rel).ok().and_then(|r| r.children.into_iter().next());
        if let (Some(hp), Some(t)) = (host.find_mut("TrackHeaderPanel"), tpl) {
            let rh = ((hp.rect.h as f32 - TIMELINE_H) / ENGINES as f32).floor() as i32;
            let w = hp.rect.w;
            for e in 0..ENGINES {
                let mut h = t.clone();
                h.set_rect(Rect::new(0, TIMELINE_H as i32 + e as i32 * rh, w, rh));
                if let Some(l) = h.find_mut("TrackNumberLabel") {
                    l.text = format!("{}", e + 1);
                }
                h.set_attr("songTrack", e.to_string());
                // A press on the header selects the engine [re:b8b0c6].
                h.overload = "SongTrackHeader".into();
                hp.children.push(h);
            }
        }
        let popup_tpl = crate::front::load(skin, "Song/SongPatternPopupPanel.skin").ok().map(|mut c| {
            c.name = POPUP.into();
            c.zorder = 2000;
            c
        });
        Ok(SongPanel {
            page: page.to_vec(),
            named: Default::default(),
            zoom: 16.0,
            insert_pattern: 0,
            erase: false,
            anchor: None,
            pattern_menu: None,
            popup_tpl,
            tool: saved_tool(),
            follow_on: crate::options::Options::load().bool("SongLastFollowMode", false),
            scroll: 0.0,
            selected: BTreeSet::new(),
            op: None,
            working: None,
            band: None,
            clipboard: Vec::new(),
            quantize_mode: 0,
            menu: None,
            menu_part: None,
            rename: None,
            marker: 0.0,
            timeline: None,
            body_w: std::cell::Cell::new(800.0),
            rec_state: 0,
            recording: Default::default(),
            retro: Default::default(),
        })
    }

    /// Snap [re:165e78]: with fewer than 4 bars visible a 16th, under 16
    /// bars a beat, else whole bars (a power of two of them per 32 bars).
    pub(crate) fn snap_qn(&self) -> f64 {
        let bars = self.body_w.get() as f64 / (4.0 * self.zoom as f64);
        if bars < 4.0 {
            0.25
        } else if bars < 16.0 {
            1.0
        } else {
            4.0 * (2f64.powf((bars * 0.03125).log2().floor()) as i64).max(1) as f64
        }
    }

    fn snap(&self, d: f64) -> f64 {
        let q = self.snap_qn();
        (d / q).round() * q
    }

    /// Parts touching the box `r` (body px; a point selects the part
    /// under it), as v1's select-by-rectangle.
    fn parts_in_rect(&self, song: &Song, r: RectF, rh: f32) -> BTreeSet<(usize, i64)> {
        let (q0, q1) = (r.x as f64 / self.zoom as f64 + self.scroll as f64, (r.x + r.w) as f64 / self.zoom as f64 + self.scroll as f64);
        let mut set = BTreeSet::new();
        for (t, ps) in song.tracks.iter().enumerate() {
            let (ty0, ty1) = (TIMELINE_H + t as f32 * rh, TIMELINE_H + (t + 1) as f32 * rh);
            if ty1 < r.y || ty0 > r.y + r.h {
                continue;
            }
            for p in ps.iter().filter(|p| p.end() > q0 && p.start < q1.max(q0 + 1e-6)) {
                set.insert((t, key(p.start)));
            }
        }
        set
    }

    /// The left press operation for the current tool ([re:4e6b67]) on
    /// `track`'s parts at `qn` (`x` / `row_y` px in the body and the row).
    /// Draw and Erase edit parts only with Cmd; Cmd under Select and Multi
    /// draws a part (Shift: box).
    fn left_op(&self, parts: &[SongPart], qn: f64, x: f32, row_y: f32, rh: f32, shift: bool, cmd: bool) -> LeftOp {
        let c = closest_edge(parts, qn);
        // Edges within 4 px of a part at least 16 px wide; the lower band
        // of a part is the phase anchor.
        let edge = |c: &Closest| {
            let p = &parts[c.idx];
            let wide = (p.end() - p.start) as f32 * self.zoom >= 16.0;
            let near = (((c.edge as f32 - self.scroll) * self.zoom) - x).abs() < 4.0;
            (wide && near).then_some(if c.end { LeftOp::Resize } else { LeftOp::Start })
        };
        let wide = |c: &Closest| (parts[c.idx].end() - parts[c.idx].start) as f32 * self.zoom >= 16.0;
        let lower = row_y > rh - FOOT_H;
        let part_edit = |c: Option<Closest>| {
            let Some(c) = c else { return LeftOp::BandNew };
            if let Some(op) = edge(&c) {
                return op;
            }
            if !c.inside || shift {
                LeftOp::BandNew
            } else if lower && wide(&c) {
                LeftOp::Phase
            } else {
                LeftOp::Move
            }
        };
        let draw = if shift { LeftOp::BandAdd } else { LeftOp::Insert };
        match (self.tool, cmd) {
            (SongTool::Draw, false) => draw,
            (SongTool::Erase, false) => LeftOp::Erase,
            (SongTool::Select | SongTool::Multi, true) => draw,
            (SongTool::Draw | SongTool::Erase | SongTool::Select, _) => part_edit(c),
            (SongTool::Multi, false) => {
                if shift {
                    return LeftOp::BandAdd;
                }
                let Some(c) = c.filter(|c| c.inside) else { return LeftOp::Insert };
                if let Some(op) = edge(&c) {
                    op
                } else if lower && wide(&c) {
                    LeftOp::Phase
                } else {
                    LeftOp::Move
                }
            }
        }
    }

    /// Start a press operation ([re:a03abd]) on `song`.
    fn start_left_op(&mut self, model: &Model, lop: LeftOp, song: &Song, p: Press) {
        let snapq = self.snap_qn();
        let mut part = p.part;
        // The press part joins the selection (a part outside it clears it).
        let select_part = |me: &mut Self, part: Option<(usize, i64)>| {
            if let Some(k) = part {
                if !me.selected.contains(&k) {
                    me.selected.clear();
                }
                me.selected.insert(k);
            }
        };
        let mut working = None;
        let mut new_part = None;
        let kind = match lop {
            LeftOp::Move => {
                // Shift toggles the part.
                match part {
                    Some(k) if p.shift => {
                        if !self.selected.remove(&k) {
                            self.selected.insert(k);
                        }
                    }
                    _ => select_part(self, part),
                }
                self.anchor = part;
                OpKind::Move { moving: false }
            }
            LeftOp::Start | LeftOp::Resize | LeftOp::Phase => {
                select_part(self, part);
                self.anchor = part;
                match lop {
                    LeftOp::Start => OpKind::Start,
                    LeftOp::Resize => OpKind::Resize,
                    _ => OpKind::Phase,
                }
            }
            // Band 2 clears the selection unless Shift; band 3 when the
            // press is off the parts. The point selects at once.
            LeftOp::BandNew | LeftOp::BandAdd => {
                if lop == LeftOp::BandNew && !p.shift {
                    self.selected.clear();
                }
                if !p.inside {
                    if lop == LeftOp::BandAdd {
                        self.selected.clear();
                    }
                    part = None;
                }
                OpKind::Band
            }
            // A part of the draw pattern at the draw marker (the press
            // floored to the snap), one pattern cycle long but no further
            // than the next part's start [re:781003]; it becomes the
            // selection.
            LeftOp::Insert => {
                let (t, pat) = (p.track, self.insert_pattern);
                let start = quantize_round((p.qn / snapq).floor() * snapq).max(0.0);
                let mut len = cycle_qn(model, t, pat);
                if len < ONE_TICK {
                    return;
                }
                let next = song.tracks.get(t).and_then(|ps| ps.iter().filter(|q| q.start > start + ONE_TICK && q.start < start + len).map(|q| q.start).reduce(f64::min));
                if let Some(n) = next {
                    len = quantize_round(n - start);
                }
                let np = SongPart { start, length: len, pattern: pat, phase: 0.0, mute: false };
                let mut s = song.clone();
                add_part_cut(model, &mut s, t, np.clone());
                working = Some(s);
                new_part = Some(np);
                self.selected = BTreeSet::from([(t, key(start))]);
                OpKind::Insert { track: t, extending: false }
            }
            // Erase: a press on a selected part takes the whole selection,
            // elsewhere just the part under the pointer [re:a03abd].
            LeftOp::Erase => {
                if part.is_some_and(|k| !self.selected.contains(&k)) || !p.inside {
                    self.selected.clear();
                }
                let mut gone = std::mem::take(&mut self.selected);
                gone.extend(self.parts_in_rect(song, RectF::new(p.x, p.y, 0.0, 0.0), p.rh));
                let mut s = song.clone();
                for (t, ps) in s.tracks.iter_mut().enumerate() {
                    ps.retain(|q| !gone.contains(&(t, key(q.start))));
                }
                working = Some(s);
                OpKind::Erase
            }
        };
        self.op = Some(Op { kind, qn0: p.qn, x0: p.x, y0: p.y, snapshot: song.clone(), saved: self.selected.clone(), part, new_part });
        self.working = working;
        if kind == OpKind::Band {
            self.apply_op(model, p.qn, p.x, p.y, p.rh, false);
        }
    }

    /// Apply the current operation's delta to its snapshot.
    fn apply_op(&mut self, model: &Model, qn: f64, x: f32, y: f32, rh: f32, alt: bool) {
        let snapq = self.snap_qn();
        let Some(op) = &mut self.op else { return };
        match &mut op.kind {
            // A press on a part moves it once the pointer has gone 3 px.
            OpKind::Move { moving } if !*moving => {
                if ((x - op.x0).powi(2) + (y - op.y0).powi(2)).sqrt() <= 3.0 {
                    return;
                }
                *moving = true;
            }
            // The new part repeats with the pointer once it is two snap
            // steps past the start, at least one step long [re:00d39d].
            OpKind::Insert { track, extending } => {
                let Some(np) = op.new_part.as_mut() else { return };
                let at = quantize_round((qn / snapq).round() * snapq);
                if !*extending {
                    if at >= np.start + 2.0 * snapq {
                        *extending = true;
                    }
                    return;
                }
                let len = quantize_round((at - np.start).max(snapq));
                if (len - np.length).abs() < 1e-9 {
                    return;
                }
                np.length = len;
                let (t, np) = (*track, np.clone());
                let mut song = op.snapshot.clone();
                add_part_cut(model, &mut song, t, np);
                self.working = Some(song);
                return;
            }
            OpKind::Erase => {
                let r = RectF::new(op.x0.min(x), op.y0.min(y), (x - op.x0).abs(), (y - op.y0).abs());
                let mut song = self.working.clone().unwrap_or_else(|| op.snapshot.clone());
                let gone = self.parts_in_rect(&song, r, rh);
                for (t, ps) in song.tracks.iter_mut().enumerate() {
                    ps.retain(|p| !gone.contains(&(t, key(p.start))));
                }
                self.selected.clear();
                self.working = Some(song);
                return;
            }
            _ => {}
        }
        let Some(op) = &self.op else { return };
        let delta = self.snap(qn - op.qn0);
        let mut song = op.snapshot.clone();
        let sel = op.saved.clone();
        let picked = selected_parts(&op.snapshot, &sel);
        match op.kind {
            OpKind::Band => {
                let r = RectF::new(op.x0.min(x), op.y0.min(y), (x - op.x0).abs(), (y - op.y0).abs());
                self.band = Some(r);
                let mut set = sel;
                set.extend(self.parts_in_rect(&op.snapshot, r, rh));
                self.selected = set;
                return;
            }
            OpKind::Insert { .. } | OpKind::Erase => return,
            // Move: no earlier than 0; with Alt the originals stay
            // (copy) [re:f6991f] [re:3a3ee7].
            OpKind::Move { .. } => {
                let min_start = picked.iter().map(|(_, p)| p.start).fold(f64::MAX, f64::min);
                let d = delta.max(-min_start);
                if !alt {
                    for (t, p) in &picked {
                        let tr = &mut song.tracks[*t];
                        tr.retain(|q| key(q.start) != key(p.start));
                    }
                }
                let mut new_sel = BTreeSet::new();
                for (t, p) in &picked {
                    let np = SongPart { start: quantize_round(p.start + d), ..p.clone() };
                    new_sel.insert((*t, key(np.start)));
                    add_part_cut(model, &mut song, *t, np);
                }
                self.selected = new_sel;
            }
            // Start change [re:4a9546]: end fixed, phase follows.
            OpKind::Start => {
                let min_len = picked.iter().map(|(_, p)| p.length).fold(f64::MAX, f64::min);
                let min_start = picked.iter().map(|(_, p)| p.start).fold(f64::MAX, f64::min);
                let d = delta.min((min_len - snapq).max(snapq)).max(-min_start);
                let mut new_sel = BTreeSet::new();
                for (t, p) in &picked {
                    let cycle = cycle_qn(model, *t, p.pattern).max(1e-6);
                    let np = SongPart { start: quantize_round(p.start + d), length: p.length - d, phase: phase_mod(p.phase + d, cycle), ..p.clone() };
                    song.tracks[*t].retain(|q| key(q.start) != key(p.start));
                    new_sel.insert((*t, key(np.start)));
                    add_part_cut(model, &mut song, *t, np);
                }
                self.selected = new_sel;
            }
            // Resize: length, at least one snap step.
            OpKind::Resize => {
                for (t, p) in &picked {
                    let len = (p.length + delta).max(snapq);
                    song.tracks[*t].retain(|q| key(q.start) != key(p.start));
                    add_part_cut(model, &mut song, *t, SongPart { length: len, ..p.clone() });
                }
            }
            // Phase change [re:ab5f6c]: phase wraps in the cycle.
            OpKind::Phase => {
                for (t, p) in &picked {
                    let cycle = cycle_qn(model, *t, p.pattern).max(1e-6);
                    let ph = (p.phase - delta).rem_euclid(cycle);
                    for q in song.tracks[*t].iter_mut().filter(|q| key(q.start) == key(p.start)) {
                        q.phase = ph;
                    }
                }
            }
        }
        self.working = Some(song);
    }

    /// Emit the changed tracks of `song` against `before`.
    fn commit(before: &Song, song: &Song, out: &mut Vec<Command>) {
        let changed: Vec<(usize, Vec<SongPart>)> = song
            .tracks
            .iter()
            .enumerate()
            .filter(|(t, ps)| before.tracks.get(*t) != Some(*ps))
            .map(|(t, ps)| {
                // Sorted here: the engine swaps them in on the audio thread.
                let mut ps = ps.clone();
                ps.sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap_or(std::cmp::Ordering::Equal));
                (t, ps)
            })
            .collect();
        if !changed.is_empty() {
            out.push(Command::SongTracks(Box::new(changed)));
        }
    }

    /// Pattern IDs by delta [re:061a2f] (delta clamped so every selected
    /// part stays in 0..=23) / set [re:490bc2] on the selection,
    /// committed as "Change pattern IDs".
    fn change_patterns(&mut self, model: &Model, delta: Option<i32>, set: Option<usize>, out: &mut Vec<Command>) {
        let before = song_of(model);
        let sel = selected_parts(&before, &self.selected);
        if sel.is_empty() {
            return;
        }
        let lo = sel.iter().map(|(_, p)| p.pattern as i32).min().unwrap_or(0);
        let hi = sel.iter().map(|(_, p)| p.pattern as i32).max().unwrap_or(0);
        let d = delta.map(|d| d.max(-lo).min(23 - hi)).unwrap_or(0);
        if delta.is_some() && d == 0 {
            return;
        }
        let mut song = before.clone();
        for (t, ps) in song.tracks.iter_mut().enumerate() {
            for p in ps.iter_mut().filter(|p| self.selected.contains(&(t, key(p.start)))) {
                p.pattern = match set {
                    Some(n) => n,
                    None => (p.pattern as i32 + d) as usize,
                };
            }
        }
        Self::commit(&before, &song, out);
    }

    /// `ShowPatternPopup(draw, ...)` [re:6a573c]: the 24 pattern keys of
    /// the track (v1's popup panel) under `anchor`, the selection's common
    /// pattern (or the draw pattern) lit.
    fn open_pattern_menu(&mut self, app: &mut App, model: &Model, anchor: &[usize], draw: bool) {
        let song = song_of(model);
        let sel = selected_parts(&song, &self.selected);
        let (track, current) = if draw {
            (model.engine, Some(self.insert_pattern))
        } else {
            let Some(&(t, _)) = sel.first() else { return };
            (t, sel.first().map(|(_, p)| p.pattern).filter(|c| sel.iter().all(|(_, p)| p.pattern == *c)))
        };
        let Some(mut popup) = self.popup_tpl.clone() else { return };
        // Under the anchor, inside the page.
        let (ax, ay) = app.origin(anchor);
        let (px, py) = app.origin(&self.page);
        let ah = app.ctrl(anchor).map(|c| c.rect.h).unwrap_or(17);
        let Some(page) = app.ctrl_mut(&self.page) else { return };
        let (w, h) = (popup.rect.w, popup.rect.h);
        let x = (ax - px).clamp(0, (page.rect.w - w).max(0));
        let y = if ay - py + ah + h <= page.rect.h { ay - py + ah } else { (ay - py - h).max(0) };
        popup.set_rect(Rect::new(x, y, w, h));
        for k in popup.children.iter_mut() {
            let Some(n) = k.name.strip_prefix("Key").and_then(|n| n.parse::<usize>().ok()) else { continue };
            k.on = current == Some(n);
            k.text = (n + 1).to_string();
            k.display_name = {
                let name = model.pattern_name(track, n);
                if name.is_empty() { format!("Pattern {}", n + 1) } else { format!("{}: {name}", n + 1) }
            };
            k.set_attr("patternUsed", model.pattern_used(track, n).to_string());
        }
        page.children.retain(|c| c.name != POPUP);
        page.children.push(popup);
        self.pattern_menu = Some(draw);
        app.dirty = true;
    }

    /// The open pattern popup's path.
    fn popup_path(&self, app: &App) -> Option<Vec<usize>> {
        let page = app.ctrl(&self.page)?;
        let i = page.children.iter().position(|c| c.name == POPUP)?;
        Some([self.page.clone(), vec![i]].concat())
    }

    /// Close the pattern popup.
    fn close_popup(&mut self, app: &mut App) {
        self.pattern_menu = None;
        if let Some(page) = app.ctrl_mut(&self.page) {
            page.children.retain(|c| c.name != POPUP);
        }
        app.dirty = true;
    }

    /// Part buttons [re:381bce]: the three part buttons inside the anchor
    /// part's top-right corner, hidden when the part is narrower than the
    /// buttons plus 8 px.
    fn place_part_buttons(&self, app: &mut App, model: &Model) {
        let body = self.named.path_in(app, &self.page, "SongBody");
        let geom = body.as_ref().and_then(|b| {
            let c = app.ctrl(b)?;
            let (bx, by) = app.origin(b);
            let (px, py) = app.origin(&self.page);
            Some((bx - px, by - py, row_h(c)))
        });
        let song = song_of(model);
        let part = self.anchor.and_then(|(t, k)| song.tracks.get(t).and_then(|ps| ps.iter().find(|p| key(p.start) == k)).map(|p| (t, p.clone())));
        let Some(page) = app.ctrl_mut(&self.page) else { return };
        let widths: Vec<i32> = ["PartPrevPatternButton", "PartNextPatternButton", "PartSelectPatternButton"].iter().map(|n| page.find(n).map(|c| c.rect.w).unwrap_or(10)).collect();
        let show = match (part, geom) {
            (Some((t, p)), Some((bx, by, rh))) => {
                let x0 = bx + ((p.start as f32 - self.scroll) * self.zoom) as i32;
                let w = ((p.end() - p.start) as f32 * self.zoom) as i32;
                let y0 = by + (TIMELINE_H + t as f32 * rh) as i32;
                (w >= widths.iter().sum::<i32>() + 8).then_some((x0, w, y0))
            }
            _ => None,
        };
        let names = ["PartPrevPatternButton", "PartNextPatternButton", "PartSelectPatternButton"];
        for (i, n) in names.iter().enumerate() {
            let Some(c) = page.find_mut(n) else { continue };
            match show {
                Some((x0, w, y0)) => {
                    let right = x0 + w;
                    let x = match i {
                        2 => right - 2 - widths[2],
                        1 => right - 4 - widths[2] - widths[1],
                        _ => right - 6 - widths[2] - widths[1] - widths[0],
                    };
                    c.rect.x = x;
                    c.rect.y = y0 + 2;
                    c.visible = true;
                    c.zorder = 10;
                }
                None => c.visible = false,
            }
        }
    }

    /// The context menu at `qn`, for `part` (the part under the pointer):
    /// Edit / Rename Pattern only when there is one ([re:ef2c92]).
    fn context_menu(&mut self, app: &mut App, anchor: &[usize], qn: f64, part: Option<(usize, SongPart)>) {
        let any = !self.selected.is_empty();
        // v1's list ([re:ef2c92]).
        let mut items: Vec<(String, MenuAction, bool)> = vec![
            ("Pen Tool".into(), MenuAction::Tool(0), true),
            ("Delete Tool".into(), MenuAction::Tool(1), true),
            ("Select Tool".into(), MenuAction::Tool(2), true),
            ("Multi Tool".into(), MenuAction::Tool(3), true),
            (String::new(), MenuAction::None, true),
            ("Cut Parts".into(), MenuAction::Cut, any),
            ("Copy Parts".into(), MenuAction::Copy, any),
            ("Paste Parts".into(), MenuAction::Paste, !self.clipboard.is_empty()),
            ("Delete Parts".into(), MenuAction::Delete, any),
            ("Duplicate Parts".into(), MenuAction::Duplicate, any),
            (String::new(), MenuAction::None, true),
            ("Quantize Parts (start)".into(), MenuAction::QuantizeStart, any),
            ("Quantize Parts (start/end)".into(), MenuAction::QuantizeBoth, any),
            ("Quantize Mode ▸".into(), MenuAction::QuantizeModeMenu, true),
            (String::new(), MenuAction::None, true),
            ("Select All Parts".into(), MenuAction::SelectAll, true),
            ("Select Empty Parts".into(), MenuAction::SelectEmpty, true),
            (String::new(), MenuAction::None, true),
            ("Clear All Parts".into(), MenuAction::ClearAll, true),
        ];
        if part.is_some() {
            items.push((String::new(), MenuAction::None, true));
            items.push(("Edit Pattern".into(), MenuAction::EditPattern, any));
            items.push(("Rename Pattern".into(), MenuAction::RenamePattern, any));
        }
        self.menu_part = part;
        let labels = items.iter().map(|(s, _, on)| if *on || s.is_empty() { s.clone() } else { format!("({s})") }).collect();
        self.menu = Some((items.into_iter().map(|(_, a, on)| if on { a } else { MenuAction::None }).collect(), qn));
        app.open_menu(anchor, labels, None);
    }

    /// The song page's rename box centred on part `p` of track `t` (kept
    /// inside the track area), holding the pattern's name and editing
    /// [re:f98408].
    fn show_rename(&mut self, app: &mut App, model: &Model, t: usize, p: &SongPart) {
        let Some(body) = self.named.path_in(app, &self.page, "SongBody") else { return };
        let Some(c) = app.ctrl(&body) else { return };
        let (bw, rh) = (c.rect.w, row_h(c));
        let Some(rel) = app.ctrl(&self.page).and_then(|pg| pg.path_of("PatternRenameTextBox")) else { return };
        let path = [self.page.clone(), rel].concat();
        let (bx, by) = app.origin(&body);
        let (hx, hy) = app.origin(&path[..path.len() - 1]);
        let (x0, w) = ((p.start as f32 - self.scroll) * self.zoom, (p.end() - p.start) as f32 * self.zoom);
        let y0 = TIMELINE_H + t as f32 * rh;
        let name = model.pattern_name(t, p.pattern);
        let Some(b) = app.ctrl_mut(&path) else { return };
        let (ew, eh) = (b.rect.w, b.rect.h);
        let left = bx - hx;
        let x = (left + (x0 + w / 2.0) as i32 - ew / 2).min(left + bw - ew).max(left);
        let y = by - hy + (y0 + (rh - FOOT_H) / 2.0) as i32 - eh / 2;
        b.set_rect(Rect::new(x, y, ew, eh));
        b.text = name;
        b.visible = true;
        self.rename = Some((t, p.pattern));
        app.begin_edit(&path);
    }

    /// Quantize [re:faf6db]: start (and end) to the snap grid;
    /// the phase follows the start in "Maintain Sync", stays in "Maintain
    /// Start Phase", and in "Auto" follows unless the pattern restarts.
    fn quantize(&self, model: &Model, song: &mut Song, end: bool) {
        let q = self.snap_qn();
        for (t, p) in selected_parts(song, &self.selected) {
            let s0 = quantize_round((p.start / q).round() * q);
            let mut e0 = p.end();
            if end {
                e0 = quantize_round((e0 / q).round() * q);
            }
            if s0 >= e0 {
                continue;
            }
            let restart = model.pattern(t, p.pattern).sync.restarts();
            let cycle = cycle_qn(model, t, p.pattern).max(1e-6);
            let phase = match self.quantize_mode {
                1 => p.phase,
                0 if restart => p.phase,
                _ => phase_mod(p.phase + s0 - p.start, cycle),
            };
            song.tracks[t].retain(|x| key(x.start) != key(p.start));
            add_part_cut(model, song, t, SongPart { start: s0, length: e0 - s0, phase, ..p });
        }
    }

    pub fn sync(&mut self, app: &mut App, model: &Model) {
        self.tick(model);
        self.place_part_buttons(app, model);
        let song = song_of(model);
        let ip = self.insert_pattern;
        let erase = self.erase;
        let tool = self.tool;
        let name = {
            let n = model.pattern_name(model.engine, ip);
            if n.is_empty() { format!("Pattern {}", ip + 1) } else { n }
        };
        // The rename box shows while it is being edited.
        if !app.is_editing() {
            self.rename = None;
        }
        let renaming = self.rename.is_some();
        let Some(page) = app.ctrl_mut(&self.page) else { return };
        // The blackout covers the lanes until song mode is on; its Activate
        // button turns it on.
        if let Some(b) = page.find_mut("BlackoutPanel") {
            b.visible = !song.active;
        }
        if let Some(b) = page.find_mut("PatternRenameTextBox") {
            b.visible = renaming;
        }
        if let Some(b) = page.find_mut("ShuffleButton") {
            b.visible = false;
        }
        if let Some(g) = page.find_mut("ToolButtonGroup") {
            for (b, t) in g.children.iter_mut().zip(SONG_TOOLS) {
                b.on = t == tool;
            }
        }
        let follow = self.follow_on;
        if let Some(b) = page.find_mut("FollowButton") {
            b.on = follow;
        }
        let _ = name;
        // Pattern buttons [re:4bd29a]: enabled with a selection.
        let any = !self.selected.is_empty();
        for n in ["SelectPatternButton", "NextPatternButton", "PrevPatternButton"] {
            if let Some(b) = page.find_mut(n) {
                b.enabled = any;
            }
        }
        if let Some(b) = page.find_mut("EraseButton") {
            b.on = erase;
        }
        // Scroll bar over the song length plus eight bars.
        let visible = self.body_w.get() / self.zoom;
        let total = (song.length_qn() as f32 + 32.0).max(visible);
        let (page_frac, pos) = (visible / total, if total > visible { self.scroll / (total - visible) } else { 0.0 });
        if let Some(b) = page.find_mut("HorizontalScrollBar") {
            b.set_attr("realPage", page_frac.to_string());
            b.value = pos.clamp(0.0, 1.0);
        }
        if let Some(hp) = page.find_mut("TrackHeaderPanel") {
            for h in &mut hp.children {
                let sel = h.attr("songTrack").and_then(|v| v.parse::<usize>().ok()) == Some(model.engine);
                h.style = if sel { "SongTrackPanelSelectedStyle".to_string() } else { format!("SongTrackPanelStyle{}", h.attr("songTrack").and_then(|v| v.parse::<usize>().ok()).unwrap_or(0) % 2) };
            }
        }
    }

    /// Hover cursor over the song body: the operation a left press would
    /// start (as `handle`'s Down) through the operation's cursor [re:d4f18d] —
    /// 2/3 band Selection, 5 resize ResizeRight, 6 change start ResizeLeft,
    /// 7 phase ResizeBiDir, 10 insert Pen, 12 erase Delete, 13 / 14 / 15 loop start / end /
    /// move ResizeLeft / ResizeRight / ResizeBiDir, others the arrow.
    pub fn hover_cursor(&self, app: &App, model: &Model, path: &[usize], overload: &str, x: f32, y: f32) -> Option<dreist_gui::cursor::Cursor> {
        use dreist_gui::cursor::Cursor;
        if overload != "SongBody" {
            return None;
        }
        let c = app.ctrl(path)?;
        let rh = row_h(c);
        let qn = (x / self.zoom + self.scroll) as f64;
        let song = song_of(model);
        let op = if y < TIMELINE_H {
            let (ls, le) = ((song.loop_start as f32 - self.scroll) * self.zoom, (song.loop_end as f32 - self.scroll) * self.zoom);
            if song.loop_enabled && y > 9.0 && x > ls - 4.0 && x < le + 4.0 {
                let (ds, de) = ((x - ls).abs(), (x - le).abs());
                if ds < 4.0 && ds < de {
                    13
                } else if de < 4.0 && de <= ds {
                    14
                } else {
                    15
                }
            } else {
                16
            }
        } else {
            let track = ((y - TIMELINE_H) / rh) as usize;
            if track >= ENGINES {
                return Some(Cursor::Arrow);
            }
            // As the press would start, for the tool [re:4e6b67]
            // [re:d4f18d] (5 is the part's end, 6 its start).
            let parts = song.tracks.get(track).map(Vec::as_slice).unwrap_or(&[]);
            let row_y = y - TIMELINE_H - track as f32 * rh;
            match self.left_op(parts, qn, x, row_y, rh, app.mods.shift, app.mods.cmd) {
                LeftOp::Move => 1,
                LeftOp::BandNew => 2,
                LeftOp::BandAdd => 3,
                LeftOp::Resize => 5,
                LeftOp::Start => 6,
                LeftOp::Phase => 7,
                LeftOp::Insert => 10,
                LeftOp::Erase => 12,
            }
        };
        Some(match op {
            2 | 3 => Cursor::image("Selection"),
            5 | 14 => Cursor::image("ResizeRight"),
            6 | 13 => Cursor::image("ResizeLeft"),
            7 | 15 => Cursor::image("ResizeBiDir"),
            8 | 12 => Cursor::image("Delete"),
            9 => Cursor::image("DragCopy"),
            10 | 11 => Cursor::image("Pen"),
            _ => Cursor::Arrow,
        })
    }

    pub fn handle(&mut self, app: &mut App, model: &mut Model, ev: &UiEvent, out: &mut Vec<Command>) -> bool {
        // A press outside the open pattern popup closes it.
        if self.pattern_menu.is_some() {
            let outside = match ev {
                UiEvent::Clicked { path, .. } | UiEvent::Mouse { path, kind: MouseKind::Down | MouseKind::RightDown, .. } => !self.popup_path(app).is_some_and(|p| path.starts_with(&p)),
                _ => false,
            };
            if outside {
                self.close_popup(app);
            }
        }
        match ev {
            UiEvent::Mouse { path, overload, kind, x, y } if overload == "SongBody" => {
                let Some(c) = app.ctrl(path) else { return false };
                let rh = row_h(c);
                let qn = (*x / self.zoom + self.scroll) as f64;
                // Timeline (left press [re:3fed38] / mouse move).
                if let Some((op, grab, s0, e0)) = self.timeline {
                    let snap = |v: f64| quantize_round((v / self.snap_qn()).round() * self.snap_qn()).max(0.0);
                    match kind {
                        MouseKind::Drag => {
                            match op {
                                13 => out.push(msg(id::SONG_SET_INTERNAL_LOOP_RANGE, &[], &[snap(qn).min(e0 - self.snap_qn()) as f32, e0 as f32])),
                                14 => out.push(msg(id::SONG_SET_INTERNAL_LOOP_RANGE, &[], &[s0 as f32, snap(qn).max(s0 + self.snap_qn()) as f32])),
                                15 => {
                                    let d = (snap(qn - grab + s0) - s0).max(-s0);
                                    out.push(msg(id::SONG_SET_INTERNAL_LOOP_RANGE, &[], &[(s0 + d) as f32, (e0 + d) as f32]));
                                }
                                _ => {
                                    self.marker = snap(qn);
                                    if song_of(model).active && !model.playing {
                                        out.push(msg(id::SET_CURRENT_TIME, &[], &[self.marker as f32]));
                                    }
                                }
                            }
                            return true;
                        }
                        MouseKind::Up => {
                            self.timeline = None;
                            return true;
                        }
                        _ => {}
                    }
                }
                if *y < TIMELINE_H && *kind == MouseKind::Down {
                    let song = song_of(model);
                    let (ls, le) = ((song.loop_start as f32 - self.scroll) * self.zoom, (song.loop_end as f32 - self.scroll) * self.zoom);
                    let op = if song.loop_enabled && *y > 9.0 && *x > ls - 4.0 && *x < le + 4.0 {
                        let (ds, de) = ((*x - ls).abs(), (*x - le).abs());
                        if ds < 4.0 && ds < de {
                            13
                        } else if de < 4.0 && de <= ds {
                            14
                        } else {
                            15
                        }
                    } else {
                        self.marker = quantize_round((qn / self.snap_qn()).round() * self.snap_qn()).max(0.0);
                        // Current time reset [re:e9cde4]: with the song on and the
                        // transport stopped, 0x1c `e_setCurrentTime`.
                        if song.active && !model.playing {
                            out.push(msg(id::SET_CURRENT_TIME, &[], &[self.marker as f32]));
                        }
                        16
                    };
                    self.timeline = Some((op, qn, song.loop_start, song.loop_end));
                    return true;
                }
                match kind {
                    MouseKind::Drag => {
                        if self.op.is_none() {
                            return false;
                        }
                        self.apply_op(model, qn, *x, *y, rh, app.mods.alt);
                        return true;
                    }
                    // Release [re:22dc97] (the right button's too under the
                    // Multi tool [re:739c34]).
                    MouseKind::Up => {
                        let Some(op) = self.op.take() else { return false };
                        self.band = None;
                        let mods = app.mods;
                        match op.kind {
                            // A click on a part (no Shift / Cmd) selects just it.
                            OpKind::Move { moving: false } if !mods.shift && !mods.cmd => {
                                self.selected = self.parts_in_rect(&op.snapshot, RectF::new(*x, *y, 0.0, 0.0), rh);
                            }
                            OpKind::Band => {
                                if op.part.is_none() && ((*x - op.x0).powi(2) + (*y - op.y0).powi(2)).sqrt() < 2.0 {
                                    self.selected.clear();
                                } else {
                                    let r = RectF::new(op.x0.min(*x), op.y0.min(*y), (*x - op.x0).abs(), (*y - op.y0).abs());
                                    let mut set = op.saved.clone();
                                    set.extend(self.parts_in_rect(&op.snapshot, r, rh));
                                    self.selected = set;
                                }
                            }
                            _ => {}
                        }
                        if let Some(song) = self.working.take() {
                            Self::commit(&op.snapshot, &song, out);
                        }
                        return true;
                    }
                    _ => {}
                }
                if *y < TIMELINE_H {
                    return false;
                }
                let track = ((*y - TIMELINE_H) / rh) as usize;
                if track >= ENGINES {
                    return false;
                }
                let song = song_of(model);
                let parts = song.tracks.get(track).cloned().unwrap_or_default();
                let mods = app.mods;
                let snapq = self.snap_qn();
                let closest = closest_edge(&parts, qn);
                let on_part = closest.as_ref().map(|c| (track, key(parts[c.idx].start)));
                let inside = closest.as_ref().is_some_and(|c| c.inside);
                match kind {
                    MouseKind::Down => {
                        // [re:9dbf73]: the press selects the engine (not with Shift).
                        if !mods.shift {
                            model.engine = track;
                        }
                        let row_y = *y - TIMELINE_H - track as f32 * rh;
                        let lop = self.left_op(&parts, qn, *x, row_y, rh, mods.shift, mods.cmd);
                        // The select tool's press moves the insert marker
                        // (not on an edge) [re:6f728c].
                        if self.tool == SongTool::Select && !matches!(lop, LeftOp::Resize | LeftOp::Start) {
                            self.marker = quantize_round((qn / snapq).round() * snapq).max(0.0);
                        }
                        self.start_left_op(model, lop, &song, Press { track, qn, x: *x, y: *y, rh, part: on_part, inside, shift: mods.shift });
                        true
                    }
                    MouseKind::RightDown => {
                        // Multi: the right button erases, Alt+right opens the
                        // menu [re:0fe2ad].
                        if self.tool == SongTool::Multi && !mods.alt {
                            self.start_left_op(model, LeftOp::Erase, &song, Press { track, qn, x: *x, y: *y, rh, part: on_part, inside, shift: mods.shift });
                            return true;
                        }
                        // The part under the pointer joins the selection (a
                        // part outside it, or empty space, clears it first);
                        // Draw and Select move the insert marker there
                        // [re:11e284] [re:ffe2b0].
                        let hit = parts.iter().find(|p| p.start <= qn && qn < p.end()).cloned();
                        let hk = hit.as_ref().map(|p| (track, key(p.start)));
                        if !hk.is_some_and(|k| self.selected.contains(&k)) {
                            self.selected.clear();
                        }
                        if let Some(k) = hk {
                            self.selected.insert(k);
                        }
                        if matches!(self.tool, SongTool::Draw | SongTool::Select) {
                            self.marker = quantize_round((qn / snapq).round() * snapq).max(0.0);
                        }
                        let path = path.clone();
                        self.context_menu(app, &path, qn, hit.map(|p| (track, p)));
                        true
                    }
                    _ => false,
                }
            }
            // A press on a track header selects its engine [re:b8b0c6].
            UiEvent::Mouse { path, overload, kind: MouseKind::Down, .. } if overload == "SongTrackHeader" => {
                let Some(t) = app.ctrl(path).and_then(|c| c.attr("songTrack")).and_then(|v| v.parse::<usize>().ok()) else { return false };
                model.engine = t.min(ENGINES - 1);
                true
            }
            // The rename box: the pattern's new name (every part of it shows
            // it) [re:78f000].
            UiEvent::TextCommitted { name, text, .. } if name == "PatternRenameTextBox" => {
                if let Some((t, p)) = self.rename.take() {
                    model.set_pattern_name(t, p, text.trim());
                }
                if let Some(b) = app.ctrl_mut(&self.page).and_then(|pg| pg.find_mut("PatternRenameTextBox")) {
                    b.visible = false;
                }
                true
            }
            // Erase on while pressed.
            UiEvent::Pressed { path, name } if name == "EraseButton" && path.starts_with(&self.page) => {
                self.erase = true;
                out.push(msg(id::SONG_SET_ERASE_STATE, &[model.engine as i32], &[1.0]));
                true
            }
            // A pattern picked from the popup → pattern IDs set (or the
            // draw tool's pattern).
            UiEvent::Clicked { path, name, .. } if self.pattern_menu.is_some() && self.popup_path(app).is_some_and(|p| path.starts_with(&p)) => {
                let draw = self.pattern_menu == Some(true);
                if let Some(n) = name.strip_prefix("Key").and_then(|n| n.parse::<usize>().ok()).filter(|n| *n < 24) {
                    if draw {
                        self.insert_pattern = n;
                    } else {
                        self.change_patterns(model, None, Some(n), out);
                    }
                }
                self.close_popup(app);
                true
            }
            UiEvent::MenuSelected { index, .. } if self.menu.is_some() => {
                let (actions, qn) = self.menu.take().unwrap();
                let a = actions.get(*index).copied().unwrap_or(MenuAction::None);
                self.menu_action(app, model, a, qn, out)
            }
            UiEvent::Wheel { overload, dy, .. } if overload == "SongBody" => {
                self.scroll = (self.scroll - dy * 4.0).max(0.0);
                true
            }
            UiEvent::ValueChanged { path, name, value, .. } if name == "HorizontalScrollBar" && path.starts_with(&self.page) => {
                let song = song_of(model);
                let visible = self.body_w.get() / self.zoom;
                let total = (song.length_qn() as f32 + 32.0).max(visible);
                self.scroll = value * (total - visible).max(0.0);
                true
            }
            UiEvent::Clicked { path, .. } if path.len() > 1 && path.starts_with(&self.page) && app.ctrl(&path[..path.len() - 1]).is_some_and(|g| g.name == "ToolButtonGroup") => {
                self.select_tool(SONG_TOOLS[path[path.len() - 1].min(3)]);
                true
            }
            UiEvent::Clicked { path, name, .. } if path.starts_with(&self.page) => {
                match name.as_str() {
                    // Control event [re:cacf78] → pattern IDs by delta.
                    "NextPatternButton" => self.change_patterns(model, Some(1), None, out),
                    "PrevPatternButton" => self.change_patterns(model, Some(-1), None, out),
                    // The blackout's Activate: song mode on (`e_songSetActive`).
                    "PowerButton" => out.push(msg(id::SONG_SET_ACTIVE, &[], &[1.0, 0.0])),
                    // Scroll to follow (`SongLastFollowMode`).
                    "FollowButton" => {
                        self.follow_on = !self.follow_on;
                        let mut o = crate::options::Options::load();
                        o.set("SongLastFollowMode", self.follow_on);
                        if !cfg!(test) {
                            o.save();
                        }
                    }
                    "SelectPatternButton" => self.open_pattern_menu(app, model, path, false),
                    // The draw tool's pattern.
                    "SetDrawPatternButton" => self.open_pattern_menu(app, model, path, true),
                    // Part buttons: select the anchor part, then as above.
                    "PartNextPatternButton" | "PartPrevPatternButton" | "PartSelectPatternButton" => {
                        if let Some(a) = self.anchor {
                            if !self.selected.contains(&a) {
                                self.selected.clear();
                            }
                            self.selected.insert(a);
                        }
                        match name.as_str() {
                            "PartNextPatternButton" => self.change_patterns(model, Some(1), None, out),
                            "PartPrevPatternButton" => self.change_patterns(model, Some(-1), None, out),
                            _ => self.open_pattern_menu(app, model, path, false),
                        }
                    }
                    // Momentary: the release ends the live erase.
                    "EraseButton" => {
                        self.erase = false;
                        out.push(msg(id::SONG_SET_ERASE_STATE, &[model.engine as i32], &[0.0]));
                    }
                    "ClearAllButton" => out.push(msg(id::SONG_CLEAR_ALL, &[], &[])),
                    "DuplicateButton" => {
                        self.menu_action(app, model, MenuAction::Duplicate, 0.0, out);
                    }
                    "ZoomInButton" => self.zoom = (self.zoom * 2.0).min(512.0),
                    "ZoomOutButton" => self.zoom = (self.zoom / 2.0).max(0.25),
                    _ => return false,
                }
                true
            }
            _ => false,
        }
    }

    fn menu_action(&mut self, app: &mut App, model: &mut Model, a: MenuAction, qn: f64, out: &mut Vec<Command>) -> bool {
        let song = song_of(model);
        let mut next = song.clone();
        let picked = selected_parts(&song, &self.selected);
        let remove = |s: &mut Song, parts: &[(usize, SongPart)]| {
            for (t, p) in parts {
                s.tracks[*t].retain(|q| key(q.start) != key(p.start));
            }
        };
        match a {
            MenuAction::Tool(k) => {
                self.select_tool(SONG_TOOLS[k.min(3)]);
                return true;
            }
            MenuAction::None => return false,
            MenuAction::Copy | MenuAction::Cut => {
                let t0 = picked.iter().map(|(_, p)| p.start).fold(f64::MAX, f64::min);
                self.clipboard = picked.iter().map(|(t, p)| (*t, SongPart { start: p.start - t0, ..p.clone() })).collect();
                if a == MenuAction::Copy {
                    return false;
                }
                remove(&mut next, &picked);
                self.selected.clear();
            }
            MenuAction::Delete => {
                remove(&mut next, &picked);
                self.selected.clear();
            }
            MenuAction::Paste => {
                let _ = qn;
                let at = self.marker;
                self.selected.clear();
                for (t, p) in self.clipboard.clone() {
                    let np = SongPart { start: quantize_round(at + p.start), ..p };
                    self.selected.insert((t, key(np.start)));
                    add_part_cut(model, &mut next, t, np);
                }
            }
            // Duplicate [re:b00d9a]: everything when nothing is selected;
            // copies follow the selection's end and become the selection.
            MenuAction::Duplicate => {
                let picked = if picked.is_empty() { song.tracks.iter().enumerate().flat_map(|(t, ps)| ps.iter().map(move |p| (t, p.clone()))).collect() } else { picked };
                if picked.is_empty() {
                    return false;
                }
                let lo = picked.iter().map(|(_, p)| p.start).fold(f64::MAX, f64::min);
                let hi = picked.iter().map(|(_, p)| p.end()).fold(0.0, f64::max);
                self.selected.clear();
                for (t, p) in picked {
                    let np = SongPart { start: quantize_round(p.start + hi - lo), ..p };
                    self.selected.insert((t, key(np.start)));
                    add_part_cut(model, &mut next, t, np);
                }
            }
            MenuAction::QuantizeStart | MenuAction::QuantizeBoth => {
                self.quantize(model, &mut next, a == MenuAction::QuantizeBoth);
                self.selected.clear();
            }
            MenuAction::QuantizeModeMenu => {
                if let Some(p) = app.root.path_of("SongBody") {
                    self.menu = Some(((0..3).map(MenuAction::QuantizeMode).collect(), qn));
                    app.open_menu(&p, QUANTIZE_MODES.iter().map(|s| s.to_string()).collect(), Some(self.quantize_mode));
                }
                return false;
            }
            MenuAction::QuantizeMode(m) => {
                self.quantize_mode = m;
                return false;
            }
            MenuAction::SelectAll => {
                self.selected = song.tracks.iter().enumerate().flat_map(|(t, ps)| ps.iter().map(move |p| (t, key(p.start)))).collect();
                return true;
            }
            // Parts whose pattern has no steps.
            MenuAction::SelectEmpty => {
                let m: &Model = model;
                let mut set = BTreeSet::new();
                for (t, ps) in song.tracks.iter().enumerate() {
                    for p in ps.iter().filter(|p| !m.pattern_used(t, p.pattern)) {
                        set.insert((t, key(p.start)));
                    }
                }
                self.selected = set;
                return true;
            }
            MenuAction::ClearAll => {
                out.push(msg(id::SONG_CLEAR_ALL, &[], &[]));
                self.selected.clear();
                return true;
            }
            // The part under the pointer when the menu opened, whatever the
            // selection [re:88c33b]: Edit selects its engine and pattern
            // and shows the pattern page [re:b87e75].
            MenuAction::EditPattern => {
                let Some((t, p)) = self.menu_part.clone() else { return false };
                model.engine = t;
                out.push(msg(id::SELECT_PATTERN, &[t as i32, -1, -1, p.pattern as i32], &[0.0]));
                let st = model.engine_mut(t).child_mut_or_insert("EngineState", "EngineState");
                st.set("CurrentPattern", p.pattern);
                model.editor_tab = 0;
                return true;
            }
            // Rename: the part alone selected, the song page's name box over
            // it [re:f98408].
            MenuAction::RenamePattern => {
                let Some((t, p)) = self.menu_part.clone() else { return false };
                self.selected = BTreeSet::from([(t, key(p.start))]);
                self.show_rename(app, model, t, &p);
                return true;
            }
        }
        // Selection keys after the change.
        Self::commit(&song, &next, out);
        true
    }

    /// Recorder messages ([re:7b9cde]: 500, 0x1f2, 499, 0x1f8).
    /// Returns whether a new recording started (the editor's undo point
    /// "Record Song Parts").
    pub fn song_event(&mut self, model: &Model, ev: &SongEvent, out: &mut Vec<Command>) -> bool {
        match ev {
            SongEvent::State(st) => {
                let started = *st == 2 && self.rec_state != 2;
                self.rec_state = *st;
                if started {
                    self.retro = Default::default();
                    self.recording = Default::default();
                }
                started
            }
            SongEvent::Started { track, pattern, time, phase, .. } => {
                if let Some(slot) = self.recording.get_mut(*track) {
                    let playing = model.song_pos.is_some() && song_of(model).active;
                    *slot = Some((SongPart { start: *time, length: 1.0 / 960.0, pattern: *pattern, phase: *phase, mute: false }, playing));
                }
                false
            }
            // `PatternStopped` [re:7c078d].
            SongEvent::Stopped { track, pattern, time, recording } => {
                let Some((mut part, playing)) = self.recording.get_mut(*track).and_then(Option::take) else { return false };
                if part.pattern != *pattern {
                    return false;
                }
                let end = quantize_round(*time);
                if end > part.start {
                    part.length = end - part.start;
                    if *recording {
                        let before = song_of(model);
                        let mut song = before.clone();
                        add_part_cut(model, &mut song, *track, part);
                        Self::commit(&before, &song, out);
                    } else {
                        self.retro[*track].push((part, playing));
                    }
                }
                false
            }
            SongEvent::NewRetro => {
                self.retro = Default::default();
                false
            }
        }
    }

    /// Timer tick [re:1f0ef0]: parts being recorded grow to the
    /// song position; a jump back before a part's start drops it.
    pub fn tick(&mut self, model: &Model) {
        let Some(pos) = model.song_pos else { return };
        for slot in &mut self.recording {
            if let Some((p, _)) = slot {
                if pos > p.start {
                    p.length = pos - p.start;
                } else if quantize_round(pos.floor()) < p.start {
                    *slot = None;
                }
            }
        }
    }

    pub fn has_retro(&self) -> bool {
        self.retro.iter().any(|t| !t.is_empty())
    }

    /// Retro record [re:4aa5f6] (Retro button): the buffered
    /// parts go into the song where they were played, selected.
    pub fn apply_retro(&mut self, model: &Model, out: &mut Vec<Command>) -> bool {
        if !self.has_retro() {
            return false;
        }
        // [re:4aa5f6]: the parts go where they were played, cutting what
        // they overlap (whether the song was playing does not matter).
        let before = song_of(model);
        let mut song = before.clone();
        self.selected.clear();
        for (t, parts) in std::mem::take(&mut self.retro).into_iter().enumerate() {
            for (p, _) in parts {
                self.selected.insert((t, key(p.start)));
                add_part_cut(model, &mut song, t, p);
            }
        }
        Self::commit(&before, &song, out);
        true
    }

    /// [re:1f0ef0] with
    /// FollowButton on: a new playback position outside the visible
    /// body (no drag going on) scrolls it to the middle. True when it moved.
    /// A new tool, remembered as `SongLastTool` ([re:9bf684]).
    fn select_tool(&mut self, t: SongTool) {
        self.tool = t;
        let mut o = crate::options::Options::load();
        o.set("SongLastTool", tool_id(t));
        if !cfg!(test) {
            o.save();
        }
    }

    pub fn follow(&mut self, pos: f64) -> bool {
        if self.op.is_some() || self.band.is_some() {
            return false;
        }
        let (left, w, p) = (self.scroll, self.body_w.get() / self.zoom, pos as f32);
        if p >= left && p <= left + w {
            return false;
        }
        let s = (p - w / 2.0).max(0.0);
        let moved = s != self.scroll;
        self.scroll = s;
        moved
    }

    /// Body-local x of the playback position line, or `None` when parts
    /// are being recorded (they grow with the position: the whole body
    /// changes).
    pub fn playhead_x(&self, pos: f64) -> Option<f32> {
        if self.recording.iter().any(Option::is_some) {
            return None;
        }
        Some((pos as f32 * self.zoom - self.scroll * self.zoom).round() + 0.5)
    }

    pub fn draw(&self, model: &Model, r: &Renderer, p: &mut Painter, c: &Ctrl) -> bool {
        if c.overload == "SongHScroll" {
            draw_scroll(r, p, c);
            return true;
        }
        if c.overload != "SongBody" {
            return false;
        }
        let s = r.styles;
        let song = self.working.clone().unwrap_or_else(|| song_of(model));
        let (w, h) = (c.rect.w as f32, c.rect.h as f32);
        self.body_w.set(w);
        let rh = row_h(c);
        let off = self.scroll * self.zoom;
        let colour = |st: &str, attr: &str, default: Colour| crate::front::colour_or(s, st, attr, default);
        let position = colour("GridPositionStyle", "colFill", Colour::rgba(236, 189, 47, 255));
        // Track rows: v1's empty lanes are darker than the parts, odd
        // tracks darker again (v1-song.png).
        for e in 0..ENGINES {
            let st = if e == model.engine {
                "SongLaneSelectedStyle"
            } else if e % 2 == 1 {
                "SongLaneAltStyle"
            } else {
                "SongLaneStyle"
            };
            let y = TIMELINE_H + e as f32 * rh;
            p.fill_rect(RectF::new(0.0, y, w, rh), colour(st, "colFill", Colour::WHITE));
        }
        // The ruler (`SongRulerStyle`) and the bar lines: solid through the
        // ruler and the empty lanes, darker at the labelled bars
        // (`SongBarLineStyle`; v1-song.png). Parts cover them.
        p.fill_rect(RectF::new(0.0, 0.0, w, TIMELINE_H), colour(RULER_STYLE, "colFill", Colour::rgba(210, 217, 229, 255)));
        let font = s.font("TimelineLabelStyle");
        let text = colour("TimelineLabelStyle", "colText", Colour::BLACK);
        let (minor, major) = (colour(BAR_STYLE, "colFill", Colour::rgba(92, 98, 114, 255)), colour(BAR_STYLE, "colAltFill", Colour::rgba(49, 55, 74, 255)));
        let bar_qn = 4.0f32;
        let first_bar = (self.scroll / bar_qn).floor() as usize;
        let bars = (w / (bar_qn * self.zoom)).ceil() as usize + 2;
        let label_every = ((48.0 / (bar_qn * self.zoom)).ceil() as usize).max(1);
        for b in first_bar..first_bar + bars {
            let x = (b as f32 * bar_qn * self.zoom - off).round() + 0.5;
            p.line(x, 0.0, x, h, if b % label_every == 0 { major } else { minor }, 1.0);
            if b % label_every == 0 {
                p.text(RectF::new(x + 3.0, 1.0, 50.0, TIMELINE_H - 3.0), &format!("{}", b + 1), text, &font, HAlign::Left, VAlign::Center);
            }
            if self.zoom * bar_qn >= 32.0 {
                for beat in 1..4 {
                    let bx = ((b as f32 * bar_qn + beat as f32) * self.zoom - off).round() + 0.5;
                    p.line(bx, TIMELINE_H - 6.0, bx, TIMELINE_H - 2.0, minor, 1.0);
                }
            }
        }
        // The ruler's frame: a line on top, 2 px at the bottom, the left edge.
        let frame = colour(RULER_STYLE, "colBorder", Colour::rgba(38, 45, 60, 255));
        p.fill_rect(RectF::new(0.0, 0.0, w, 1.0), frame);
        p.fill_rect(RectF::new(0.0, TIMELINE_H - 2.0, w, 2.0), frame);
        p.fill_rect(RectF::new(0.0, 0.0, 1.0, TIMELINE_H), frame);
        // Loop range (`SongLoopStyle`, v1's `LoopHorizontal` / `LoopGrab`):
        // a line along the ruler between two upright pairs.
        if song.loop_enabled {
            let col = colour("SongLoopStyle", "colFill", Colour::rgba(18, 25, 60, 255));
            let (x0, x1) = ((song.loop_start as f32 * self.zoom - off).round(), (song.loop_end as f32 * self.zoom - off).round());
            p.fill_rect(RectF::new(x0, 8.0, (x1 - x0).max(0.0), 1.0), col);
            for x in [x0 - 2.0, x0, x1 - 1.0, x1 + 1.0] {
                p.fill_rect(RectF::new(x, 6.0, 1.0, 5.0), col);
            }
        }
        // Parts: a flat face with a navy top edge and 2 px sides (one
        // divider between touching parts), dotted pattern ends; under it
        // the navy foot with notched ends, a dot at each pattern end and a
        // grip in each pattern repeat (v1's `PartNormal*` art).
        let fill = colour(PART_STYLE, "colFill", Colour::rgba(222, 229, 237, 255));
        let border = colour(PART_STYLE, "colBorder", Colour::BLACK);
        let ptext = colour(PART_STYLE, "colText", Colour::BLACK);
        let pfont = s.font(PART_STYLE);
        let radius = s.float(PART_STYLE, "realCornerRounding", 7.0).floor().min(rh / 2.0);
        let sel_col = colour(PART_SELECTED_STYLE, "colFill", Colour::rgba(236, 189, 47, 255));
        let checker = colour(PART_SELECTED_STYLE, "colAltFill", Colour::rgba(217, 205, 157, 255));
        let dots = colour("SongBarDotStyle", "colFill", Colour::rgba(18, 25, 60, 255));
        let foot = colour(FOOT_STYLE, "colFill", Colour::rgba(44, 48, 77, 255));
        let marks = colour(FOOT_STYLE, "colText", Colour::rgba(200, 206, 224, 255));
        for (e, parts) in song.tracks.iter().enumerate() {
            let y = TIMELINE_H + e as f32 * rh;
            let fy = y + rh - FOOT_H;
            for part in parts {
                let start = part.start;
                let x0 = start as f32 * self.zoom - off;
                let x1 = (start + part.length) as f32 * self.zoom - off;
                if x1 < 0.0 || x0 > w {
                    continue;
                }
                let rect = RectF::new(x0.round(), y, (x1 - x0).round(), rh - FOOT_H);
                let selected = self.selected.contains(&(e, key(part.start)));
                let base = if selected { sel_col } else { fill };
                let col = if part.mute { base.with_alpha(base.f_alpha() * 0.5) } else { base };
                p.fill_rounded(rect, radius, [true; 4], col);
                if selected {
                    check(p, rect, checker);
                }
                p.fill_rect(RectF::new(rect.x, rect.y, rect.w, 1.0), border);
                p.fill_rect(RectF::new(rect.x, rect.y, 2.0, rect.h), border);
                let next = parts.iter().any(|q| (q.start - (start + part.length)).abs() < 1e-6);
                if !next {
                    p.fill_rect(RectF::new(rect.x + rect.w - 2.0, rect.y, 2.0, rect.h), border);
                }
                // The foot, its bottom corners notched.
                let (fx, fw) = (rect.x, rect.w);
                p.fill_rect(RectF::new(fx, fy, fw, FOOT_H - 3.0), foot);
                p.fill_rect(RectF::new(fx + 2.0, fy + FOOT_H - 3.0, (fw - 4.0).max(0.0), 2.0), foot);
                p.fill_rect(RectF::new(fx + 4.0, fy + FOOT_H - 1.0, (fw - 8.0).max(0.0), 1.0), foot);
                // Pattern ends (after the phase offset): dotted on the face,
                // a dot in the foot; a grip in the middle of each repeat
                // wide enough for one.
                let plen = cycle_qn(model, e, part.pattern) as f32;
                let mut ends = vec![x0];
                if plen > 0.0 {
                    let mut t = start as f32 + (plen - (part.phase as f32).rem_euclid(plen)) % plen;
                    if t <= start as f32 + 1e-3 {
                        t += plen;
                    }
                    while t < (start + part.length) as f32 - 1e-3 {
                        let dx = (t * self.zoom - off).round();
                        let mut dy = y + 1.0;
                        while dy < fy {
                            p.fill_rect(RectF::new(dx, dy, 1.0, 1.0), dots);
                            dy += 2.0;
                        }
                        p.fill_rect(RectF::new(dx, fy + 5.0, 1.0, 1.0), marks);
                        ends.push(t * self.zoom - off);
                        t += plen;
                    }
                }
                ends.push(x1);
                for pair in ends.windows(2) {
                    let (a, b) = (pair[0].max(x0 + 2.0), pair[1].min(x1 - 2.0));
                    if b - a >= 12.0 {
                        let cx = ((a + b) / 2.0).round();
                        for k in 0..4 {
                            p.fill_rect(RectF::new(cx - 3.0 + k as f32 * 2.0, fy + 2.0, 1.0, 5.0), marks);
                        }
                    }
                }
                let name = if part.pattern == STOP_PART {
                    String::new()
                } else {
                    let n = model.pattern_name(e, part.pattern);
                    if n.is_empty() { format!("Pattern {}", part.pattern + 1) } else { n }
                };
                // The pattern number over the name, centred; a name too
                // long for the part is cut in the middle (v1's shortening
                // labels, video 01 04-20).
                if rect.w > 14.0 && part.pattern != STOP_PART {
                    let tw = rect.w - 6.0;
                    let number = p.fonts.shorten(&pfont, &format!("{}", part.pattern + 1), tw, Shorten::Middle);
                    let name = p.fonts.shorten(&pfont, &name, tw, Shorten::Middle);
                    p.text(RectF::new(rect.x + 3.0, y + 5.0, tw, 12.0), &number, ptext, &pfont, HAlign::Center, VAlign::Center);
                    p.text(RectF::new(rect.x + 3.0, y + 17.0, tw, 12.0), &name, ptext, &pfont, HAlign::Center, VAlign::Center);
                }
            }
        }
        // Parts being recorded (`SongPartRecordingStyle`) and retro parts
        // (`SongPartRetroStyle`): fill and edge.
        let rec_col = (colour("SongPartRecordingStyle", "colFill", Colour::rgba(244, 116, 116, 255)), colour("SongPartRecordingStyle", "colBorder", border));
        let retro_col = (colour("SongPartRetroStyle", "colFill", Colour::rgba(244, 116, 116, 160)), colour("SongPartRetroStyle", "colBorder", border));
        let grow = |p: &SongPart| model.song_pos.filter(|pos| *pos > p.start).map(|pos| pos - p.start).unwrap_or(p.length);
        let extra = self
            .recording
            .iter()
            .enumerate()
            .filter_map(|(t, x)| x.as_ref().map(|(p, _)| (t, p.start, grow(p), rec_col)))
            .chain(self.retro.iter().enumerate().flat_map(|(t, ps)| ps.iter().map(move |(p, _)| (t, p.start, p.length, retro_col))));
        for (t, start, length, (col, edge)) in extra {
            let y = TIMELINE_H + t as f32 * rh;
            let x0 = start as f32 * self.zoom - off;
            let x1 = (start + length) as f32 * self.zoom - off;
            if x1 < 0.0 || x0 > w {
                continue;
            }
            let rect = RectF::new(x0.round() - 1.0, y + 1.0, (x1 - x0).round().max(1.0) + 1.0, rh - 2.0);
            p.fill_rounded(rect, radius, [true; 4], col);
            p.stroke_rounded(rect.inset(0.5), radius, [true; 4], edge, 1.0);
        }
        // Insert marker (`SongMarkerStyle`): a head on the ruler's top
        // edge over a short line (v1-song.png).
        let mx = ((self.marker as f32 - self.scroll) * self.zoom).round();
        let mcol = colour("SongMarkerStyle", "colFill", Colour::rgba(18, 25, 60, 255));
        p.fill_polygon(&[(mx - 3.0, 0.0), (mx + 4.0, 0.0), (mx + 0.5, 4.0)], mcol);
        p.fill_rect(RectF::new(mx, 0.0, 1.0, 7.0), mcol);
        // Rubber band.
        if let Some(b) = self.band {
            let col = sel_col;
            p.fill_rect(b, col.with_alpha(0.25));
            p.stroke_rect(b.inset(0.5), col, 1.0);
        }
        // Playback position.
        if let Some(pos) = model.song_pos.filter(|_| song.active) {
            let x = (pos as f32 * self.zoom - off).round() + 0.5;
            p.line(x, 0.0, x, h, position, 1.0);
        }
        true
    }
}

/// The song page's mouse tools as v1's ([re:4e6b67] [re:a03abd]
/// [re:00d39d] [re:22dc97]).
#[cfg(test)]
mod tool_tests {
    use super::*;
    use crate::Editor;
    use dreist_gui::app::{Mods, MouseButton};

    fn editor() -> Option<Editor> {
        crate::app::test_setup();
        let mut ed = Editor::new(dreist_gui::skin::default_theme(), Model::new(crate::project::new_document())).ok()?;
        ed.model.editor_tab = SONG_TAB;
        ed.sync();
        let p = ed.app.root.path_of("PowerButton")?;
        ed.handle(&UiEvent::Clicked { path: p, name: "PowerButton".into(), id: 0, on: false });
        // v1 starts on Draw ([re:7fdc18]); the tests start on Multi.
        assert_eq!(ed.song.as_ref()?.tool, SongTool::Draw);
        ed.song.as_mut()?.tool = SongTool::Multi;
        Some(ed)
    }

    fn song(ed: &Editor) -> Song {
        song_of(&ed.model)
    }

    fn panel(ed: &mut Editor) -> &mut SongPanel {
        ed.song.as_mut().unwrap()
    }

    /// Body origin (root px) and row height.
    fn body(ed: &Editor) -> (f32, f32, f32) {
        let b = ed.app.root.path_of("SongBody").unwrap();
        let (x, y) = ed.app.origin(&b);
        (x as f32, y as f32, row_h(ed.app.ctrl(&b).unwrap()))
    }

    /// Root px of `qn` in the middle (upper half) of track `t`'s row.
    fn at(ed: &Editor, t: usize, qn: f64) -> (f32, f32) {
        let (bx, by, rh) = body(ed);
        let sp = ed.song.as_ref().unwrap();
        (bx + (qn as f32 - sp.scroll) * sp.zoom, by + TIMELINE_H + t as f32 * rh + rh * 0.3)
    }

    /// Press `button` at `from` with `mods`, move through `path`, release.
    fn gesture(ed: &mut Editor, button: MouseButton, mods: Mods, from: (f32, f32), path: &[(f32, f32)]) {
        ed.app.mods = mods;
        for ev in ed.app.mouse_move(from.0, from.1) {
            ed.handle(&ev);
        }
        for ev in ed.app.mouse_down(button) {
            ed.handle(&ev);
        }
        for &(x, y) in path {
            for ev in ed.app.mouse_move(x, y) {
                ed.handle(&ev);
            }
        }
        for ev in ed.app.mouse_up(button) {
            ed.handle(&ev);
        }
        ed.app.mods = Mods::default();
    }

    fn click(ed: &mut Editor, mods: Mods, t: usize, qn: f64) {
        let p = at(ed, t, qn);
        gesture(ed, MouseButton::Left, mods, p, &[]);
    }

    fn spans(ed: &Editor, t: usize) -> Vec<(f64, f64)> {
        song(ed).tracks[t].iter().map(|p| (p.start, p.length)).collect()
    }

    const CMD: Mods = Mods { shift: false, cmd: true, alt: false, ctrl: false };
    const SHIFT: Mods = Mods { shift: true, cmd: false, alt: false, ctrl: false };
    const ALT: Mods = Mods { shift: false, cmd: false, alt: true, ctrl: false };
    const NONE: Mods = Mods { shift: false, cmd: false, alt: false, ctrl: false };

    #[test]
    fn draw_drag_extends_the_new_part() {
        let Some(mut ed) = editor() else { return };
        panel(&mut ed).tool = SongTool::Draw;
        let snap = panel(&mut ed).snap_qn();
        // Under two snap steps from the start the part keeps its pattern
        // length; past them it follows the pointer.
        let from = at(&ed, 0, 0.3);
        let (a, b) = (at(&ed, 0, 2.0 * snap + 0.1), at(&ed, 0, 10.0));
        gesture(&mut ed, MouseButton::Left, NONE, from, &[a, b]);
        assert_eq!(spans(&ed, 0), vec![(0.0, 10.0)]);
        assert_eq!(ed.song.as_ref().unwrap().selected, BTreeSet::from([(0, 0)]), "the new part is the selection");
        // Shorter than the pattern too, at least one snap step.
        let from = at(&ed, 1, 0.3);
        let (a, b) = (at(&ed, 1, 2.0 * snap + 0.1), at(&ed, 1, 0.0));
        gesture(&mut ed, MouseButton::Left, NONE, from, &[a, b]);
        assert_eq!(spans(&ed, 1), vec![(0.0, snap)]);
    }

    #[test]
    fn draw_on_top_of_a_part_cuts_it() {
        let Some(mut ed) = editor() else { return };
        panel(&mut ed).tool = SongTool::Draw;
        // A 16 QN part, then a 4 QN part from QN 4 in it: the old part is
        // split around the new one, its tail keeping its phase.
        let from = at(&ed, 0, 0.3);
        let (a, b) = (at(&ed, 0, 3.0), at(&ed, 0, 16.0));
        gesture(&mut ed, MouseButton::Left, NONE, from, &[a, b]);
        assert_eq!(spans(&ed, 0), vec![(0.0, 16.0)]);
        click(&mut ed, NONE, 0, 4.2);
        assert_eq!(spans(&ed, 0), vec![(0.0, 4.0), (4.0, 4.0), (8.0, 8.0)]);
        assert!(song(&ed).tracks[0][2].phase.abs() < 1e-9, "{:?}", song(&ed).tracks[0][2]);
        // A new part stops at the next part's start.
        click(&mut ed, NONE, 0, 18.5);
        click(&mut ed, NONE, 0, 16.0);
        assert_eq!(spans(&ed, 0).last().copied(), Some((18.0, 4.0)));
        assert!(spans(&ed, 0).contains(&(16.0, 2.0)), "{:?}", spans(&ed, 0));
    }

    #[test]
    fn draw_and_erase_edit_parts_only_with_cmd() {
        let Some(mut ed) = editor() else { return };
        panel(&mut ed).tool = SongTool::Draw;
        click(&mut ed, NONE, 0, 0.3);
        // Cmd-drag moves.
        let (from, to) = (at(&ed, 0, 2.0), at(&ed, 0, 10.0));
        gesture(&mut ed, MouseButton::Left, CMD, from, &[to]);
        assert_eq!(spans(&ed, 0), vec![(8.0, 4.0)]);
        // Cmd + Alt copies.
        let (from, to) = (at(&ed, 0, 10.0), at(&ed, 0, 18.0));
        gesture(&mut ed, MouseButton::Left, Mods { alt: true, ..CMD }, from, &[to]);
        assert_eq!(spans(&ed, 0), vec![(8.0, 4.0), (16.0, 4.0)]);
        // A plain press on a part draws over it.
        click(&mut ed, NONE, 0, 10.2);
        assert_eq!(spans(&ed, 0), vec![(8.0, 2.0), (10.0, 4.0), (16.0, 4.0)]);
        // Erase: Cmd-drag moves instead of erasing.
        panel(&mut ed).tool = SongTool::Erase;
        let (from, to) = (at(&ed, 0, 17.0), at(&ed, 0, 25.0));
        gesture(&mut ed, MouseButton::Left, CMD, from, &[to]);
        assert_eq!(spans(&ed, 0), vec![(8.0, 2.0), (10.0, 4.0), (24.0, 4.0)]);
        // Shift-press under Draw is a box adding to the selection.
        panel(&mut ed).tool = SongTool::Draw;
        panel(&mut ed).selected.clear();
        let (from, to) = (at(&ed, 0, 7.0), at(&ed, 0, 11.0));
        gesture(&mut ed, MouseButton::Left, SHIFT, from, &[to]);
        assert_eq!(spans(&ed, 0).len(), 3, "no new part");
        assert_eq!(ed.song.as_ref().unwrap().selected.len(), 2);
    }

    #[test]
    fn alt_drag_copies_under_multi() {
        let Some(mut ed) = editor() else { return };
        assert_eq!(ed.song.as_ref().unwrap().tool, SongTool::Multi);
        click(&mut ed, NONE, 0, 0.3);
        let (from, to) = (at(&ed, 0, 2.0), at(&ed, 0, 10.0));
        gesture(&mut ed, MouseButton::Left, ALT, from, &[to]);
        assert_eq!(spans(&ed, 0), vec![(0.0, 4.0), (8.0, 4.0)]);
        assert_eq!(ed.song.as_ref().unwrap().selected, BTreeSet::from([(0, key(8.0))]), "the copies are the selection");
    }

    #[test]
    fn select_cmd_click_adds_a_part() {
        let Some(mut ed) = editor() else { return };
        panel(&mut ed).tool = SongTool::Select;
        let snap = panel(&mut ed).snap_qn();
        click(&mut ed, NONE, 0, 4.6);
        assert!(spans(&ed, 0).is_empty());
        assert_eq!(ed.song.as_ref().unwrap().marker, (4.6 / snap).round() * snap, "a press moves the insert marker");
        let from = at(&ed, 0, 4.5);
        let (a, b) = (at(&ed, 0, 4.0 + 2.0 * snap + 0.1), at(&ed, 0, 12.0));
        gesture(&mut ed, MouseButton::Left, CMD, from, &[a, b]);
        assert_eq!(spans(&ed, 0), vec![(4.0, 8.0)]);
    }

    #[test]
    fn erase_click_takes_the_part_or_the_selection() {
        let Some(mut ed) = editor() else { return };
        for t in 0..3 {
            click(&mut ed, NONE, t, 0.3);
        }
        panel(&mut ed).selected = BTreeSet::from([(0, 0), (1, 0)]);
        panel(&mut ed).tool = SongTool::Erase;
        // An unselected part: just it.
        click(&mut ed, NONE, 2, 1.0);
        assert!(spans(&ed, 2).is_empty());
        assert_eq!((spans(&ed, 0).len(), spans(&ed, 1).len()), (1, 1), "the selection stays in the song");
        assert!(ed.song.as_ref().unwrap().selected.is_empty());
    }

    #[test]
    fn erase_click_on_a_selected_part_takes_the_selection() {
        let Some(mut ed) = editor() else { return };
        for t in 0..3 {
            click(&mut ed, NONE, t, 0.3);
        }
        panel(&mut ed).tool = SongTool::Erase;
        panel(&mut ed).selected = BTreeSet::from([(0, 0), (1, 0)]);
        click(&mut ed, NONE, 1, 1.0);
        assert!(spans(&ed, 0).is_empty() && spans(&ed, 1).is_empty());
        assert_eq!(spans(&ed, 2).len(), 1);
        // Dragging erases everything the box crosses.
        panel(&mut ed).tool = SongTool::Draw;
        click(&mut ed, NONE, 0, 8.3);
        click(&mut ed, NONE, 1, 8.3);
        panel(&mut ed).tool = SongTool::Erase;
        let (from, to) = (at(&ed, 0, 7.0), at(&ed, 1, 9.0));
        gesture(&mut ed, MouseButton::Left, NONE, from, &[to]);
        assert!(spans(&ed, 0).is_empty() && spans(&ed, 1).is_empty());
        assert_eq!(spans(&ed, 2).len(), 1);
    }

    #[test]
    fn multi_right_button_erases_alt_opens_the_menu() {
        let Some(mut ed) = editor() else { return };
        click(&mut ed, NONE, 0, 0.3);
        click(&mut ed, NONE, 0, 8.3);
        let p = at(&ed, 0, 1.0);
        gesture(&mut ed, MouseButton::Right, NONE, p, &[]);
        assert_eq!(spans(&ed, 0), vec![(8.0, 4.0)]);
        assert!(ed.app.menu.is_none());
        let p = at(&ed, 0, 9.0);
        gesture(&mut ed, MouseButton::Right, ALT, p, &[]);
        assert!(ed.app.menu.is_some());
        assert_eq!(spans(&ed, 0), vec![(8.0, 4.0)]);
    }

    /// Run the context menu item `a` (opened at `qn` on track `t`).
    fn menu(ed: &mut Editor, t: usize, qn: f64, a: MenuAction) {
        let p = at(ed, t, qn);
        gesture(ed, MouseButton::Right, NONE, p, &[]);
        let m = ed.app.menu.clone().expect("menu");
        let index = ed.song.as_ref().unwrap().menu.as_ref().unwrap().0.iter().position(|x| *x == a).expect("menu item");
        ed.handle(&UiEvent::MenuSelected { anchor: m.anchor, name: String::new(), index });
        ed.app.menu = None;
    }

    #[test]
    fn menu_moves_the_insert_marker_and_pastes_there() {
        let Some(mut ed) = editor() else { return };
        panel(&mut ed).tool = SongTool::Select;
        // Multi's right button erases: draw with Cmd under Select.
        click(&mut ed, CMD, 0, 0.3);
        menu(&mut ed, 0, 1.0, MenuAction::Copy);
        menu(&mut ed, 1, 12.2, MenuAction::Paste);
        assert_eq!(ed.song.as_ref().unwrap().marker, 12.0);
        assert_eq!(spans(&ed, 0), vec![(0.0, 4.0), (12.0, 4.0)], "the clipboard keeps its track");
    }

    #[test]
    fn edit_pattern_takes_the_part_under_the_pointer() {
        let Some(mut ed) = editor() else { return };
        panel(&mut ed).tool = SongTool::Select;
        panel(&mut ed).insert_pattern = 2;
        click(&mut ed, CMD, 0, 0.3);
        panel(&mut ed).insert_pattern = 5;
        click(&mut ed, CMD, 1, 0.3);
        panel(&mut ed).selected = BTreeSet::from([(0, 0), (1, 0)]);
        menu(&mut ed, 1, 1.0, MenuAction::EditPattern);
        assert_eq!(ed.model.engine, 1);
        assert_eq!(ed.model.current_pattern(1), 5);
        assert_eq!(ed.model.editor_tab, 0);
    }

    #[test]
    fn rename_pattern_stays_on_the_song_page() {
        let Some(mut ed) = editor() else { return };
        panel(&mut ed).tool = SongTool::Select;
        panel(&mut ed).insert_pattern = 3;
        click(&mut ed, CMD, 0, 0.3);
        click(&mut ed, CMD, 0, 8.3);
        menu(&mut ed, 0, 9.0, MenuAction::RenamePattern);
        assert_eq!(ed.model.editor_tab, SONG_TAB);
        ed.sync();
        let b = ed.app.root.path_of("PatternRenameTextBox").unwrap();
        assert!(ed.app.ctrl(&b).unwrap().visible && ed.app.is_editing());
        assert_eq!(ed.song.as_ref().unwrap().selected, BTreeSet::from([(0, key(8.0))]));
        ed.handle(&UiEvent::TextCommitted { path: b.clone(), name: "PatternRenameTextBox".into(), text: "Verse".into() });
        assert_eq!(ed.model.pattern_name(0, 3), "Verse");
        assert!(!ed.app.ctrl(&b).unwrap().visible);
    }

    #[test]
    fn track_header_press_selects_the_engine() {
        let Some(mut ed) = editor() else { return };
        let hp = ed.app.root.path_of("TrackHeaderPanel").unwrap();
        let h = [hp, vec![3]].concat();
        let (x, y) = ed.app.origin(&h);
        gesture(&mut ed, MouseButton::Left, NONE, (x as f32 + 3.0, y as f32 + 3.0), &[]);
        assert_eq!(ed.model.engine, 3);
    }
}
