"""The main window (RootSkin.skin)."""

from . import N, skin


@skin("RootSkin.skin")
def root_skin():
    return [
        N("Panel", "BackPanel", (0, 0, 1135, 640), id=0, style="RootPanelStyle", kids=[
            N("Panel", "BrowsePanel", (0, 0, 308, 640), id=0, style="EmptyPanelStyle", kids=[
                N("Panel", "BackBrowser", (2, 3, 304, 633), id=0, legacyClass="CtrlImageView", style="ScreenBezelStyle", kids=[
                    N("Panel", None, (3, 20, 298, 607), style="ScreenLcdStyle"),
                    N("Panel", None, (3, 20, 298, 20), style="WellShadowTopStyle"),
                    N("Panel", None, (3, 20, 18, 607), style="WellShadowLeftStyle"),
                ]),
                N("Label", "Logo", (5, 6, 40, 17), id=0, legacyClass="CtrlImageView", style="WordmarkStyle", text="einst."),
                N("ButtonGroup", "BrowserTabButtons", (140, 4, 160, 19), id=0, style="BasePanelStyle", kids=[
                    N("Button", None, (0, 0, 78, 19), id=0, optIsMomentary=False, style="BrowserTabStyle", text="BROWSER"),
                    N("Button", None, (80, 0, 78, 19), id=0, optIsMomentary=False, style="MappingTabStyle", text="MAPPING"),
                ]),
                N("Label", "FPSLabel", (72, 5, 46, 20), id=100000, style="StandardFXLabelCenteredStyle", text=""),
                N("Panel", "BrowserTabs", (7, 23, 295, 608), id=0, overloadClass="TabbedPanel"),
            ]),
            N("Panel", "MovePanel", (306, 0, 829, 640), id=0, style="EmptyPanelStyle", kids=[
                N("Panel", None, (1, 423, 826, 215), style="ChassisFrontStyle"),
                N("Panel", None, (2, 428, 283, 11), style="ChassisBarStyle"),
                N("Panel", None, (290, 428, 392, 11), style="ChassisBarMidStyle"),
                N("Panel", None, (688, 428, 138, 11), style="ChassisBarRightStyle"),
                N("Panel", None, (290, 500, 392, 11), style="ChassisBarMidStyle"),
                N("Panel", None, (2, 443, 58, 58), style="ChassisFrameStyle"),
                N("Panel", None, (2, 504, 58, 56), style="ChassisWellStyle"),
                N("Panel", None, (688, 437, 138, 160), style="ChassisFrameStyle"),
                N("Panel", None, (421, 602, 316, 34), style="ChassisBackingStyle"),
                N("Panel", "BackMain", (1, 3, 827, 420), id=0, legacyClass="CtrlImageView", style="ScreenBezelStyle", kids=[
                    N("Panel", None, (3, 20, 821, 397), style="ScreenLcdStyle"),
                    N("Panel", None, (3, 20, 821, 18), style="ScreenShadeTopStyle"),
                    N("Panel", None, (3, 20, 8, 397), style="ScreenShadeLeftStyle"),
                ]),
                N("Panel", "BackEng", (288, 442, 396, 57), id=0, legacyClass="CtrlImageView", style="StatusStripStyle"),
                N("Label", "EngText", (633, 429, 45, 9), id=0, legacyClass="CtrlImageView", style="SectionLabelStyle",
                  text="ENGINES"),
                N("Label", "PatText", (626, 501, 54, 9), id=0, legacyClass="CtrlImageView", style="SectionLabelStyle",
                  text="PATTERNS"),
                N("Label", "MasterText", (780, 429, 42, 9), id=0, legacyClass="CtrlImageView", style="SectionLabelStyle",
                  text="MASTER"),
                N("Label", "PadText", (256, 429, 27, 9), id=0, legacyClass="CtrlImageView", style="SectionLabelStyle",
                  text="PADS"),
                N("Label", "ModesText", (17, 429, 37, 9), id=0, legacyClass="CtrlImageView", style="SectionLabelLeftStyle",
                  text="MODES"),
                N("ButtonGroup", "MainTabButtons", (32, 4, 795, 19), id=0, style="BasePanelStyle", kids=[
                    N("Button", None, (0, 0, 78, 19), id=0, optIsMomentary=False, style="PatternTabStyle", text="PATTERN"),
                    N("Button", None, (80, 0, 78, 19), id=0, optIsMomentary=False, style="SceneTabStyle", text="SCENE"),
                    N("Button", None, (160, 0, 78, 19), id=0, optIsMomentary=False, style="SongTabStyle", text="SONG"),
                    N("Button", None, (260, 0, 78, 19), id=0, optIsMomentary=False, style="PadsTabStyle", text="PAD / LAYERS"),
                    N("Button", None, (366, 0, 78, 19), id=0, optIsMomentary=False, style="LMixerTabStyle", text="LAYER MIXER"),
                    N("Button", None, (446, 0, 78, 19), id=0, optIsMomentary=False, style="PMixerTabStyle", text="PAD MIXER"),
                    N("Button", None, (526, 0, 78, 19), id=0, optIsMomentary=False, style="EMixerTabStyle", text="ENGINE MIXER"),
                    N("Button", None, (615, 0, 78, 19), id=0, optIsMomentary=False, style="GMixerTabStyle", text="GLOBAL MIXER"),
                    N("Button", None, (715, 0, 78, 19), id=0, optIsMomentary=False, style="SamplerTabStyle", text="SAMPLER"),
                ]),
                N("Button", "ShowHideButton", (8, 4, 19, 19), id=100000, optIsMomentary=False, style="ExposeButtonStyle",
                  text="\ue903"),
                N("Panel", "MainTabs", (5, 23, 820, 395), id=0, overloadClass="TabbedPanel"),
                N("Panel", "PadModePanel", (2, 437, 70, 200), id=0, overloadClass="PadModeControlsPanel"),
                N("Panel", "EditorEngineSelectors", (291, 442, 393, 56), id=0, overloadClass="EditorEngineSelectors"),
                N("Panel", "EditorPads", (65, 442, 224, 194), id=0, overloadClass="EditorPads"),
                N("Panel", "PatternSelectors", (344, 511, 340, 85), id=0, overloadClass="PatternSelectors"),
                N("Panel", "EngineControls", (290, 498, 57, 100), id=0, overloadClass="EngineControls"),
                N("Panel", "MasterControls", (690, 440, 135, 156), id=0, overloadClass="MasterControlsPanel"),
                N("Panel", "UtilityControls", (690, 447, 135, 12), id=100002, overloadClass="UtilityControlsPanel"),
                N("Panel", "Transport", (295, 600, 532, 39), id=0, overloadClass="Transport"),
            ]),
        ]),
    ]
