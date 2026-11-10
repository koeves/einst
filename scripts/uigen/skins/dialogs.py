"""Dialogs and the master / utility strips: export, import, find files, options (Global/)."""

from . import N, skin


@skin("Global/ExportPanel.skin")
def export_panel():
    return [
        N("Panel", "CtrlPanel", (7, 7, 435, 264), id=0, style="ModalPanelStyle", kids=[
            N("Label", "CtrlLabel", (7, 5, 97, 15), id=100003, style="ModalLabelStyle", text="Output Source"),
            N("Label", "CtrlLabel", (144, 5, 97, 15), id=100003, style="ModalLabelStyle", text="Output Files"),
            N("Panel", "CtrlPanel", (6, 22, 130, 75), id=100005, style="ModalPanelCollectionStyle", kids=[
                N("ButtonGroup", "SourceButtonGroup", (107, 2, 23, 78), id=100000, style="BasePanelStyle", kids=[
                    N("Button", "CtrlButton", (5, 5, 14, 15), id=100001, optIsMomentary=False,
                      renderer="CheckBoxButtonRenderer", style="ModalCheckButtonStyle", text=""),
                    N("Button", "CtrlButton", (5, 29, 14, 15), id=100001, optIsMomentary=False,
                      renderer="CheckBoxButtonRenderer", style="ModalCheckButtonStyle", text=""),
                    N("Button", "CtrlButton", (5, 53, 14, 15), id=100001, optIsMomentary=False,
                      renderer="CheckBoxButtonRenderer", style="ModalCheckButtonStyle", text=""),
                ]),
                N("Label", "CtrlLabel", (-51, 8, 160, 15), id=100002, style="ModalLabelRightAlignedStyle",
                  text="All Engine Patterns"),
                N("Label", "CtrlLabel", (-71, 32, 180, 15), id=100002, style="ModalLabelRightAlignedStyle",
                  text="Current Engine Pattern"),
                N("Label", "CtrlLabel", (-51, 56, 160, 15), id=100002, style="ModalLabelRightAlignedStyle", text="Song"),
            ]),
            N("Panel", "CtrlPanel", (143, 22, 285, 75), id=100006, style="ModalPanelCollectionStyle", kids=[
                N("ButtonGroup", "OutputTypeButtonGroup", (160, 2, 23, 78), id=100000, style="BasePanelStyle", kids=[
                    N("Button", "CtrlButton", (5, 5, 14, 15), id=100001, optIsMomentary=False,
                      renderer="CheckBoxButtonRenderer", style="ModalCheckButtonStyle", text=""),
                    N("Button", "CtrlButton", (5, 29, 14, 15), id=100001, optIsMomentary=False,
                      renderer="CheckBoxButtonRenderer", style="ModalCheckButtonStyle", text=""),
                    N("Button", "CtrlButton", (5, 53, 14, 15), id=100001, optIsMomentary=False,
                      renderer="CheckBoxButtonRenderer", style="ModalCheckButtonStyle", text=""),
                ]),
                N("Label", "CtrlLabel", (22, 8, 140, 15), id=100002, style="ModalLabelRightAlignedStyle", text="Stereo Mixdown"),
                N("PopupButton", "SingleOutputComboBox", (181, 31, 100, 17), id=100004, legacyClass="CtrlComboBox",
                  optSingleClickEdit=True, style="ModalComboBoxStyle"),
                N("Label", "CtrlLabel", (32, 32, 130, 15), id=100002, style="ModalLabelRightAlignedStyle", text="Single Output"),
                N("Label", "CtrlLabel", (-61, 56, 223, 15), id=100002, style="ModalLabelRightAlignedStyle",
                  text="All assigned outputs (multiple files)"),
            ]),
            N("Panel", "CtrlPanel", (78, 112, 275, 60), id=100006, style="ModalPanelCollectionStyle", kids=[
                N("Label", "CtrlLabel", (-56, 7, 160, 17), id=100003, style="ModalLabelRightAlignedStyle",
                  text="Normalization mode"),
                N("PopupButton", "NormalizeComboBox", (106, 7, 156, 17), id=100002, legacyClass="CtrlComboBox",
                  optSingleClickEdit=True, style="ModalComboBoxStyle"),
                N("Label", "CtrlLabel", (7, 28, 97, 17), id=100003, style="ModalLabelStyle", text="Tail Length (seconds)"),
                N("TextBox", "TailTextBox", (106, 28, 50, 17), id=100004, optSingleClickEdit=True, style="ModalEditBoxStyle"),
                N("Button", "LoopButton", (159, 28, 50, 17), id=100005, optIsMomentary=False, style="ModalLatchingButtonStyle",
                  text="Loop"),
                N("Button", "AutoCropButton", (212, 28, 50, 17), id=100005, optIsMomentary=False,
                  style="ModalLatchingButtonStyle", text="AutoCrop"),
            ]),
            N("Label", "ProgressLabel", (-2, 186, 437, 15), id=100001, style="ModalLabelCenteredStyle", text="Progress"),
            N("ProgressBar", "ProgressBar", (6, 202, 421, 24), id=100002, legacyClass="VProgressBar", style="ModalProgressStyle"),
            N("Button", "CancelButton", (168, 239, 50, 17), id=100003, optIsMomentary=True, style="ModalMomentaryButtonStyle",
              text="Cancel"),
            N("Button", "StartButton", (222, 239, 50, 17), id=100003, optIsMomentary=True, style="ModalMomentaryButtonStyle",
              text="Export"),
        ]),
    ]


