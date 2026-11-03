"""Vector looks standing in for every bitmap a v1 skin names.

v1 draws almost everything from PNGs (`imUp` / `imDown` buttons,
`imBackground` panels, `imKnobNormal` film strips, VU strips, `image=`
views). None of those images is used here: each reference is matched by
the rules below to a look, which is

- `parent`: the theme style (theme.py) the converted style inherits from,
- `attrs`: extra style attributes (colours, rounding, font),
- `text`: a caption or an Einst-Icons glyph that replaces the drawn image
  text (written onto every control using the style; `upper` upper-cases
  the control's own text instead),
- `ctrl`: attributes written onto those controls (`renderer`, ...),
- `cls`: the class a v1 image view becomes (a panel or a label),
- `decor`: unnamed panels standing in for the parts of a bitmap one style
  cannot draw (the screen inside a bezel, knob bodies), each
  `(where, x, y, w, h, style)`: `inside` the control (its first children)
  or `behind` it (siblings drawn before it); x, y relative to the control,
  w, h below 1 relative to its size.

Rules match the image file name (no folder, no extension) with a regular
expression; the first match wins. Buttons use their `imUp` image.
"""

import re

from theme import G, icon_font, text_font


def L(parent, text=None, upper=False, ctrl=None, cls=None, decor=None, **attrs):
    return {"parent": parent, "text": text, "upper": upper, "ctrl": ctrl or {}, "cls": cls, "decor": decor or [], "attrs": attrs}


POPUP = {"renderer": "PopupButtonRenderer"}
CHECK = {"renderer": "CheckBoxButtonRenderer"}

# ---------------------------------------------------------------- buttons

