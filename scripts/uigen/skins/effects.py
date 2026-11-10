"""Effect panels, one per effect type, and the effect chain (Effects/)."""

from . import N, skin


@skin("Effects/BreverbHall.skin")
def breverb_hall():
    return [
        N("Panel", "CtrlPanel", (0, 0, 193, 150), id=0, style="BreverbPanelStyle", kids=[
            N("Panel", None, (0, 0, 193, 150), style="ReverbCaseStyle"),
            N("Panel", None, (4, 22, 183, 121), style="ReverbScreenStyle"),
            N("Label", "CtrlLabel", (67, 0, 116, 20), id=100000, style="StandardFXLabelCenteredStyle", text="HALL"),
            N("Knob", "Control1", (21, 35, 24, 26), id=100022, realDialThickness=0.04, style="LEDKnobSmallLR"),
            N("Knob", "Control4", (82, 35, 24, 26), id=100025, realDialThickness=0.04, style="LEDKnobSmallLR"),
            N("Knob", "Control2", (144, 35, 24, 26), id=100023, realDialThickness=0.04, style="LEDKnobSmallLR"),
            N("Label", "CtrlLabel", (4, 65, 56, 15), id=100028, style="StandardLabelCenteredStyle", text="Time"),
            N("Label", "CtrlLabel", (75, 65, 37, 15), id=100031, style="StandardLabelCenteredStyle", text="Size"),
            N("Label", "CtrlLabel", (126, 65, 58, 15), id=100029, style="StandardLabelCenteredStyle", text="Predelay"),
            N("Knob", "Control6", (19, 90, 24, 26), id=100021, realDialThickness=0.04, style="LEDKnobSmallLR"),
            N("Knob", "Control5", (82, 90, 24, 26), id=100020, realDialThickness=0.04, style="LEDKnobSmallLR"),
            N("Knob", "Control3", (144, 90, 24, 26), id=100024, realDialThickness=0.04, style="LEDKnobSmallLR"),
            N("Label", "CtrlLabel", (3, 120, 56, 15), id=100027, style="StandardLabelCenteredStyle", text="Shape"),
            N("Label", "CtrlLabel", (65, 120, 56, 15), id=100026, style="StandardLabelCenteredStyle", text="Spread"),
            N("Label", "CtrlLabel", (128, 120, 56, 15), id=100030, style="StandardLabelCenteredStyle", text="Diffusion"),
        ]),
    ]


@skin("Effects/BreverbInverse.skin")
def breverb_inverse():
    return [
        N("Panel", "CtrlPanel", (0, 0, 193, 150), id=0, style="BreverbPanelStyle", kids=[
            N("Panel", None, (0, 0, 193, 150), style="ReverbCaseStyle"),
            N("Panel", None, (4, 22, 183, 121), style="ReverbScreenStyle"),
            N("Label", "CtrlLabel", (59, 0, 116, 20), id=100000, style="StandardFXLabelCenteredStyle", text="INVERSE"),
            N("Knob", "Control1", (21, 62, 24, 26), id=100022, realDialThickness=0.04, style="LEDKnobSmallLR"),
            N("Knob", "Control2", (82, 62, 24, 26), id=100023, realDialThickness=0.04, style="LEDKnobSmallLR"),
            N("Knob", "Control3", (144, 62, 24, 26), id=100024, realDialThickness=0.04, style="LEDKnobSmallLR"),
            N("Label", "CtrlLabel", (4, 92, 56, 15), id=100028, style="StandardLabelCenteredStyle", text="Time"),
            N("Label", "CtrlLabel", (64, 92, 58, 15), id=100029, style="StandardLabelCenteredStyle", text="Predelay"),
            N("Label", "CtrlLabel", (128, 92, 56, 15), id=100030, style="StandardLabelCenteredStyle", text="Diffusion"),
        ]),
    ]


@skin("Effects/BreverbPlate.skin")
def breverb_plate():
    return [
        N("Panel", "CtrlPanel", (0, 0, 193, 150), id=0, style="BreverbPanelStyle", kids=[
            N("Panel", None, (0, 0, 193, 150), style="ReverbCaseStyle"),
            N("Panel", None, (4, 22, 183, 121), style="ReverbScreenStyle"),
            N("Label", "CtrlLabel", (59, 0, 116, 20), id=100000, style="StandardFXLabelCenteredStyle", text="PLATE"),
            N("Knob", "Control1", (21, 35, 24, 26), id=100022, realDialThickness=0.04, style="LEDKnobSmallLR"),
            N("Knob", "Control4", (82, 35, 24, 26), id=100025, realDialThickness=0.04, style="LEDKnobSmallLR"),
            N("Knob", "Control2", (144, 35, 24, 26), id=100023, realDialThickness=0.04, style="LEDKnobSmallLR"),
            N("Label", "CtrlLabel", (4, 65, 56, 15), id=100028, style="StandardLabelCenteredStyle", text="Time"),
            N("Label", "CtrlLabel", (75, 65, 37, 15), id=100031, style="StandardLabelCenteredStyle", text="Size"),
            N("Label", "CtrlLabel", (126, 65, 58, 15), id=100029, style="StandardLabelCenteredStyle", text="Predelay"),
            N("Knob", "Control5", (53, 90, 24, 26), id=100020, realDialThickness=0.04, style="LEDKnobSmallLR"),
            N("Knob", "Control3", (113, 90, 24, 26), id=100024, realDialThickness=0.04, style="LEDKnobSmallLR"),
            N("Label", "CtrlLabel", (35, 120, 56, 15), id=100026, style="StandardLabelCenteredStyle", text="Spread"),
            N("Label", "CtrlLabel", (98, 120, 56, 15), id=100030, style="StandardLabelCenteredStyle", text="Diffusion"),
        ]),
    ]


@skin("Effects/BreverbRoom.skin")
def breverb_room():
    return [
        N("Panel", "CtrlPanel", (0, 0, 193, 150), id=0, style="BreverbPanelStyle", kids=[
            N("Panel", None, (0, 0, 193, 150), style="ReverbCaseStyle"),
            N("Panel", None, (4, 22, 183, 121), style="ReverbScreenStyle"),
            N("Label", "CtrlLabel", (59, 0, 116, 20), id=100000, style="StandardFXLabelCenteredStyle", text="ROOM"),
            N("Knob", "Control1", (21, 35, 24, 26), id=100022, realDialThickness=0.04, style="LEDKnobSmallLR"),
            N("Knob", "Control4", (82, 35, 24, 26), id=100025, realDialThickness=0.04, style="LEDKnobSmallLR"),
            N("Knob", "Control2", (144, 35, 24, 26), id=100023, realDialThickness=0.04, style="LEDKnobSmallLR"),
            N("Label", "CtrlLabel", (4, 65, 56, 15), id=100028, style="StandardLabelCenteredStyle", text="Time"),
            N("Label", "CtrlLabel", (75, 65, 37, 15), id=100031, style="StandardLabelCenteredStyle", text="Size"),
            N("Label", "CtrlLabel", (126, 65, 58, 15), id=100029, style="StandardLabelCenteredStyle", text="Predelay"),
            N("Knob", "Control5", (53, 90, 24, 26), id=100020, realDialThickness=0.04, style="LEDKnobSmallLR"),
            N("Knob", "Control3", (113, 90, 24, 26), id=100024, realDialThickness=0.04, style="LEDKnobSmallLR"),
            N("Label", "CtrlLabel", (35, 120, 56, 15), id=100026, style="StandardLabelCenteredStyle", text="Spread"),
            N("Label", "CtrlLabel", (98, 120, 56, 15), id=100030, style="StandardLabelCenteredStyle", text="Diffusion"),
        ]),
    ]