@skin("Global/FileFindPanel.skin")
def file_find_panel():
    return [
        N("Panel", "CtrlPanel", (0, 0, 390, 410), id=0, style="ModalPanelStyle", kids=[
            N("Panel", "HoldingPanel", (0, 0, 390, 410), id=0, style="EmptyPanelStyle", kids=[
                N("Label", "CtrlLabel", (20, 2, 350, 20), id=100003, style="ModalLabelStyle",
                  text="The listed audio files could not be found whilst loading"),
                N("ListBox", "FileList", (3, 24, 382, 350), id=100002, style="FindFileListBoxStyle"),
                N("Button", "IgnoreButton", (115, 383, 50, 17), id=100004, optIsMomentary=True,
                  style="ModalMomentaryButtonStyle", text="Close"),
                N("Button", "ManualButton", (168, 383, 50, 17), id=100006, optIsMomentary=True,
                  style="ModalMomentaryButtonStyle", text="Manual"),
                N("Button", "AutoButton", (221, 383, 50, 17), id=100005, optIsMomentary=True, style="ModalMomentaryButtonStyle",
                  text="Auto"),
            ]),
            N("Panel", "ScanningPanel", (19, 24, 350, 60), id=0, style="EmptyPanelStyle", kids=[
                N("Label", "CtrlLabel", (3, 2, 85, 20), id=100004, style="ModalLabelStyle", text="Currently scanning"),
                N("Label", "ScanLabel", (98, 2, 250, 20), id=100003, style="ModalLabelStyle", text=""),
                N("Button", "AbortButton", (143, 31, 50, 17), id=100002, optIsMomentary=True, style="ModalMomentaryButtonStyle",
                  text="Abort"),
            ]),
        ]),
    ]


@skin("Global/ImportPanel.skin")
def import_panel():
    return [
        N("Panel", "CtrlPanel", (0, 0, 400, 175), id=0, style="ModalPanelStyle", kids=[
            N("Label", "Label", (-8, 16, 416, 50), id=100001, style="ModalLargeLabelCenteredNotificationStyle",
              text="Converting file. Please wait..."),
            N("Knob", "Spinner", (167, 90, 50, 50), id=100002, realDialThickness=0.1, realSpinnerArrow=0.26,
              realSpinnerInset=4.5, style="LargeSpinnerStyle"),
        ]),
    ]


