"""The mixer: pad, engine, layer, aux and master strips and layouts (Mixer/)."""

from . import N, skin


@skin("Mixer/EngineMixerPanel.skin", origin="MIxer/EngineMixerPanel.skin")
def engine_mixer_panel():
    return [
        N("Panel", "CtrlPanel", (0, 0, 815, 395), id=0, style="EmptyPanelStyle", kids=[
            N("Panel", "Channel0", (51, 232, 97, 161), id=0, overloadClass="EditorEngineMixerChannel"),
            N("Panel", "Channel1", (146, 232, 97, 161), id=1, overloadClass="EditorEngineMixerChannel"),
            N("Panel", "Channel2", (241, 232, 97, 161), id=2, overloadClass="EditorEngineMixerChannel"),
            N("Panel", "Channel3", (336, 232, 97, 161), id=3, overloadClass="EditorEngineMixerChannel"),
            N("Panel", "Channel4", (431, 232, 97, 161), id=4, overloadClass="EditorEngineMixerChannel"),
            N("Panel", "Channel5", (526, 232, 97, 161), id=5, overloadClass="EditorEngineMixerChannel"),
            N("Panel", "Channel6", (621, 232, 97, 161), id=6, overloadClass="EditorEngineMixerChannel"),
            N("Panel", "Channel7", (716, 232, 97, 161), id=7, overloadClass="EditorEngineMixerChannel"),
            N("Panel", None, (38, 250, 12, 146), style="LcdSideTabSpineStyle"),
            N("ButtonGroup", "ModeGroup", (4, 248, 46, 150), id=100001, style="BasePanelStyle", kids=[
                N("Button", "MixButton", (0, 2, 34, 45), displayName="View mixer", id=100001, optIsMomentary=False,
                  style="LEDMMixButton", text="MIX"),
                N("Button", "SendButton", (0, 50, 34, 45), displayName="View Sends", id=100002, optIsMomentary=False,
                  style="LEDMSendButton", text="SEND"),
                N("Button", "SetupButton", (0, 98, 34, 45), displayName="View engine settings", id=100000, optIsMomentary=False,
                  style="LEDMSetupButton", text="SETUP"),
            ]),
            N("Panel", "FXPanel", (1, 2, 812, 245), id=100009, overloadClass="EditorEffectChainPanel"),
            N("Panel", "EngineGlue", (43, 232, 92, 11), id=0, legacyClass="CtrlImageView", style="GlueMediumImageStyle"),
            N("Panel", "GlueMicro", (808, 235, 3, 8), id=0, legacyClass="CtrlImageView", style="GlueMicroImageStyle"),
        ]),
    ]


