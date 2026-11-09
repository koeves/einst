//! The browser's tree icons and file-type badges, drawn as vectors (v1
//! drew them from 20 × 20 bitmaps): a 16 × 16 design scaled to the rect's
//! smaller side and centred, in a style's colours: outlines colBorder,
//! faces colFill, dark parts (the book's cover) colAltFill, details and
//! the badge's type text colText (in the style's font).

use dreist_gui::Colour;
use dreist_gui::paint::{Painter, RectF};
use dreist_gui::render::Renderer;
use dreist_gui::style::{HAlign, VAlign};

/// The icon style, and the one for a selected row.
pub const STYLE: &str = "BrowserIconStyle";
pub const SELECTED_STYLE: &str = "BrowserIconSelectedStyle";

/// Icon kinds: `book` (a content library), `home`, `folder`, `desktop`,
/// `drive`, `disc`, `search`, `list` (a shortlist), `file` (a page with
/// the type `label`, the file-type badge).
pub fn draw(r: &Renderer, p: &mut Painter, style: &str, rect: RectF, kind: &str, label: &str) {
    let s = r.styles;
    let line = s.colour(style, "colBorder", Colour::BLACK);
    let face = s.colour(style, "colFill", Colour::WHITE);
    let dark = s.colour(style, "colAltFill", Colour::BLACK);
    let text = s.colour(style, "colText", Colour::BLACK);
    let k = rect.w.min(rect.h) / 16.0;
    let (ox, oy) = (rect.x + ((rect.w - 16.0 * k) / 2.0).floor(), rect.y + ((rect.h - 16.0 * k) / 2.0).floor());
    let rr = |x: f32, y: f32, w: f32, h: f32| RectF::new(ox + x * k, oy + y * k, w * k, h * k);
    let pt = |x: f32, y: f32| (ox + x * k, oy + y * k);
    match kind {
        "book" => {
            p.fill_rect(rr(2.0, 1.0, 12.0, 14.0), dark);
            p.fill_polygon(&[pt(3.0, 2.0), pt(8.0, 4.0), pt(8.0, 14.0), pt(3.0, 12.0)], face);
            p.fill_rect(rr(8.0, 2.0, 1.0, 12.0), line);
        }
        "home" => {
            p.fill_rect(rr(3.5, 7.0, 9.0, 7.5), face);
            p.stroke_rect(rr(3.5, 7.5, 9.0, 7.0), line, 1.0);
            p.fill_polygon(&[pt(1.0, 8.0), pt(8.0, 1.5), pt(15.0, 8.0)], line);
            p.fill_polygon(&[pt(3.5, 7.5), pt(8.0, 3.5), pt(12.5, 7.5)], face);
            p.fill_rect(rr(7.0, 10.0, 3.0, 4.5), line);
        }
        "folder" => {
            p.fill_polygon(&[pt(1.0, 4.0), pt(6.0, 4.0), pt(7.5, 5.5), pt(15.0, 5.5), pt(15.0, 14.0), pt(1.0, 14.0)], face);
            p.polyline(&[pt(1.5, 14.0), pt(1.5, 3.5), pt(6.0, 3.5), pt(7.5, 5.5), pt(14.5, 5.5), pt(14.5, 13.5), pt(1.5, 13.5)], line, 1.0);
            p.polyline(&[pt(1.5, 7.5), pt(14.5, 7.5)], line, 1.0);
        }
        "desktop" => {
            p.fill_rect(rr(1.5, 2.5, 13.0, 9.0), face);
            p.stroke_rect(rr(1.5, 2.5, 13.0, 9.0), line, 1.0);
            p.fill_rect(rr(6.0, 12.0, 4.0, 1.5), line);
            p.fill_rect(rr(4.0, 13.5, 8.0, 1.5), line);
        }
        "drive" => {
            p.fill_rounded(rr(3.0, 1.0, 10.0, 14.0), 2.0, [true; 4], face);
            p.stroke_rounded(rr(3.5, 1.5, 9.0, 13.0), 2.0, [true; 4], line, 1.0);
            p.fill_rect(rr(4.0, 10.0, 8.0, 1.0), line);
            p.fill_rect(rr(10.0, 12.0, 1.5, 1.5), text);
        }
        "disc" => {
            p.fill_ellipse(rr(1.5, 1.5, 13.0, 13.0), face);
            p.stroke_ellipse(rr(1.5, 1.5, 13.0, 13.0), line, 1.0);
            p.stroke_ellipse(rr(5.0, 5.0, 6.0, 6.0), line, 1.0);
        }
        "search" => {
            p.fill_ellipse(rr(1.5, 1.5, 10.0, 10.0), face);
            p.stroke_ellipse(rr(1.5, 1.5, 10.0, 10.0), line, 1.5);
            p.polyline(&[pt(10.0, 10.0), pt(14.5, 14.5)], line, 2.0);
        }
        "list" => {
            p.fill_rect(rr(2.0, 1.5, 12.0, 13.0), face);
            p.stroke_rect(rr(2.5, 2.0, 11.0, 12.5), line, 1.0);
            for y in [5.0, 8.0, 11.0] {
                p.fill_rect(rr(4.5, y, 7.0, 1.0), text);
            }
        }
        "file" => {
            // A page with a turned corner and the type label (the badge).
            p.fill_polygon(&[pt(2.0, 1.0), pt(11.0, 1.0), pt(14.0, 4.0), pt(14.0, 15.0), pt(2.0, 15.0)], face);
            p.polyline(&[pt(11.0, 1.5), pt(2.5, 1.5), pt(2.5, 14.5), pt(13.5, 14.5), pt(13.5, 4.0), pt(11.0, 1.5), pt(11.0, 4.0), pt(13.5, 4.0)], line, 1.0);
            let font = s.font(style);
            p.text(rr(2.5, 8.0, 11.0, 6.5), label, text, &font, HAlign::Center, VAlign::Center);
        }
        _ => {}
    }
}