@skin("Effects/BusComp.skin")
def bus_comp():
    return [
        N("Panel", "CtrlPanel", (0, 0, 193, 150), id=0, style="EmptyPanelStyle", kids=[
            N("Meter", "DasMeter", (43, 2, 107, 50), id=100031, style="StandardDBReduceMeterStyle"),
            N("Knob", "Control1", (21, 57, 24, 26), id=100009, realDialThickness=0.04, style="LEDKnobSmallLR"),
            N("Knob", "Control3", (82, 57, 24, 26), id=100020, realDialThickness=0.04, style="LEDKnobSmallLR"),
            N("Knob", "Control5", (144, 57, 24, 26), id=100032, realDialThickness=0.04, style="LEDKnobSmallLR"),
            N("Label", "CtrlLabel", (-1, 87, 66, 15), id=100010, style="StandardLabelCenteredStyle", text="Threshold"),
            N("Label", "CtrlLabel", (74, 87, 41, 15), id=100026, style="StandardLabelCenteredStyle", text="Ratio"),
            N("Label", "CtrlLabel", (122, 87, 66, 15), id=100033, style="StandardLabelCenteredStyle", text="Make up"),
            N("Knob", "Control2", (21, 107, 24, 26), id=100021, realDialThickness=0.04, style="LEDKnobSmallLR"),
            N("Knob", "Control4", (82, 107, 24, 26), id=100011, realDialThickness=0.04, style="LEDKnobSmallLR"),
            N("Button", "Control6", (148, 107, 16, 17), id=100034, optIsMomentary=False, style="LEDSmallLatchingButtonStyle",
              text=""),
            N("Label", "CtrlLabel", (4, 137, 56, 15), id=100027, style="StandardLabelCenteredStyle", text="Attack"),
            N("Label", "CtrlLabel", (66, 137, 56, 15), id=100014, style="StandardLabelCenteredStyle", text="Release"),
            N("Label", "CtrlLabel", (135, 137, 41, 15), id=100036, style="StandardLabelCenteredStyle", text="Limit"),
        ]),
    ]


@skin("Effects/ChanComp.skin")
def chan_comp():
    return [
        N("Panel", "CtrlPanel", (0, 0, 193, 150), id=0, style="EmptyPanelStyle", kids=[
            N("Meter", "DasMeter", (43, 2, 107, 50), id=100031, style="StandardDBReduceMeterStyle"),
            N("PopupButton", "Control4", (76, 57, 40, 17), id=100029, legacyClass="CtrlComboBox", optSingleClickEdit=True,
              style="LEDSmallComboBoxStyle"),
            N("Knob", "Control2", (17, 78, 24, 26), id=100021, realDialThickness=0.04, style="LEDKnobSmallLR"),
            N("Knob", "Control3", (152, 78, 24, 26), id=100011, realDialThickness=0.04, style="LEDKnobSmallLR"),
            N("Knob", "Control1", (62, 107, 24, 26), id=100009, realDialThickness=0.04, style="LEDKnobSmallLR"),
            N("Knob", "Control5", (106, 107, 24, 26), id=100020, realDialThickness=0.04, style="LEDKnobSmallLR"),
            N("Label", "CtrlLabel", (68, 78, 56, 15), id=100030, style="StandardLabelCenteredStyle", text="Ratio"),
            N("Label", "CtrlLabel", (3, 108, 51, 15), id=100027, style="StandardLabelCenteredStyle", text="Attack"),
            N("Label", "CtrlLabel", (47, 137, 51, 15), id=100010, style="StandardLabelCenteredStyle", text="Input"),
            N("Label", "CtrlLabel", (93, 137, 51, 15), id=100026, style="StandardLabelCenteredStyle", text="Output"),
            N("Label", "CtrlLabel", (137, 108, 56, 15), id=100014, style="StandardLabelCenteredStyle", text="Release"),
        ]),
    ]


@skin("Effects/Chorus.skin")
def chorus():
    return [
        N("Panel", "CtrlPanel", (0, 0, 193, 150), id=0, style="EmptyPanelStyle", kids=[
            N("Knob", "Control1", (7, 44, 50, 46), id=100014, realDialThickness=0.08, style="LEDKnobLargeLR"),
            N("Knob", "Control2", (71, 44, 50, 46), id=100016, realDialThickness=0.08, style="LEDKnobLargeLR"),
            N("Knob", "Control3", (136, 44, 50, 46), id=100018, realDialThickness=0.08, style="LEDKnobLargeLR"),
            N("Label", "CtrlLabel", (-1, 94, 66, 15), id=100015, style="StandardLabelCenteredStyle", text="Frequency"),
            N("Label", "CtrlLabel", (63, 94, 66, 15), id=100017, style="StandardLabelCenteredStyle", text="Depth"),
            N("Label", "CtrlLabel", (128, 94, 66, 15), id=100019, style="StandardLabelCenteredStyle", text="Spread"),
        ]),
    ]


@skin("Effects/Comb.skin")
def comb():
    return [
        N("Panel", "CtrlPanel", (0, 0, 193, 150), id=0, style="EmptyPanelStyle", kids=[
            N("Button", "Control2", (88, 16, 16, 17), id=100001, optIsMomentary=False, style="LEDSmallLatchingButtonStyle",
              text=""),
            N("Label", "CtrlLabel", (72, 37, 46, 15), id=100010, style="StandardLabelCenteredStyle", text="Mode"),
            N("Knob", "Control1", (7, 68, 50, 46), displayName="Drive", id=100020, realDialThickness=0.08,
              style="LEDKnobLargeLR"),
            N("Knob", "Control3", (71, 68, 50, 46), displayName="Pitch", id=100021, realDialThickness=0.08,
              style="LEDKnobLargeLR"),
            N("Knob", "Control4", (136, 68, 50, 46), displayName="Resonance", id=100012, realDialThickness=0.08,
              style="LEDKnobLargeLR"),
            N("Label", "CtrlLabel", (-1, 118, 66, 15), id=100026, style="StandardLabelCenteredStyle", text="Input Drive"),
            N("Label", "CtrlLabel", (63, 118, 66, 15), id=100027, style="StandardLabelCenteredStyle", text="Pitch"),
            N("Label", "CtrlLabel", (128, 118, 66, 15), id=100015, style="StandardLabelCenteredStyle", text="Resonance"),
        ]),
    ]