@skin("Mixer/EngineMixerStrip.skin", origin="MIxer/EngineMixerStrip.skin")
def engine_mixer_strip():
    return [
        N("Panel", "CtrlPanel", (0, 0, 97, 161), id=0, style="EngineChannelBackgroundOdd", kids=[
            N("Panel", None, (0, 2, 97, 13), style="LcdStripHeadStyle"),
            N("Panel", "ClickPanel", (0, 0, 97, 161), id=0, style="EmptyPanelStyle", kids=[
                N("Label", "NameLabel", (-6, 17, 109, 13), displayName=" ", id=100011, style="StandardLabelCenteredStyle",
                  text="Engine X"),
                N("PanSlider", "PanKnob", (11, 49, 75, 12), displayName="Pan", id=100000, realTrackInset=0.0,
                  style="LargePanStyle"),
                N("Button", "MuteButton", (30, 29, 19, 19), displayName="Mute", id=100019, optIsMomentary=False,
                  style="LEDMuteButtonStyle", text="M"),
                N("Button", "SoloButton", (50, 29, 19, 19), displayName="Solo", id=100018, optIsMomentary=False,
                  style="LEDSoloButtonStyle", text="S"),
                N("Label", "EnginePitchLabel", (18, 85, 30, 13), id=100013, style="StandardLabelStyle", text="Pitch"),
                N("Knob", "EnginePitchKnob", (18, 97, 20, 20), displayName="Engine Pitch", id=100003, realDialThickness=0.04,
                  style="LEDKnobSmallLR"),
                N("Slider", "GainSlider", (41, 63, 15, 75), displayName="Engine Gain", id=100003, realTrackInset=0.0,
                  style="SmallVerticalFaderStyle"),
                N("LevelStrip", "VUMeterLeft", (58, 63, 3, 75), id=100016, style="VU75Dark"),
                N("LevelStrip", "VUMeterRight", (63, 63, 3, 75), id=100017, style="VU75Dark"),
                N("Label", "TempoLabel", (14, 126, 69, 15), id=100005, style="StandardLabelStyle", text="Tempo modifier"),
                N("ValueBox", "EngineNumBox", (16, 140, 30, 17), id=100001, optSingleClickEdit=False,
                  style="LEDSmallValueBoxCenteredStyle"),
                N("ValueBox", "EngineDenomBox", (51, 140, 30, 17), id=100002, optSingleClickEdit=False,
                  style="LEDSmallValueBoxCenteredStyle"),
                N("Label", "SwingDepthLabel", (58, 85, 30, 13), id=100013, style="StandardLabelStyle", text="Swing"),
                N("Label", "SwingLabel", (3, 50, 91, 15), id=100005, style="StandardLabelCenteredStyle", text="Swing Template"),
                N("Button", "EngineSwingTemplateCombo", (11, 65, 75, 17), id=100004, optIsMomentary=True,
                  renderer="PopupButtonRenderer", style="LEDLargeMenuButtonStyle"),
                N("Knob", "SwingKnob", (60, 97, 20, 20), displayName="Swing Template Depth", id=100000, realDialThickness=0.04,
                  style="LEDKnobSmallLR"),
                N("PopupButton", "OutputCombo", (11, 140, 75, 17), id=100010, legacyClass="CtrlComboBox",
                  optSingleClickEdit=True, style="LEDComboBoxStyle"),
                N("Label", "SendLabel0", (56, 49, 35, 17), id=100013, style="StandardLabelStyle", text="S1"),
                N("Label", "SendLabel1", (56, 72, 35, 17), id=100013, style="StandardLabelStyle", text="S2"),
                N("Label", "SendLabel2", (56, 95, 35, 17), id=100013, style="StandardLabelStyle", text="S3"),
                N("Label", "SendLabel3", (56, 118, 35, 17), id=100013, style="StandardLabelStyle", text="S4"),
                N("Knob", "SendKnob0", (35, 49, 17, 17), displayName="Send 1", id=0, realDialThickness=0.05,
                  style="LEDKnobMicroLR"),
                N("Knob", "SendKnob1", (35, 72, 17, 17), displayName="Send 2", id=1, realDialThickness=0.05,
                  style="LEDKnobMicroLR"),
                N("Knob", "SendKnob2", (35, 95, 17, 17), displayName="Send 3", id=2, realDialThickness=0.05,
                  style="LEDKnobMicroLR"),
                N("Knob", "SendKnob3", (35, 118, 17, 17), displayName="Send 4", id=3, realDialThickness=0.05,
                  style="LEDKnobMicroLR"),
            ]),
        ]),
    ]