BUTTONS = [
    # Page tabs on the screen bezels: text labels instead of image text.
    (r"^Tab(On|Off|Disabled)", L("TabButtonStyle", upper=True)),
    (r"^(Show|Hide)$", L("BezelIconButtonStyle", text=G["chev_left"])),
    # Transport (hardware strip).
    (r"^PlayButton", L("HwPlayButtonStyle", text=G["right"])),
    (r"^RecButton", L("HwRecordButtonStyle", text=G["record"])),
    (r"^SongButton", L("HwButtonStyle", text="SONG")),
    (r"^RestartButton", L("HwIconButtonStyle", text=G["skip_start"])),
    (r"^LoopButton", L("HwIconButtonStyle", text=G["loop"])),
    (r"^RetroButton", L("HwIconButtonStyle", text=G["retrigger"])),
    (r"^PanicButton", L("HwIconButtonStyle", text=G["warning"])),
    (r"^LearnButton", L("HwButtonStyle", text="LEARN", font=text_font(8, True))),
    (r"^Quantize", L("HwIconButtonStyle", text=G["quantise"])),
    (r"^MetroButton", L("HwIconButtonStyle", text=G["metronome"])),
    (r"^UndoButton", L("HwIconButtonStyle", text=G["undo"])),
    (r"^RedoButton", L("HwIconButtonStyle", text=G["redo"])),
    (r"^UndoRedoDropArrow", L("HwSmallIconButtonStyle", text=G["chev_down"])),
    # Pad mode column.
    (r"^PadModePad", L("HwButtonStyle", text="PAD")),
    (r"^PadModeChro", L("HwButtonStyle", text="CHR")),
    (r"^PadModeVel", L("HwButtonStyle", text="VEL")),
    (r"^PadModeEMS", L("HwButtonStyle", text="M/S")),
    (r"^FullLevel", L("HwButtonStyle", text="FULL", font=text_font(8, True))),
    (r"^Erase(Up|Down)", L("HwIconButtonStyle", text=G["eraser"], font=icon_font(12))),
    (r"^Follow", L("HwButtonStyle", text="FLW", font=text_font(8, True))),
    (r"^RepeatOptions", L("HwButtonStyle", font=text_font(11), enumTextHorizontalAlign="left", realTextOffset="5")),
    (r"^Repeat", L("HwButtonStyle", text="REPEAT")),
    # Utility menus under the master section (40 x 10).
    (r"^(Load|Save)Button", L("HwTinyButtonStyle", upper=True)),
    # (the third of them, the options menu, is matched by name in convert.py)
    # Engine selectors.
    (r"^EngineButtonUpPowerOff", L("EngineButtonPowerOffStyle")),
    (r"^EngineButtonUp", L("EngineButtonStyle")),
    (r"^PatternPower", L("HwPowerButtonStyle", text=G["power"])),
    (r"^MuteIndi", L("IndicatorMuteStyle", text="")),
    (r"^SoloIndi", L("IndicatorSoloStyle", text="")),
    (r"^HARDMute", L("HwMuteButtonStyle")),
    (r"^HARDSolo", L("HwSoloButtonStyle")),
    # Master section.
    (r"^MIDIIndiError", L("IndicatorErrorStyle", text="MIDI")),
    (r"^MIDIIndi", L("IndicatorStyle", text="MIDI")),
    (r"^VoiceIndi", L("IndicatorStyle", text="VOICE")),
    # Pattern memory keys.
    (r"^PatternButtonOff", L("PatternKeyStyle")),
    (r"^PatternButtonOn", L("PatternKeyAllocatedStyle")),
    (r"^PatternButtonEditing", L("PatternKeyEditingStyle")),
    (r"^PatternButtonSelected", L("PatternKeySelectedStyle")),
    # LCD buttons.
    (r"^LED(Standard|Link)Button|^LinkButton", L("LcdButtonStyle")),
    (r"^LEDSmallButton", L("LcdIconButtonStyle", text=G["power"])),
    (r"^LEDMuteButton", L("LcdMuteButtonStyle")),
    (r"^LEDSoloButton", L("LcdSoloButtonStyle")),
    (r"^LED(Ultra|Large)?MenuButton|^LEDEffectCombo|^DragEditButton", L("LcdMenuButtonStyle", ctrl=POPUP)),
    (r"^LEDRadio(Large|Micro)Left", L("LcdButtonStyle", optRoundTopRight="false", optRoundBottomRight="false")),
    (r"^LEDRadio(Large|Micro)Center", L("LcdButtonStyle", realCornerRounding="0")),
    (r"^LEDRadio(Large|Micro)Right", L("LcdButtonStyle", optRoundTopLeft="false", optRoundBottomLeft="false")),
    (r"^LEDAddButton", L("LcdIconButtonStyle", text=G["plus"])),
    (r"^LEDSubtractButton", L("LcdIconButtonStyle", text=G["minus"])),
    (r"^LEDClear", L("LcdIconButtonStyle", text=G["close"])),
    (r"^LEDLock", L("LcdIconButtonStyle", text=G["lock"])),
    (r"^(LEDCheck|ModalCheckBox)", L("CheckBoxStyle", text="", ctrl=CHECK)),
    (r"^LEDZoomIn", L("LcdIconButtonStyle", text=G["plus"], font=icon_font(8))),
    (r"^LEDZoomOut", L("LcdIconButtonStyle", text=G["minus"], font=icon_font(8))),
    (r"^LEDZoomStartEnd", L("LcdIconButtonStyle", text=G["range"], font=icon_font(8))),
    (r"^LEDZoomStart", L("LcdIconButtonStyle", text=G["to_start"], font=icon_font(8))),
    (r"^LEDZoomEnd", L("LcdIconButtonStyle", text=G["to_end"], font=icon_font(8))),
    (r"^LEDNoZoom", L("LcdIconButtonStyle", text=G["ring"], font=icon_font(8))),
    (r"^LEDDiv2", L("LcdButtonStyle", text="÷2", font=text_font(11))),
    (r"^LEDx2", L("LcdButtonStyle", text="×2", font=text_font(11))),
    (r"^LEDM?(Mix|Send|Setup)Button", L("LcdBigButtonStyle", upper=True)),
    (r"^LargePowerButton", L("LcdPowerButtonStyle", text=G["power"])),
    (r"^LargePresetBackButton", L("LcdFlatIconButtonStyle", text=G["left"])),
    (r"^LargePresetNextButton", L("LcdFlatIconButtonStyle", text=G["right"])),
    (r"^PrevPresetButton", L("LcdFlatIconButtonStyle", text=G["left"], font=icon_font(8))),
    (r"^NextPresetButton", L("LcdFlatIconButtonStyle", text=G["right"], font=icon_font(8))),
    # Pattern and song tools.
    (r"^PenPopupTool", L("LcdFlatIconButtonStyle", text=G["chev_down"], font=icon_font(8))),
    (r"^PenTool", L("LcdButtonStyle", text="✎", font=text_font(12))),
    (r"^DeleteTool", L("LcdIconButtonStyle", text=G["eraser"])),
    (r"^SelectTool", L("LcdButtonStyle", text="⬚", font=text_font(11))),
    (r"^MultiTool", L("LcdButtonStyle", text="✛", font=text_font(11))),
    (r"^MultiTrack", L("LcdButtonStyle", text="≡", font=text_font(12), optRoundTopRight="false", optRoundBottomRight="false")),
    (r"^SingleTrack", L("LcdButtonStyle", text="−", font=text_font(12), optRoundTopLeft="false", optRoundBottomLeft="false")),
    (r"^NudgeLeftButton", L("LcdIconButtonStyle", text=G["left"], font=icon_font(8))),
    (r"^NudgeRightButton", L("LcdIconButtonStyle", text=G["right"], font=icon_font(8))),
    (r"^AutoButton", L("LcdButtonStyle", font=text_font(11, True))),
    (r"^Part(Prev)", L("LcdFlatIconButtonStyle", text=G["left"], font=icon_font(8))),
    (r"^Part(Next)", L("LcdFlatIconButtonStyle", text=G["right"], font=icon_font(8))),
    (r"^Part(Popup)", L("LcdFlatIconButtonStyle", text=G["chev_down"], font=icon_font(8))),
    # Layer setup / timestretch.
    (r"^PhaseButton", L("LcdIconButtonStyle", text=G["phase"], font=icon_font(14))),
    (r"^ReverseButton", L("LcdIconButtonStyle", text=G["reverse"], font=icon_font(14))),
    (r"^BPMButton", L("LcdButtonStyle", text="BPM", font=text_font(8, True))),
    (r"^AlgoButton", L("LcdButtonStyle", text="ALG", font=text_font(8, True))),
    # Browser header and footer.
    (r"^PrevUp", L("LcdFlatIconButtonStyle", text=G["left"])),
    (r"^NextUp", L("LcdFlatIconButtonStyle", text=G["right"])),
    (r"^PrevFile", L("LcdFlatIconButtonStyle", text=G["up"], font=icon_font(8))),
    (r"^NextFile", L("LcdFlatIconButtonStyle", text=G["down"], font=icon_font(8))),
    (r"^EnterButton", L("LcdIconButtonStyle", text=G["folder"], font=icon_font(14))),
    (r"^ParentButtonTile", L("LcdButtonStyle")),
    (r"^ParentButton", L("LcdIconButtonStyle", text=G["parent"], font=icon_font(14))),
    (r"^RefreshButton", L("LcdIconButtonStyle", text=G["refresh"], font=icon_font(14))),
    (r"^StopButton", L("LcdIconButtonStyle", text=G["stop"], font=icon_font(14))),
    (r"^LockPads", L("LcdButtonStyle", text="Pads", font=text_font(10, True))),
    (r"^LockPatterns", L("LcdButtonStyle", text="Ptns", font=text_font(10, True))),
    (r"^SearchOptions", L("LcdFlatIconButtonStyle", text=G["chev_down"], font=icon_font(8))),
    (r"^ViewOptionsButton", L("LcdFlatIconButtonStyle", text=G["settings"])),
    # Scene page.
    (r"^ScenePageSnapshotNextButton", L("LcdBigButtonStyle", text=G["snapshot"], font=icon_font(18))),
    (r"^PreviousSceneButton", L("LcdBigButtonStyle", text=G["left"], font=icon_font(16))),
    (r"^NextSceneButton", L("LcdBigButtonStyle", text=G["right"], font=icon_font(16))),
    (r"^ScenePageClearAllButton", L("LcdBigButtonStyle", text=G["close"], font=icon_font(16))),
    (r"^ScenePageUpButton", L("LcdIconButtonStyle", text=G["up"])),
    (r"^ScenePageDownButton", L("LcdIconButtonStyle", text=G["down"])),
    (r"^SceneRowUpButton", L("LcdIconButtonStyle", text=G["chev_up"])),
    (r"^SceneRowDownButton", L("LcdIconButtonStyle", text=G["chev_down"])),
    (r"^SceneSnapshotButton", L("LcdIconButtonStyle", text=G["plus"])),
    (r"^SceneClearButton", L("LcdIconButtonStyle", text=G["close"])),
    (r"^SmallSceneSelectButton", L("LcdButtonStyle")),
    # Sampler page.
    (r"^SamplerArmButton", L("LcdBigButtonStyle", text="ARM", colAltFill="Amber", colAltText="Ink")),
    (r"^SamplerRecButton", L("LcdBigButtonStyle", text=G["record"], font=icon_font(18), colText="HwRed", colAltFill="HwRed", colAltText="LcdLight")),
]

