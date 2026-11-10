"""The browser's header, footer and search panels (Browser/)."""

from . import N, skin


@skin("Browser/Footer.skin")
def footer():
    return [
        N("Panel", "ChooserPanel", (0, 0, 295, 25), id=0, style="EmptyPanelStyle", kids=[
            N("Button", "StopButton", (44, 1, 23, 24), displayName="Stop Preview", id=100001, optIsMomentary=True,
              style="StopButtonStyle", text="\ue904"),
            N("Button", "PreviewButton", (68, 1, 23, 24), displayName="Preview audio files when clicked in browser", id=100000,
              optIsMomentary=False, style="AutoButtonStyle", text=""),
            N("Button", "LockPadsButton", (104, 1, 35, 24), displayName="Lock / Unlock all pads", id=100001,
              optIsMomentary=True, style="LockPadsStyle", text=""),
            N("Button", "LockPatternsButton", (140, 1, 35, 24), displayName="Lock / Unlock all patterns", id=100002,
              optIsMomentary=True, style="LockPatternStyle", text=""),
            N("Button", "HitLoopModeButton", (188, 4, 50, 17), displayName="Slice audio files when clicked in browser",
              id=100007, optIsMomentary=False, style="LEDLatchingButtonStyle", text="Slice"),
            N("Button", "InteractionModeButton", (240, 4, 50, 17), displayName="Auto allocate files when clicked in browser",
              id=100004, optIsMomentary=False, style="LEDLatchingButtonStyle", text="Auto load"),
            N("Knob", "PreviewVolumeKnob", (2, 4, 17, 17), displayName="Preview volume", id=100005, realDialThickness=0.05,
              style="LEDKnobMicroLR"),
            N("Label", "CtrlImageView", (22, 7, 12, 10), id=100000, legacyClass="CtrlImageView", style="GlyphLabelStyle",
              text=""),
        ]),
    ]


@skin("Browser/Header.skin")
def header():
    return [
        N("Panel", "ChooserPanel", (0, 0, 295, 25), id=0, style="EmptyPanelStyle", kids=[
            N("Panel", "NavLocationImage", (2, 1, 56, 24), id=100000, legacyClass="CtrlImageView",
              style="NavigationBackgroundImageStyle"),
            N("Button", "PrevLocnButton", (3, 2, 14, 21), displayName="Return to the previous folder", id=100000,
              optIsMomentary=True, style="BrowserPrevLocationButtonStyle", text="\ue901"),
            N("Button", "PrevFileButton", (18, 2, 24, 10), displayName="Move to the previous file", id=100000,
              optIsMomentary=True, style="BrowserPrevFileButtonStyle", text="\ue900"),
            N("Button", "NextFileButton", (18, 13, 24, 10), displayName="Move to the next file", id=100000, optIsMomentary=True,
              style="BrowserNextFileButtonStyle", text="\ue902"),
            N("Button", "NextLocnButton", (43, 2, 14, 21), displayName="Move to the next folder", id=100000,
              optIsMomentary=True, style="BrowserNextLocationButtonStyle", text="\ue903"),
            N("Button", "EnterButton", (60, 1, 23, 24), displayName="Enter the selected folder", id=100000, optIsMomentary=True,
              style="EnterButtonStyle", text=""),
            N("Button", "ParentButton", (84, 1, 23, 24), displayName="Navigate to the parent of the current directory",
              id=100000, optIsMomentary=True, style="BrowserParentButtonStyle", text=""),
            N("Button", "RefreshButton", (108, 1, 23, 24), displayName="Refresh current file list", id=100000,
              optIsMomentary=True, style="RefreshButtonStyle", text="\ue912"),
            N("Knob", "Spinner", (138, 2, 20, 20), id=100001, realDialThickness=0.13, realSpinnerArrow=0.36, realSpinnerInset=1,
              style="SpinnerStyle"),
            N("Panel", "CtrlImageView", (161, 1, 126, 24), id=100006, legacyClass="CtrlImageView", style="StupidImageStyle"),
            N("SearchBox", "SearchBox", (180, 3, 105, 18), id=0, legacyClass="CtrlSearchTextBox", optSingleClickEdit=True,
              style="SearchBoxStyle"),
            N("Button", "SearchOptionsButton", (165, 7, 14, 11), displayName="Alter search settings", id=0, optIsMomentary=True,
              style="SearchOptionsStyle", text=""),
        ]),
    ]


@skin("Browser/SearchOptionItem.skin")
def search_option_item():
    return [
        N("Panel", "ChooserPanel", (0, 0, 291, 31), id=0, style="EmptyPanelStyle", kids=[
            N("PopupButton", "SearchType", (1, 7, 75, 17), displayName="Click here to set the type of search", id=100004,
              legacyClass="CtrlComboBox", optSingleClickEdit=True, style="LEDComboBoxStyle"),
            N("PopupButton", "SubSearchType", (83, 7, 75, 17), displayName="Click here to set the sub type of search",
              id=100004, legacyClass="CtrlComboBox", optSingleClickEdit=True, style="LEDComboBoxStyle"),
            N("TextBox", "SearchTermBox", (164, 7, 125, 17), displayName="Click here to set the term to search for", id=100002,
              optSingleClickEdit=True, style="LEDEditBoxStyle"),
        ]),
    ]


@skin("Browser/SearchPane.skin")
def search_pane():
    return [
        N("Panel", "CtrlPanel", (0, 0, 295, 125), id=0, style="EmptyPanelStyle", kids=[
            N("Panel", "OptionsPlaceHolder", (2, 2, 291, 95), id=100005, style="EmptyPanelStyle"),
            N("Knob", "Spinner", (215, 98, 20, 20), id=100000, realDialThickness=0.13, realSpinnerArrow=0.36,
              realSpinnerInset=1, style="SpinnerStyle"),
            N("Button", "DoSearchButton", (239, 99, 50, 17), id=100003, optIsMomentary=True, style="LEDMomentaryButtonStyle",
              text="Search"),
            N("Button", "MatchAllButton", (5, 101, 14, 15), id=100005, optIsMomentary=False, renderer="CheckBoxButtonRenderer",
              style="StandardCheckButtonStyle", text=""),
            N("Label", "CtrlLabel", (23, 103, 80, 15), id=100006, style="StandardLabelStyle", text="Match all options"),
        ]),
    ]