@skin("Mixer/LayerMixerPanel.skin", origin="MIxer/LayerMixerPanel.skin")
def layer_mixer_panel():
    return [
        N("Panel", "CtrlPanel", (0, 0, 815, 395), id=0, style="EmptyPanelStyle", kids=[
            N("Panel", "Channel0", (51, 232, 97, 161), id=0, overloadClass="EditorLayerMixerChannel"),
            N("Panel", "Channel1", (146, 232, 97, 161), id=1, overloadClass="EditorLayerMixerChannel"),
            N("Panel", "Channel2", (241, 232, 97, 161), id=2, overloadClass="EditorLayerMixerChannel"),
            N("Panel", "Channel3", (336, 232, 97, 161), id=3, overloadClass="EditorLayerMixerChannel"),
            N("Panel", "Channel4", (431, 232, 97, 161), id=4, overloadClass="EditorLayerMixerChannel"),
            N("Panel", "Channel5", (526, 232, 97, 161), id=5, overloadClass="EditorLayerMixerChannel"),
            N("Panel", "Channel6", (621, 232, 97, 161), id=6, overloadClass="EditorLayerMixerChannel"),
            N("Panel", "Channel7", (716, 232, 97, 161), id=7, overloadClass="EditorLayerMixerChannel"),
            N("Button", "AllLayers", (4, 248, 46, 15), displayName="Apply changes to all layers of this pad", id=100002,
              optIsMomentary=False, style="LinkLargeButtonStyle", text="Layers"),
            N("Button", "AllPads", (4, 264, 46, 15), displayName="Apply changes to all selected layers on all pads (Alt)",
              id=100000, optIsMomentary=False, style="LinkLargeButtonStyle", text="Pads"),
            N("Button", "AllGroups", (4, 280, 46, 15),
              displayName="Apply changes to all selected layers on all pads in this group (Alt + Shift)", id=100001,
              optIsMomentary=False, style="LinkLargeButtonStyle", text="Row"),
            N("Panel", None, (38, 298, 12, 49), style="LcdSideTabSpineStyle"),
            N("Button", "MixButton", (4, 298, 34, 45), displayName="View mixer", id=100001, optIsMomentary=False,
              style="LEDMixButton", text="MIX"),
            N("Panel", None, (38, 346, 12, 49), style="LcdSideTabSpineStyle"),
            N("Button", "SendButton", (4, 346, 34, 45), displayName="View aux sends", id=100002, optIsMomentary=False,
              style="LEDSendButton", text="SEND"),
            N("Panel", "FXPanel", (1, 2, 812, 245), id=100009, overloadClass="EditorEffectChainPanel"),
            N("Panel", "LayerGlue", (52, 233, 92, 11), id=0, legacyClass="CtrlImageView", style="GlueMediumImageStyle"),
            N("Panel", "GlueMicro", (808, 235, 3, 8), id=0, legacyClass="CtrlImageView", style="GlueMicroImageStyle"),
        ]),
    ]


@skin("Mixer/LayerMixerStrip.skin", origin="MIxer/LayerMixerStrip.skin")
def layer_mixer_strip():
    return [
        N("Panel", "CtrlPanel", (0, 0, 97, 161), id=0, style="LayerChannelBackgroundOdd", kids=[
            N("Panel", None, (0, 2, 97, 13), style="LcdStripHeadStyle"),
            N("Label", "NameLabel", (-6, 17, 109, 13), displayName=" ", id=100011, overloadClass="AutoShorteningLabel",
              style="StandardLabelCenteredStyle", text="Layer X"),
            N("PanSlider", "PanKnob", (11, 49, 75, 12), displayName="Pan", id=100000, realTrackInset=0.0, style="LargePanStyle"),
            N("Button", "MuteButton", (30, 29, 19, 19), displayName="Mute", id=100019, optIsMomentary=False,
              style="LEDMuteButtonStyle", text="M"),
            N("Button", "SoloButton", (50, 29, 19, 19), displayName="Solo", id=100018, optIsMomentary=False,
              style="LEDSoloButtonStyle", text="S"),
            N("Slider", "GainSlider", (41, 63, 15, 75), displayName="Gain", id=100003, realTrackInset=0.0,
              style="SmallVerticalFaderStyle"),
            N("LevelStrip", "VUMeterLeft", (58, 63, 3, 75), id=100016, style="VU75Dark"),
            N("LevelStrip", "VUMeterRight", (63, 63, 3, 75), id=100017, style="VU75Dark"),
            N("PopupButton", "OutputCombo", (11, 140, 75, 17), id=100010, legacyClass="CtrlComboBox", optSingleClickEdit=True,
              style="LEDComboBoxStyle"),
            N("Label", "SendLabel0", (56, 49, 35, 17), id=100013, style="StandardLabelStyle", text="S1"),
            N("Label", "SendLabel1", (56, 72, 35, 17), id=100013, style="StandardLabelStyle", text="S2"),
            N("Label", "SendLabel2", (56, 95, 35, 17), id=100013, style="StandardLabelStyle", text="S3"),
            N("Label", "SendLabel3", (56, 118, 35, 17), id=100013, style="StandardLabelStyle", text="S4"),
            N("Knob", "SendKnob0", (35, 49, 17, 17), displayName="Send 1", id=0, realDialThickness=0.05, style="LEDKnobMicroLR"),
            N("Knob", "SendKnob1", (35, 72, 17, 17), displayName="Send 2", id=1, realDialThickness=0.05, style="LEDKnobMicroLR"),
            N("Knob", "SendKnob2", (35, 95, 17, 17), displayName="Send 3", id=2, realDialThickness=0.05, style="LEDKnobMicroLR"),
            N("Knob", "SendKnob3", (35, 118, 17, 17), displayName="Send 4", id=3, realDialThickness=0.05, style="LEDKnobMicroLR"),
        ]),
    ]


