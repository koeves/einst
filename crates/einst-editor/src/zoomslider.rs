//! [re:55b154] ([re:55b154]..) and the [re:a31f0f] zoom logic
//! shared by the layer wave display: the view is (start, size) as fractions
//! of the file. See the dreist-re GUI notes (Zoom slider, Wave display zoom).

use dreist_gui::colour::Colour;
use dreist_gui::paint::{Painter, RectF};
use dreist_gui::style::StyleSheet;

pub use dreist_editor_core::zoomslider::*;

/// [re:bcee78]: the thumb in colFill, 2 px edge
/// lines and 10 px discs (white there; colAltFill) with 6 px centres
/// (black; colBorder), over the track (colTrack).
pub fn draw(p: &mut Painter, s: &StyleSheet, style: &str, w: f32, h: f32, view: (f32, f32)) {
    p.fill_rect(RectF::new(0.0, 0.0, w, h), s.colour(style, "colTrack", Colour::TRANSPARENT));
    let (tx, tw) = thumb(view, w);
    p.fill_rect(RectF::new(tx, 0.0, tw, h), s.colour(style, "colFill", Colour::rgba(255, 192, 203, 255)));
    let (edge, centre) = (s.colour(style, "colAltFill", Colour::WHITE), s.colour(style, "colBorder", Colour::BLACK));
    for ex in [tx, tx + tw] {
        p.fill_rect(RectF::new(ex - 1.0, 0.0, 2.0, h), edge);
    }
    let cy = (h as i32 / 2) as f32;
    for ex in [tx, tx + tw] {
        p.fill_ellipse(RectF::new(ex - 5.0, cy - 5.0, 10.0, 10.0), edge);
        p.fill_ellipse(RectF::new(ex - 3.0, cy - 3.0, 6.0, 6.0), centre);
    }
}

/// [re:079fd4]: the wave displays' drag signs (zoom, scroll). Dragging down
/// widens the view and dragging right scrolls right; the preferences
/// `GuiDragDownToZoom` / `GuiDragLeftToScrollRight` flip them.
pub fn drag_signs() -> (f32, f32) {
    let o = crate::options::Options::load();
    let s = |k: &str| if o.bool(k, false) { -1.0 } else { 1.0 };
    (s("GuiDragDownToZoom"), s("GuiDragLeftToScrollRight"))
}
