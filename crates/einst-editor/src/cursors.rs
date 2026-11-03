//! Pointer feedback: hover and drag cursors (`VMouse::SetCursor` callers).
//!
//! - Hover: the layer wave display's markers and the zoom sliders' thumb
//!   edges show the left / right cursor (`WaveDisplay::OnMouseMove`
//!   0x3cfe50, `WaveDisplayZoomSlider::OnMouseMove` 0x3e4040); elsewhere
//!   the arrow (the app resets it on every move).
//! - Pattern grid (multi tool, `SetCursorFromToolOnMouseMove` 0x28b4d0):
//!   NotAllowed / Selection / ResizeRightThin / Multi.
//! - Song body (`GetCursorForOperation` 0x568f0): the cursor of the
//!   operation a press would start.
//! - Drag: the drop target's `DragMove` sets the source's drag cursor
//!   (`ICtrlDragSource::GetDragCursor`). The original renders pad and
//!   pattern ghosts (`EditorPads::MakeDragCursors` 0x1d8670: the pad at 65 %
//!   over an arrow with a swap / copy / move sign); the port uses the
//!   matching images from `Resources/Cursors` — DragPad / DragPadCopy,
//!   DragPattern / DragPatternCopy, DragAudio / DragFile / DragFiles /
//!   DragFolder for browser items — and NotAllowed over a place that takes
//!   nothing.

use geist_gui::app::UiEvent;
use geist_gui::cursor::Cursor;

use crate::Editor;

impl Editor {
    /// Hover / drag-over handling; true when something visible changed.
    pub(crate) fn pointer_feedback(&mut self, ev: &UiEvent) -> bool {
        let before = self.app.cursor.clone();
        match ev {
            UiEvent::Hover { path, overload, x, y } => {
                let c = self
                    .inspector
                    .hover_cursor(&self.app, &self.model, path, overload, *x, *y)
                    .or_else(|| self.slicer.hover_cursor(&self.app, path, overload, *x, *y))
                    .or_else(|| self.pattern.as_ref().and_then(|p| p.hover_cursor(&self.app, &self.model, path, overload, *x, *y)))
                    .or_else(|| self.song.as_ref().and_then(|s| s.hover_cursor(&self.app, &self.model, path, overload, *x, *y)))
                    .or_else(|| self.midimap.as_ref().and_then(|m| m.hover_cursor(overload, *x, *y)));
                if let Some(c) = c {
                    self.app.cursor = c;
                }
            }
            UiEvent::DragOver { from, to, x, y } => {
                self.app.cursor = self.drag_cursor(from, to);
                self.pad_drag_flash(&from.clone(), &to.clone(), *x, *y);
                self.drop_highlights(&from.clone(), &to.clone());
            }
            _ => {}
        }
        before != self.app.cursor
    }

    pub(crate) fn drag_cursor(&self, from: &[usize], to: &[usize]) -> Cursor {
        let files = |items: &[std::path::PathBuf]| match items {
            [] => Cursor::Arrow,
            [one] if one.is_dir() => Cursor::image("DragFolder"),
            [one] if crate::search::is_audio(&one.extension().map(|e| e.to_string_lossy().into_owned()).unwrap_or_default()) => Cursor::image("DragAudio"),
            [_] => Cursor::image("DragFile"),
            _ => Cursor::image("DragFiles"),
        };
        if from.is_empty() {
            let Some(items) = self.external_drag.as_ref() else { return Cursor::Arrow };
            return if self.accepts_drop(from, to) { files(items) } else { Cursor::NotAllowed };
        }
        let Some(c) = self.app.ctrl(from) else { return Cursor::Arrow };
        let alt = self.app.mods.alt;
        let source = match c.overload.as_str() {
            "EditorPad" => Cursor::image(if alt { "DragPadCopy" } else { "DragPad" }),
            "PatternSelector" => Cursor::image(if alt { "DragPatternCopy" } else { "DragPattern" }),
            "BrowserFileList" => {
                let Some(items) = self.browser.as_ref().map(|b| b.drag_items()) else { return Cursor::Arrow };
                match items.as_slice() {
                    [] => return Cursor::Arrow,
                    [one] if one.is_dir() => Cursor::image("DragFolder"),
                    [one] if crate::search::is_audio(&one.extension().map(|e| e.to_string_lossy().into_owned()).unwrap_or_default()) => Cursor::image("DragAudio"),
                    [_] => Cursor::image("DragFile"),
                    _ => Cursor::image("DragFiles"),
                }
            }
            // Knob / slider drags and the like keep their cursor.
            _ => return self.app.cursor.clone(),
        };
        if self.accepts_drop(from, to) { source } else { Cursor::NotAllowed }
    }
}
