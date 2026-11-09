//! The pad view of the Pad / Layers page (`PadControls.skin`, manual
//! §5:2): layer play mode, Vel-Amp and the velocity split lanes.
//!
//! - `SelModeCombo`: the five [re:1cbdd2] modes →
//!   `e_setPadLayerMode` 0x51 [e, p] [mode].
//! - `V2A` knob: `e_setPadParameter` 0x4e [e, p] [v, 2 (V2A), 0].
//! - "Auto" (enabled in velocity-split mode only, update
//!   [re:f8c41d]) posts `e_autoVelocitySplitPad` at once ([re:ab11a3];
//!   v1 has no overlap choices).
//! - No right-click menu on the knobs (v1's panel ignores right clicks).
//! - Eight [re:924f85]s (`Split0..7`), one per layer, shown in
//!   velocity-split mode (the manual: "only visible when the Layer mode is
//!   set to Velocity split"), enabled when the layer has audio and is not
//!   locked. Releasing a drag posts min 0x7e, min turnover 0x7f, max
//!   turnover 0x81, max 0x80 [e, p, l] ([re:b8fa20], undo "Engine %d Pad
//!   %d velocity split" from a second callback).
//!
//! [re:924f85] ([re:924f85]..): velocity v sits at x = v·(w−8)/127
//! ([re:355118]; back with +0.5 truncation,
//! [re:dcdd9d]). Four 8×8 handles: lowest at the
//! bottom, the turnovers at the top, highest at the bottom (layout
//! [re:924f85]). Drawing [re:272fc5]: 126 grid lines at floor(i·w/127) in colBorder
//! (colAltBorder every 8th), the bounds in colBorder; enabled, the
//! trapezoid (lowest, h) → (low turn, 0) → (high turn + 8, 0) → (highest + 8,
//! h) filled colFill α0.4 and stroked colFill, then the handles: dragged
//! colAltFill, others colAltFill darker 0.4 at α0.5 (hover lighter 0.4 is not
//! tracked). A left press [re:4e003f] grabs a handle, or the whole
//! split when none is hit; a mouse move [re:64a9e4] applies the clamps of
//! [`drag`].

use dreist_engine::binding::id;
use dreist_engine::velsplit::Split;
use dreist_gui::app::{App, MouseKind, UiEvent};
use dreist_gui::paint::{Painter, RectF};
use dreist_gui::render::Renderer;
use dreist_gui::{Colour, Ctrl};

use crate::command::{Command, msg};
use crate::model::{LAYERS, Model};

pub const HANDLE: f32 = 8.0;
pub const STYLE: &str = "VSplitStyle";
/// The velocity ruler over the lanes (v1's marker strip; `LcdRulerStyle`).
pub const RULER: &str = "VelocityRuler";

/// The velocity ruler: a tick at every velocity (colTick), labels 0, 32,
/// 64, 96, 127 (colText) on colAltFill boxes, on colFill, over the lanes'
/// 0..127 scale.
fn draw_ruler(r: &Renderer, p: &mut Painter, c: &Ctrl) {
    use dreist_gui::style::{HAlign, VAlign};
    let s = r.styles;
    let st = c.style.as_str();
    let (w, h) = (c.rect.w as f32, c.rect.h as f32);
    p.fill_rect(RectF::new(0.0, 0.0, w, h), s.colour(st, "colFill", Colour::TRANSPARENT));
    let (tick, text) = (s.colour(st, "colTick", Colour::BLACK), s.colour(st, "colText", Colour::BLACK));
    let knock = s.colour(st, "colAltFill", Colour::TRANSPARENT);
    let font = s.font(st);
    // v1's `GraphMarkers` (508 × 10): a line along the top and bottom and a
    // full-height tick at every velocity; 0, 32, 64, 96 and 127 labelled on
    // light boxes.
    p.fill_rect(RectF::new(0.0, 0.0, w, 1.0), tick);
    p.fill_rect(RectF::new(0.0, h - 1.0, w, 1.0), tick);
    let step = w * 0.007874016;
    for v in 0..=127u32 {
        let x = ((v as f32 * step).floor() + 0.5).min(w - 0.5);
        p.line(x, 0.0, x, h, tick, 1.0);
    }
    for v in [0u32, 32, 64, 96, 127] {
        let x = ((v as f32 * step).floor() + 0.5).min(w - 0.5);
        let label = v.to_string();
        let (tw, _) = p.fonts.measure(&font, &label);
        let bw = tw.ceil() + 3.0;
        let bx = if v == 127 { w - bw } else { x + 0.5 };
        p.fill_rect(RectF::new(bx, 1.0, bw, h - 2.0), knock);
        let align = if v == 127 { HAlign::Right } else { HAlign::Left };
        p.text_fx(RectF::new(bx + 1.0, 0.0, bw - 2.0, h), &label, text, &font, align, VAlign::Center, r.text_fx(c));
    }
}
pub const OVERLOAD: &str = "VelocitySplitEditor";