@skin("Effects/Crusher.skin")
def crusher():
    return [
        N("Panel", "CtrlPanel", (0, 0, 193, 150), id=0, style="EmptyPanelStyle", kids=[
            N("Panel", "Control4", (43, 3, 107, 15), id=100001, style="EmptyPanelStyle", kids=[
                N("Label", "CtrlLabel", (-8, 0, 123, 15), id=0, style="StandardLabelCenteredStyle", text="High Cut"),
            ]),
            N("RangeSlider", "SplitFader", (43, 20, 107, 19), id=100000, legacyClass="CtrlPanel", overloadClass="SplitFader",
              style="LcdRangeStyle"),
            N("Panel", "Control5", (43, 42, 107, 15), id=100001, style="EmptyPanelStyle", kids=[
                N("Label", "CtrlLabel", (-8, 1, 123, 15), id=0, style="StandardLabelCenteredStyle", text="Low Cut"),
            ]),
            N("Knob", "Control1", (14, 80, 24, 26), id=100020, realDialThickness=0.04, style="LEDKnobSmallLR"),
            N("Knob", "Control6", (61, 80, 24, 26), id=100022, realDialThickness=0.04, style="LEDKnobSmallLR"),
            N("Knob", "Control2", (108, 80, 24, 26), id=100021, realDialThickness=0.04, style="LEDKnobSmallLR"),
            N("Knob", "Control3", (155, 80, 24, 26), id=100023, realDialThickness=0.04, style="LEDKnobSmallLR"),
            N("Label", "CtrlLabel", (1, 110, 51, 15), id=100026, style="StandardLabelCenteredStyle", text="Drive"),
            N("Label", "CtrlLabel", (39, 110, 66, 15), id=100028, style="StandardLabelCenteredStyle", text="Tone"),
            N("Label", "CtrlLabel", (85, 110, 66, 15), id=100027, style="StandardLabelCenteredStyle", text="Freq"),
            N("Label", "CtrlLabel", (132, 110, 66, 15), id=100029, style="StandardLabelCenteredStyle", text="Bits"),
        ]),
    ]


@skin("Effects/DJEQ.skin")
def djeq():
    return [
        N("Panel", "CtrlPanel", (0, 0, 193, 150), id=0, style="EmptyPanelStyle", kids=[
            N("Plot", "Plot", (43, 2, 107, 50), id=100000, style="EQGraphStyle"),
            N("Knob", "Control1", (14, 80, 24, 26), id=100009, realDialThickness=0.04, style="LEDKnobSmallLR"),
            N("Knob", "Control4", (155, 80, 24, 26), id=100011, realDialThickness=0.04, style="LEDKnobSmallLR"),
            N("Knob", "Control2", (61, 80, 24, 26), id=100020, realDialThickness=0.04, style="LEDKnobSmallLR"),
            N("Knob", "Control3", (108, 80, 24, 26), id=100021, realDialThickness=0.04, style="LEDKnobSmallLR"),
            N("Label", "CtrlLabel", (-8, 110, 66, 15), id=100010, style="StandardLabelCenteredStyle", text="L Gain"),
            N("Label", "CtrlLabel", (133, 110, 66, 15), id=100014, style="StandardLabelCenteredStyle", text="H Gain"),
            N("Label", "CtrlLabel", (39, 110, 66, 15), id=100026, style="StandardLabelCenteredStyle", text="M Gain"),
            N("Label", "CtrlLabel", (85, 110, 66, 15), id=100027, style="StandardLabelCenteredStyle", text="M Freq"),
        ]),
    ]


@skin("Effects/Delay.skin")
def delay():
    return [
        N("Panel", "CtrlPanel", (0, 0, 193, 150), id=0, style="EmptyPanelStyle", kids=[
            N("Panel", "Control3", (84, 3, 107, 15), id=100001, style="EmptyPanelStyle", kids=[
                N("Label", "CtrlLabel", (-8, 0, 123, 15), id=0, style="StandardLabelCenteredStyle", text="High Cut"),
            ]),
            N("RangeSlider", "SplitFader", (84, 20, 107, 19), id=100000, legacyClass="CtrlPanel", overloadClass="SplitFader",
              style="LcdRangeStyle"),
            N("Panel", "Control4", (84, 42, 107, 15), id=100001, style="EmptyPanelStyle", kids=[
                N("Label", "CtrlLabel", (-8, 0, 123, 15), id=0, style="StandardLabelCenteredStyle", text="Low Cut"),
            ]),
            N("Button", "Control6", (25, 21, 16, 17), id=100002, optIsMomentary=False, style="LEDSmallLatchingButtonStyle",
              text=""),
            N("Label", "CtrlLabel", (-1, 43, 66, 13), id=100003, style="StandardLabelCenteredStyle", text="Sync"),
            N("Knob", "Control1", (7, 68, 50, 46), displayName="Delay time", id=100001, realDialThickness=0.08,
              style="LEDKnobLargeLR"),
            N("Knob", "Control2", (71, 68, 50, 46), id=100021, realDialThickness=0.08, style="LEDKnobLargeLR"),
            N("Knob", "Control5", (136, 68, 50, 46), id=100027, realDialThickness=0.08, style="LEDKnobLargeLR"),
            N("Label", "CtrlLabel", (-1, 118, 66, 15), id=100020, style="StandardLabelCenteredStyle", text="Delay Time"),
            N("Label", "CtrlLabel", (63, 118, 66, 15), id=100022, style="StandardLabelCenteredStyle", text="Feedback"),
            N("Label", "CtrlLabel", (128, 118, 66, 15), id=100028, style="StandardLabelCenteredStyle", text="Swing"),
        ]),
    ]


@skin("Effects/Distortion.skin")
def distortion():
    return [
        N("Panel", "CtrlPanel", (0, 0, 193, 150), id=0, style="EmptyPanelStyle", kids=[
            N("Panel", "Control3", (84, 3, 107, 15), id=100001, style="EmptyPanelStyle", kids=[
                N("Label", "CtrlLabel", (-8, 0, 123, 15), id=0, style="StandardLabelCenteredStyle", text="High Cut"),
            ]),
            N("RangeSlider", "SplitFader", (84, 20, 107, 19), id=100000, legacyClass="CtrlPanel", overloadClass="SplitFader",
              style="LcdRangeStyle"),
            N("PopupButton", "Control1", (5, 21, 75, 17), id=100002, legacyClass="CtrlComboBox", optSingleClickEdit=True,
              style="LEDComboBoxStyle"),
            N("Label", "CtrlLabel", (-3, 44, 91, 15), id=100001, style="StandardLabelCenteredStyle", text="Type"),
            N("Panel", "Control4", (84, 42, 107, 15), id=100001, style="EmptyPanelStyle", kids=[
                N("Label", "CtrlLabel", (-8, 0, 123, 15), id=0, style="StandardLabelCenteredStyle", text="Low Cut"),
            ]),
            N("Knob", "Control5", (7, 68, 50, 46), id=100021, realDialThickness=0.08, style="LEDKnobLargeLR"),
            N("Knob", "Control2", (71, 68, 50, 46), id=100020, realDialThickness=0.08, style="LEDKnobLargeLR"),
            N("Knob", "Control6", (136, 68, 50, 46), id=100001, realDialThickness=0.08, style="LEDKnobLargeLR"),
            N("Label", "CtrlLabel", (-1, 118, 66, 15), id=100027, style="StandardLabelCenteredStyle", text="Tone"),
            N("Label", "CtrlLabel", (63, 118, 66, 15), id=100026, style="StandardLabelCenteredStyle", text="Drive"),
            N("Label", "CtrlLabel", (128, 118, 66, 15), id=100002, style="StandardLabelCenteredStyle", text="Clean"),
        ]),
    ]


