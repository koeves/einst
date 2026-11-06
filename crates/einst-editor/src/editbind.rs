//! Editor-only bindings (`Binding` `EditorOnly`): controls whose MIDI
//! / host binding posts a message back to the editor instead of the engine.
//! v1 makes them for the transport's Undo / Redo ([re:a54ccb]) and the
//! browser's header and footer buttons ([re:c8c512], [re:dc44d8]); the
//! scene triggers bind to the engine ([re:8b4856], `learning`).
//!
//! The editor acts on their messages: browser Auto / Slice / Preview mode
//! 0x15c–0x15e set the toggle from the value (≥ 0.5); next / previous file
//! 0x155 / 0x156, next / previous location 0x157 / 0x158, refresh 0x159,
//! enter 0x15a, parent 0x15b press the button; undo / redo 0x18d / 0x18e.
//!
//! v2 only (v1 has no binding for them; their messages are still handled):
//! the pad view toggles, the pattern tool bar, the page tabs, the mixer
//! modes and the scene snapshot ([re:59ad1d], [re:b2db08], [re:23be19],
//! [re:69fc1c]).

/// One editor binding: the control (`name` inside a container named
/// `scope`), its binding path / name / type and message.
pub struct EditorBinding {
    pub scope: &'static str,
    pub control: &'static str,
    pub group: &'static str,
    pub name: &'static str,
    pub kind: &'static str,
    pub id: u32,
}

const fn b(scope: &'static str, control: &'static str, group: &'static str, name: &'static str, kind: &'static str, id: u32) -> EditorBinding {
    EditorBinding { scope, control, group, name, kind, id }
}

pub const TABLE: &[EditorBinding] = &[
    b("Transport", "UndoButton", "Global", "Undo", "Trigger", 0x18d),
    b("Transport", "RedoButton", "Global", "Redo", "Trigger", 0x18e),
    b("BrowserTabs", "InteractionModeButton", "Browser", "Auto Mode", "Trigger", 0x15c),
    b("BrowserTabs", "HitLoopModeButton", "Browser", "Slice Mode", "Trigger", 0x15d),
    b("BrowserTabs", "PreviewButton", "Browser", "Preview", "Trigger", 0x15e),
    // [re:c8c512]: v1's lock buttons (engine messages there; here the
    // editor locks the pads / patterns).
    b("BrowserTabs", "LockPadsButton", "Browser", "Lock all pads", "Trigger", 0x84),
    b("BrowserTabs", "LockPatternsButton", "Browser", "Lock all patterns", "Trigger", 0x85),
    b("BrowserTabs", "NextFileButton", "Browser", "Next file", "Trigger", 0x155),
    b("BrowserTabs", "PrevFileButton", "Browser", "Previous file", "Trigger", 0x156),
    b("BrowserTabs", "NextLocnButton", "Browser", "Next location", "Trigger", 0x157),
    b("BrowserTabs", "PrevLocnButton", "Browser", "Previous location", "Trigger", 0x158),
    b("BrowserTabs", "RefreshButton", "Browser", "Refresh", "Trigger", 0x159),
    b("BrowserTabs", "EnterButton", "Browser", "Enter location", "Trigger", 0x15a),
    b("BrowserTabs", "ParentButton", "Browser", "Parent location", "Trigger", 0x15b),
];

/// LCD page buttons ([re:23be19]: button id → panel; the pages the
/// original's panel messages name).
pub const EDITOR_TABS: [(usize, &str, u32); 6] = [(0, "Pattern", 0x146), (1, "Scenes", 0x147), (2, "Song", 0x148), (4, "Layer Mixer", 0x14b), (5, "Pad Mixer", 0x14a), (7, "Global Mixer", 0x149)];
/// Browser column buttons.
pub const UTILITY_TABS: [(usize, &str, u32); 2] = [(0, "Browser", 0x14e), (1, "Mapping", 0x14c)];
/// Mixer ModeGroup children ([re:69fc1c]).
pub const MIXER_MODES: [(&str, u32); 3] = [("Mix", 0x14f), ("Sends", 0x150), ("Effects", 0x151)];

pub fn by_id(id: u32) -> Option<&'static EditorBinding> {
    TABLE.iter().find(|e| e.id == id)
}
