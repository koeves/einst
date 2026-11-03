"""The `Einst` theme of the v1-layout UI: palette, type scale and the base
control styles every converted v1 style inherits from.

The palette follows v1's own (light blue-grey "LCD", dark-blue ink, amber
selection; the RE survey's palette section) and adds the chassis colours
the v1 bitmaps carried (silver panel, black screen bezels, grey hardware
buttons), picked by eye from those bitmaps' proportions and tones, not
copied from them.
"""

# ---------------------------------------------------------------- palette

# name -> (r, g, b) or (r, g, b, a)
COLOURS = {
    # The LCD (screens, browser, edit boxes).
    "Ink": (18, 25, 60),
    "InkMid": (76, 83, 111),
    "InkSoft": (148, 155, 172),
    "Lcd": (210, 217, 227),
    "LcdLight": (222, 229, 237),
    "LcdMid": (199, 206, 217),
    "LcdDim": (169, 176, 191),
    "LcdShade": (185, 192, 205),
    "Amber": (236, 189, 47),
    "AmberSoft": (232, 204, 91),
    "AmberDeep": (226, 165, 20),
    "AmberTint": (232, 204, 91, 120),
    # The chassis around the screens.
    "Chassis": (212, 213, 216),
    "ChassisEdge": (58, 59, 62),
    "ChassisBar": (150, 151, 154),
    "ChassisLine": (128, 129, 132),
    "ChassisWell": (128, 129, 133),
    "ChassisText": (236, 236, 236),
    "Bezel": (10, 11, 13),
    "BezelText": (196, 201, 212),
    "BezelTextOn": (150, 200, 255),
    "BezelTabOn": (38, 46, 78),
    # Hardware buttons, keys and pads.
    "HwFace": (234, 235, 237),
    "HwFaceOn": (196, 199, 205),
    "HwFrame": (104, 106, 110),
    "HwFrameDark": (52, 54, 58),
    "HwInk": (62, 64, 68),
    "HwInkOn": (18, 25, 60),
    "HwGreen": (34, 150, 46),
    "HwGreenOn": (60, 200, 70),
    "HwRed": (200, 34, 30),
    "HwRedOn": (240, 60, 50),
    "PadFace": (88, 89, 93),
    "PadFrame": (48, 49, 52),
    "EngineFace": (58, 58, 61),
    "EngineFaceOff": (36, 36, 38),
    "EngineRim": (8, 8, 9),
    "EngineDigit": (205, 205, 208),
    "EngineDigitOff": (110, 110, 114),
    # Status strips (black displays in the hardware area).
    "Strip": (6, 6, 7),
    "StripFrame": (90, 92, 96),
    "Readout": (112, 175, 240),
    "ReadoutDim": (36, 48, 66),
    "LedOff": (54, 55, 58),
    "LedMute": (230, 40, 36),
    "LedSolo": (246, 214, 40),
    "LedGreen": (70, 210, 80),
    # LCD state colours.
    "MuteOn": (232, 84, 76),
    "SoloOn": (240, 214, 64),
    "VuTrack": (120, 127, 148),
    "VuLit": (18, 25, 60),
    # Modal dialogs.
    "ModalFill": (237, 237, 237),
    "ModalGroup": (229, 229, 229),
    "ModalBorder": (137, 137, 137),
    "ModalField": (243, 243, 243),
    "ModalOn": (189, 196, 205),
    # Names dreist-gui itself looks up (window backdrop, base defaults).
    "PanelDarkest": (58, 59, 62),
    "StandardControlBackground": (222, 229, 237),
    "StandardControlBorder": (18, 25, 60),
    "StandardControlText": (18, 25, 60),
    "StandardControlHighlight": (18, 25, 60),
    "ToolTipPanelColour": (255, 250, 214),
    "ToolTipBorderColour": (18, 25, 60),
    "ToolTipTextColour": (18, 25, 60),
    "LogoMark": (236, 189, 47),
}


def colour_value(c):
    """`#RRGGBB` for opaque colours, `r, g, b, a` otherwise."""
    if len(c) == 3 or c[3] == 255:
        return "#%02X%02X%02X" % tuple(c[:3])
    return "%d, %d, %d, %d" % tuple(c)


# ---------------------------------------------------------------- fonts

TEXT_FAMILY = "Liberation Sans"  # metric stand-in for v1's Arial
ICON_FAMILY = "Einst-Icons"

# (size, bold) of every text font a converted style may name, plus the
# theme's own; named Sans<size>[Bold].
TEXT_SIZES = [(7, True), (8, False), (8, True), (9, False), (9, True), (10, False), (10, True), (11, False), (11, True),
              (12, False), (12, True), (13, False), (14, False), (14, True), (18, False), (24, False), (32, False)]
ICON_SIZES = [8, 10, 11, 12, 14, 16, 18, 22]


