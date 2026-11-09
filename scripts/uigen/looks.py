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
  or `behind` it (siblings drawn before it, placed on v1's rect, before
  any `inset`); x, y relative to the control (a negative y from its
  bottom), w, h below 1 relative to its size,
- `inset`: `(left, top, right, bottom)` taken off the rect of every
  control using the look: the part of v1's bitmap the art filled (round
  engine buttons, knob dials, pattern keys had transparent margins).

Rules match the image file name (no folder, no extension) with a regular
expression; the first match wins. Buttons use their `imUp` image.
"""

import math
import re

from theme import G, icon_font, text_font


def L(parent, text=None, upper=False, ctrl=None, cls=None, decor=None, inset=None, **attrs):
    return {"parent": parent, "text": text, "upper": upper, "ctrl": ctrl or {}, "cls": cls, "decor": decor or [], "inset": inset,
            "attrs": attrs}


# Pattern keys: the key face inside v1's 24 x 41 cell, with its lit strip.
KEY = dict(inset=(0, 0, 3, 5))

# Transport buttons: the face inside v1's 33 px cells (the rest was the
# bitmap's shadow on the backing).
TB = (0, 0, 2, 5)

POPUP = {"renderer": "PopupButtonRenderer"}

# Pad mode buttons: the face inside v1's 25 x 27 cells.
PM = (1, 1, 3, 5)
PMF = dict(realBorderWidth="2", colBorder="HwFramePad")
GRID_ICON = [("inside", 3 + 5 * (i % 3), 4 + 5 * (i // 3), 4, 4, "PadModeIconStyle") for i in range(9)]
# (E with a small M over S: two 3 x 3 marks right of the E)
EMS_ICON = [("inside", 15, 5, 3, 3, "PadModeIconStyle"), ("inside", 15, 11, 3, 3, "PadModeIconStyle")]
CHECK = {"renderer": "CheckBoxButtonRenderer"}

# The learn button's MIDI socket: a white disc with five pins round its
# upper half (v1 drew it filled, not as an outline).
MIDI_ICON = [("inside", 7, 5, 16, 16, "MidiDiscStyle")] + [
    ("inside", round(15 + 4.6 * math.cos(math.radians(a)) - 1), round(13 - 4.6 * math.sin(math.radians(a)) - 1), 3, 3, "MidiPinStyle")
    for a in (180, 135, 90, 45, 0)]

# The layer / pad mixers' MIX / SEND tabs: each with its part of the ink
# spine right of its face (behind it: the face is inset off the spine).
SIDE_TAB_SPINE = [("behind", 34, 0, 12, 0, "LcdSideTabSpineStyle")]
SAMPLER_KEY = dict(font=text_font(14, True), realCornerRounding="1", colFill="LcdLight", colBorder="InkMid", realGradientShade="0",
                   colText="BigButtonText", colAltFill="Ink", colAltBorder="Ink", colAltText="LcdLight")

# ---------------------------------------------------------------- buttons

BUTTONS = [
    # Page tabs on the screen bezels: text labels instead of image text.
    (r"^Tab(On|Off|Disabled)", L("TabButtonStyle", upper=True)),  # ("/" spaced, see convert.py)
    (r"^(Show|Hide)$", L("BezelIconButtonStyle", text=G["right"])),
    # Transport (hardware strip).
    (r"^PlayButton", L("HwPlayButtonStyle", text=G["right"], inset=TB, font=icon_font(22))),
    (r"^RecButton", L("HwRecordButtonStyle", text=G["record"], inset=TB)),
    (r"^SongButton", L("HwSongButtonStyle", text="SONG", inset=TB)),
    (r"^RestartButton", L("HwIconButtonStyle", text=G["skip_start"], inset=TB, colText="HwBlueIcon", colAltText="HwBlueIcon")),
    (r"^LoopButton", L("HwIconButtonStyle", text=G["loop"], inset=TB, colText="HwIconDim", colAltText="HwIconDim")),
    (r"^RetroButton", L("HwIconButtonStyle", text=G["retrigger"], inset=TB, colText="HwRed", colAltText="HwRedOn")),
    (r"^PanicButton", L("HwIconButtonStyle", text=G["warning"], inset=TB)),
    (r"^LearnButton", L("HwGreyIconButtonStyle", text="", inset=TB, decor=MIDI_ICON)),
    (r"^Quantize", L("HwGreyIconButtonStyle", text=G["quantise"], inset=TB)),
    (r"^MetroButton", L("HwGreyIconButtonStyle", text=G["metronome"], inset=TB)),
    (r"^UndoButton", L("HwDarkButtonStyle", text="UNDO", optRoundTopRight="false", optRoundBottomRight="false", realTextOffset="3",
                       realLetterSpacing="1", font=text_font(10, True), realTextOffsetY="0", inset=(0, 0, 0, 3))),
    (r"^RedoButton", L("HwDarkButtonStyle", text="REDO", optRoundTopRight="false", optRoundBottomRight="false", realTextOffset="3",
                       realLetterSpacing="1", font=text_font(10, True), realTextOffsetY="0", inset=(0, 0, 0, 3))),
    # (UNDO / REDO and their arrows: one 24 px high bezel, the arrow part
    # 12 px wide; the rest of v1's cells was the bitmap's shadow)
    (r"^UndoRedoDropArrow", L("HwDarkIconButtonStyle", text=G["down"], optRoundTopLeft="false", optRoundBottomLeft="false",
                              inset=(0, 0, 4, 3))),
    # Pad mode column.
    # (icons: a 3 x 3 grid of squares, a note, a velocity ramp, E with M/S)
    # (a heavy 2 px frame; the velocity ramp spans the face, low and wide)
    (r"^PadModePad", L("HwButtonStyle", text="", inset=PM, decor=GRID_ICON, **PMF)),
    (r"^PadModeChro", L("HwButtonStyle", text="♩", font=text_font(24), inset=PM, enumTextShortenAlgorithm="none", **PMF)),
    (r"^PadModeVel", L("HwButtonStyle", text="◢", font=text_font(18), inset=PM, realTextScaleX="1.5", realTextOffsetY="2",
                       enumTextShortenAlgorithm="none", **PMF)),
    (r"^PadModeEMS", L("HwButtonStyle", text="E", font=text_font(14, True), inset=PM, decor=EMS_ICON, realTextOffset="-3", **PMF)),
    (r"^FullLevel", L("HwButtonStyle", text="FULL", font=text_font(8, True))),
    (r"^Erase(Up|Down)", L("HwIconButtonStyle", text=G["eraser"], font=icon_font(12))),
    (r"^Follow", L("HwButtonStyle", text="FLW", font=text_font(8, True))),
    (r"^RepeatOptions", L("HwButtonStyle", font=text_font(11), enumTextHorizontalAlign="left", realTextOffset="5")),
    (r"^Repeat", L("HwButtonStyle", text="REPEAT")),
    # Utility menus under the master section (40 x 10).
    (r"^(Load|Save)Button", L("ReadoutMenuStyle", upper=True)),  # ("..." -> "▼", see convert.py)
    # (the third of them, the options menu, is matched by name in convert.py)
    # Engine selectors.
    (r"^EngineButtonUpPowerOff", L("EngineButtonPowerOffStyle", inset=(6, 6, 6, 6))),
    (r"^EngineButtonUp", L("EngineButtonStyle", inset=(6, 6, 6, 6))),
    (r"^PatternPower", L("HwPowerButtonStyle", text=G["power"])),
    (r"^MuteIndi", L("IndicatorMuteStyle", text="")),
    (r"^SoloIndi", L("IndicatorSoloStyle", text="")),
    (r"^HARDMute", L("HwMuteButtonStyle")),
    (r"^HARDSolo", L("HwSoloButtonStyle")),
    # Master section.
    (r"^MIDIIndiError", L("IndicatorLabelStyle", text="● MIDI", colText="LedMute")),
    (r"^MIDIIndi", L("IndicatorLabelStyle", text="● MIDI")),
    (r"^VoiceIndi", L("IndicatorLabelStyle", text="● VOICE")),
    # Pattern memory keys.
    (r"^PatternButtonOff", L("PatternKeyStyle", **KEY)),
    (r"^PatternButtonOn", L("PatternKeyAllocatedStyle", **KEY)),
    (r"^PatternButtonEditing", L("PatternKeyEditingStyle", **KEY)),
    (r"^PatternButtonSelected", L("PatternKeySelectedStyle", **KEY)),
    # LCD buttons.
    (r"^LED(Standard|Link)Button|^LinkButton", L("LcdButtonStyle")),
    # (v1: an ink disc when on, a grey one with an ink edge when off, with
    # an upright slot through it; 16 x 17 ovals, the chain's 24 px round)
    (r"^LEDSmallButton", L("LcdRoundToggleStyle", text="", realCornerRounding="8", realGlyphWidth="4", realGlyphHeight="11")),
    # (v1's mute / solo bitmaps: a 2 px frame a pixel above and below the cell;
    # the letters mid grey when off, as in v1's `LEDMute/SoloButtonUp`)
    (r"^LEDMuteButton", L("LcdMuteButtonStyle", inset=(0, -1, 0, -1), colText="RowEdge")),
    (r"^LEDSoloButton", L("LcdSoloButtonStyle", inset=(0, -1, 0, -1), colText="RowEdge")),
    # (the chain preset menu, 24 px high: v1's small arrow ~7 px in)
    (r"^LEDEffectCombo", L("LcdMenuButtonStyle", ctrl=POPUP, realPopupArrowSize="0.2", realPopupArrowBoxWidth="14")),
    # (the effect slots' type / preset menus, 17 px high: v1's 5 px arrow)
    (r"^LEDUltraMenuButton", L("LcdMenuButtonStyle", ctrl=POPUP, realPopupArrowSize="0.3")),
    (r"^LED(Large)?MenuButton|^DragEditButton", L("LcdMenuButtonStyle", ctrl=POPUP)),
    # (v1: the selected segment mid grey with dark text, the outer ends pills)
    (r"^LEDRadio(Large|Micro)Left", L("LcdRadioStyle", realCornerRounding="8", optRoundTopRight="false", optRoundBottomRight="false")),
    (r"^LEDRadio(Large|Micro)Center", L("LcdRadioStyle", realCornerRounding="0")),
    (r"^LEDRadio(Large|Micro)Right", L("LcdRadioStyle", realCornerRounding="8", optRoundTopLeft="false", optRoundBottomLeft="false")),
    (r"^LEDAddButton", L("LcdIconButtonStyle", text=G["plus"])),
    (r"^LEDSubtractButton", L("LcdIconButtonStyle", text=G["minus"])),
    (r"^LEDClear", L("LcdIconButtonStyle", text=G["close"])),
    (r"^LEDLock", L("LcdIconButtonStyle", text=G["lock"])),
    (r"^(LEDCheck|ModalCheckBox)", L("CheckBoxStyle", text="", ctrl=CHECK)),
    # (v1's 12 x 12 zoom keys: a navy frame round a light face)
    (r"^LEDZoomIn", L("LcdIconButtonStyle", text=G["plus"], font=icon_font(8), colBorder="Ink")),
    (r"^LEDZoomOut", L("LcdIconButtonStyle", text=G["minus"], font=icon_font(8), colBorder="Ink")),
    (r"^LEDZoomStartEnd", L("LcdIconButtonStyle", text=G["range"], font=icon_font(8), colBorder="Ink")),
    (r"^LEDZoomStart", L("LcdIconButtonStyle", text=G["to_start"], font=icon_font(8), colBorder="Ink")),
    (r"^LEDZoomEnd", L("LcdIconButtonStyle", text=G["to_end"], font=icon_font(8), colBorder="Ink")),
    (r"^LEDNoZoom", L("LcdIconButtonStyle", text=G["ring"], font=icon_font(8), colBorder="Ink")),
    (r"^LEDDiv2", L("LcdButtonStyle", text="÷2", font=text_font(11))),
    (r"^LEDx2", L("LcdButtonStyle", text="×2", font=text_font(11))),
    # Mixer view tabs (v1, 46 x 49: a 34 px face, then the ink spine; the
    # engine mixer's group draws one spine for its three, convert.py).
    (r"^LEDM(Mix|Send|Setup)Button", L("LcdSideTabStyle", upper=True, inset=(0, 2, 12, 2))),
    (r"^LED(Mix|Send|Setup)Button", L("LcdSideTabStyle", upper=True, inset=(0, 2, 12, 2), decor=SIDE_TAB_SPINE)),
    (r"^LargePowerButton", L("LcdRoundToggleStyle", text="", realCornerRounding="12", realGlyphWidth="4", realGlyphHeight="16")),
    # (v1: filled grey triangles with an ink edge, ink when pressed)
    (r"^LargePresetBackButton", L("LcdTriangleButtonStyle", text="", enumGlyph="TriangleLeft")),
    (r"^LargePresetNextButton", L("LcdTriangleButtonStyle", text="", enumGlyph="TriangleRight")),
    (r"^PrevPresetButton", L("LcdTriangleButtonStyle", text="", enumGlyph="TriangleLeft")),
    (r"^NextPresetButton", L("LcdTriangleButtonStyle", text="", enumGlyph="TriangleRight")),
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
    # v1's track graph toggle is a bare disclosure triangle (the reference
    # screenshots): flat, ► closed and ▼ open (pattern.rs sets the text).
    (r"^AutoButton", L("TrackArrowStyle", text="►", font=text_font(9))),
    (r"^Part(Prev)", L("LcdFlatIconButtonStyle", text=G["left"], font=icon_font(8))),
    (r"^Part(Next)", L("LcdFlatIconButtonStyle", text=G["right"], font=icon_font(8))),
    (r"^Part(Popup)", L("LcdFlatIconButtonStyle", text=G["chev_down"], font=icon_font(8))),
    # Layer setup / timestretch.
    # (v1: 24 px round buttons, the tempo ones reading a bold "bpm")
    (r"^PhaseButton", L("LcdRoundButtonStyle", text=G["phase"], font=icon_font(14))),
    (r"^ReverseButton", L("LcdRoundButtonStyle", text=G["reverse"], font=icon_font(14))),
    (r"^BPMButton", L("LcdRoundButtonStyle", text="bpm", font=text_font(9, True))),
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
    # (v1's 40 x 41 keys: square, a light face in a navy edge, bold grey
    # caps; navy with light caps when on or flashing)
    (r"^SamplerArmButton", L("LcdBigButtonStyle", text="ARM", **SAMPLER_KEY)),
    (r"^SamplerRecButton", L("LcdBigButtonStyle", text="REC", **SAMPLER_KEY)),
]

# Looks by (neutral) style name, before the image rules: styles whose
# bitmap is shared by controls that need different captions, and the
# options menu button (its v1 name carries the product name).
BY_NAME = {
    "OptionsButtonStyle": L("ReadoutMenuStyle", text="EINST▼"),
    "AutoButtonStyle": L("LcdButtonStyle", text="AUTO", font=text_font(8, True)),
    "FooterLatchingButtonStyle": L("LcdButtonStyle", font=text_font(10)),
}

# Buttons without any image (flat colour styles, flash layers) keep their
# v1 colours on this base.
FLAT_BUTTON = L("LcdButtonStyle")

# ---------------------------------------------------------------- panels (imBackground)

# Mixer strips: v1's strip bitmaps start with a dark band across the top
# (2 px down, 13 px high; the bands of neighbouring strips meet).
STRIP_HEAD = [("inside", 0, 2, 0, 13, "LcdStripHeadStyle")]

FX_CARD = [("inside", 1, 2, -2, -4, "FxCardEmptyStyle"), ("inside", 3, 4, -6, 38, "FxCardHeadStyle")]


# The pad editor's panels (`PadEditBackground` / `PadControlsBackground`,
# 682 x 396): a navy bezel (2 px transparent margin, 2 px dark edge, the
# band 3 px high at top and bottom, 7 px at the sides) round a screen with a
# 2 px inner shadow. The layer view's adds four group boxes with grey
# 20 px headers (setup, timestretch, filter, envelopes).
PAD_BEZEL = [("inside", 2, 2, -5, -5, "LcdBezelStyle"), ("inside", 11, 7, -23, -15, "LcdInsetStyle")]
GROUPS = [d for x, h in ((117, 151), (255, 151), (393, 185), (531, 185))
          for d in (("inside", x, 13, 131, h, "LcdGroupStyle"), ("inside", x, 13, 131, 21, "LcdGroupHeadStyle"))]
PANELS = [
    (r"^Background", L("ChassisStyle")),
    (r"^AboutBackground", L("LcdPanelStyle", colFill="Ink", colBorder="Amber", realCornerRounding="6")),
    # (v1: the chain on a dark frame that shows between and under the
    # effect panels)
    (r"^FXGlobalBackground", L("LcdPanelStyle", colFill="InkMid", colBorder="Ink", optDrawBorder="true", realCornerRounding="3")),
    (r"^ChainHeader", L("LcdHeaderStyle", colFill="LcdMid", colBorder="InkMid", realCornerRounding="0")),
    # (v1, 199 x 196: a card 1 px in at the sides, 2 px at the top and
    # bottom, with a darker band behind the slot's menus; empty slots all
    # dark. fxchain.rs switches the card's style.)
    (r"^FXPanelBackground", L("BasePanelStyle", colFill="Transparent", optDrawBorder="false", decor=FX_CARD)),
    (r"^BreverbBackground", L("BasePanelStyle")),
    (r"^(MiniChannel|Pad|Aux)Background(Even)", L("LcdRowEvenStyle", realCornerRounding="3", decor=STRIP_HEAD)),
    (r"^(MiniChannel|Pad|Aux)Background(Odd)", L("LcdRowOddStyle", realCornerRounding="3", decor=STRIP_HEAD)),
    (r"^(MiniChannel|Pad|Aux)Background(Selected)", L("LcdStripSelectedStyle", realCornerRounding="3", decor=STRIP_HEAD)),
    (r"^PadEditBackground", L("BasePanelStyle", decor=PAD_BEZEL + GROUPS)),
    (r"^PadControlsBackground", L("BasePanelStyle", decor=PAD_BEZEL)),
    (r"^PadSelectionNotSelected", L("LcdPadRowStyle")),
    (r"^PadSelectionPrimary", L("LcdPadRowSelectedStyle")),
    (r"^LayerSelectionBackgroundEven", L("LcdLayerRowEvenStyle")),
    (r"^LayerSelectionBackgroundOdd", L("LcdLayerRowOddStyle")),
    (r"^LayerSelectionBackgroundSelected", L("LcdRowSelectedStyle")),
    (r"^(Preview|ScenePage)Background", L("BasePanelStyle")),
    # (v1: the outer tile a 2 px navy edge round 3, the inner ones square
    # with a 2 px dark inset edge)
    (r"^SamplerBackgroundOuter", L("LcdPanelStyle", colFill="InkMid", colBorder="Ink", realBorderWidth="2", realCornerRounding="3")),
    (r"^SamplerBackgroundInner", L("LcdPanelStyle", colFill="Lcd", colBorder="SamplerInsetEdge", realBorderWidth="2",
                                   realCornerRounding="1")),
    (r"^SceneBackground", L("LcdSlotStyle")),
]

# ---------------------------------------------------------------- text boxes (imBackground)

TEXT_BOXES = [
    (r"Combo", L("BaseTextBoxStyle", realPopupArrowSize="0.3", optPopupDivider="false")),
    (r"^SearchBox", L("BaseTextBoxStyle", realCornerRounding="8")),
    (r"", L("BaseTextBoxStyle")),
]

# ---------------------------------------------------------------- knobs (imKnobNormal)

# (rule, look, fit): fit = how the square dial sits in v1's rect ("top"
# for the hardware knobs, whose film frames carry a scale under the dial).
KNOBS = [
    # (v1: a light-blue value arc on the outer part of the dark ring)
    (r"^HARDKnobLarge", L("HardKnobStyle", ctrl={"realDialThickness": "0.07"}, inset=(4, 12, 8, 20)), "top"),
    (r"^HARDKnobSmall", L("HardKnobSmallStyle", ctrl={"realDialThickness": "0.07"}, inset=(2, 1, 2, 5)), "top"),
    # (v1, 50 x 46 film: a 4 px 270-degree value arc, a gap, then a knob
    # body: grey ring round a light face, with a short dark pointer)
    (r"^LEDKnobLarge", L("LcdKnobLargeStyle", ctrl={"realDialThickness": "0.09"}), "center"),
    # (v1, 24 x 26 / 17 x 17 films: no arc; a closed grey ring round a light
    # face with a dark edge and a pointer)
    (r"^LEDKnobSmall", L("LcdKnobSmallStyle", ctrl={"realDialThickness": "0.04"}), "center"),
    (r"^LEDKnobMicro", L("LcdKnobMicroStyle", ctrl={"realDialThickness": "0.05"}), "center"),
    (r"^LEDSpinner", L("SpinnerStyle", ctrl={"realDialThickness": "0.2"}), "center"),
]

# The panels drawn under knobs big enough to have them (dials over 32 px):
# (theme style, left, top, right, bottom as fractions of the dial), in
# drawing order. The large hardware knob sits in a light halo with a soft
# shadow below, inside a ring lit grey at the top and black at the bottom.
KNOB_BODY = {
    "HardKnobStyle": [
        ("HardKnobHaloStyle", -0.13, -0.13, 1.13, 1.13),
        ("HardKnobShadowStyle", 0.0, 0.6, 1.0, 1.4),
        ("HardKnobRingStyle", 0, 0, 1, 1),
        # (the scale's end marks, below left and right)
        ("HardKnobMarkStyle", -0.02, 0.99, 0.05, 1.06),
        ("HardKnobMarkStyle", 0.95, 0.99, 1.02, 1.06),
    ],
}

# ---------------------------------------------------------------- faders, pans, meters (imBackingStrip)

# LED level strips: 3 px segments 1 px apart. Faders and pans get
# rectangular caps (v1's heads: 15 x 16 / 18 x 22 on the 15 / 18 px
# faders; pans size theirs with faderHeadSize) and their track back (v1's
# noDefaultDrawing left it to the bitmap).
LED = {"intSegmentSize": "3", "intSegmentGap": "1"}
TRACK = {"colFill": "LcdFace"}
SLIDERS = [
    (r"^(HARDVU|EngineVU)", L("StripMeterStyle", **LED)),
    (r"VU", L("LcdMeterStyle", inset=(0, 0, 1, 0))),  # (v1's LCD meters: plain 3 px bars in 4 px rects)
    # (v1's pan thumb: a dark block, a light line across its middle)
    (r"PanBase", L("LcdPanStyle", ctrl={"realTrackInset": "0.0"}, realHandleRounding="1", enumGripShape="Across", intGripLines="1", **TRACK)),
    # (v1's mixer fader cap: dark, a light line across its middle)
    (r"^LEDSmVSliderBack", L("LcdFaderStyle", ctrl={"realTrackInset": "0.0"}, realHandleLength="15", intGripLines="1", enumGripShape="Across",
                             **TRACK)),
    (r"^LEDFatSliderBack", L("LcdFaderStyle", ctrl={"realTrackInset": "0.0"}, realHandleLength="20", **TRACK)),
    (r"", L("LcdFaderStyle", ctrl={"realTrackInset": "0.0"}, **TRACK)),
]

# ---------------------------------------------------------------- meters and plots (imBackground)

# (v1's meter and plot bitmaps, 107 x 50: a 2 px ink frame with 5 px
# corners; the gain-reduction meter has a dark band along the bottom with
# a bump under the needle's pivot, a thin grey scale and small grey labels)
METER = L("LcdPlotStyle", colAltBorder="InkSoft", colLine="Ink", colBorder="Ink", realBorderWidth="2", realCornerRounding="5",
          colBaseBand="InkMid", realBaseBandHeight="5", intTicks="0", colText="60, 60, 60, 255", font=text_font(8))
PLOT = L("LcdPlotStyle", colLine="Ink", colBorder="Ink", realBorderWidth="2", realCornerRounding="5")

# ---------------------------------------------------------------- image views (`image=`)

IMAGES = [
    (r"^(Background|BackgroundSmall)$", L("ChassisStyle")),
    (r"^(Screen|Browser)$", L("ScreenBezelStyle", decor=[("inside", 3, 20, -6, -23, "ScreenLcdStyle")])),
    (r"^StatusStrip", L("StatusStripStyle")),
    (r"^EngineText$", L("SectionLabelStyle", text="ENGINES", cls="CtrlLabel")),
    (r"^PatternText$", L("SectionLabelStyle", text="PATTERNS", cls="CtrlLabel")),
    (r"^MasterText$", L("SectionLabelStyle", text="MASTER", cls="CtrlLabel")),
    (r"^PadText$", L("SectionLabelStyle", text="PADS", cls="CtrlLabel")),
    (r"^ModesText$", L("SectionLabelLeftStyle", text="MODES", cls="CtrlLabel")),
    # The wordmark in the logo's place (lower-case, as v1's): text, not art.
    (r"^Logo$", L("WordmarkStyle", text="einst.", cls="CtrlLabel")),
    # (v1: a large pale triangle under the "S1" .. "S4" captions)
    (r"^SendArrow$", L("GlyphLabelStyle", text="►", cls="CtrlLabel", font=text_font(24), colText="LcdDim",
                       enumTextHorizontalAlign="left")),
    (r"^Glue", L("LcdGlueStyle", colFill="InkMid")),  # (v1's mixer glue: the chain frame's colour)
    (r"^(Pad|Layer)Glue$", L("LcdLayerGlueStyle")),
    (r"^NavigationBackground$", L("LcdFieldStyle", realCornerRounding="4")),
    (r"^GraphMarkers$", L("LcdRulerStyle", ctrl={"overloadClass": "VelocityRuler"})),  # (labelled by the editor)
    (r"^GraphIcon$", L("ChassisGlyphLabelStyle", text=G["wave"], cls="CtrlLabel")),
    (r"^PreviewVol$", L("GlyphLabelStyle", text=G["speaker"], cls="CtrlLabel", font=icon_font(10))),
    (r"^MasterTuneIcon$", L("ChassisGlyphLabelStyle", text=G["a"], cls="CtrlLabel")),
    (r"^MasterVolIcon$", L("ChassisGlyphLabelStyle", text=G["speaker"], cls="CtrlLabel")),
    (r"^SwingIcon$", L("ChassisGlyphLabelStyle", text=G["metronome"], cls="CtrlLabel", font=icon_font(12))),
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
