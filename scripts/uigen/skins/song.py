"""The song page: lanes, track headers and bodies, timeline (Song/)."""

from . import N, skin


@skin("Song/SongPanel.skin")
def song_panel():
    return [
        N("Panel", "CtrlPanel", (30, 22, 815, 395), id=0, style="EmptyPanelStyle", kids=[
            N("Panel", None, (2, 2, 811, 38), id=0, style="EmptyPanelStyle", kids=[
                N("Label", "TempTextLabel", (762, -7, 50, 38), id=100000, style="BaseLabelStyle", text=""),
                N("Button", "ShuffleButton", (109, -20, 80, 20), displayName="Shuffle Edit Mode", id=100028,
                  overloadClass="FlashingButton", text="Shuffle Edit"),
                N("ButtonGroup", "ToolButtonGroup", (3, 6, 105, 32), id=100000, style="BasePanelStyle", kids=[
                    N("Button", "CtrlButton", (2, 8, 19, 17), displayName="Draw Tool", icon="pencil", id=1,
                      optIsMomentary=False, overloadClass="IconButton", style="SongDrawButtonStyle", text=""),
                    N("Button", "CtrlButton", (38, 8, 19, 17), displayName="Erase Tool", icon="eraser", id=2,
                      optIsMomentary=False, overloadClass="IconButton", style="SongEraseButtonStyle", text=""),
                    N("Button", "CtrlButton", (59, 8, 19, 17), displayName="Select Tool", icon="select", id=0,
                      optIsMomentary=False, overloadClass="IconButton", style="SongSelectButtonStyle", text=""),
                    N("Button", "CtrlButton", (80, 8, 19, 17), displayName="Multi Tool", icon="pencil_plus", id=3,
                      optIsMomentary=False, overloadClass="IconButton", style="SongMultiButtonStyle", text=""),
                ]),
                N("Button", "SetDrawPatternButton", (25, 14, 12, 17), displayName="Change selected Parts to previous Pattern",
                  id=100028, optIsMomentary=True, overloadClass="FlashingButton", style="SongPenPopupButtonStyle", text="\ue902"),
                N("Button", "FollowButton", (150, 14, 50, 17), displayName="Scroll to Follow", id=100028, optIsMomentary=False,
                  overloadClass="FlashingButton", style="LEDLatchingButtonStyle", text="Follow"),
                N("Button", "PrevPatternButton", (263, 14, 15, 17), displayName="Change selected Parts to previous Pattern",
                  icon="minus", id=100028, optIsMomentary=True, overloadClass="IconButton", style="StandardSubtractButtonStyle",
                  text=""),
                N("Button", "SelectPatternButton", (277, 14, 50, 17), displayName="Change Pattern for selected Parts",
                  id=100028, optIsMomentary=True, overloadClass="FlashingButton", renderer="PopupButtonRenderer",
                  style="LEDMenuButtonStyle", text="Pattern"),
                N("Button", "NextPatternButton", (326, 14, 15, 17), displayName="Change selected Parts to next Pattern",
                  icon="plus", id=100028, optIsMomentary=True, overloadClass="IconButton", style="StandardAddButtonStyle",
                  text=""),
                N("Button", "EraseButton", (559, 14, 50, 17), displayName="Erase when Recording", id=100028,
                  optIsMomentary=True, overloadClass="FlashingButton", style="LEDMomentaryButtonStyle", text="Erase"),
                N("Button", "ClearAllButton", (611, 14, 50, 17), displayName="Clear Song", id=100028, optIsMomentary=True,
                  overloadClass="FlashingButton", style="LEDMomentaryButtonStyle", text="Clear All"),
                N("Button", "DuplicateButton", (663, 14, 50, 17), displayName="Duplicate selected Parts", id=100028,
                  optIsMomentary=True, overloadClass="FlashingButton", style="LEDMomentaryButtonStyle", text="Duplicate"),
            ]),
            N("Panel", "TrackHeaderPanel", (2, 41, 48, 340), id=100003, style="EmptyPanelStyle"),
            N("Panel", "TrackScrollView", (50, 41, 760, 340), id=100002, legacyClass="CtrlScrollView", style="BasePanelStyle"),
            N("TextBox", "PatternRenameTextBox", (167, 371, 151, 20), id=100000, optSingleClickEdit=True,
              style="SongPatternEditBoxStyle"),
            N("Button", "ZoomOutButton", (786, 380, 12, 12), id=100028, optIsMomentary=True, overloadClass="FlashingButton",
              style="ZoomOutButtonStyle", text="\ue91c"),
            N("Button", "ZoomInButton", (798, 380, 12, 12), id=100028, optIsMomentary=True, overloadClass="FlashingButton",
              style="ZoomInButtonStyle", text="\ue91d"),
            N("Button", "PartNextPatternButton", (87, 383, 10, 10), id=100000, optIsMomentary=True,
              style="SongPartNextButtonStyle", text="\ue903"),
            N("Button", "PartPrevPatternButton", (103, 383, 10, 10), id=100000, optIsMomentary=True,
              style="SongPartPrevButtonStyle", text="\ue901"),
            N("Button", "PartSelectPatternButton", (119, 384, 10, 10), id=100000, optIsMomentary=True,
              style="SongPartPopupButtonStyle", text="\ue905"),
            N("Panel", "BlackoutPanel", (50, 41, 760, 340), id=100001, style="SongBlackoutPanelStyle", zorder=2, kids=[
                N("Label", "CtrlLabel", (79, 38, 516, 264), id=100002, style="BlackoutTextStyle",
                  text="Click to activate Song Mode"),
                N("Button", "PowerButton", (480, 160, 50, 17), id=100000, optIsMomentary=True, style="LEDMomentaryButtonStyle",
                  text="Activate"),
            ]),
            N("ScrollBar", "HorizontalScrollBar", (52, 380, 734, 11), overloadClass="SongHScroll", style="SongScrollStyle"),
        ]),
    ]