@skin("Effects/EffectChain.skin")
def effect_chain():
    return [
        N("Panel", "CtrlPanel", (0, 0, 812, 245), id=0, style="FXGlobalBackgroundPanelStyle", kids=[
            N("Panel", "ToolsPanel", (0, 0, 812, 40), id=100002, style="EmptyPanelStyle", kids=[
                N("Panel", "HeaderPanel", (10, 3, 306, 35), id=0, style="ChainHeaderStyle", kids=[
                    N("Knob", "MixKnob", (40, 4, 18, 19), displayName="Wet / Dry Mix", id=100000, realDialThickness=0.05,
                      style="LEDKnobMicroLR"),
                    N("Button", "PresetCombo", (70, 5, 190, 24), id=100006, optIsMomentary=True, renderer="PopupButtonRenderer",
                      style="LEDEffectCombo"),
                    N("Button", "PresetPrev", (264, 8, 16, 18), displayName="Previous preset", id=100000, optIsMomentary=True,
                      style="LargePrevPresetButtonStyle", text=""),
                    N("Button", "PresetNext", (283, 8, 16, 18), displayName="Next preset", id=100001, optIsMomentary=True,
                      style="LargeNextPresetButtonStyle", text=""),
                    N("Button", "ActiveButton", (6, 5, 24, 24), displayName="Chain active", id=100006, optIsMomentary=False,
                      style="LargePowerButtonStyle", text=""),
                    N("Label", "CtrlLabel", (29, 22, 36, 13), id=100002, style="StandardLabelCenteredStyle", text="Mix"),
                ]),
                N("ScrollBar", "ScrollBar", (353, 3, 450, 35), id=100003, legacyClass="CtrlScrollViewSlider",
                  overloadClass="EditorEffectPanelScrollbar", style="FXScrollerStyle"),
            ]),
            N("Panel", "ScrollView", (8, 39, 796, 196), id=100000, legacyClass="CtrlScrollView", style="EmptyScrollViewStyle"),
        ]),
    ]


@skin("Effects/EffectChainFX.skin")
def effect_chain_fx():
    return [
        N("Panel", "ChainPanelScroll", (0, 0, 1194, 196), id=0, style="EmptyPanelStyle", kids=[
            N("Panel", "FX1Panel", (0, 0, 199, 196), id=100003, overloadClass="EditorEffectPanel"),
            N("Panel", "FX2Panel", (199, 0, 199, 196), id=100003, overloadClass="EditorEffectPanel"),
            N("Panel", "FX3Panel", (398, 0, 199, 196), id=100003, overloadClass="EditorEffectPanel"),
            N("Panel", "FX4Panel", (597, 0, 199, 196), id=100003, overloadClass="EditorEffectPanel"),
            N("Panel", "FX5Panel", (796, 0, 199, 196), id=100003, overloadClass="EditorEffectPanel"),
            N("Panel", "FX6Panel", (995, 0, 199, 196), id=100003, overloadClass="EditorEffectPanel"),
        ]),
    ]


@skin("Effects/EffectPanel.skin")
def effect_panel():
    return [
        N("Panel", "CtrlPanel", (0, 0, 199, 196), id=0, style="EffectPanelBackgroundNoFX", kids=[
            N("Panel", None, (1, 2, 197, 192), style="FxCardEmptyStyle"),
            N("Panel", None, (2, 3, 195, 190), style="FxCardInnerEmptyStyle"),
            N("Panel", None, (3, 4, 193, 38), style="FxCardHeadStyle"),
            N("Panel", None, (3, 42, 193, 1), style="FxCardHeadLineEmptyStyle"),
            N("Panel", "SetupPanel", (0, 0, 199, 50), id=100001, style="EmptyPanelStyle", kids=[
                N("Knob", "MixKnob", (158, 4, 18, 19), displayName="Effect wet / dry mix", id=100004, realDialThickness=0.05,
                  style="LEDKnobMicroLR"),
                N("Button", "EffectTypeCombo", (4, 5, 130, 17), displayName="Effect Type", id=100002, optIsMomentary=True,
                  renderer="PopupButtonRenderer", style="LEDUltraMenuButtonStyle"),
                N("Button", "ActiveButton", (136, 5, 16, 17), displayName="Effect active", id=100006, optIsMomentary=False,
                  style="LEDSmallLatchingButtonStyle", text=""),
                N("Label", "Label1", (178, 7, 20, 13), id=100009, style="StandardLabelStyle", text="Mix"),
                N("Button", "LastPresetButton", (135, 26, 9, 11), displayName="Previous preset", id=100000, optIsMomentary=True,
                  style="PrevFXPresetButtonStyle", text=""),
                N("Button", "NextPresetButton", (145, 26, 9, 11), displayName="Next preset", id=100001, optIsMomentary=True,
                  style="NextFXPresetButtonStyle", text=""),
                N("Button", "PresetCombo", (4, 23, 130, 17), displayName="Current preset", id=100006, optIsMomentary=True,
                  renderer="PopupButtonRenderer", style="LEDUltraMenuButtonStyle"),
                N("Knob", "OutputKnob", (158, 23, 18, 19), displayName="Effect output level", id=100005, realDialThickness=0.05,
                  style="LEDKnobMicroLR"),
                N("Label", "Label2", (178, 27, 15, 13), id=100010, style="StandardLabelStyle", text="Lvl"),
            ]),
            N("Panel", "EffectPanel", (3, 42, 193, 150), id=100012, style="EmptyPanelStyle"),
        ]),
    ]


@skin("Effects/EnvShaper.skin")
def env_shaper():
    return [
        N("Panel", "CtrlPanel", (0, 0, 193, 150), id=0, style="EmptyPanelStyle", kids=[
            N("Knob", "Control1", (7, 5, 50, 46), displayName="Drive", id=100020, realDialThickness=0.08, style="LEDKnobLargeLR"),
            N("Knob", "Control2", (136, 5, 50, 46), displayName="Mode", id=100009, realDialThickness=0.08,
              style="LEDKnobLargeLR"),
            N("Label", "CtrlLabel", (-1, 55, 66, 15), id=100026, style="StandardLabelCenteredStyle", text="Attack"),
            N("Label", "CtrlLabel", (128, 55, 66, 15), id=100010, style="StandardLabelCenteredStyle", text="Sustain"),
            N("Button", "Control5", (72, 66, 50, 17), id=100000, optIsMomentary=False, style="LEDLatchingButtonStyle",
              text="Linked"),
            N("Knob", "Control4", (7, 79, 50, 46), displayName="Resonance", id=100012, realDialThickness=0.08,
              style="LEDKnobLargeLR"),
            N("Knob", "Control3", (136, 79, 50, 46), displayName="Pitch", id=100021, realDialThickness=0.08,
              style="LEDKnobLargeLR"),
            N("Label", "CtrlLabel", (-1, 129, 66, 15), id=100015, style="StandardLabelCenteredStyle", text="Sensitivy"),
            N("Label", "CtrlLabel", (128, 129, 66, 15), id=100027, style="StandardLabelCenteredStyle", text="Gain"),
        ]),
    ]


