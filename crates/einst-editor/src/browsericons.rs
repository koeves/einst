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
