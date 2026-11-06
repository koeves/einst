//! Keyboard shortcuts ([re:a191fd], ev_16eeb3a7…, ev_ce262c09…,
//! ev_68554242…).
//!
//! [re:9db09e] hands each key to
//! [re:9454a7], which looks the key up in
//! the accelerator table (the menus' key equivalents) and
//! runs the entry's function [re:df170c]:
//! - nothing happens unless option `RespondToKeyboard` (default on) is set;
//! - an entry matches when the key matches and either both sides have no
//!   modifiers or the modifier sets are equal;
//! - entries without modifiers only respond with `RespondToExtendedKeyboard`
//!   (default off), except the browser arrows 0x24–0x27.
//!
//! The standalone app also carries the table as its menu bar's key
//! equivalents ([re:c60fcb]; [re:ed265e] runs
//! the function regardless of the options), so there every entry works.

/// Modifier set of an entry (shift, command, option, control).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Mods {
    pub shift: bool,
    pub cmd: bool,
    pub alt: bool,
    pub ctrl: bool,
}

impl Mods {
    pub fn any(&self) -> bool {
        self.shift || self.cmd || self.alt || self.ctrl
    }
}

/// What an entry's function does (54 functions).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// Editor message `id`.
    Editor(u32),
    /// Engine message `id` with the flag flipped, for a transport toggle
    /// (the header button with that message).
    Toggle(&'static str),
    /// Engine message `id` for a transport trigger (the header button).
    Press(&'static str),
    /// Loop: an empty function.
    Nothing,
    /// View menu: show main page `n` (v1's `e_updateSelectView`).
    View(usize),
}

pub struct Accel {
    pub id: u32,
    /// Key as `App::shortcut` names it (a lowercase character, " ",
    /// "left" / "right" / "up" / "down", "enter").
    pub key: &'static str,
    pub mods: Mods,
    /// Menu item title (`Construct*Menu`).
    pub name: &'static str,
    pub action: Action,
}

const N: Mods = Mods { shift: false, cmd: false, alt: false, ctrl: false };
const C: Mods = Mods { shift: false, cmd: true, alt: false, ctrl: false };
const SC: Mods = Mods { shift: true, cmd: true, alt: false, ctrl: false };
const CA: Mods = Mods { shift: false, cmd: true, alt: true, ctrl: false };
const S: Mods = Mods { shift: true, cmd: false, alt: false, ctrl: false };
const A: Mods = Mods { shift: false, cmd: false, alt: true, ctrl: false };

const fn a(id: u32, key: &'static str, mods: Mods, name: &'static str, action: Action) -> Accel {
    Accel { id, key, mods, name, action }
}

/// The entries with a key (menu items without one are left out). Editor
/// messages: e_updateOpenPrefs 0x16a, e_updateOpenPreset / Engine / Kit
/// 0x16c–0x16e, e_updateSavePreset 0x174, e_updateSavePresetAs 0x176,
/// e_updateSaveKit / As 0x178 / 0x179, e_updateUndo / Redo 0x18d / 0x18e,
/// e_updateDelete 0x18f, e_updateBrowserStartFilter / StartFind 0x160 /
/// 0x161, e_grabRetroRecording 0x23, browser 0x155–0x15b, panels
/// 0x146–0x14e, e_resetUISizeToDefault / Scale 0x152 / 0x153. Synth:
/// e_togglePlayback 0x15 (![re:64b7fd]), e_toggleRecord 0x14,
/// e_toggleMetronome 0x18, e_toggleQuantize 0x19,
/// e_songSetActive 0x117 (!active, 0), e_returnToZero 0x1d, e_panic 0x12.
pub const TABLE: &[Accel] = &[
    // v1's menus and key equivalents ([re:689471] [re:4accac] [re:8e7307] [re:ee7b06] [re:42a8c6] [re:33b96d]); functions as v1's
    // [re:ed265e] (ids are v1's).
    a(0x00, ",", C, "Preferences", Action::Editor(0x16a)),
    a(0x01, "o", C, "Open Preset", Action::Editor(0x16c)),
    a(0x02, "o", CA, "Open Engine", Action::Editor(0x16d)),
    a(0x03, "o", SC, "Open Kit", Action::Editor(0x16e)),
    a(0x04, "s", C, "Save Preset", Action::Editor(0x174)),
    a(0x05, "s", CA, "Save Engine", Action::Editor(0x175)),
    a(0x06, "s", SC, "Save Kit", Action::Editor(0x178)),
    a(0x08, "z", C, "Undo", Action::Editor(0x18d)),
    a(0x09, "z", SC, "Redo", Action::Editor(0x18e)),
    a(0x0b, "f", C, "Find", Action::Editor(0x161)),
    a(0x0a, "f", SC, "Filter Browser List", Action::Editor(0x160)),
    a(0x0c, " ", N, "Play", Action::Toggle("PlayButton")),
    a(0x0d, "r", N, "Record", Action::Toggle("RecButton")),
    a(0x0e, "r", A, "Retro Record", Action::Editor(0x23)),
    a(0x0f, "m", N, "Metronome", Action::Toggle("MetroButton")),
    a(0x10, "k", N, "Input Quantize", Action::Toggle("QuantizeButton")),
    a(0x11, "s", N, "Song mode", Action::Toggle("SongButton")),
    a(0x12, "0", N, "Return to zero", Action::Press("RestartButton")),
    a(0x13, "down", N, "Next File", Action::Editor(0x155)),
    a(0x14, "up", N, "Previous File", Action::Editor(0x156)),
    a(0x15, "right", N, "Next Location", Action::Editor(0x157)),
    a(0x16, "left", N, "Previous Location", Action::Editor(0x158)),
    a(0x17, "enter", N, "Enter Folder", Action::Editor(0x15a)),
    a(0x18, "left", S, "Parent Folder", Action::Editor(0x15b)),
    a(0x19, "r", SC, "Refresh", Action::Editor(0x159)),
    a(0x1a, "1", N, "Pattern", Action::View(crate::front::PATTERN_PAGE)),
    a(0x1b, "2", N, "Scenes", Action::View(crate::front::SCENE_PAGE)),
    a(0x1c, "3", N, "Song", Action::View(crate::front::SONG_PAGE)),
    a(0x1d, "4", N, "Pads/Layers", Action::View(crate::front::PADS_PAGE)),
    a(0x1e, "5", N, "Layers Mixer", Action::View(crate::front::LAYER_MIXER_PAGE)),
    a(0x1f, "6", N, "Pad Mixer", Action::View(crate::front::PAD_MIXER_PAGE)),
    a(0x20, "7", N, "Engine Mixer", Action::View(crate::front::ENGINE_MIXER_PAGE)),
    a(0x21, "8", N, "Global Mixer", Action::View(crate::front::GLOBAL_MIXER_PAGE)),
    a(0x22, "9", N, "Sampler", Action::View(crate::front::SAMPLER_PAGE)),
];

/// Menu items without a key (File menu): id, title, editor
/// message (functions 4–8, 11, 12, 15–20).
pub const UNKEYED: &[(u32, &str, u32)] = &[];

/// A menu bar entry.
pub enum Menu {
    Item(u32),
    Separator,
}

use Menu::{Item as I, Separator as Sep};

/// The standalone's menus (File, Edit, Browser, Transport, View;
/// Preferences sits in the app menu).
pub const MENUS: &[(&str, &[Menu])] = &[
    ("File", &[I(0x01), I(0x02), I(0x03), Sep, I(0x04), I(0x05), I(0x06)]),
    ("Edit", &[I(0x08), I(0x09), Sep, I(0x0b), I(0x0a)]),
    ("Browser", &[I(0x13), I(0x14), I(0x15), I(0x16), I(0x17), I(0x18), I(0x19)]),
    ("Transport", &[I(0x0c), I(0x0d), I(0x0e), I(0x0f), I(0x10), I(0x11), I(0x12)]),
    ("View", &[I(0x1a), I(0x1b), I(0x1c), I(0x1d), I(0x1e), I(0x1f), I(0x20), I(0x21), I(0x22)]),
];

/// An entry's title and function (keyed or not).
pub fn entry(id: u32) -> Option<(&'static str, Action)> {
    TABLE.iter().find(|e| e.id == id).map(|e| (e.name, e.action)).or_else(|| UNKEYED.iter().find(|u| u.0 == id).map(|u| (u.1, Action::Editor(u.2))))
}

/// The entry for `key` with `mods`, if it responds.
/// `menu_bar`: the standalone's menu key equivalents (no option gating).
pub fn lookup(key: &str, mods: Mods, keyboard: bool, extended: bool, menu_bar: bool) -> Option<&'static Accel> {
    let e = TABLE.iter().find(|e| e.key == key && e.mods == mods)?;
    if menu_bar {
        return Some(e);
    }
    if !keyboard {
        return None;
    }
    (e.mods.any() || extended || (0x13..=0x16).contains(&e.id)).then_some(e)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gating() {
        crate::app::test_setup();
        // Space plays from the menu bar; in a plugin only with extended keys.
        assert_eq!(lookup(" ", N, true, false, true).map(|e| e.id), Some(0x0c));
        assert!(lookup(" ", N, true, false, false).is_none());
        assert_eq!(lookup(" ", N, true, true, false).map(|e| e.id), Some(0x0c));
        // Modified keys need only RespondToKeyboard; the browser arrows
        // always respond.
        assert_eq!(lookup("z", SC, true, false, false).map(|e| e.id), Some(0x09));
        assert!(lookup("z", SC, false, false, false).is_none());
        assert_eq!(lookup("down", N, true, false, false).map(|e| e.id), Some(0x13));
        assert!(lookup("enter", N, true, false, false).is_none());
        // Modifiers must match exactly.
        assert!(lookup("z", CA, true, true, true).is_none());
        assert_eq!(lookup("left", S, true, false, true).map(|e| e.id), Some(0x18));
        // v1: Retro Record is alt+R.
        assert_eq!(lookup("r", A, true, false, false).map(|e| e.id), Some(0x0e));
    }
}