@skin("Global/MasterControls.skin")
def master_controls():
    return [
        N("Panel", "CtrlPanel", (0, 0, 135, 156), id=0, style="EmptyPanelStyle", kids=[
            N("Panel", "BackStat", (2, 2, 132, 57), id=0, legacyClass="CtrlImageView", style="StatusStripStyle"),
            N("LevelStrip", "MasterVULeft", (8, 22, 120, 3), id=100012, style="HardVU120"),
            N("LevelStrip", "MasterVURight", (8, 26, 120, 3), id=100013, style="HardVU120"),
            N("Button", "MIDIIndi", (30, 32, 40, 8), displayName="MIDI activity", id=100001, optIsMomentary=True,
              style="MIDIIndiStyle", text="● MIDI"),
            N("Button", "VoiceIndi", (72, 32, 40, 8), displayName="Indicates a voice has been stolen", id=100001,
              optIsMomentary=True, style="VoiceIndiStyle", text="● VOICE"),
            N("Button", "TapBox", (5, 40, 38, 16), id=1, legacyClass="Ctrl", overloadClass="TapTempoBox",
              style="ReadoutMenuStyle", text=""),
            N("ValueBox", "BPMTextBox", (42, 40, 44, 16), id=100003, overloadClass="TempoBox", style="ReadoutStyle"),
            N("Label", "MemoryUsageBox", (81, 40, 54, 16), displayName="Memory usage", id=100006, style="GlobalLabelStyle",
              text=""),
            N("Panel", None, (5, 72, 55, 55), style="HardKnobHaloStyle"),
            N("Panel", None, (11, 104, 43, 34), style="HardKnobShadowStyle"),
            N("Panel", None, (10, 77, 45, 45), style="HardKnobRingStyle"),
            N("Panel", None, (10, 121, 3, 3), style="HardKnobMarkStyle"),
            N("Panel", None, (52, 121, 3, 3), style="HardKnobMarkStyle"),
            N("Knob", "MasterPitch", (11, 78, 43, 43), displayName="Master pitch", id=100000, optCentered=True,
              realDialThickness=0.047, style="HardKnobLargeC"),
            N("Panel", None, (72, 72, 55, 55), style="HardKnobHaloStyle"),
            N("Panel", None, (78, 104, 43, 34), style="HardKnobShadowStyle"),
            N("Panel", None, (77, 77, 45, 45), style="HardKnobRingStyle"),
            N("Panel", None, (77, 121, 3, 3), style="HardKnobMarkStyle"),
            N("Panel", None, (119, 121, 3, 3), style="HardKnobMarkStyle"),
            N("Knob", "MasterSlider", (78, 78, 43, 43), displayName="Master gain", id=100000, realDialThickness=0.047,
              style="HardKnobLargeLR"),
            N("Label", "CtrlImageView", (28, 136, 12, 11), id=100000, legacyClass="CtrlImageView",
              style="ChassisGlyphLabelStyle", text="", kids=[
                N("Panel", None, (4, 0, 1, 8), style="ChassisGlyphBarStyle"),
                N("Panel", None, (0, 6, 5, 4), style="ChassisGlyphDotStyle"),
                N("Panel", None, (9, 1, 2, 1), style="ChassisGlyphBarStyle"),
                N("Panel", None, (8, 2, 3, 1), style="ChassisGlyphBarStyle"),
                N("Panel", None, (7, 3, 5, 1), style="ChassisGlyphBarStyle"),
                N("Panel", None, (9, 8, 2, 1), style="ChassisGlyphBarStyle"),
                N("Panel", None, (8, 7, 3, 1), style="ChassisGlyphBarStyle"),
                N("Panel", None, (7, 6, 5, 1), style="ChassisGlyphBarStyle"),
            ]),
            N("Label", "CtrlImageView", (95, 136, 12, 10), id=100001, legacyClass="CtrlImageView",
              style="ChassisGlyphLabelStyle", text="", kids=[
                N("Panel", None, (0, 9, 1, 1), style="ChassisGlyphBarStyle"),
                N("Panel", None, (1, 8, 1, 2), style="ChassisGlyphBarStyle"),
                N("Panel", None, (2, 8, 1, 2), style="ChassisGlyphBarStyle"),
                N("Panel", None, (3, 7, 1, 3), style="ChassisGlyphBarStyle"),
                N("Panel", None, (4, 6, 1, 4), style="ChassisGlyphBarStyle"),
                N("Panel", None, (5, 6, 1, 4), style="ChassisGlyphBarStyle"),
                N("Panel", None, (6, 5, 1, 5), style="ChassisGlyphBarStyle"),
                N("Panel", None, (7, 4, 1, 6), style="ChassisGlyphBarStyle"),
                N("Panel", None, (8, 3, 1, 7), style="ChassisGlyphBarStyle"),
                N("Panel", None, (9, 2, 1, 8), style="ChassisGlyphBarStyle"),
                N("Panel", None, (10, 2, 1, 8), style="ChassisGlyphBarStyle"),
                N("Panel", None, (11, 1, 1, 9), style="ChassisGlyphBarStyle"),
            ]),
        ]),
    ]


