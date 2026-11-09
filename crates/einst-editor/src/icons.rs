//! Button icons drawn as vectors over a button's face (`IconButton`
//! controls; v1 drew them in its button bitmaps): the song and pattern
//! tools' pencil, eraser, dashed frame and pencil with a plus.
//!
//! The control's `icon` attribute names the icon and `iconSize` its height
//! in px. It is drawn centred in the style's text colour (colAltText while
//! the button is on); holes (the pencil's light stripe) take the face colour (colFill / colAltFill).

use dreist_gui::paint::{Painter, RectF};
use dreist_gui::render::Renderer;
use dreist_gui::{Colour, Ctrl};

/// Overload class of the controls drawing an icon.
pub const OVERLOAD: &str = "IconButton";

/// The icon of an `IconButton`, over its face (`CustomDraw::draw_over`).
pub fn draw_button(r: &Renderer, p: &mut Painter, c: &Ctrl) {
    let s = r.styles;
    let st = r.style(c);
    let (ink, face) =
        if c.on { (s.colour(st, "colAltText", Colour::WHITE), s.colour(st, "colAltFill", Colour::BLACK)) } else { (s.colour(st, "colText", Colour::BLACK), s.colour(st, "colFill", Colour::WHITE)) };
    let size = c.attr("iconSize").and_then(|v| v.trim().parse::<f32>().ok());
    draw(p, c.attr("icon").unwrap_or(""), RectF::new(0.0, 0.0, c.rect.w as f32, c.rect.h as f32), size, ink, face);
}

/// Icon `name` centred in `r`, `size` px high (each icon has its own
/// default), in `ink` with holes in `face`.
pub fn draw(p: &mut Painter, name: &str, r: RectF, size: Option<f32>, ink: Colour, face: Colour) {
    // The design box (w, h) and default height of each icon.
    let (dw, dh, default) = match name {
        "pencil" | "eraser" => (13.0, 13.0, 13.0),
        "pencil_plus" => (14.0, 13.0, 13.0),
        "select" => (13.0, 10.0, 10.0),
        _ => return,
    };
    let k = size.unwrap_or(default) / dh;
    let (ox, oy) = ((r.x + (r.w - dw * k) / 2.0).round(), (r.y + (r.h - dh * k) / 2.0).round());
    let pt = |x: f32, y: f32| (ox + x * k, oy + y * k);
    let rr = |x: f32, y: f32, w: f32, h: f32| RectF::new(ox + x * k, oy + y * k, w * k, h * k);
    match name {
        "pencil" => pencil(p, pt(0.5, 12.5), 16.0 * k, k, ink, face),
        "pencil_plus" => {
            pencil(p, pt(0.5, 10.5), 13.0 * k, k, ink, face);
            p.fill_rect(rr(8.5, 9.0, 5.5, 1.5), ink);
            p.fill_rect(rr(10.5, 7.0, 1.5, 5.5), ink);
        }
        // (a block tilted like the pencil: the upper half filled)
        "eraser" => {
            let (c, u, n) = ((6.5, 6.5), (0.7071, -0.7071), (0.7071, 0.7071));
            let at = |a: f32, b: f32| pt(c.0 + u.0 * a + n.0 * b, c.1 + u.1 * a + n.1 * b);
            p.fill_polygon(&[at(0.0, -3.0), at(6.0, -3.0), at(6.0, 3.0), at(0.0, 3.0)], ink);
            p.polyline(&[at(-6.0, -3.0), at(6.0, -3.0), at(6.0, 3.0), at(-6.0, 3.0), at(-6.0, -3.0)], ink, 1.3 * k);
        }
        // (a dashed frame: 2 px dashes, 1 px gaps)
        "select" => {
            let t = 1.3 * k;
            let mut x = 0.0;
            while x < 13.0 {
                let l = 2.0f32.min(13.0 - x);
                p.fill_rect(RectF::new(ox + x * k, oy, l * k, t), ink);
                p.fill_rect(RectF::new(ox + x * k, oy + 10.0 * k - t, l * k, t), ink);
                x += 3.0;
            }
            let mut y = 0.0;
            while y < 10.0 {
                let l = 2.0f32.min(10.0 - y);
                p.fill_rect(RectF::new(ox, oy + y * k, t, l * k), ink);
                p.fill_rect(RectF::new(ox + 13.0 * k - t, oy + y * k, t, l * k), ink);
                y += 3.0;
            }
        }
        _ => {}
    }
}

/// A pencil from its tip at `tip` up to the right, `len` px long: a dark
/// body with a light stripe, the point and a band before the end.
fn pencil(p: &mut Painter, tip: (f32, f32), len: f32, k: f32, ink: Colour, face: Colour) {
    let (u, n) = ((0.7071, -0.7071), (0.7071, 0.7071));
    let at = |a: f32, b: f32| (tip.0 + u.0 * a + n.0 * b, tip.1 + u.1 * a + n.1 * b);
    let (hw, point) = (1.9 * k, 4.0 * k);
    p.fill_polygon(&[at(0.0, 0.0), at(point, hw), at(len, hw), at(len, -hw), at(point, -hw)], ink);
    let (a, b) = (at(point + 0.8 * k, 0.0), at(len - 3.0 * k, 0.0));
    p.line(a.0, a.1, b.0, b.1, face, 0.9 * k);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every icon draws inside its rect.
    #[test]
    fn icons_draw_inside_their_rect() {
        crate::app::test_setup();
        let ink = Colour::rgba(18, 25, 60, 255);
        let face = Colour::rgba(222, 229, 237, 255);
        for name in ["pencil", "pencil_plus", "eraser", "select"] {
            let mut pm = dreist_gui::tiny_skia::Pixmap::new(40, 40).unwrap();
            let mut fonts = dreist_gui::text::Fonts::new(std::path::Path::new("."));
            let mut p = Painter::new(&mut pm, &mut fonts, 1.0);
            draw(&mut p, name, RectF::new(5.0, 5.0, 30.0, 30.0), None, ink, face);
            drop(p);
            let (mut inked, mut outside) = (0, 0);
            for y in 0..40 {
                for x in 0..40 {
                    let px = pm.pixel(x, y).unwrap();
                    if px.alpha() > 0 {
                        inked += 1;
                        if !(4..36).contains(&x) || !(4..36).contains(&y) {
                            outside += 1;
                        }
                    }
                }
            }
            assert!(inked > 4, "{name} draws nothing");
            assert_eq!(outside, 0, "{name} draws outside its rect");
        }
    }
}