@skin("Mixer/MixerAux.skin", origin="MIxer/MixerAux.skin")
def mixer_aux():
    return [
        N("Panel", "CtrlPanel", (0, 0, 193, 161), id=0, style="AuxChannelBackgroundOdd", kids=[
            N("Panel", None, (0, 2, 193, 13), style="LcdStripHeadStyle"),
            N("Panel", "ClickPanel", (0, 0, 193, 161), id=0, style="EmptyPanelStyle", kids=[
                N("Label", "NameLabel", (-6, 17, 205, 13), id=100011, style="StandardLabelCenteredStyle", text="Global aux"),
                N("Button", "MuteButton", (17, 29, 19, 19), displayName="Mute", id=100019, optIsMomentary=False,
                  style="LEDMuteButtonStyle", text="M"),
                N("Button", "SoloButton", (37, 29, 19, 19), displayName="Solo", id=100018, optIsMomentary=False,
                  style="LEDSoloButtonStyle", text="S"),
                N("ButtonGroup", "PrePostButton", (64, 30, 120, 17), id=100000, style="BasePanelStyle", kids=[
                    N("Button", "PreButton", (0, 0, 60, 17), displayName="Pre fade", id=100001, optIsMomentary=False,
                      style="RadioButtonLeftStyle", text="Pre"),
                    N("Button", "PostButton", (58, 0, 60, 17), displayName="Post fade", id=100002, optIsMomentary=False,
                      style="RadioButtonRightStyle", text="Post"),
                ]),
                N("Slider", "GainSlider", (89, 63, 15, 75), displayName="Gain", id=100012, realTrackInset=0.0,
                  style="SmallVerticalFaderStyle"),
                N("LevelStrip", "VUMeterLeft", (106, 63, 3, 75), id=100000, style="VU75Dark"),
                N("LevelStrip", "VUMeterRight", (111, 63, 3, 75), id=100001, style="VU75Dark"),
                N("PopupButton", "OutputCombo", (59, 140, 75, 17), id=100010, legacyClass="CtrlComboBox",
                  optSingleClickEdit=True, style="LEDComboBoxStyle"),
            ]),
        ]),
    ]


@skin("Mixer/MixerMaster.skin", origin="MIxer/MixerMaster.skin")
def mixer_master():
    return [
        N("Panel", "CtrlPanel", (0, 0, 48, 161), id=0, style="MasterChannelBackground", kids=[
            N("Panel", None, (0, 2, 48, 13), style="LcdStripHeadStyle"),
            N("Panel", "ClickPanel", (0, 0, 48, 161), id=0, style="EmptyPanelStyle", kids=[
                N("Label", "NameLabel", (-8, 17, 62, 13), id=100011, style="StandardLabelCenteredStyle", text="Master"),
                N("Slider", "GainSlider", (12, 63, 15, 75), displayName="Gain", id=100012, realTrackInset=0.0,
                  style="SmallVerticalFaderStyle"),
                N("LevelStrip", "VUMeterLeft", (29, 63, 3, 75), id=100000, style="VU75Dark"),
                N("LevelStrip", "VUMeterRight", (34, 63, 3, 75), id=100001, style="VU75Dark"),
            ]),
        ]),
    ]