/// Split → handle x positions.
pub fn to_px(s: Split, w: f32) -> [f32; 4] {
    let k = (w - HANDLE) * 0.007874016;
    s.map(|v| v as f32 * k)
}

/// Handle x positions → split.
pub fn from_px(px: [f32; 4], w: f32) -> Split {
    let k = 1.0 / (w - HANDLE);
    px.map(|x| ((x * k * 127.0 + 0.5) as i32).clamp(0, 127) as u32)
}

/// Handle under (x, y): 0 lowest, 1 low turn, 2 high turn, 3 highest.
pub fn hit(px: [f32; 4], h: f32, x: f32, y: f32) -> Option<usize> {
    let ys = [h - HANDLE, 0.0, 0.0, h - HANDLE];
    (0..4).find(|&k| (ys[k]..ys[k] + HANDLE).contains(&y) && (px[k]..px[k] + HANDLE).contains(&x))
}

/// Mouse move [re:64a9e4]: move `handle` (or everything) by `dx` pixels.
pub fn drag(px: &mut [f32; 4], handle: Option<usize>, dx: f32, w: f32) {
    let max = w - HANDLE;
    let [p0, p1, p2, p3] = px;
    match handle {
        Some(0) => {
            *p0 += dx;
            *p1 += dx;
            if *p1 >= *p2 {
                *p1 = *p2 - 1.0;
            }
            if *p0 < 0.0 {
                *p0 = 0.0;
                *p1 = 0.0;
            }
            if *p0 >= *p2 {
                *p0 = *p2 - 1.0;
            }
            if *p0 >= *p3 {
                *p0 = *p3 - 1.0;
            }
            if *p1 <= *p0 {
                *p1 = *p0;
            }
        }
        Some(1) => {
            *p1 = (*p1 + dx).max(0.0);
            if *p1 >= *p2 {
                *p1 = *p2 - 1.0;
            }
            if *p1 >= *p3 {
                *p1 = *p3 - 1.0;
            }
            if *p1 < *p0 {
                *p1 = *p0;
            }
        }
        Some(2) => {
            *p2 = (*p2 + dx).min(max);
            if *p2 > *p3 {
                *p2 = *p3;
            }
            if *p2 <= *p1 {
                *p2 = *p1 + 1.0;
            }
        }
        Some(3) => {
            *p2 += dx;
            *p3 += dx;
            if *p3 > max {
                let over = *p3 - max;
                *p2 -= (over + 0.5).trunc();
                *p3 = max;
            }
            if *p2 <= *p1 {
                *p2 = *p1 + 1.0;
            }
            if *p2 > *p3 {
                *p2 = *p3;
            }
            if *p3 <= *p1 {
                *p3 = *p1 + 1.0;
            }
            if *p3 <= *p0 {
                *p3 = *p0 + 1.0;
            }
        }
        Some(_) => {}
        None => {
            for p in px.iter_mut() {
                *p += dx;
            }
            while px[0] < 0.0 {
                px.iter_mut().for_each(|p| *p += 1.0);
            }
            while px[3] > max && px[0] > 0.0 {
                px.iter_mut().for_each(|p| *p -= 1.0);
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Drag {
    layer: usize,
    handle: Option<usize>,
    last_x: i32,
    px: [f32; 4],
    w: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Menu {
    Mode,
}

#[derive(Default)]
pub struct VelSplit {
    /// The pad view (`PadControls`).
    panel: Option<Vec<usize>>,
    drag: Option<Drag>,
    menu: Option<Menu>,
}

fn layer_split(model: &Model, e: usize, p: usize, l: usize) -> Split {
    model.layer_node(e, p, l).map(dreist_engine::engine::doc::layer_split).unwrap_or(dreist_engine::velsplit::DEFAULT)
}

/// Pad layer mode index (`PadMode` attribute).
pub fn layer_mode(model: &Model, e: usize, p: usize) -> usize {
    let name = model.pad_node(e, p).and_then(|n| n.attr("PadMode")).unwrap_or("");
    dreist_dsp::voices::LAYER_MODE_NAMES.iter().position(|m| *m == name).unwrap_or(0)
}

fn is_v2a(name: &str) -> bool {
    matches!(name, "V2A" | "v2a")
}

impl VelSplit {
    /// The pad view's controls (its `Split0..7` lanes tagged by the page).
    pub fn build(panel: Option<Vec<usize>>) -> VelSplit {
        VelSplit { panel, ..VelSplit::default() }
    }

    pub fn sync(&mut self, app: &mut App, model: &Model) {
        let Some(pp) = self.panel.clone() else { return };
        let e = model.engine;
        let p = model.pad[e];
        let mode = layer_mode(model, e, p);
        let pad = model.pad_node(e, p);
        let v2a = pad.and_then(|n| n.f32(dreist_formats::PadParam::V2A.name())).unwrap_or(dreist_formats::PadParam::V2A.default_value());
        let pad_locked = pad.is_some_and(|n| n.locked());
        let enabled: Vec<bool> = (0..LAYERS)
            .map(|l| {
                let f = model.layer(e, p, l);
                f.valid && !pad_locked && !model.layer_node(e, p, l).is_some_and(|n| n.locked())
            })
            .collect();
        let split = mode == 1;
        let Some(panel) = app.ctrl_mut(&pp) else { return };
        let label = format!("Engine {} Pad {} velocity split", e + 1, p + 1);
        panel.walk_mut(&mut |c: &mut Ctrl| match c.name.as_str() {
            "SelModeCombo" => c.text = dreist_dsp::voices::LAYER_MODE_NAMES[mode].to_string(),
            n if is_v2a(n) => c.value = v2a,
            "Auto" | "Insert" => c.enabled = split && !pad_locked,
            // The velocity ruler over the lanes.
            "CtrlImageView" => c.visible = split,
            _ => {
                if let Some(l) = c.attr("layer").and_then(|v| v.parse::<usize>().ok()).filter(|_| c.overload == OVERLOAD) {
                    c.visible = split;
                    c.enabled = split && enabled[l.min(LAYERS - 1)];
                    c.display_name = label.clone();
                }
            }
        });
    }

    /// [re:fe83c5] [re:b8fa20]: a double click on Vel-Amp resets it
    /// ([re:b8fa20]).
    pub fn reset_value(&self, path: &[usize], name: &str) -> Option<f32> {
        (is_v2a(name) && self.panel.as_ref().is_some_and(|pp| path.starts_with(pp))).then(|| dreist_formats::PadParam::V2A.default_value())
    }

    pub fn handle(&mut self, app: &mut App, model: &mut Model, ev: &UiEvent, out: &mut Vec<Command>) -> bool {
        let e = model.engine;
        let p = model.pad[e];
        let ep = [e as i32, p as i32];
        let in_panel = |path: &[usize]| self.panel.as_ref().is_some_and(|pp| path.starts_with(pp));
        match ev {
            UiEvent::Clicked { path, name, .. } if in_panel(path) && name == "SelModeCombo" => {
                let items = dreist_dsp::voices::LAYER_MODE_NAMES.iter().map(|s| s.to_string()).collect();
                app.open_menu(path, items, Some(layer_mode(model, e, p)));
                self.menu = Some(Menu::Mode);
                true
            }
            // [re:ab11a3]: Auto splits the pad's layers at once.
            UiEvent::Clicked { path, name, .. } if in_panel(path) && matches!(name.as_str(), "Auto" | "Insert") => {
                out.push(msg(id::AUTO_VELOCITY_SPLIT_PAD, &ep, &[]));
                true
            }
            UiEvent::ValueChanged { path, name, value, .. } if in_panel(path) && is_v2a(name) => {
                out.push(msg(id::SET_PAD_PARAMETER, &ep, &[*value, dreist_formats::PadParam::V2A as u8 as f32, 0.0]));
                true
            }
            UiEvent::MenuSelected { anchor, index, .. } if self.menu.is_some() && in_panel(anchor) => {
                match self.menu.take() {
                    Some(Menu::Mode) => out.push(msg(id::SET_PAD_LAYER_MODE, &ep, &[(*index).min(4) as f32])),
                    None => {}
                }
                true
            }
            UiEvent::Mouse { path, overload, kind, x, y } if overload == OVERLOAD => {
                let Some((l, w, h)) = app.ctrl(path).and_then(|c| Some((c.attr("layer")?.parse::<usize>().ok()?, c.rect.w as f32, c.rect.h as f32))) else { return false };
                let xi = (*x as i32).clamp(0, w as i32);
                match kind {
                    MouseKind::Down => {
                        let px = to_px(layer_split(model, e, p, l), w);
                        self.drag = Some(Drag { layer: l, handle: hit(px, h, *x, *y), last_x: xi, px, w });
                        true
                    }
                    MouseKind::Drag => {
                        let Some(d) = self.drag.as_mut().filter(|d| d.layer == l) else { return false };
                        drag(&mut d.px, d.handle, (xi - d.last_x) as f32, d.w);
                        d.last_x = xi;
                        true
                    }
                    MouseKind::Up => {
                        let Some(d) = self.drag.take() else { return false };
                        let s = from_px(d.px, d.w);
                        let idx = [e as i32, p as i32, d.layer as i32];
                        out.push(msg(id::SET_LAYER_VELOCITY_SPLIT_MIN, &idx, &[s[0] as f32]));
                        out.push(msg(id::SET_LAYER_VELOCITY_SPLIT_MIN_TURNOVER, &idx, &[s[1] as f32]));
                        out.push(msg(id::SET_LAYER_VELOCITY_SPLIT_MAX_TURNOVER, &idx, &[s[2] as f32]));
                        out.push(msg(id::SET_LAYER_VELOCITY_SPLIT_MAX, &idx, &[s[3] as f32]));
                        true
                    }
                    MouseKind::RightDown => false,
                }
            }
            _ => false,
        }
    }

    /// [re:55d929]: the four
    /// velocities (`%03d`), "N: low | low fade | high fade | high", or the
    /// dragged handle's "N: Low velocity 012" and so on.
    pub fn tooltip(&self, model: &Model, c: &Ctrl) -> Option<String> {
        if c.overload != OVERLOAD {
            return None;
        }
        let l: usize = c.attr("layer")?.parse().ok()?;
        let e = model.engine;
        let p = model.pad[e];
        let (s, handle) = match self.drag.filter(|d| d.layer == l) {
            Some(d) => (from_px(d.px, d.w), d.handle),
            None => (layer_split(model, e, p, l), None),
        };
        let v = s.map(|x| format!("{x:03}"));
        let n = &c.display_name;
        Some(match handle {
            Some(0) => format!("{n}: Low velocity {}", v[0]),
            Some(1) => format!("{n}: Low fade velocity {}", v[1]),
            Some(2) => format!("{n}: High fade velocity {}", v[2]),
            Some(3) => format!("{n}: High velocity {}", v[3]),
            _ => format!("{n}: {} | {} | {} | {}", v[0], v[1], v[2], v[3]),
        })
    }

    pub fn draw(&self, model: &Model, r: &Renderer, p: &mut Painter, c: &Ctrl) -> bool {
        if c.overload == RULER {
            draw_ruler(r, p, c);
            return true;
        }
        if c.overload != OVERLOAD {
            return false;
        }
        let Some(l) = c.attr("layer").and_then(|v| v.parse::<usize>().ok()) else { return false };
        let (w, h) = (c.rect.w as f32, c.rect.h as f32);
        let st = &r.styles;
        // `VSplitStyle`: the split's area, the dragged handle, tick lines
        // (every eighth colAltBorder), the handles.
        let fill = st.colour(STYLE, "colFill", Colour::WHITE);
        let alt = st.colour(STYLE, "colAltFill", Colour::WHITE);
        let border = st.colour(STYLE, "colBorder", Colour::rgba(0x5a, 0x5a, 0x5a, 40));
        let alt_border = st.colour(STYLE, "colAltBorder", Colour::rgba(0x5a, 0x5a, 0x5a, 10));
        let step = w * 0.007874016;
        for i in 1..127u32 {
            let x = (i as f32 * step).floor() + 0.5;
            p.line(x, 0.0, x, h, if i % 8 == 0 { alt_border } else { border }, 1.0);
        }
        p.stroke_rect(RectF::new(0.5, 0.5, w - 1.0, h - 1.0), border, 1.0);
        if !c.enabled {
            return true;
        }
        let e = model.engine;
        let pad = model.pad[e];
        let (px, grabbed) = match self.drag.filter(|d| d.layer == l) {
            Some(d) => (d.px, d.handle),
            None => (to_px(layer_split(model, e, pad, l), w), None),
        };
        let poly = [(px[0], h), (px[1], 0.0), (px[2] + HANDLE, 0.0), (px[3] + HANDLE, h)];
        p.fill_polygon(&poly, fill.with_alpha(0.4));
        let mut closed = poly.to_vec();
        closed.push(poly[0]);
        p.polyline(&closed, fill, 1.0);
        let normal = st.colour(STYLE, "colHandle", alt);
        let ys = [h - HANDLE, 0.0, 0.0, h - HANDLE];
        for k in 0..4 {
            p.fill_rect(RectF::new(px[k], ys[k], HANDLE, HANDLE), if grabbed == Some(k) { alt } else { normal });
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn px_round_trip() {
        crate::app::test_setup();
        let w = 670.0;
        for v in [[0, 0, 127, 127], [10, 20, 90, 100], [43, 43, 85, 85]] {
            assert_eq!(from_px(to_px(v, w), w), v);
        }
    }

    #[test]
    fn handle_drags_clamp() {
        crate::app::test_setup();
        let w = 135.0; // 1 px per velocity.
        let mut px = to_px([10, 20, 90, 100], w);
        drag(&mut px, Some(0), -30.0, w);
        assert_eq!(from_px(px, w), [0, 0, 90, 100]);
        let mut px = to_px([10, 20, 90, 100], w);
        drag(&mut px, Some(1), 200.0, w);
        assert_eq!(from_px(px, w), [10, 89, 90, 100]);
        let mut px = to_px([10, 20, 90, 100], w);
        drag(&mut px, Some(2), -200.0, w);
        assert_eq!(from_px(px, w), [10, 20, 21, 100]);
        let mut px = to_px([10, 20, 90, 100], w);
        drag(&mut px, Some(3), 50.0, w);
        assert_eq!(from_px(px, w), [10, 20, 117, 127]);
        let mut px = to_px([10, 20, 90, 100], w);
        drag(&mut px, None, 50.0, w);
        assert_eq!(from_px(px, w), [37, 47, 117, 127]);
        assert_eq!(hit(to_px([10, 20, 90, 100], w), 33.0, 12.0, 30.0), Some(0));
        assert_eq!(hit(to_px([10, 20, 90, 100], w), 33.0, 92.0, 2.0), Some(2));
        assert_eq!(hit(to_px([10, 20, 90, 100], w), 33.0, 50.0, 15.0), None);
    }
}
