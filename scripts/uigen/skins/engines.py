"""The engine controls and selectors (Engines/)."""

from . import N, skin


@skin("Engines/EngineControls.skin")
def engine_controls():
    return [
        N("Panel", "CtrlPanel", (0, 0, 57, 100), id=0, style="EmptyPanelStyle", kids=[
            N("Panel", None, (-1, 14, 55, 55), style="HardKnobHaloStyle"),
            N("Panel", None, (5, 46, 43, 34), style="HardKnobShadowStyle"),
            N("Panel", None, (4, 19, 45, 45), style="HardKnobRingStyle"),
            N("Panel", None, (4, 63, 3, 3), style="HardKnobMarkStyle"),
            N("Panel", None, (46, 63, 3, 3), style="HardKnobMarkStyle"),
            N("Knob", "EngineSwingDepthKnob", (5, 20, 43, 43), displayName="Swing Template Depth", id=100010,
              realDialThickness=0.047, style="HardKnobLargeLR"),
            N("Label", "SwingIcon", (22, 80, 13, 14), id=0, legacyClass="CtrlImageView", style="SwingIconImageStyle",
              text="\ue90e"),
        ]),
    ]


@skin("Engines/EngineSelector.skin")
def engine_selector():
    return [
        N("Panel", "CtrlPanel", (0, 0, 49, 56), id=0, style="EmptyPanelStyle", kids=[
            N("Button", "EngineSelectionButton", (9, 7, 26, 25), displayName="Engine selector", id=100001, optIsMomentary=False,
              style="EngineButtonStyle1"),
            N("LevelStrip", "VULeft", (41, 3, 5, 34), id=100004, style="EngineVUStyle"),
            N("Button", "EngineMutedIcon", (4, 36, 15, 3), id=100000, optIsMomentary=False, style="MuteIndiStyle", text=""),
            N("Button", "EngineSoloedIcon", (18, 36, 15, 3), id=100001, optIsMomentary=False, style="SoloIndiStyle", text=""),
            N("Button", "MuteButton", (5, 39, 13, 13), displayName="Mute", id=100002, optIsMomentary=False,
              style="HARDMuteButtonStyle", text="M"),
            N("Button", "SoloButton", (19, 39, 13, 13), displayName="Solo", id=100003, optIsMomentary=False,
              style="HARDSoloButtonStyle", text="S"),
            N("Button", "PatternPower", (33, 39, 13, 13), displayName="Pattern Power", id=100001, optIsMomentary=False,
              style="EnginePatternPowerStyle", text="\ue909"),
        ]),
    ]


@skin("Engines/EngineSelectors.skin")
def engine_selectors():
    return [
        N("Panel", "CtrlPanel", (0, 0, 393, 56), id=0, style="EmptyPanelStyle", kids=[
            N("Panel", "EngineSelector0", (1, 0, 49, 56), id=100001, overloadClass="EditorEngineSelector"),
            N("Panel", "EngineSelector1", (49, 0, 49, 56), id=100002, overloadClass="EditorEngineSelector"),
            N("Panel", "EngineSelector2", (97, 0, 49, 56), id=100002, overloadClass="EditorEngineSelector"),
            N("Panel", "EngineSelector3", (145, 0, 49, 56), id=100002, overloadClass="EditorEngineSelector"),
            N("Panel", "EngineSelector4", (193, 0, 49, 56), id=100001, overloadClass="EditorEngineSelector"),
            N("Panel", "EngineSelector5", (241, 0, 49, 56), id=100002, overloadClass="EditorEngineSelector"),
            N("Panel", "EngineSelector6", (289, 0, 49, 56), id=100002, overloadClass="EditorEngineSelector"),
            N("Panel", "EngineSelector7", (337, 0, 49, 56), id=100002, overloadClass="EditorEngineSelector"),
        ]),
    ]
