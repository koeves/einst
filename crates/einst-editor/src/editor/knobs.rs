//! Knob editing: the value edit box and the double-click reset. v1's knobs
//! have no right-click menu ([re:1d6fd1]: the click goes to the panel).

use crate::*;

/// The typed knob edit box ([re:c8b896]).
pub(crate) const KNOB_EDIT: &str = "KnobValueEdit";

/// The unit a typed edit shows for a conversion.
fn curve_suffix(c: dreist_formats::params::Curve) -> &'static str {
    use dreist_formats::params::Curve;
    match c {
        Curve::Decibels => " dB",
        Curve::Cutoff => " Hz",
        Curve::Stretch => " %",
        Curve::Linear(lo, hi) if lo == 0.0 && hi == 100.0 => " %",
        _ => "",
    }
}

impl Editor {
    /// A knob's reset value from the panel that owns it.
    pub(crate) fn knob_reset_value(&self, path: &[usize], name: &str) -> Option<f32> {
        self.inspector
            .reset_value(path, name)
            .or_else(|| self.mixers.iter().find_map(|m| m.reset_value(&self.app, path)))
            .or_else(|| self.transmod.reset_value(&self.app, &self.model, path, name))
            .or_else(|| self.pattern.as_ref().and_then(|p| p.reset_value(path, name)))
    }

    /// Set a knob as if moved to `v` (base value: transmod source 0).
    pub(crate) fn knob_set(&mut self, path: &[usize], name: &str, id: i32, v: f32) {
        if let Some(c) = self.app.ctrl_mut(path) {
            c.value = v;
        }
        let source = std::mem::replace(&mut self.model.source, 0);
        self.handle(&UiEvent::ValueChanged { path: path.to_vec(), name: name.to_string(), id, value: v });
        self.model.source = source;
        self.sync();
        self.app.dirty = true;
    }

    /// [re:c8b896]: a text box over the control with
    /// its value in its units; the typed value sets it.
    pub(crate) fn start_knob_edit(&mut self, path: &[usize], name: &str, id: i32) {
        use dreist_formats::params::Curve;
        let Some(c) = self.app.ctrl(path).cloned() else { return };
        let curve = self.inspector.edit_curve(path, name).or_else(|| self.mixers.iter().find_map(|m| m.edit_curve(&self.app, path))).unwrap_or(if name == header::MASTER_GAIN {
            Curve::Decibels
        } else {
            Curve::Linear(0.0, 100.0)
        });
        let u = curve.to_units(c.value);
        let text = match curve {
            Curve::Decibels => format!("{:.1} dB", u.max(-100.0)),
            Curve::Cutoff => format!("{u:.0} Hz"),
            _ => format!("{}{}", (u * 100.0).round() / 100.0, curve_suffix(curve)),
        };
        let rect = dreist_gui::Rect::new(c.rect.x + c.rect.w / 2 - 35, c.rect.y + c.rect.h / 2 - 9, 70, 18);
        self.app.open_edit_box(&path[..path.len() - 1], KNOB_EDIT, rect, text);
        self.knob_edit = Some((path.to_vec(), name.to_string(), id, curve));
        self.app.dirty = true;
    }

    /// Knob / slider double clicks with `DoubleClickResets` (default on):
    /// the control goes to its default and reports it as a move (the
    /// original sets the value and notifies the listeners, or posts the default
    /// directly), always the base value (transmod source 0). Off, the
    /// original opens a typed edit ([re:c8b896]), as
    /// the knob menu's "Edit" does.
    pub(crate) fn double_click_reset(&mut self, ev: &UiEvent) -> bool {
        let UiEvent::DoubleClicked { path, name } = ev else { return false };
        let Some(c) = self.app.ctrl(path) else { return false };
        if !matches!(c.class.as_str(), "CtrlKnob" | "CtrlTransmodKnob" | "CtrlSlider" | "CtrlTransmodSlider" | "CtrlPanSlider") || name == header::MASTER_GAIN {
            return false;
        }
        let id = c.id;
        let Some(v) = self.knob_reset_value(path, name) else { return false };
        if !options::Options::load().bool("DoubleClickResets", true) {
            self.start_knob_edit(path, name, id);
            return true;
        }
        self.knob_set(path, name, id, v);
        true
    }
}