@skin("Mixer/MixerPanelGlobalLayout.skin", origin="MIxer/MixerPanelGlobalLayout.skin")
def mixer_panel_global_layout():
    return [
        N("Panel", "CtrlPanel", (0, 0, 815, 395), id=0, style="EmptyPanelStyle", kids=[
            N("Panel", "AuxChannel0", (1, 232, 193, 161), id=100001, overloadClass="EditorMixerAuxChannel"),
            N("Panel", "AuxChannel1", (192, 232, 193, 161), id=100003, overloadClass="EditorMixerAuxChannel"),
            N("Panel", "AuxChannel2", (383, 232, 193, 161), id=100004, overloadClass="EditorMixerAuxChannel"),
            N("Panel", "AuxChannel3", (574, 232, 193, 161), id=100005, overloadClass="EditorMixerAuxChannel"),
            N("Panel", "MasterChannel", (765, 232, 48, 161), id=100008, overloadClass="EditorMixerMasterChannel"),
            N("Panel", "FXPanel", (1, 2, 812, 245), id=100009, overloadClass="EditorEffectChainPanel"),
            N("Panel", "LargeGlue", (1, 232, 189, 11), id=0, legacyClass="CtrlImageView", style="GlueLargeImageStyle"),
            N("Panel", "MasterGlue", (767, 235, 44, 15), id=0, legacyClass="CtrlImageView", style="GlueMasterImageStyle"),
            N("Panel", "GlueMicro", (3, 235, 3, 8), id=0, legacyClass="CtrlImageView", style="GlueMicroImageStyle"),
        ]),
    ]


@skin("Mixer/MixerPanelPadLayout.skin", origin="MIxer/MixerPanelPadLayout.skin")
def mixer_panel_pad_layout():
    return [
        N("Panel", "CtrlPanel", (0, 0, 815, 395), id=0, style="EmptyPanelStyle", kids=[
            N("Panel", "PadChan0", (60, 232, 48, 161), id=0, overloadClass="EditorMixerPadChannel"),
            N("Panel", "PadChan1", (107, 232, 48, 161), id=1, overloadClass="EditorMixerPadChannel"),
            N("Panel", "PadChan2", (154, 232, 48, 161), id=2, overloadClass="EditorMixerPadChannel"),
            N("Panel", "PadChan3", (201, 232, 48, 161), id=3, overloadClass="EditorMixerPadChannel"),
            N("Panel", "PadChan4", (248, 232, 48, 161), id=4, overloadClass="EditorMixerPadChannel"),
            N("Panel", "PadChan5", (295, 232, 48, 161), id=5, overloadClass="EditorMixerPadChannel"),
            N("Panel", "PadChan6", (342, 232, 48, 161), id=6, overloadClass="EditorMixerPadChannel"),
            N("Panel", "PadChan7", (389, 232, 48, 161), id=7, overloadClass="EditorMixerPadChannel"),
            N("Panel", "PadChan8", (436, 232, 48, 161), id=8, overloadClass="EditorMixerPadChannel"),
            N("Panel", "PadChan9", (483, 232, 48, 161), id=9, overloadClass="EditorMixerPadChannel"),
            N("Panel", "PadChan10", (530, 232, 48, 161), id=10, overloadClass="EditorMixerPadChannel"),
            N("Panel", "PadChan11", (577, 232, 48, 161), id=11, overloadClass="EditorMixerPadChannel"),
            N("Panel", "PadChan12", (624, 232, 48, 161), id=12, overloadClass="EditorMixerPadChannel"),
            N("Panel", "PadChan13", (671, 232, 48, 161), id=13, overloadClass="EditorMixerPadChannel"),
            N("Panel", "PadChan14", (718, 232, 48, 161), id=14, overloadClass="EditorMixerPadChannel"),
            N("Panel", "PadChan15", (765, 232, 48, 161), id=15, overloadClass="EditorMixerPadChannel"),
            N("Button", "AllPads", (13, 264, 46, 15), displayName="Apply changes to all pads (Alt)", id=100000,
              optIsMomentary=False, style="LinkLargeButtonStyle", text="Pads"),
            N("Button", "AllGroups", (13, 280, 46, 15), displayName="Apply changes to all pads in this group (Alt + Shift)",
              id=100001, optIsMomentary=False, style="LinkLargeButtonStyle", text="Row"),
            N("Panel", None, (47, 298, 12, 49), style="LcdSideTabSpineStyle"),
            N("Button", "MixButton", (13, 298, 34, 45), displayName="View mixer", id=100001, optIsMomentary=False,
              style="LEDMixButton", text="MIX"),
            N("Panel", None, (47, 346, 12, 49), style="LcdSideTabSpineStyle"),
            N("Button", "SendButton", (13, 346, 34, 45), displayName="View aux sends", id=100002, optIsMomentary=False,
              style="LEDSendButton", text="SEND"),
            N("Panel", "FXPanel", (1, 2, 812, 245), id=100009, overloadClass="EditorEffectChainPanel"),
            N("Panel", "PadGlue", (61, 232, 44, 11), id=0, legacyClass="CtrlImageView", style="GlueSmallImageStyle"),
            N("Panel", "PadGlueMicro", (808, 231, 3, 8), id=0, legacyClass="CtrlImageView", style="GlueMicroImageStyle"),
        ]),
    ]