# Looks by (neutral) style name, before the image rules: styles whose
# bitmap is shared by controls that need different captions, and the
# options menu button (its v1 name carries the product name).
BY_NAME = {
    "OptionsButtonStyle": L("HwTinyButtonStyle", upper=True),
    "AutoButtonStyle": L("LcdButtonStyle", text="AUTO", font=text_font(8, True)),
    "FooterLatchingButtonStyle": L("LcdButtonStyle", font=text_font(10)),
}

# Buttons without any image (flat colour styles, flash layers) keep their
# v1 colours on this base.
FLAT_BUTTON = L("LcdButtonStyle")

# ---------------------------------------------------------------- panels (imBackground)

PANELS = [
    (r"^Background", L("ChassisStyle")),
    (r"^AboutBackground", L("LcdPanelStyle", colFill="Ink", colBorder="Amber", realCornerRounding="6")),
    (r"^FXGlobalBackground", L("LcdPanelStyle", colFill="LcdMid")),
    (r"^ChainHeader", L("LcdHeaderStyle")),
    (r"^FXPanelBackground", L("LcdPanelStyle")),
    (r"^BreverbBackground", L("BasePanelStyle")),
    (r"^(MiniChannel|Pad|Aux)Background(Even)", L("LcdRowEvenStyle", realCornerRounding="3")),
    (r"^(MiniChannel|Pad|Aux)Background(Odd)", L("LcdRowOddStyle", realCornerRounding="3")),
    (r"^(MiniChannel|Pad|Aux)Background(Selected)", L("LcdRowSelectedStyle", realCornerRounding="3")),
    (r"^Pad(Controls|Edit)Background", L("LcdPanelStyle", colFill="LcdMid")),
    (r"^PadSelectionNotSelected", L("LcdRowEvenStyle")),
    (r"^PadSelectionPrimary", L("LcdRowSelectedStyle")),
    (r"^LayerSelectionBackgroundEven", L("LcdRowEvenStyle")),
    (r"^LayerSelectionBackgroundOdd", L("LcdRowOddStyle")),
    (r"^LayerSelectionBackgroundSelected", L("LcdRowSelectedStyle")),
    (r"^(Preview|ScenePage)Background", L("BasePanelStyle")),
    (r"^SamplerBackgroundOuter", L("LcdPanelStyle", colFill="InkMid", colBorder="Ink", realCornerRounding="6")),
    (r"^SamplerBackgroundInner", L("LcdPanelStyle", colFill="Lcd", realCornerRounding="6")),
    (r"^SceneBackground", L("LcdSlotStyle")),
]