def text_font(size, bold=False):
    return f"Sans{size}{'Bold' if bold else ''}"


def icon_font(size):
    return f"Icons{size}"


# ---------------------------------------------------------------- glyphs

# Einst-Icons code points (the icon font shared with assets/ui).
G = {
    "up": "", "left": "", "down": "", "right": "", "stop": "",
    "chev_down": "", "chev_left": "", "chev_right": "", "chev_up": "",
    "power": "", "phase": "", "redo": "", "undo": "", "settings": "",
    "metronome": "", "record": "", "parent": "", "folder": "", "refresh": "",
    "reverse": "", "speaker": "", "lock": "", "eraser": "", "midi": "",
    "to_end": "", "to_start": "", "range": "", "crossfade": "", "minus": "",
    "plus": "", "left_right": "", "sine": "", "wave": "", "pulse": "",
    "close": "", "snapshot": "", "skip_start": "", "loop": "", "quantise": "",
    "retrigger": "", "warning": "", "a": "", "ring": "", "linked": "",
}


# ---------------------------------------------------------------- base styles

def base_styles():
    """(name, attrs) of the theme's own styles, in file order."""
    S = []

    def st(name, parent="ApplicationStyle", **a):
        S.append((name, dict(parentStyle0=parent, **a)))

    # Element bases (one per v1 style class).
    st("BaseStyle", realCornerRounding="0", optDrawShadow="false", font=text_font(12), colText="Ink", colAltText="Ink",
       colFill="Transparent", colAltFill="Transparent", colBorder="Transparent", colAltBorder="Transparent")
    st("BasePanelStyle", "BaseStyle", optDrawBorder="false")
    st("BaseLabelStyle", "BaseStyle", optDrawBorder="false", enumTextHorizontalAlign="left")
    st("BaseTextBoxStyle", "BaseStyle", colFill="LcdLight", colBorder="Ink", realCornerRounding="2",
       enumTextHorizontalAlign="left", realTextOffset="3")
    st("BaseButtonStyle", "BaseStyle", colFill="LcdLight", colBorder="Ink", colAltFill="Ink", colAltBorder="Ink",
       colAltText="LcdLight", realCornerRounding="2", enumTextShortenAlgorithm="end")
    st("BaseKnobStyle", "BaseStyle", colFill="InkSoft", colFillHighlight="Ink", colAltFill="LcdLight", colAltBorder="Ink")
    st("BaseSliderStyle", "BaseStyle", colFill="InkSoft", colFillHighlight="Ink", colAltFill="LcdLight",
       colAltBorder="Ink", realCornerRounding="2")
    st("BaseImageStyle", "BasePanelStyle")
    st("BaseMeterStyle", "BaseStyle", optDrawBorder="false", colFill="VuTrack", colAltFill="VuLit", colAltBorder="VuLit")
    st("BaseTableStyle", "BaseStyle", colFill="LcdMid", colBorder="Ink", optDrawBorder="true")

    # Chassis and screens.
    st("ChassisStyle", "BasePanelStyle", colFill="Chassis", colBorder="ChassisEdge", optDrawBorder="true", realCornerRounding="5")
    st("ChassisBarStyle", "BasePanelStyle", colFill="ChassisBar", realCornerRounding="1")
    st("ChassisFrameStyle", "BasePanelStyle", colBorder="ChassisLine", optDrawBorder="true", realCornerRounding="2")
    st("ChassisWellStyle", "BasePanelStyle", colFill="ChassisWell", realCornerRounding="5")
    st("ChassisBackingStyle", "BasePanelStyle", colFill="ChassisBar", realCornerRounding="5")
    st("SectionLabelStyle", "BaseLabelStyle", colText="ChassisText", font=text_font(7, True), enumTextHorizontalAlign="right")
    st("ScreenBezelStyle", "BasePanelStyle", colFill="Bezel", colBorder="ChassisEdge", optDrawBorder="true", realCornerRounding="4")
    st("ScreenLcdStyle", "BasePanelStyle", colFill="Lcd", realCornerRounding="9")
    st("StatusStripStyle", "BasePanelStyle", colFill="Strip", colBorder="StripFrame", optDrawBorder="true", realCornerRounding="3")
    st("LogoStyle", "BasePanelStyle", enumDisabledDrawMode="NormalDrawing", colFill="Amber", colAltFill="AmberSoft")
    st("GlyphLabelStyle", "BaseLabelStyle", colText="InkMid", font=icon_font(12), enumTextHorizontalAlign="center")
    st("ChassisGlyphLabelStyle", "GlyphLabelStyle", colText="HwInk", font=icon_font(10))

    # LCD panels.
    st("LcdPanelStyle", "BasePanelStyle", colFill="Lcd", colBorder="InkMid", optDrawBorder="true", realCornerRounding="4")
    st("LcdHeaderStyle", "BasePanelStyle", colFill="LcdDim", colBorder="InkMid", optDrawBorder="true", realCornerRounding="3")
    st("LcdSlotStyle", "BasePanelStyle", colFill="LcdMid", colBorder="InkSoft", optDrawBorder="true", realCornerRounding="3")
    st("LcdRowEvenStyle", "BasePanelStyle", colFill="LcdMid", colBorder="InkSoft", optDrawBorder="true", realCornerRounding="2")
    st("LcdRowOddStyle", "LcdRowEvenStyle", colFill="LcdShade")
    st("LcdRowSelectedStyle", "LcdRowEvenStyle", colFill="AmberSoft", colBorder="Ink")
    st("LcdGlueStyle", "BasePanelStyle", colFill="AmberSoft")
    st("LcdRulerStyle", "BasePanelStyle", colFill="LcdDim", realCornerRounding="1")
    st("LcdFieldStyle", "BasePanelStyle", colFill="LcdLight", colBorder="Ink", optDrawBorder="true", realCornerRounding="2")
    st("LcdPlotStyle", "BasePanelStyle", colFill="LcdLight", colBorder="InkMid", optDrawBorder="true", realCornerRounding="2")

    # LCD buttons.
    st("LcdButtonStyle", "BaseButtonStyle")
    st("LcdIconButtonStyle", "LcdButtonStyle", font=icon_font(10))
    st("LcdMuteButtonStyle", "LcdButtonStyle", colAltFill="MuteOn", colAltText="Ink")
    st("LcdSoloButtonStyle", "LcdButtonStyle", colAltFill="SoloOn", colAltText="Ink")
    st("LcdMenuButtonStyle", "LcdButtonStyle", enumTextHorizontalAlign="left", realTextOffset="5")
    st("LcdBigButtonStyle", "LcdButtonStyle", font=text_font(11, True), realCornerRounding="3")
    st("LcdPowerButtonStyle", "LcdIconButtonStyle", font=icon_font(14), colAltFill="Amber", colAltText="Ink", realCornerRounding="3")
    st("LcdFlatButtonStyle", "LcdButtonStyle", colFill="Transparent", colBorder="Transparent", colText="InkMid",
       colAltFill="Transparent", colAltBorder="Transparent", colAltText="Ink")
    st("LcdFlatIconButtonStyle", "LcdFlatButtonStyle", font=icon_font(10))

    # Bezel tabs.
    st("TabButtonStyle", "BaseButtonStyle", colFill="Transparent", colBorder="Transparent", colText="BezelText",
       colAltFill="BezelTabOn", colAltBorder="BezelTabOn", colAltText="BezelTextOn", font=text_font(10, True),
       realCornerRounding="3", optRoundBottomLeft="false", optRoundBottomRight="false")
    st("BezelIconButtonStyle", "TabButtonStyle", font=icon_font(11), colAltFill="Transparent", colAltBorder="Transparent")

    # Hardware buttons.
    st("HwButtonStyle", "BaseButtonStyle", colFill="HwFace", colBorder="HwFrame", colText="HwInk", colAltFill="HwFaceOn",
       colAltBorder="HwFrameDark", colAltText="HwInkOn", realCornerRounding="3", font=text_font(9, True))
    st("HwIconButtonStyle", "HwButtonStyle", font=icon_font(14))
    st("HwSmallIconButtonStyle", "HwButtonStyle", font=icon_font(10))
    st("HwPlayButtonStyle", "HwIconButtonStyle", colText="HwGreen", colAltText="HwGreenOn", font=icon_font(16))
    st("HwRecordButtonStyle", "HwIconButtonStyle", colText="HwRed", colAltText="HwRedOn", font=icon_font(16))
    st("HwTinyButtonStyle", "HwButtonStyle", font=text_font(7, True), realCornerRounding="2")
    st("HwMuteButtonStyle", "HwTinyButtonStyle", colAltFill="LedMute", colAltBorder="HwFrameDark", colAltText="HwFace")
    st("HwSoloButtonStyle", "HwTinyButtonStyle", colAltFill="LedSolo", colAltBorder="HwFrameDark", colAltText="HwInk")
    st("HwPowerButtonStyle", "HwSmallIconButtonStyle", font=icon_font(8), colAltFill="LedGreen", colAltText="HwInk",
       realCornerRounding="2")
    st("IndicatorStyle", "BaseButtonStyle", colFill="LedOff", colBorder="Strip", colText="HwInk", colAltFill="Readout",
       colAltBorder="Strip", colAltText="Strip", realCornerRounding="1", font=text_font(7, True))
    st("IndicatorMuteStyle", "IndicatorStyle", colAltFill="LedMute", optDrawBorder="false")
    st("IndicatorSoloStyle", "IndicatorStyle", colAltFill="LedSolo", optDrawBorder="false")
    st("IndicatorErrorStyle", "IndicatorStyle", colFill="LedMute")

    # Engine selectors, pattern keys, pads.
    st("EngineButtonStyle", "BaseButtonStyle", colFill="EngineFace", colBorder="EngineRim", colText="EngineDigit",
       colAltFill="EngineFace", colAltBorder="Amber", colAltText="ChassisText", realBorderWidth="2",
       realCornerRounding="19", font=text_font(18))
    st("EngineButtonPowerOffStyle", "EngineButtonStyle", colFill="EngineFaceOff", colAltFill="EngineFaceOff",
       colText="EngineDigitOff", colAltText="EngineDigitOff")
    st("PatternKeyStyle", "BaseButtonStyle", colFill="HwFace", colBorder="HwFrame", colText="HwInk", colAltFill="HwFaceOn",
       colAltBorder="HwFrameDark", realCornerRounding="3", font=text_font(8))
    st("PatternKeyAllocatedStyle", "PatternKeyStyle", colFill="LcdMid", colAltFill="LcdDim")
    st("PatternKeyEditingStyle", "PatternKeyStyle", colFill="AmberSoft", colAltFill="AmberSoft")
    st("PatternKeySelectedStyle", "PatternKeyStyle", colFill="Amber", colAltFill="AmberDeep", colBorder="HwFrameDark")
    st("PadSlotStyle", "BasePanelStyle")
    st("PadStyle", "BasePanelStyle", colFill="PadFace", colBorder="PadFrame", optDrawBorder="true", realCornerRounding="4",
       colText="ChassisText")

    # Knobs, faders, meters.
    st("HardKnobStyle", "BaseKnobStyle", colFill="HwFrameDark", colFillHighlight="Readout", colAltFill="HwFace", colAltBorder="HwFrameDark")
    st("HardKnobBodyStyle", "BasePanelStyle", colFill="HwFace", colBorder="HwFrame", optDrawBorder="true", realCornerRounding="100")
    st("LcdKnobStyle", "BaseKnobStyle")
    st("LcdKnobBodyStyle", "BasePanelStyle", colFill="LcdLight", colBorder="InkSoft", optDrawBorder="true", realCornerRounding="100")
    st("SpinnerStyle", "BaseKnobStyle", colFill="LcdDim", colFillHighlight="LcdDim")
    st("LcdFaderStyle", "BaseSliderStyle")
    st("LcdScrollBarStyle", "BaseSliderStyle", colFill="LcdLight", colAltFill="InkMid")
    st("LcdMeterStyle", "BaseMeterStyle")
    st("StripMeterStyle", "BaseMeterStyle", colFill="ReadoutDim", colAltFill="Readout", colAltBorder="Readout")

    # Readouts on the status strips.
    st("ReadoutStyle", "BaseLabelStyle", colText="Readout", enumTextHorizontalAlign="center", font=text_font(11, True))
    st("ReadoutButtonStyle", "BaseButtonStyle", colFill="Transparent", colBorder="ReadoutDim", colText="Readout",
       colAltFill="ReadoutDim", colAltBorder="Readout", colAltText="Readout", font=text_font(9, True))

    # Modal dialogs.
    st("ModalButtonStyle", "BaseButtonStyle", colFill="ModalGroup", colAltFill="ModalOn", colBorder="Ink", colAltBorder="Ink",
       colAltText="Ink", realCornerRounding="3")
    st("ModalFieldStyle", "BaseTextBoxStyle", colFill="ModalField", colBorder="Ink")
    st("ModalProgressStyle", "BaseStyle", optDrawBorder="true", colFill="ModalField", colBorder="Ink", colAltFill="InkMid", colAltBorder="Ink")
    st("CheckBoxStyle", "BaseStyle", colFill="LcdLight", colBorder="Ink", colAltFill="LcdLight", colAltBorder="Ink",
       colTick="Ink", realCornerRounding="2", realTickInset="3", realTickLineWidth="2")

    # Tooltips and menus (read by dreist-gui by these names).
    st("TooltipStyle", "BaseStyle", optDrawBorder="true", enumDisabledDrawMode="NormalDrawing", colFill="ToolTipPanelColour",
       colBorder="ToolTipBorderColour", colText="ToolTipTextColour")
    st("MenuStyle", "BaseStyle", optDrawBorder="true", optDrawShadow="true", colFill="LcdLight", colBorder="Ink", colAltFill="Amber",
       colDivider="InkSoft", colSubMenuArrow="Ink", colText="Ink", realCornerRounding="3")
    return S


# Style classes of the theme styles that dreist-gui dispatches on by class.
STYLE_CLASS = {"CheckBoxStyle": "CheckBoxButtonRendererStyle", "MenuStyle": "CtrlOptionMenuStyle"}