@skin("Mixer/MixerStripPad.skin", origin="MIxer/MixerStripPad.skin")
def mixer_strip_pad():
    return [
        N("Panel", "CtrlPanel", (0, 0, 48, 161), id=0, style="PadChannelBackgroundOdd", kids=[
            N("Panel", None, (0, 2, 48, 13), style="LcdStripHeadStyle"),
            N("Label", "NameLabel", (-8, 17, 62, 13), displayName="Sample name", id=100011, overloadClass="AutoShorteningLabel",
              style="StandardLabelCenteredStyle", text="Pad X"),
            N("PanSlider", "PanKnob", (4, 49, 41, 12), displayName="Pan", id=100000, realTrackInset=0.0, style="SmallPanStyle"),
            N("Button", "MuteButton", (5, 29, 19, 19), displayName="Mute", id=100019, optIsMomentary=False,
              style="LEDMuteButtonStyle", text="M"),
            N("Button", "SoloButton", (25, 29, 19, 19), displayName="Solo", id=100018, optIsMomentary=False,
              style="LEDSoloButtonStyle", text="S"),
            N("Slider", "GainSlider", (17, 63, 15, 75), displayName="Gain", id=100003, realTrackInset=0.0,
              style="SmallVerticalFaderStyle"),
            N("LevelStrip", "VUMeterLeft", (34, 63, 3, 75), id=100016, style="VU75Dark"),
            N("LevelStrip", "VUMeterRight", (39, 63, 3, 75), id=100017, style="VU75Dark"),
            N("Label", "SendLabel0", (33, 49, 35, 17), id=100013, style="StandardLabelStyle", text="S1"),
            N("Label", "SendLabel1", (33, 72, 35, 17), id=100013, style="StandardLabelStyle", text="S2"),
            N("Label", "SendLabel2", (33, 95, 35, 17), id=100013, style="StandardLabelStyle", text="S3"),
            N("Label", "SendLabel3", (33, 118, 35, 17), id=100013, style="StandardLabelStyle", text="S4"),
            N("Knob", "Send0", (13, 49, 17, 17), displayName="Send 1", id=0, realDialThickness=0.05, style="LEDKnobMicroLR"),
            N("Knob", "Send1", (13, 72, 17, 17), displayName="Send 2", id=1, realDialThickness=0.05, style="LEDKnobMicroLR"),
            N("Knob", "Send2", (13, 95, 17, 17), displayName="Send 3", id=2, realDialThickness=0.05, style="LEDKnobMicroLR"),
            N("Knob", "Send3", (13, 118, 17, 17), displayName="Send 4", id=3, realDialThickness=0.05, style="LEDKnobMicroLR"),
            N("PopupButton", "OutputCombo", (4, 140, 40, 17), id=100010, legacyClass="CtrlComboBox", optSingleClickEdit=True,
              style="LEDSmallComboBoxStyle"),
        ]),
    ]
