//! MIDI learn ([re:99f0ce]..): the header LearnButton toggles
//! learn mode ([re:f00b25]).
//!
//! - In learn mode every learnable control gets corner brackets a quarter of
//!   its size in `LearnTargetColour`, its binding text in `LearnTextColour`,
//!   and the current target a 2 px outline ([re:22b95d]).
//! - A left click picks the target instead of operating the control
//!   ([re:be689e]) and sends `e_setTargetLearnBinding`
//!   (0xd) with the control's binding ([re:c89236]); the
//!   next MIDI CC is bound to it.
//! - A right click does nothing but on the mapping table, as v1
//!   ([re:318e14]); host parameters are bound from the Host parameters panel.
//!
//! Port: a control's binding prototype is the message its normal handler
//! posts (captured by running the event and discarding the result); the
//! value slot carrying the control value becomes the stuffer. Binding text
//! is known for bindings made in this session.

use std::collections::HashMap;

use dreist_engine::binding::Message;
use dreist_gui::app::UiEvent;
use dreist_gui::{Ctrl, Rect};

#[derive(Clone, Debug)]
pub struct Target {
    pub path: Vec<usize>,
    pub proto: Message,
    pub stuffer: usize,
    pub name: String,
    /// Binding `Path` ("Global", or e.g. "Engine 1 | Pattern 1 | Track 1").
    pub group: String,
    /// Binding type: Continuous for values, Latching / Trigger for buttons.
    pub kind: &'static str,
}

#[derive(Default)]
pub struct Learn {
    pub active: bool,
    pub target: Option<Target>,
    /// Binding text per control path.
    pub labels: HashMap<Vec<usize>, String>,
    pub last_cc: u32,
    /// Absolute rects of learnable controls (filled while active).
    pub rects: Vec<(Rect, Option<String>, bool)>,
}

pub fn is_control_event(ev: &UiEvent) -> bool {
    matches!(ev, UiEvent::Clicked { .. } | UiEvent::ValueChanged { .. })
}

pub fn event_path(ev: &UiEvent) -> Option<&[usize]> {
    match ev {
        UiEvent::Clicked { path, .. } | UiEvent::ValueChanged { path, .. } | UiEvent::RightClicked { path, .. } | UiEvent::Pressed { path, .. } => Some(path),
        _ => None,
    }
}

/// The value slot of `m` carrying `v`.
pub fn stuffer_for(m: &Message, v: f32) -> usize {
    (0..3).find(|&i| (m.value[i] - v).abs() < 1e-6).unwrap_or(0)
}

pub fn learnable(c: &Ctrl) -> bool {
    // Plus the value boxes / popups with an editor binding (pattern length,
    // step length).
    let editor = matches!(c.class.as_str(), "CtrlTextBox" | "CtrlPopupButton") && crate::editbind::TABLE.iter().any(|b| b.control == c.name);
    (matches!(c.class.as_str(), "CtrlKnob" | "CtrlTransmodKnob" | "CtrlSlider" | "CtrlTransmodSlider" | "CtrlPanSlider" | "CtrlButton" | "CtrlTempoBox") || editor)
        && c.enabled
        && c.visible
        && !c.passive
}

impl Learn {
    /// Collect the learnable controls' absolute rects.
    pub fn collect(&mut self, root: &Ctrl) {
        fn rec(c: &Ctrl, ox: i32, oy: i32, path: &mut Vec<usize>, out: &mut Vec<(Rect, Vec<usize>)>) {
            if !c.visible {
                return;
            }
            for (i, k) in c.children.iter().enumerate() {
                let (x, y) = (ox + k.rect.x, oy + k.rect.y);
                path.push(i);
                if learnable(k) && k.name != "LearnButton" {
                    out.push((Rect::new(x, y, k.rect.w, k.rect.h), path.clone()));
                }
                rec(k, x, y, path, out);
                path.pop();
            }
        }
        let mut v = Vec::new();
        rec(root, root.rect.x, root.rect.y, &mut Vec::new(), &mut v);
        let target = self.target.as_ref().map(|t| t.path.clone());
        self.rects = v.into_iter().map(|(r, p)| (r, self.labels.get(&p).cloned(), target.as_ref() == Some(&p))).collect();
    }
}