/// The badge text of a listed file: its type, by extension.
pub fn badge(ext: &str) -> String {
    use dreist_formats::profile::{DocKind, doc_kind};
    let e = ext.to_ascii_lowercase();
    match doc_kind(&e) {
        Some(DocKind::Project) => return "PST".into(),
        Some(DocKind::Kit) => return "KIT".into(),
        Some(DocKind::Engine) => return "ENG".into(),
        Some(DocKind::Pad) => return "PAD".into(),
        Some(DocKind::Pattern) => return "PTN".into(),
        Some(DocKind::Bank) => return "BNK".into(),
        _ => {}
    }
    match e.as_str() {
        "wav" => "WAV",
        "aif" | "aiff" => "AIF",
        "rx2" | "rex" | "rcy" | "rex2" => "REX",
        "patterntemplate" => "PTN",
        "kit" => "KIT",
        "mid" | "midi" => "MID",
        _ => return e.chars().take(3).collect::<String>().to_ascii_uppercase(),
    }
    .into()
}

/// The overload of the browser header / footer buttons whose v1 art is a
/// picture (drawn over the button's face by [`draw_button`]).
pub const BUTTON_OVERLOAD: &str = "BrowserButtonIcon";

/// A browser header / footer button's icon after v1's bitmaps (measured,
/// own drawing), centred in the control, in its style's text colour
/// (colAltText while latched): EnterButton / ParentButton an arrow into /
/// out of a small folder, PreviewButton a play triangle in a ring,
/// LockPadsButton / LockPatternsButton a padlock with a pad grid / step
/// rows, SearchOptionsButton a magnifier with a small down triangle, the
/// preview volume icon a rising wedge.
pub fn draw_button(r: &Renderer, p: &mut Painter, c: &dreist_gui::Ctrl) {
    let s = r.styles;
    let ink = s.colour(&c.style, if c.on { "colAltText" } else { "colText" }, Colour::BLACK);
    let face = s.colour(STYLE, "colFill", Colour::WHITE);
    let (w, h) = (c.rect.w as f32, c.rect.h as f32);
    let (cx, cy) = ((w / 2.0).floor(), (h / 2.0).floor());
    let padlock = |p: &mut Painter, x: f32| {
        p.stroke_rounded(RectF::new(x + 1.5, cy - 5.5, 5.0, 6.0), 2.5, [true, true, false, false], ink, 1.5);
        p.fill_rounded(RectF::new(x, cy - 1.0, 8.0, 7.0), 1.0, [true; 4], ink);
    };
    match c.name.as_str() {
        "EnterButton" | "ParentButton" => {
            let (fx, fy) = (cx - 1.0, cy - 4.0);
            p.fill_rect(RectF::new(fx, fy + 1.0, 10.0, 8.0), face);
            p.polyline(&[(fx + 0.5, fy + 9.5), (fx + 0.5, fy + 0.5), (fx + 4.0, fy + 0.5), (fx + 5.0, fy + 1.5), (fx + 10.5, fy + 1.5), (fx + 10.5, fy + 9.5), (fx + 0.5, fy + 9.5)], ink, 1.0);
            if c.name == "EnterButton" {
                p.fill_ellipse(RectF::new(cx - 11.0, cy - 1.5, 3.0, 3.0), ink);
                p.line(cx - 8.0, cy, cx + 3.0, cy, ink, 2.0);
                p.fill_polygon(&[(cx + 1.0, cy - 3.5), (cx + 5.0, cy), (cx + 1.0, cy + 3.5)], ink);
            } else {
                p.fill_ellipse(RectF::new(cx + 2.0, cy - 1.5, 3.0, 3.0), ink);
                p.line(cx - 7.0, cy, cx + 3.0, cy, ink, 2.0);
                p.fill_polygon(&[(cx - 6.0, cy - 3.5), (cx - 10.0, cy), (cx - 6.0, cy + 3.5)], ink);
            }
        }
        "PreviewButton" => {
            p.stroke_ellipse(RectF::new(cx - 7.0, cy - 7.0, 14.0, 14.0), ink, 2.0);
            p.fill_polygon(&[(cx - 2.0, cy - 4.0), (cx + 4.0, cy), (cx - 2.0, cy + 4.0)], ink);
        }
        "LockPadsButton" => {
            padlock(p, cx - 13.0);
            for k in 0..16 {
                let (i, j) = ((k % 4) as f32, (k / 4) as f32);
                p.fill_rect(RectF::new(cx - 1.0 + i * 3.5, cy - 6.0 + j * 3.5, 2.0, 2.0), ink);
            }
        }
        "LockPatternsButton" => {
            padlock(p, cx - 13.0);
            for j in 0..3 {
                for i in 0..4 {
                    p.fill_rect(RectF::new(cx - 1.0 + i as f32 * 3.5, cy - 4.0 + j as f32 * 4.0, 2.5, 2.0), ink);
                }
            }
        }
        "SearchOptionsButton" => {
            p.stroke_ellipse(RectF::new(1.0, 1.0, 6.0, 6.0), ink, 1.5);
            p.line(6.0, 6.0, 9.0, 9.5, ink, 2.0);
            p.fill_polygon(&[(9.0, 2.0), (14.0, 2.0), (11.5, 5.0)], ink);
        }
        _ => {
            // The preview volume icon.
            p.fill_polygon(&[(0.0, h), (w, h), (w, 0.0)], ink);
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn badges() {
        crate::app::test_setup();
        assert_eq!(super::badge("WAV"), "WAV");
        assert_eq!(super::badge("aiff"), "AIF");
        assert_eq!(super::badge("rx2"), "REX");
        assert_eq!(super::badge("gru"), "GRU");
    }
}