@skin("Effects/Filter.skin")
def filter():
    return [
        N("Panel", "CtrlPanel", (0, 0, 193, 150), id=0, style="EmptyPanelStyle", kids=[
            N("Plot", "Plot", (43, 2, 107, 50), id=100000, style="EQGraphStyle"),
            N("Button", "Control1", (13, 16, 16, 17), displayName="HP Active", id=100004, optIsMomentary=False,
              style="LEDSmallLatchingButtonStyle", text=""),
            N("Button", "Control3", (164, 16, 16, 17), displayName="LP Active", id=100003, optIsMomentary=False,
              style="LEDSmallLatchingButtonStyle", text=""),
            N("Label", "CtrlLabel", (-8, 37, 56, 15), id=100006, style="StandardLabelCenteredStyle", text="HP On"),
            N("Label", "CtrlLabel", (144, 37, 56, 15), id=100005, style="StandardLabelCenteredStyle", text="LP On"),
            N("Knob", "Control2", (36, 68, 50, 46), displayName="HP Frequency", id=100021, realDialThickness=0.08,
              style="LEDKnobLargeLR"),
            N("Knob", "Control4", (106, 68, 50, 46), displayName="LP Frequency", id=100020, realDialThickness=0.08,
              style="LEDKnobLargeLR"),
            N("Label", "CtrlLabel", (28, 121, 66, 15), id=100027, style="StandardLabelCenteredStyle", text="HP Freq"),
            N("Label", "CtrlLabel", (98, 121, 66, 15), id=100026, style="StandardLabelCenteredStyle", text="LP Freq"),
        ]),
    ]


@skin("Effects/Filter101.skin")
def filter101():
    return [
        N("Panel", "CtrlPanel", (0, 0, 193, 150), id=0, style="EmptyPanelStyle", kids=[
            N("PopupButton", "Control2", (59, 16, 75, 17), id=100002, legacyClass="CtrlComboBox", optSingleClickEdit=True,
              style="LEDComboBoxStyle"),
            N("Label", "CtrlLabel", (51, 37, 91, 15), id=100010, style="StandardLabelCenteredStyle", text="Mode"),
            N("Knob", "Control3", (71, 68, 50, 46), displayName="Pitch", id=100021, realDialThickness=0.08,
              style="LEDKnobLargeLR"),
            N("Knob", "Control4", (136, 68, 50, 46), displayName="Resonance", id=100012, realDialThickness=0.08,
              style="LEDKnobLargeLR"),
            N("Knob", "Control1", (7, 68, 50, 46), displayName="Drive", id=100020, realDialThickness=0.08,
              style="LEDKnobLargeLR"),
            N("Label", "CtrlLabel", (63, 118, 66, 15), id=100027, style="StandardLabelCenteredStyle", text="Pitch"),
            N("Label", "CtrlLabel", (128, 118, 66, 15), id=100015, style="StandardLabelCenteredStyle", text="Resonance"),
            N("Label", "CtrlLabel", (-1, 118, 66, 15), id=100026, style="StandardLabelCenteredStyle", text="Input Drive"),
        ]),
    ]


@skin("Effects/Filter106.skin")
def filter106():
    return [
        N("Panel", "CtrlPanel", (0, 0, 193, 150), id=0, style="EmptyPanelStyle", kids=[
            N("PopupButton", "Control2", (59, 16, 75, 17), id=100002, legacyClass="CtrlComboBox", optSingleClickEdit=True,
              style="LEDComboBoxStyle"),
            N("Label", "CtrlLabel", (51, 37, 91, 15), id=100010, style="StandardLabelCenteredStyle", text="Mode"),
            N("Knob", "Control3", (71, 68, 50, 46), displayName="Pitch", id=100021, realDialThickness=0.08,
              style="LEDKnobLargeLR"),
            N("Knob", "Control4", (136, 68, 50, 46), displayName="Resonance", id=100012, realDialThickness=0.08,
              style="LEDKnobLargeLR"),
            N("Knob", "Control1", (7, 68, 50, 46), displayName="Drive", id=100020, realDialThickness=0.08,
              style="LEDKnobLargeLR"),
            N("Label", "CtrlLabel", (63, 118, 66, 15), id=100027, style="StandardLabelCenteredStyle", text="Pitch"),
            N("Label", "CtrlLabel", (128, 118, 66, 15), id=100015, style="StandardLabelCenteredStyle", text="Resonance"),
            N("Label", "CtrlLabel", (-1, 118, 66, 15), id=100026, style="StandardLabelCenteredStyle", text="Input Drive"),
        ]),
    ]


@skin("Effects/Filter303.skin")
def filter303():
    return [
        N("Panel", "CtrlPanel", (0, 0, 193, 150), id=0, style="EmptyPanelStyle", kids=[
            N("PopupButton", "Control2", (59, 16, 75, 17), id=100002, legacyClass="CtrlComboBox", optSingleClickEdit=True,
              style="LEDComboBoxStyle"),
            N("Label", "CtrlLabel", (51, 37, 91, 15), id=100010, style="StandardLabelCenteredStyle", text="Mode"),
            N("Knob", "Control3", (71, 68, 50, 46), displayName="Pitch", id=100021, realDialThickness=0.08,
              style="LEDKnobLargeLR"),
            N("Knob", "Control4", (136, 68, 50, 46), displayName="Resonance", id=100012, realDialThickness=0.08,
              style="LEDKnobLargeLR"),
            N("Knob", "Control1", (7, 68, 50, 46), displayName="Drive", id=100020, realDialThickness=0.08,
              style="LEDKnobLargeLR"),
            N("Label", "CtrlLabel", (63, 118, 66, 15), id=100027, style="StandardLabelCenteredStyle", text="Pitch"),
            N("Label", "CtrlLabel", (128, 118, 66, 15), id=100015, style="StandardLabelCenteredStyle", text="Resonance"),
            N("Label", "CtrlLabel", (-1, 118, 66, 15), id=100026, style="StandardLabelCenteredStyle", text="Input Drive"),
        ]),
    ]


@skin("Effects/FilterLadder.skin")
def filter_ladder():
    return [
        N("Panel", "CtrlPanel", (0, 0, 193, 150), id=0, style="EmptyPanelStyle", kids=[
            N("PopupButton", "Control2", (59, 16, 75, 17), id=100002, legacyClass="CtrlComboBox", optSingleClickEdit=True,
              style="LEDComboBoxStyle"),
            N("Label", "CtrlLabel", (51, 37, 91, 15), id=100010, style="StandardLabelCenteredStyle", text="Mode"),
            N("Knob", "Control3", (71, 68, 50, 46), displayName="Pitch", id=100021, realDialThickness=0.08,
              style="LEDKnobLargeLR"),
            N("Knob", "Control4", (136, 68, 50, 46), displayName="Resonance", id=100012, realDialThickness=0.08,
              style="LEDKnobLargeLR"),
            N("Knob", "Control1", (7, 68, 50, 46), displayName="Drive", id=100020, realDialThickness=0.08,
              style="LEDKnobLargeLR"),
            N("Label", "CtrlLabel", (63, 118, 66, 15), id=100027, style="StandardLabelCenteredStyle", text="Pitch"),
            N("Label", "CtrlLabel", (128, 118, 66, 15), id=100015, style="StandardLabelCenteredStyle", text="Resonance"),
            N("Label", "CtrlLabel", (-1, 118, 66, 15), id=100026, style="StandardLabelCenteredStyle", text="Input Drive"),
        ]),
    ]