# ---------------------------------------------------------------- text boxes (imBackground)

TEXT_BOXES = [
    (r"Combo", L("BaseTextBoxStyle")),
    (r"^SearchBox", L("BaseTextBoxStyle", realCornerRounding="8")),
    (r"", L("BaseTextBoxStyle")),
]

# ---------------------------------------------------------------- knobs (imKnobNormal)

# (rule, look, fit): fit = how the square dial sits in v1's rect ("top"
# for the hardware knobs, whose film frames carry a scale under the dial).
KNOBS = [
    (r"^HARDKnobLarge", L("HardKnobStyle", ctrl={"realDialThickness": "0.16"}), "top"),
    (r"^HARDKnobSmall", L("HardKnobStyle", ctrl={"realDialThickness": "0.2"}), "top"),
    (r"^LEDKnobLarge", L("LcdKnobStyle", ctrl={"realDialThickness": "0.14"}), "center"),
    (r"^LEDKnob(Small|Micro)", L("LcdKnobStyle", ctrl={"realDialThickness": "0.22"}), "center"),
    (r"^LEDSpinner", L("SpinnerStyle", ctrl={"realDialThickness": "0.2"}), "center"),
]

# The body drawn under knobs big enough to have one: (theme style, inset as
# a fraction of the dial).
KNOB_BODY = {"HardKnobStyle": ("HardKnobBodyStyle", 0.2), "LcdKnobStyle": ("LcdKnobBodyStyle", 0.2)}

