"""The pattern editor: track, automation and global controls (Pattern Editor/)."""

from . import N, skin


@skin("Pattern Editor/AutomationControls.skin")
def automation_controls():
    return [
        N("Panel", "CtrlPanel", (0, 0, 150, 75), id=0, style="EmptyPanelStyle", kids=[
            N("Button", "GraphCombo", (2, 1, 130, 17), id=100009, optIsMomentary=True, renderer="PopupButtonRenderer",
              style="LEDUltraMenuButtonStyle"),
            N("Button", "InsertButton", (2, 20, 50, 17), id=100009, optIsMomentary=True, renderer="PopupButtonRenderer",
              style="LEDMenuButtonStyle", text="Insert"),
            N("Button", "Div2Button", (53, 20, 19, 17), id=100000, optIsMomentary=True, style="Div2ButtonStyle", text="/2"),
            N("Button", "x2Button", (73, 20, 19, 17), id=100001, optIsMomentary=True, style="x2ButtonStyle", text="X2"),
            N("Button", "LearnButton", (93, 20, 50, 17), id=100014, optIsMomentary=False, style="LEDLatchingButtonStyle",
              text="Learn"),
            N("Label", "CtrlLabel", (3, 40, 26, 13), id=100002, style="StandardLabelStyle", text="Offset"),
            N("Label", "CtrlLabel", (32, 40, 26, 13), id=100006, style="StandardLabelStyle", text="Comp"),
            N("Label", "CtrlLabel", (65, 40, 26, 13), id=100003, style="StandardLabelStyle", text="Rand"),
            N("Label", "CtrlLabel", (85, 40, 66, 13), id=100002, style="StandardLabelCenteredStyle", text="Mode"),
            N("Knob", "ScaleKnob", (5, 52, 17, 17), displayName="Offset", id=100001, overloadClass="TimedKnob",
              realDialThickness=0.05, style="LEDKnobMicroLR"),
            N("Knob", "CompKnob", (37, 52, 17, 17), displayName="Compression", id=100005, overloadClass="TimedKnob",
              realDialThickness=0.05, style="LEDKnobMicroLR"),
            N("Knob", "RandKnob", (69, 52, 17, 17), displayName="Randomisation", id=100002, overloadClass="TimedKnob",
              realDialThickness=0.05, style="LEDKnobMicroLR"),
            N("ButtonGroup", "ModeGroup", (93, 52, 50, 19), id=100000, style="BasePanelStyle", kids=[
                N("Button", "CtrlButton", (0, 0, 25, 17), id=100000, optIsMomentary=False, style="MicroRadioButtonLeftStyle",
                  text="Edit"),
                N("Button", "CtrlButton", (23, 0, 25, 17), id=100001, optIsMomentary=False, style="MicroRadioButtonRightStyle",
                  text="Live"),
            ]),
        ]),
    ]