@skin("Effects/FilterMS20.skin")
def filter_ms20():
    return [
        N("Panel", "CtrlPanel", (0, 0, 193, 150), id=0, style="EmptyPanelStyle", kids=[
            N("PopupButton", "Control2", (59, 16, 75, 17), id=100002, legacyClass="CtrlComboBox", optSingleClickEdit=True,
              style="LEDComboBoxStyle"),
            N("Label", "CtrlLabel", (51, 37, 91, 15), id=100010, style="StandardLabelCenteredStyle", text="Mode"),
            N("Knob", "Control3", (71, 68, 50, 46), displayName="Pitch", id=100021, realDialThickness=0.08,
              style="LEDKnobLargeLR"),
            N("Knob", "Control4", (136, 68, 50, 46), displayName="Resonance", id=100012, realDialThickness=0.08,
              style="LEDKnobLargeLR"),
            N("Knob", "Control1", (7, 68, 50, 46), displayName="Drive", id=100020, realDialThickness=0.08,
              style="LEDKnobLargeLR"),
            N("Label", "CtrlLabel", (63, 118, 66, 15), id=100027, style="StandardLabelCenteredStyle", text="Pitch"),
            N("Label", "CtrlLabel", (128, 118, 66, 15), id=100015, style="StandardLabelCenteredStyle", text="Resonance"),
            N("Label", "CtrlLabel", (-1, 118, 66, 15), id=100026, style="StandardLabelCenteredStyle", text="Input Drive"),
        ]),
    ]


@skin("Effects/FilterProdigy.skin")
def filter_prodigy():
    return [
        N("Panel", "CtrlPanel", (0, 0, 193, 150), id=0, style="EmptyPanelStyle", kids=[
            N("PopupButton", "Control2", (59, 16, 75, 17), id=100002, legacyClass="CtrlComboBox", optSingleClickEdit=True,
              style="LEDComboBoxStyle"),
            N("Label", "CtrlLabel", (51, 37, 91, 15), id=100010, style="StandardLabelCenteredStyle", text="Mode"),
            N("Knob", "Control3", (71, 68, 50, 46), displayName="Pitch", id=100021, realDialThickness=0.08,
              style="LEDKnobLargeLR"),
            N("Knob", "Control4", (136, 68, 50, 46), displayName="Resonance", id=100012, realDialThickness=0.08,
              style="LEDKnobLargeLR"),
            N("Knob", "Control1", (7, 68, 50, 46), displayName="Drive", id=100020, realDialThickness=0.08,
              style="LEDKnobLargeLR"),
            N("Label", "CtrlLabel", (63, 118, 66, 15), id=100027, style="StandardLabelCenteredStyle", text="Pitch"),
            N("Label", "CtrlLabel", (128, 118, 66, 15), id=100015, style="StandardLabelCenteredStyle", text="Resonance"),
            N("Label", "CtrlLabel", (-1, 118, 66, 15), id=100026, style="StandardLabelCenteredStyle", text="Input Drive"),
        ]),
    ]


@skin("Effects/Flanger.skin")
def flanger():
    return [
        N("Panel", "CtrlPanel", (0, 0, 193, 150), id=0, style="EmptyPanelStyle", kids=[
            N("Knob", "Control1", (21, 35, 24, 26), id=100022, realDialThickness=0.04, style="LEDKnobSmallLR"),
            N("Knob", "Control2", (82, 35, 24, 26), id=100023, realDialThickness=0.04, style="LEDKnobSmallLR"),
            N("Knob", "Control4", (144, 35, 24, 26), id=100025, realDialThickness=0.04, style="LEDKnobSmallLR"),
            N("Label", "CtrlLabel", (0, 64, 66, 13), id=100028, style="StandardLabelCenteredStyle", text="Frequency"),
            N("Label", "CtrlLabel", (60, 64, 66, 13), id=100029, style="StandardLabelCenteredStyle", text="Depth"),
            N("Label", "CtrlLabel", (123, 64, 66, 13), id=100031, style="StandardLabelCenteredStyle", text="Feedback"),
            N("Knob", "Control6", (21, 90, 24, 26), id=100021, realDialThickness=0.04, style="LEDKnobSmallLR"),
            N("Knob", "Control3", (82, 90, 24, 26), id=100024, realDialThickness=0.04, style="LEDKnobSmallLR"),
            N("Knob", "Control5", (144, 90, 24, 26), id=100020, realDialThickness=0.04, style="LEDKnobSmallLR"),
            N("Label", "CtrlLabel", (-2, 119, 66, 13), id=100027, style="StandardLabelCenteredStyle", text="Phase"),
            N("Label", "CtrlLabel", (59, 119, 66, 13), id=100030, style="StandardLabelCenteredStyle", text="Pos"),
            N("Label", "CtrlLabel", (122, 119, 66, 13), id=100026, style="StandardLabelCenteredStyle", text="Spread"),
        ]),
    ]


@skin("Effects/Freezer.skin")
def freezer():
    return [
        N("Panel", "CtrlPanel", (0, 0, 193, 150), id=0, style="EmptyPanelStyle", kids=[
            N("Knob", "Control3", (71, 3, 50, 46), id=100021, realDialThickness=0.08, style="LEDKnobLargeLR"),
            N("Knob", "Control2", (136, 3, 50, 46), id=100009, realDialThickness=0.08, style="LEDKnobLargeLR"),
            N("Button", "Control1", (24, 24, 16, 17), id=100000, optIsMomentary=False, style="LEDSmallLatchingButtonStyle",
              text=""),
            N("Label", "CtrlLabel", (-1, 53, 66, 15), id=100026, style="StandardLabelCenteredStyle", text="Gate"),
            N("Label", "CtrlLabel", (58, 53, 76, 15), id=100027, style="StandardLabelCenteredStyle", text="Grain Length"),
            N("Label", "CtrlLabel", (123, 53, 76, 15), id=100010, style="StandardLabelCenteredStyle", text="Loop Length"),
            N("Knob", "Control6", (7, 79, 50, 46), id=100013, realDialThickness=0.08, style="LEDKnobLargeLR"),
            N("Knob", "Control5", (71, 79, 50, 46), id=100011, realDialThickness=0.08, style="LEDKnobLargeLR"),
            N("Knob", "Control4", (136, 79, 50, 46), id=100012, realDialThickness=0.08, style="LEDKnobLargeLR"),
            N("Label", "CtrlLabel", (-1, 129, 66, 15), id=100016, style="StandardLabelCenteredStyle", text="Speed"),
            N("Label", "CtrlLabel", (63, 129, 66, 15), id=100014, style="StandardLabelCenteredStyle", text="Scratch"),
            N("Label", "CtrlLabel", (128, 129, 66, 15), id=100015, style="StandardLabelCenteredStyle", text="Smoothing"),
        ]),
    ]