# ---------------------------------------------------------------- faders, pans, meters (imBackingStrip)

SLIDERS = [
    (r"^(HARDVU|EngineVU)", L("StripMeterStyle")),
    (r"VU", L("LcdMeterStyle")),
    (r"", L("LcdFaderStyle", ctrl={"realTrackInset": "0.36"})),
]

# ---------------------------------------------------------------- image views (`image=`)

IMAGES = [
    (r"^(Background|BackgroundSmall)$", L("ChassisStyle")),
    (r"^(Screen|Browser)$", L("ScreenBezelStyle", decor=[("inside", 3, 20, -6, -23, "ScreenLcdStyle")])),
    (r"^StatusStrip", L("StatusStripStyle")),
    (r"^EngineText$", L("SectionLabelStyle", text="ENGINE", cls="CtrlLabel")),
    (r"^PatternText$", L("SectionLabelStyle", text="PATTERN", cls="CtrlLabel")),
    (r"^MasterText$", L("SectionLabelStyle", text="MASTER", cls="CtrlLabel")),
    (r"^PadText$", L("SectionLabelStyle", text="PADS", cls="CtrlLabel")),
    (r"^ModesText$", L("SectionLabelStyle", text="MODES", cls="CtrlLabel")),
    (r"^Logo$", L("LogoStyle", cls="CtrlPanel", ctrl={"overloadClass": "Logo"})),
    (r"^SendArrow$", L("GlyphLabelStyle", text=G["right"], cls="CtrlLabel", font=icon_font(8))),
    (r"^Glue", L("LcdGlueStyle")),
    (r"^(Pad|Layer)Glue$", L("LcdGlueStyle")),
    (r"^NavigationBackground$", L("LcdFieldStyle", realCornerRounding="4")),
    (r"^GraphMarkers$", L("LcdRulerStyle")),
    (r"^GraphIcon$", L("ChassisGlyphLabelStyle", text=G["wave"], cls="CtrlLabel")),
    (r"^PreviewVol$", L("GlyphLabelStyle", text=G["speaker"], cls="CtrlLabel", font=icon_font(10))),
    (r"^MasterTuneIcon$", L("ChassisGlyphLabelStyle", text=G["a"], cls="CtrlLabel")),
    (r"^MasterVolIcon$", L("ChassisGlyphLabelStyle", text=G["speaker"], cls="CtrlLabel")),
    (r"^SwingIcon$", L("ChassisGlyphLabelStyle", text=G["pulse"], cls="CtrlLabel")),
    (r"^SearchBox$", L("LcdFieldStyle", realCornerRounding="8")),
    (r"^Scene(Empty|Full|Active|Cued)", L("LcdFieldStyle", realCornerRounding="3")),
]

# Image views that only show state the editor fills in (scene trigger
# states): a field look by style name.
IMAGE_STYLE_LOOKS = {
    "TriggerImageViewStyle": L("LcdFieldStyle", realCornerRounding="3"),
    "OverlayImageViewStyle": L("BaseImageStyle"),
}


def match(rules, stem):
    for rule in rules:
        pat, look = rule[0], rule[1]
        if re.search(pat, stem):
            return rule
    return None


def stem(path):
    """`Images/Buttons/LEDLockUp.png` -> `LEDLockUp`."""
    base = path.replace("\\", "/").rsplit("/", 1)[-1]
    return base.rsplit(".", 1)[0]