@skin("Global/OptionsPanel.skin")
def options_panel():
    return [
        N("Panel", "CtrlPanel", (437, 155, 405, 450), id=0, style="ModalPanelStyle", kids=[
            N("Panel", "CtrlPanel", (8, 6, 389, 365), id=100000, style="ModalPanelCollectionStyle", kids=[
                N("Button", "InvertButton", (211, 9, 14, 15), id=100000, optIsMomentary=False,
                  renderer="CheckBoxButtonRenderer", style="ModalCheckButtonStyle", text=""),
                N("Label", "CtrlLabel", (-54, 10, 260, 15), id=100001, style="ModalLabelRightAlignedStyle",
                  text="Invert Y axis (zoom) in wave displays"),
                N("Button", "InvertXButton", (211, 31, 14, 15), id=100000, optIsMomentary=False,
                  renderer="CheckBoxButtonRenderer", style="ModalCheckButtonStyle", text=""),
                N("Label", "CtrlLabel", (-54, 32, 260, 15), id=100001, style="ModalLabelRightAlignedStyle",
                  text="Invert X axis (scroll) in wave displays"),
                N("Button", "MultiButton", (211, 53, 14, 15), id=100000, optIsMomentary=False,
                  renderer="CheckBoxButtonRenderer", style="ModalCheckButtonStyle", text=""),
                N("Label", "CtrlLabel", (1, 54, 205, 15), id=100001, style="ModalLabelRightAlignedStyle",
                  text="Multitool Right click opens menu"),
                N("Button", "ForceModeButton", (211, 75, 14, 15), id=100000, optIsMomentary=False,
                  renderer="CheckBoxButtonRenderer", style="ModalCheckButtonStyle", text=""),
                N("Label", "CtrlLabel", (-54, 76, 260, 15), id=100001, style="ModalLabelRightAlignedStyle",
                  text="Forced layers fall through if none found"),
                N("Button", "AutoAssign", (211, 97, 14, 15), id=100000, optIsMomentary=False, renderer="CheckBoxButtonRenderer",
                  style="ModalCheckButtonStyle", text=""),
                N("Label", "CtrlLabel", (-44, 98, 250, 15), id=100001, style="ModalLabelRightAlignedStyle",
                  text="Auto assign MIDI CC's to graphs for export"),
                N("Button", "QuantizeREXButton", (211, 119, 14, 15), id=100001, optIsMomentary=False,
                  renderer="CheckBoxButtonRenderer", style="ModalCheckButtonStyle", text=""),
                N("Label", "CtrlLabel", (41, 120, 165, 15), id=100002, style="ModalLabelRightAlignedStyle",
                  text="Quantize loops on load"),
                N("Button", "PreFadeButton", (211, 141, 14, 15), id=100000, optIsMomentary=False,
                  renderer="CheckBoxButtonRenderer", style="ModalCheckButtonStyle", text=""),
                N("Label", "CtrlLabel", (11, 142, 195, 15), id=100001, style="ModalLabelRightAlignedStyle",
                  text="Auxes are pre fade by default"),
                N("Button", "ResetLayersButton", (211, 163, 14, 15), id=100000, optIsMomentary=False,
                  renderer="CheckBoxButtonRenderer", style="ModalCheckButtonStyle", text=""),
                N("Label", "CtrlLabel", (-4, 164, 210, 15), id=100001, style="ModalLabelRightAlignedStyle",
                  text="Reset layer settings on load"),
                N("Label", "CtrlLabel", (-4, 185, 210, 15), id=100001, style="ModalLabelRightAlignedStyle",
                  text="Send Active Sensing Messages"),
                N("Button", "ActiveSensingButton", (211, 185, 14, 15), id=100000, optIsMomentary=False,
                  renderer="CheckBoxButtonRenderer", style="ModalCheckButtonStyle", text=""),
                N("Button", "RestoreOutputsButton", (211, 207, 14, 15), id=6000, optIsMomentary=False,
                  renderer="CheckBoxButtonRenderer", style="ModalCheckButtonStyle", text=""),
                N("Label", None, (-3, 208, 210, 15), id=6000, style="ModalLabelRightAlignedStyle",
                  text="Restore outputs on preset load"),
                N("Button", "RetardButton", (211, 229, 14, 15), id=100000, optIsMomentary=False,
                  renderer="CheckBoxButtonRenderer", style="ModalCheckButtonStyle", text=""),
                N("Label", "RetardLabel", (-32, 230, 238, 15), id=100001, style="ModalLabelRightAlignedStyle",
                  text="Use Audio Unit host timing correction"),
                N("ButtonGroup", "MIDIModeGroup", (211, 251, 90, 17), id=100000, style="BasePanelStyle", kids=[
                    N("Button", "CtrlButton", (0, 0, 25, 17), id=0, optIsMomentary=False, style="ModalLatchingButtonStyle",
                      text=0),
                    N("Button", "CtrlButton", (24, 0, 25, 17), id=1, optIsMomentary=False, style="ModalLatchingButtonStyle",
                      text=-2),
                    N("Button", "CtrlButton", (48, 0, 25, 17), id=2, optIsMomentary=False, style="ModalLatchingButtonStyle",
                      text=-1),
                ]),
                N("Label", "CtrlLabel", (74, 252, 132, 17), id=100004, style="ModalLabelRightAlignedStyle",
                  text="MIDI numbering"),
                N("TextBox", "UndoStages", (211, 273, 50, 17), id=100001, optSingleClickEdit=True, style="ModalEditBoxStyle"),
                N("Label", "CtrlLabel", (41, 274, 165, 17), id=100000, style="ModalLabelRightAlignedStyle",
                  text="Number of undo stages"),
                N("Label", "CtrlLabel", (96, 295, 110, 17), id=100014, style="ModalLabelRightAlignedStyle", text="Metronome"),
                N("PopupButton", "MetroType", (211, 295, 75, 17), id=100008, legacyClass="CtrlComboBox",
                  optSingleClickEdit=True, style="ModalComboBoxStyle"),
                N("PopupButton", "ShadingMode", (211, 317, 75, 17), id=100000, legacyClass="CtrlComboBox",
                  optSingleClickEdit=True, style="ModalComboBoxStyle"),
                N("Label", "CtrlLabel", (46, 318, 160, 17), id=100001, style="ModalLabelRightAlignedStyle",
                  text="Pattern shading mode"),
                N("Label", "CtrlLabel", (46, 339, 160, 17), id=100001, style="ModalLabelRightAlignedStyle", text="Screen Theme"),
                N("PopupButton", "ThemeCombo", (211, 339, 75, 17), id=100000, legacyClass="CtrlComboBox",
                  optSingleClickEdit=True, style="ModalComboBoxStyle"),
            ]),
            N("Panel", "CtrlPanel", (8, 376, 389, 42), id=100000, style="ModalPanelCollectionStyle", kids=[
                N("TextBox", "DocumentsPath", (78, 14, 200, 17), id=100001, optSingleClickEdit=False,
                  style="ModalNonEditableEditBoxStyle"),
                N("Button", "BrowseButton", (281, 14, 50, 17), id=100000, optIsMomentary=True,
                  style="ModalMomentaryButtonStyle", text="Browse"),
                N("Button", "ResetPathButton", (334, 14, 50, 17), id=100000, optIsMomentary=True,
                  style="ModalMomentaryButtonStyle", text="Reset"),
                N("Label", "CtrlLabel", (-67, 15, 140, 17), id=100000, style="ModalLabelRightAlignedStyle",
                  text="Documents path"),
            ]),
            N("Panel", "DirectWritePanel", (8, 422, 280, 30), id=100007, style="ModalPanelCollectionStyle", kids=[
                N("Button", "DirectWriteButton", (154, 6, 14, 15), id=100000, optIsMomentary=False,
                  renderer="CheckBoxButtonRenderer", style="ModalCheckButtonStyle", text=""),
                N("Label", "CtrlLabel", (-55, 7, 205, 17), id=100001, style="ModalLabelRightAlignedStyle",
                  text="DirectWrite text (experimental)"),
            ]),
            N("Button", "ConfirmButton", (320, 426, 75, 17), id=100007, optIsMomentary=True, style="ModalMomentaryButtonStyle",
              text="Finished"),
            N("Panel", "LaunchpadPanel", (-10, 443, 280, 30), id=100000, style="ModalPanelCollectionStyle", kids=[
                N("Button", "LaunchpadButton", (154, 6, 14, 15), id=100000, optIsMomentary=False,
                  renderer="CheckBoxButtonRenderer", style="ModalCheckButtonStyle", text=""),
                N("Button", "ResetLaunchpadButton", (176, 6, 50, 17), id=100000, optIsMomentary=True,
                  style="ModalMomentaryButtonStyle", text="Reset"),
                N("Label", "CtrlLabel", (-56, 8, 205, 17), id=100001, style="ModalLabelRightAlignedStyle",
                  text="Launchpad Active (experimental)"),
            ]),
        ]),
    ]


@skin("Global/UtilityControls.skin")
def utility_controls():
    return [
        N("Panel", "CtrlPanel", (0, 0, 135, 12), id=0, style="EmptyPanelStyle", kids=[
            N("Button", "LoadButton", (6, 1, 40, 10), displayName="Load files", id=100001, optIsMomentary=True,
              style="LoadButtonStyle", text="LOAD▼"),
            N("Button", "SaveButton", (48, 1, 40, 10), displayName="Save files", id=100003, optIsMomentary=True,
              style="SaveButtonStyle", text="SAVE▼"),
            N("Button", "GridButton", (90, 1, 40, 10), displayName="Options", id=100002, optIsMomentary=True,
              style="OptionsButtonStyle", text="EINST▼"),
        ]),
    ]