@skin("Effects/Gain.skin")
def gain():
    return [
        N("Panel", "CtrlPanel", (0, 0, 193, 150), id=0, style="EmptyPanelStyle", kids=[
            N("Knob", "Control1", (71, 39, 50, 46), id=100014, realDialThickness=0.08, style="LEDKnobLargeLR"),
            N("Label", "CtrlLabel", (63, 90, 66, 15), id=100015, style="StandardLabelCenteredStyle", text="Gain"),
        ]),
    ]


@skin("Effects/ModFilter.skin")
def mod_filter():
    return [
        N("Panel", "CtrlPanel", (0, 0, 193, 150), id=0, style="EmptyPanelStyle", kids=[
            N("Knob", "Control4", (71, 79, 50, 46), id=100023, realDialThickness=0.08, style="LEDKnobLargeLR"),
            N("Knob", "Control5", (136, 79, 50, 46), id=100025, realDialThickness=0.08, style="LEDKnobLargeLR"),
            N("Knob", "Control6", (7, 79, 50, 46), id=100027, realDialThickness=0.08, style="LEDKnobLargeLR"),
            N("Label", "CtrlLabel", (63, 129, 66, 15), id=100024, style="StandardLabelCenteredStyle", text="Attack"),
            N("Label", "CtrlLabel", (128, 129, 66, 15), id=100026, style="StandardLabelCenteredStyle", text="Decay"),
            N("Label", "CtrlLabel", (-1, 129, 66, 15), id=100028, style="StandardLabelCenteredStyle", text="Depth"),
            N("PopupButton", "Control1", (10, 26, 40, 17), id=100017, legacyClass="CtrlComboBox", optSingleClickEdit=True,
              style="LEDSmallComboBoxStyle"),
            N("Knob", "Control2", (71, 10, 50, 46), id=100019, realDialThickness=0.08, style="LEDKnobLargeLR"),
            N("Knob", "Control3", (136, 10, 50, 46), id=100021, realDialThickness=0.08, style="LEDKnobLargeLR"),
            N("Label", "CtrlLabel", (2, 47, 56, 15), id=100018, style="StandardLabelCenteredStyle", text="Mode"),
            N("Label", "CtrlLabel", (63, 60, 66, 15), id=100020, style="StandardLabelCenteredStyle", text="Cutoff"),
            N("Label", "CtrlLabel", (128, 60, 66, 15), id=100022, style="StandardLabelCenteredStyle", text="Resonance"),
        ]),
    ]


@skin("Effects/NoiseGate.skin")
def noise_gate():
    return [
        N("Panel", "CtrlPanel", (0, 0, 193, 150), id=0, style="EmptyPanelStyle", kids=[
            N("Knob", "Control4", (21, 35, 24, 26), id=100012, realDialThickness=0.04, style="LEDKnobSmallLR"),
            N("Knob", "Control6", (82, 35, 24, 26), id=100013, realDialThickness=0.04, style="LEDKnobSmallLR"),
            N("Knob", "Control5", (144, 35, 24, 26), id=100011, realDialThickness=0.04, style="LEDKnobSmallLR"),
            N("Label", "CtrlLabel", (-1, 64, 66, 13), id=100015, style="StandardLabelCenteredStyle", text="Threshold"),
            N("Label", "CtrlLabel", (50, 64, 86, 13), id=100016, style="StandardLabelCenteredStyle", text="LPF"),
            N("Label", "CtrlLabel", (113, 64, 86, 13), id=100014, style="StandardLabelCenteredStyle", text="Hysteresis"),
            N("Knob", "Control1", (21, 90, 24, 26), id=100020, realDialThickness=0.04, style="LEDKnobSmallLR"),
            N("Knob", "Control3", (82, 90, 24, 26), id=100021, realDialThickness=0.04, style="LEDKnobSmallLR"),
            N("Knob", "Control2", (144, 90, 24, 26), id=100009, realDialThickness=0.04, style="LEDKnobSmallLR"),
            N("Label", "CtrlLabel", (-3, 119, 66, 13), id=100026, style="StandardLabelCenteredStyle", text="Attack"),
            N("Label", "CtrlLabel", (50, 119, 86, 13), id=100027, style="StandardLabelCenteredStyle", text="Release"),
            N("Label", "CtrlLabel", (112, 119, 86, 13), id=100010, style="StandardLabelCenteredStyle", text="Hold"),
        ]),
    ]


@skin("Effects/ParaEQ.skin")
def para_eq():
    return [
        N("Panel", "CtrlPanel", (0, 0, 193, 150), id=0, style="EmptyPanelStyle", kids=[
            N("Plot", "Plot", (43, 2, 107, 50), id=100000, style="EQGraphStyle"),
            N("Label", "CtrlLabel", (-2, 87, 66, 15), id=100014, style="StandardLabelCenteredStyle", text="High Freq"),
            N("Label", "CtrlLabel", (58, 87, 66, 15), id=100015, style="StandardLabelCenteredStyle", text="High Q"),
            N("Label", "CtrlLabel", (123, 87, 66, 15), id=100016, style="StandardLabelCenteredStyle", text="High Gain"),
            N("Knob", "Control5", (21, 57, 24, 26), id=100011, realDialThickness=0.04, style="LEDKnobSmallLR"),
            N("Knob", "Control4", (82, 57, 24, 26), id=100012, realDialThickness=0.04, style="LEDKnobSmallLR"),
            N("Knob", "Control6", (144, 57, 24, 26), id=100013, realDialThickness=0.04, style="LEDKnobSmallLR"),
            N("Knob", "Control2", (21, 107, 24, 26), id=100009, realDialThickness=0.04, style="LEDKnobSmallLR"),
            N("Knob", "Control1", (82, 107, 24, 26), id=100020, realDialThickness=0.04, style="LEDKnobSmallLR"),
            N("Knob", "Control3", (144, 107, 24, 26), id=100021, realDialThickness=0.04, style="LEDKnobSmallLR"),
            N("Label", "CtrlLabel", (-1, 137, 66, 15), id=100010, style="StandardLabelCenteredStyle", text="Low Freq"),
            N("Label", "CtrlLabel", (60, 137, 66, 15), id=100026, style="StandardLabelCenteredStyle", text="Low Q"),
            N("Label", "CtrlLabel", (123, 137, 66, 15), id=100027, style="StandardLabelCenteredStyle", text="Low Gain"),
        ]),
    ]