@skin("Song/SongPatternPopupPanel.skin")
def song_pattern_popup_panel():
    return [
        N("Panel", "CtrlPanel", (0, 0, 339, 93), id=0, style="SongPatternPopupPanelStyle", kids=[
            N("Button", "Key1", (18, 6, 21, 36), id=100003, optIsMomentary=False, style="SongPatternSelectorStyleNotAllocated"),
            N("Button", "Key3", (41, 6, 21, 36), id=100004, optIsMomentary=False, style="SongPatternSelectorStyleNotAllocated"),
            N("Button", "Key6", (86, 6, 21, 36), id=100005, optIsMomentary=False, style="SongPatternSelectorStyleNotAllocated"),
            N("Button", "Key8", (109, 6, 21, 36), id=100006, optIsMomentary=False, style="SongPatternSelectorStyleNotAllocated"),
            N("Button", "Key10", (132, 6, 21, 36), id=100007, optIsMomentary=False, style="SongPatternSelectorStyleNotAllocated"),
            N("Button", "Key13", (179, 6, 21, 36), id=100008, optIsMomentary=False, style="SongPatternSelectorStyleNotAllocated"),
            N("Button", "Key15", (202, 6, 21, 36), id=100009, optIsMomentary=False, style="SongPatternSelectorStyleNotAllocated"),
            N("Button", "Key18", (246, 6, 21, 36), id=100010, optIsMomentary=False, style="SongPatternSelectorStyleNotAllocated"),
            N("Button", "Key20", (269, 6, 21, 36), id=100011, optIsMomentary=False, style="SongPatternSelectorStyleNotAllocated"),
            N("Button", "Key22", (292, 6, 21, 36), id=100012, optIsMomentary=False, style="SongPatternSelectorStyleNotAllocated"),
            N("Button", "Key0", (6, 46, 21, 36), id=100000, optIsMomentary=False, style="SongPatternSelectorStyleNotAllocated"),
            N("Button", "Key2", (29, 46, 21, 36), id=100001, optIsMomentary=False, style="SongPatternSelectorStyleNotAllocated"),
            N("Button", "Key4", (52, 46, 21, 36), id=100000, optIsMomentary=False, style="SongPatternSelectorStyleNotAllocated"),
            N("Button", "Key5", (75, 46, 21, 36), id=100001, optIsMomentary=False, style="SongPatternSelectorStyleNotAllocated"),
            N("Button", "Key7", (98, 46, 21, 36), id=100000, optIsMomentary=False, style="SongPatternSelectorStyleNotAllocated"),
            N("Button", "Key9", (121, 46, 21, 36), id=100001, optIsMomentary=False, style="SongPatternSelectorStyleNotAllocated"),
            N("Button", "Key11", (144, 46, 21, 36), id=100000, optIsMomentary=False,
              style="SongPatternSelectorStyleNotAllocated"),
            N("Button", "Key12", (167, 46, 21, 36), id=100001, optIsMomentary=False,
              style="SongPatternSelectorStyleNotAllocated"),
            N("Button", "Key14", (190, 46, 21, 36), id=100000, optIsMomentary=False,
              style="SongPatternSelectorStyleNotAllocated"),
            N("Button", "Key16", (213, 46, 21, 36), id=100001, optIsMomentary=False,
              style="SongPatternSelectorStyleNotAllocated"),
            N("Button", "Key17", (236, 46, 21, 36), id=100000, optIsMomentary=False,
              style="SongPatternSelectorStyleNotAllocated"),
            N("Button", "Key19", (259, 46, 21, 36), id=100001, optIsMomentary=False,
              style="SongPatternSelectorStyleNotAllocated"),
            N("Button", "Key21", (282, 46, 21, 36), id=100000, optIsMomentary=False,
              style="SongPatternSelectorStyleNotAllocated"),
            N("Button", "Key23", (305, 46, 21, 36), id=100001, optIsMomentary=False,
              style="SongPatternSelectorStyleNotAllocated"),
        ]),
    ]


@skin("Song/SongTimelinePanel.skin")
def song_timeline_panel():
    return [
        N("Panel", "CtrlPanel", (30, 22, 800, 20), id=0, style="EmptyPanelStyle"),
    ]


@skin("Song/SongTrackBodyPanel.skin")
def song_track_body_panel():
    return [
        N("Panel", "CtrlPanel", (30, 22, 800, 40), id=0, style="EmptyPanelStyle"),
    ]


@skin("Song/SongTrackHeaderPanel.skin")
def song_track_header_panel():
    return [
        N("Panel", "CtrlPanel", (30, 22, 48, 40), id=0, style="SongTrackPanelSelectedStyle", kids=[
            N("Label", "TrackNumberLabel", (9, 5, 36, 30), id=100000, style="StandardLargeLabelCenteredNotificationStyle",
              text=8),
        ]),
    ]