@skin("Pattern Editor/GlobalControls.skin")
def global_controls():
    return [
        N("Panel", "CtrlPanel", (0, 0, 815, 35), id=0, style="EmptyPanelStyle", kids=[
            N("Label", "CtrlLabel", (11, -1, 66, 13), id=100000, style="StandardLabelCenteredStyle", text="Tools"),
            N("Label", "CtrlLabel", (83, -1, 68, 13), id=100002, style="StandardLabelCenteredStyle", text="Drag edits..."),
            N("Label", "CtrlLabel", (150, -1, 66, 13), id=100000, style="StandardLabelCenteredStyle", text="View mode"),
            N("Label", "CtrlLabel", (253, -1, 121, 13), id=100001, style="StandardLabelCenteredStyle", text="Pattern name"),
            N("Label", "CtrlLabel", (361, -1, 56, 13), id=100007, style="StandardLabelCenteredStyle", text="Length"),
            N("Label", "CtrlLabel", (406, -1, 66, 13), id=100003, style="StandardLabelCenteredStyle", text="Steps"),
            N("Label", "CtrlLabel", (488, -1, 66, 13), id=100000, style="StandardLabelCenteredStyle", text="Play mode"),
            N("Label", "CtrlLabel", (566, -1, 221, 13), id=100000, style="StandardLabelCenteredStyle", text="Editing"),
            N("Label", "CtrlLabel", (774, -1, 46, 13), id=100000, style="StandardLabelCenteredStyle", text="Nudge"),
            N("Panel", "ToolsPanel", (3, 1, 86, 35), id=100000, overloadClass="PatternToolsPanel"),
            N("Button", "DragEditButton", (91, 11, 65, 17), id=100001, optIsMomentary=True, renderer="PopupButtonRenderer",
              style="DragEditButtonStyle", text="Velocity"),
            N("ButtonGroup", "ModeGroup", (160, 11, 50, 20), id=100000, style="BasePanelStyle", kids=[
                N("Button", "MultiButton", (0, 0, 24, 17), displayName="Multi track view", id=100001, optIsMomentary=False,
                  style="MultiTrackStyle", text="", kids=[
                    N("Panel", None, (6, 4, 3, 2), style="ToolMarkStyle"),
                    N("Panel", None, (10, 4, 3, 2), style="ToolMarkStyle"),
                    N("Panel", None, (14, 4, 3, 2), style="ToolMarkStyle"),
                    N("Panel", None, (6, 7, 3, 2), style="ToolMarkStyle"),
                    N("Panel", None, (10, 7, 3, 2), style="ToolMarkStyle"),
                    N("Panel", None, (14, 7, 3, 2), style="ToolMarkStyle"),
                    N("Panel", None, (6, 10, 3, 2), style="ToolMarkStyle"),
                    N("Panel", None, (10, 10, 3, 2), style="ToolMarkStyle"),
                    N("Panel", None, (14, 10, 3, 2), style="ToolMarkStyle"),
                ]),
                N("Button", "SingleButton", (23, 0, 24, 17), displayName="Multi graph view", id=100002, optIsMomentary=False,
                  style="SingleTrackStyle", text="", kids=[
                    N("Panel", None, (6, 4, 3, 1), style="ToolMarkStyle"),
                    N("Panel", None, (10, 4, 3, 1), style="ToolMarkStyle"),
                    N("Panel", None, (14, 4, 3, 1), style="ToolMarkStyle"),
                    N("Panel", None, (5, 12, 1, 1), style="ToolMarkStyle"),
                    N("Panel", None, (6, 12, 1, 1), style="ToolMarkStyle"),
                    N("Panel", None, (7, 12, 1, 1), style="ToolMarkStyle"),
                    N("Panel", None, (8, 11, 1, 2), style="ToolMarkStyle"),
                    N("Panel", None, (9, 11, 1, 2), style="ToolMarkStyle"),
                    N("Panel", None, (10, 10, 1, 3), style="ToolMarkStyle"),
                    N("Panel", None, (11, 10, 1, 3), style="ToolMarkStyle"),
                    N("Panel", None, (12, 10, 1, 3), style="ToolMarkStyle"),
                    N("Panel", None, (13, 9, 1, 4), style="ToolMarkStyle"),
                    N("Panel", None, (14, 9, 1, 4), style="ToolMarkStyle"),
                    N("Panel", None, (15, 8, 1, 5), style="ToolMarkStyle"),
                    N("Panel", None, (16, 8, 1, 5), style="ToolMarkStyle"),
                    N("Panel", None, (17, 7, 1, 6), style="ToolMarkStyle"),
                    N("Panel", None, (18, 7, 1, 6), style="ToolMarkStyle"),
                ]),
            ]),
            N("Button", "Follow", (210, 11, 50, 17), displayName="Follow playback position", id=100000, optIsMomentary=False,
              style="LEDLatchingButtonStyle", text="Follow"),
            N("TextBox", "PatternNameBox", (262, 11, 105, 17), id=100000, optSingleClickEdit=True, style="LEDEditBoxStyle"),
            N("PopupButton", "StepLengthCombo", (369, 11, 40, 17), id=100004, legacyClass="CtrlComboBox",
              optSingleClickEdit=True, style="LEDSmallComboBoxStyle"),
            N("Button", "MinusButton", (411, 11, 15, 17), displayName="Reduce steps", icon="minus", id=100000,
              optIsMomentary=True, overloadClass="IconButton", style="StandardSubtractButtonStyle", text=""),
            N("ValueBox", "LengthBox", (425, 11, 30, 17), id=100002, optSingleClickEdit=True,
              style="LEDSmallNonDragableValueBoxStyle"),
            N("Button", "PlusButton", (454, 11, 15, 17), displayName="Increase steps", icon="plus", id=100001,
              optIsMomentary=True, overloadClass="IconButton", style="StandardAddButtonStyle", text=""),
            N("Button", "RestartButton", (471, 11, 50, 17), displayName="Restart pattern from beginning on pattern change",
              id=100001, optIsMomentary=False, style="LEDLatchingButtonStyle", text="Restart"),
            N("Button", "GateButton", (522, 11, 50, 17), displayName="Pattern requires MIDI trigger to playback", id=100000,
              optIsMomentary=False, style="LEDLatchingButtonStyle", text="Gated"),
            N("Button", "MonoButton", (574, 11, 50, 17), displayName="Remix mode", id=100001, optIsMomentary=False,
              style="LEDLatchingButtonStyle", text="Remix!"),
            N("Button", "QuantizeButton", (626, 11, 50, 17), displayName="Quantize", id=100000, optIsMomentary=True,
              style="LEDMomentaryButtonStyle", text="Quantize"),
            N("Button", "RepeatButton", (678, 11, 50, 17), displayName="Repeat selected notes / pattern", id=100007,
              optIsMomentary=True, style="LEDMomentaryButtonStyle", text="Duplicate"),
            N("Button", "InsertButton", (730, 11, 50, 17), id=100008, optIsMomentary=True, renderer="PopupButtonRenderer",
              style="LEDMenuButtonStyle", text="Insert"),
            N("Button", "NudgeBack", (783, 11, 15, 17), displayName="Nudge selected notes / pattern", id=100000,
              optIsMomentary=True, style="NudgeLeftStyle", text="\ue901", kids=[
                N("Panel", None, (9, 5, 2, 7), style="ToolMarkStyle"),
            ]),
            N("Button", "NudgeNext", (797, 11, 15, 17), displayName="Nudge selected notes / pattern", id=100001,
              optIsMomentary=True, style="NudgeRightStyle", text="\ue903", kids=[
                N("Panel", None, (4, 5, 2, 7), style="ToolMarkStyle"),
            ]),
        ]),
    ]


@skin("Pattern Editor/TrackControls.skin")
def track_controls():
    return [
        N("Panel", "CtrlPanel", (0, 0, 150, 20), id=0, style="EmptyPanelStyle", kids=[
            N("Button", "AutomateButton", (0, 0, 20, 20), displayName="Show graphs", id=100003, optIsMomentary=False,
              style="TrackControlAutoStyle", text="►"),
            N("Label", "SelectArea", (19, 1, 68, 18), id=100000, style="BaseLabelStyle", text="SelectArea"),
            N("Knob", "LevelSlider", (88, 1, 17, 17), displayName="Pad Gain", id=100000, realDialThickness=0.05,
              style="LEDKnobMicroLR"),
            N("Button", "MuteButton", (108, 0, 19, 19), displayName="Mute track steps", id=100002, optIsMomentary=False,
              style="LEDMuteButtonStyle", text="M"),
            N("Button", "SoloButton", (128, 0, 19, 19), displayName="Solo track steps", id=100004, optIsMomentary=False,
              style="LEDSoloButtonStyle", text="S"),
        ]),
    ]