@skin("Effects/Phaser.skin")
def phaser():
    return [
        N("Panel", "CtrlPanel", (0, 0, 193, 150), id=0, style="EmptyPanelStyle", kids=[
            N("Knob", "Control1", (71, 10, 50, 46), displayName="Drive", id=100020, realDialThickness=0.08,
              style="LEDKnobLargeLR"),
            N("Knob", "Control5", (136, 10, 50, 46), id=100000, realDialThickness=0.08, style="LEDKnobLargeLR"),
            N("PopupButton", "Control2", (10, 26, 40, 17), id=100004, legacyClass="CtrlComboBox", optSingleClickEdit=True,
              style="LEDSmallComboBoxStyle"),
            N("Label", "CtrlLabel", (2, 47, 56, 15), id=100010, style="StandardLabelCenteredStyle", text="Mode"),
            N("Label", "CtrlLabel", (63, 60, 66, 15), id=100026, style="StandardLabelCenteredStyle", text="Input Drive"),
            N("Label", "CtrlLabel", (128, 60, 66, 15), id=100001, style="StandardLabelCenteredStyle", text="Frequency"),
            N("Knob", "Control6", (7, 79, 50, 46), id=100002, realDialThickness=0.08, style="LEDKnobLargeLR"),
            N("Knob", "Control3", (71, 79, 50, 46), displayName="Pitch", id=100021, realDialThickness=0.08,
              style="LEDKnobLargeLR"),
            N("Knob", "Control4", (136, 79, 50, 46), displayName="Resonance", id=100012, realDialThickness=0.08,
              style="LEDKnobLargeLR"),
            N("Label", "CtrlLabel", (-1, 129, 66, 15), id=100003, style="StandardLabelCenteredStyle", text="Depth"),
            N("Label", "CtrlLabel", (63, 129, 66, 15), id=100027, style="StandardLabelCenteredStyle", text="Pitch"),
            N("Label", "CtrlLabel", (128, 129, 66, 15), id=100015, style="StandardLabelCenteredStyle", text="Resonance"),
        ]),
    ]


@skin("Effects/RingSting.skin")
def ring_sting():
    return [
        N("Panel", "CtrlPanel", (0, 0, 193, 150), id=0, style="EmptyPanelStyle", kids=[
            N("PopupButton", "Control2", (59, 16, 75, 17), id=100007, legacyClass="CtrlComboBox", optSingleClickEdit=True,
              style="LEDComboBoxStyle"),
            N("Label", "CtrlLabel", (51, 37, 91, 15), id=100008, style="StandardLabelCenteredStyle", text="Mode"),
            N("Knob", "Control3", (36, 68, 50, 46), id=100020, realDialThickness=0.08, style="LEDKnobLargeLR"),
            N("Knob", "Control1", (106, 68, 50, 46), id=100021, realDialThickness=0.08, style="LEDKnobLargeLR"),
            N("Label", "CtrlLabel", (28, 121, 66, 15), id=100026, style="StandardLabelCenteredStyle", text="Pitch"),
            N("Label", "CtrlLabel", (98, 121, 66, 15), id=100027, style="StandardLabelCenteredStyle", text="Drive"),
        ]),
    ]


@skin("Effects/Shifter.skin")
def shifter():
    return [
        N("Panel", "CtrlPanel", (0, 0, 193, 150), id=0, style="EmptyPanelStyle", kids=[
            N("Knob", "Control1", (36, 39, 50, 46), id=100020, realDialThickness=0.08, style="LEDKnobLargeLR"),
            N("Knob", "Control2", (106, 39, 50, 46), id=100021, realDialThickness=0.08, style="LEDKnobLargeLR"),
            N("Label", "CtrlLabel", (28, 90, 66, 15), id=100026, style="StandardLabelCenteredStyle", text="Pitch"),
            N("Label", "CtrlLabel", (98, 90, 66, 15), id=100027, style="StandardLabelCenteredStyle", text="Amp"),
        ]),
    ]


@skin("Effects/StereoWidth.skin")
def stereo_width():
    return [
        N("Panel", "CtrlPanel", (0, 0, 193, 150), id=0, style="EmptyPanelStyle", kids=[
            N("Knob", "Control2", (106, 39, 50, 46), id=100001, realDialThickness=0.08, style="LEDKnobLargeLR"),
            N("Knob", "Control1", (36, 39, 50, 46), id=100014, realDialThickness=0.08, style="LEDKnobLargeLR"),
            N("Label", "CtrlLabel", (28, 90, 66, 15), id=100015, style="StandardLabelCenteredStyle", text="Gain"),
            N("Label", "CtrlLabel", (98, 90, 66, 15), id=100003, style="StandardLabelCenteredStyle", text="Width"),
        ]),
    ]


@skin("Effects/TinCan.skin")
def tin_can():
    return [
        N("Panel", "CtrlPanel", (0, 0, 193, 150), id=0, style="EmptyPanelStyle", kids=[
            N("Knob", "Control2", (71, 10, 50, 46), id=100019, realDialThickness=0.08, style="LEDKnobLargeLR"),
            N("Knob", "Control1", (136, 10, 50, 46), id=100021, realDialThickness=0.08, style="LEDKnobLargeLR"),
            N("Label", "CtrlLabel", (63, 60, 66, 15), id=100022, style="StandardLabelCenteredStyle", text="Size"),
            N("Label", "CtrlLabel", (128, 60, 66, 15), id=100020, style="StandardLabelCenteredStyle", text="Decay"),
            N("Knob", "Control5", (7, 79, 50, 46), id=100027, realDialThickness=0.08, style="LEDKnobLargeLR"),
            N("Knob", "Control3", (71, 79, 50, 46), id=100023, realDialThickness=0.08, style="LEDKnobLargeLR"),
            N("Knob", "Control4", (136, 79, 50, 46), id=100025, realDialThickness=0.08, style="LEDKnobLargeLR"),
            N("Label", "CtrlLabel", (-2, 129, 66, 15), id=100028, style="StandardLabelCenteredStyle", text="Squeeze"),
            N("Label", "CtrlLabel", (63, 129, 66, 15), id=100024, style="StandardLabelCenteredStyle", text="Damp"),
            N("Label", "CtrlLabel", (128, 129, 66, 15), id=100026, style="StandardLabelCenteredStyle", text="Pinch"),
        ]),
    ]


@skin("Effects/VariEQ.skin")
def vari_eq():
    return [
        N("Panel", "CtrlPanel", (0, 0, 193, 150), id=0, style="EmptyPanelStyle", kids=[
            N("Plot", "Plot", (43, 2, 107, 50), id=100000, style="EQGraphStyle"),
            N("PopupButton", "Control1", (59, 54, 75, 17), id=100000, legacyClass="CtrlComboBox", optSingleClickEdit=True,
              style="LEDComboBoxStyle"),
            N("Knob", "Control2", (7, 78, 50, 46), id=100020, realDialThickness=0.08, style="LEDKnobLargeLR"),
            N("Knob", "Control4", (71, 78, 50, 46), id=100011, realDialThickness=0.08, style="LEDKnobLargeLR"),
            N("Knob", "Control3", (136, 78, 50, 46), id=100021, realDialThickness=0.08, style="LEDKnobLargeLR"),
            N("Label", "CtrlLabel", (-1, 128, 66, 15), id=100026, style="StandardLabelCenteredStyle", text="Frequency"),
            N("Label", "GainLabel", (63, 128, 66, 15), id=100014, style="StandardLabelCenteredStyle", text="Gain"),
            N("Label", "CtrlLabel", (128, 128, 66, 15), id=100027, style="StandardLabelCenteredStyle", text="Q"),
        ]),
    ]
