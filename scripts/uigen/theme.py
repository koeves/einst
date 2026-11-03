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
    "Chassis": (207, 208, 210),
    "ChassisEdge": (58, 59, 62),
    "ChassisBar": (128, 129, 131),
    "ChassisBacking": (118, 119, 121),
    "ChassisLine": (128, 129, 132),
    "ChassisWell": (128, 129, 133),
    "ChassisText": (236, 236, 236),
    "Bezel": (10, 11, 13),
    "BezelText": (178, 182, 188),
    "BezelTextOn": (176, 212, 245),
    "BezelTabOn": (38, 46, 78),
    # Hardware buttons, keys and pads.
    "HwFace": (226, 227, 229),
    "HwFaceOn": (236, 228, 160),
    "HwFrame": (120, 122, 126),
    "HwFrameDark": (52, 54, 58),
    "HwInk": (62, 64, 68),
    "HwInkOn": (18, 25, 60),
    "HwGreen": (34, 150, 46),
    "HwGreenOn": (60, 200, 70),
    "HwRed": (200, 34, 30),
    "HwRedOn": (240, 60, 50),
    "PadFace": (74, 75, 77),
    "PadFaceTop": (84, 85, 87),
    "PadText": (214, 214, 217),
    "PadFrame": (38, 39, 41),
    "EngineFace": (78, 79, 81),
    "EngineFaceOff": (40, 40, 42),
    "EngineRim": (150, 152, 156),
    "EngineDigit": (212, 212, 215),
    "EngineDigitOff": (110, 110, 114),
    # Status strips (black displays in the hardware area).
    "Strip": (10, 13, 16),
    "StripBand": (26, 28, 31),
    "StripFrame": (90, 92, 96),
    "Readout": (112, 175, 240),
    "ReadoutDim": (36, 48, 66),
    "LedOff": (54, 55, 58),
    "HwDark": (40, 41, 44),
    "HwDarkFrame": (22, 23, 25),
    "HwDarkText": (92, 94, 98),
    "HwDarkTextLight": (232, 233, 235),
    "HwBlueOn": (46, 84, 122),
    "HwBlueText": (120, 170, 220),
    "SongText": (84, 158, 200),
    "KeyLed": (186, 205, 231),
    "KnobFace": (196, 197, 199),
    "KnobRing": (44, 45, 47),
    "ReadoutMid": (62, 82, 106),
    "LedMute": (230, 40, 36),
    "LedSolo": (246, 214, 40),
    "LedGreen": (70, 210, 80),
    # LCD state colours.
    "MuteOn": (232, 84, 76),
    "SoloOn": (240, 214, 64),
    "VuTrack": (120, 127, 148),
    "VuLit": (18, 25, 60),
    # LCD content drawn by code (grid, song parts, browser).
    "RowSelected": (232, 226, 190),
    "PlayPosition": (158, 180, 198),
    "StepOn": (98, 106, 136),
    "LaneAlt": (204, 211, 221),
    "PartSelected": (229, 216, 158),
    "Crumb": (190, 195, 204),
    "BrowserSel": (150, 157, 177),
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
# v1's font sizes drew smaller than the same pixel size here (v1's screens:
# "Clear All" spans about 37 px where Liberation Sans 12 needs 46), so
# converted sizes are scaled by this.
TEXT_SCALE = 0.83
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
    st("BaseStyle", realCornerRounding="0", optDrawShadow="false", font=text_font(10), colText="Ink", colAltText="Ink",
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
    st("ChassisBackingStyle", "BasePanelStyle", colFill="ChassisBacking", realCornerRounding="5")
    st("SectionLabelStyle", "BaseLabelStyle", colText="ChassisText", font=text_font(9, True), enumTextHorizontalAlign="right")
    st("SectionLabelLeftStyle", "SectionLabelStyle", enumTextHorizontalAlign="left")
    st("WordmarkStyle", "BaseLabelStyle", colText="ChassisText", font=text_font(14, True), enumTextHorizontalAlign="left")
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
    st("LcdSideTabStyle", "LcdBigButtonStyle", colFill="InkMid", colBorder="Ink", colText="LcdLight", colAltFill="Ink",
       colAltBorder="Ink", colAltText="LcdLight", font=text_font(10, True))
    st("LcdPowerButtonStyle", "LcdIconButtonStyle", font=icon_font(14), colAltFill="Amber", colAltText="Ink", realCornerRounding="3")
    st("LcdFlatButtonStyle", "LcdButtonStyle", colFill="Transparent", colBorder="Transparent", colText="InkMid",
       colAltFill="Transparent", colAltBorder="Transparent", colAltText="Ink")
    st("LcdFlatIconButtonStyle", "LcdFlatButtonStyle", font=icon_font(10))

    # Bezel tabs.
    st("TabButtonStyle", "BaseButtonStyle", colFill="Transparent", colBorder="Transparent", colText="BezelText",
       colAltFill="Transparent", colAltBorder="Transparent", colAltText="BezelTextOn", font=text_font(10, True),
       realCornerRounding="3", optRoundBottomLeft="false", optRoundBottomRight="false")
    st("BezelIconButtonStyle", "TabButtonStyle", font=icon_font(11), colText="ChassisText", colAltText="ChassisText")

    # Hardware buttons.
    st("HwButtonStyle", "BaseButtonStyle", colFill="HwFace", colBorder="HwFrame", colText="HwInk", colAltFill="HwFaceOn",
       colAltBorder="HwFrameDark", colAltText="HwInkOn", realCornerRounding="3", font=text_font(9, True))
    st("HwIconButtonStyle", "HwButtonStyle", font=icon_font(14))
    st("HwSmallIconButtonStyle", "HwButtonStyle", font=icon_font(10))
    st("HwPlayButtonStyle", "HwIconButtonStyle", colText="HwGreen", colAltText="HwGreenOn", colAltFill="HwFace", font=icon_font(16))
    st("HwRecordButtonStyle", "HwIconButtonStyle", colText="HwRed", colAltText="HwRedOn", font=icon_font(16))
    st("HwTinyButtonStyle", "HwButtonStyle", font=text_font(7, True), realCornerRounding="2")
    st("HwDarkTinyButtonStyle", "HwTinyButtonStyle", colFill="HwDark", colBorder="HwDarkFrame", colText="HwDarkText",
       colAltBorder="HwDarkFrame", realCornerRounding="0")
    st("HwMuteButtonStyle", "HwDarkTinyButtonStyle", colAltFill="LedMute", colAltText="HwFace")
    st("HwSoloButtonStyle", "HwDarkTinyButtonStyle", colAltFill="LedSolo", colAltText="HwInk")
    st("HwPowerButtonStyle", "HwDarkTinyButtonStyle", font=icon_font(8), colAltFill="HwBlueOn", colAltText="HwBlueText")
    st("HwDarkButtonStyle", "HwButtonStyle", colFill="HwDark", colBorder="HwDarkFrame", colText="HwDarkTextLight", realDisabledAlpha="0.3",
       colAltFill="HwDarkFrame", colAltBorder="HwDarkFrame", colAltText="HwDarkTextLight")
    st("HwDarkIconButtonStyle", "HwDarkButtonStyle", font=icon_font(8))
    st("HwSongButtonStyle", "HwButtonStyle", colText="SongText", colAltText="SongText", colAltFill="LcdLight")
    st("IndicatorStyle", "BaseButtonStyle", colFill="LedOff", colBorder="Strip", colText="HwInk", colAltFill="Readout",
       colAltBorder="Strip", colAltText="Strip", realCornerRounding="1", font=text_font(7, True))
    st("IndicatorLabelStyle", "IndicatorStyle", colFill="Transparent", colBorder="Transparent", colText="ReadoutMid",
       colAltFill="Transparent", colAltBorder="Transparent", colAltText="Readout", enumTextHorizontalAlign="left")
    st("IndicatorMuteStyle", "IndicatorStyle", colAltFill="LedMute", optDrawBorder="false")
    st("IndicatorSoloStyle", "IndicatorStyle", colAltFill="LedSolo", optDrawBorder="false")
    st("IndicatorErrorStyle", "IndicatorStyle", colFill="LedMute")

    # Engine selectors, pattern keys, pads.
    st("EngineButtonStyle", "BaseButtonStyle", colFill="EngineFace", colBorder="EngineRim", colText="EngineDigit",
       colAltFill="EngineFace", colAltBorder="Amber", colAltText="ChassisText", realBorderWidth="2",
       realCornerRounding="13", font=text_font(14, True))
    st("EngineButtonPowerOffStyle", "EngineButtonStyle", colFill="EngineFaceOff", colAltFill="EngineFaceOff",
       colText="EngineDigitOff", colAltText="EngineDigitOff")
    st("PatternKeyStyle", "BaseButtonStyle", colFill="HwFace", colBorder="HwFrame", colText="HwInk", colAltFill="HwFace",
       colAltBorder="HwFrameDark", realCornerRounding="3", font=text_font(8))
    st("PatternKeyAllocatedStyle", "PatternKeyStyle", colFill="LcdLight", colAltFill="LcdLight", colBorder="InkMid")
    st("PatternKeyEditingStyle", "PatternKeyStyle", colBorder="AmberSoft", colAltBorder="AmberSoft", realBorderWidth="2")
    st("PatternKeySelectedStyle", "PatternKeyStyle", colBorder="Amber", colAltBorder="Amber", realBorderWidth="2")
    st("PatternKeyLedStyle", "BasePanelStyle", colFill="KeyLed", realCornerRounding="1")
    st("PadSlotStyle", "BasePanelStyle")
    st("PadStyle", "BasePanelStyle", colFill="PadFace", colBorder="PadFrame", optDrawBorder="true", realCornerRounding="4",
       colText="PadText")
    st("PadSelectedStyle", "PadStyle", colBorder="Amber", realBorderWidth="2")
    st("PadNameStyle", "BaseLabelStyle", colText="PadText", font=text_font(10), enumTextHorizontalAlign="center")
    st("PadLedStyle", "BasePanelStyle", colFill="PadFrame", realCornerRounding="1")
    for i in range(1, 5):  # a pad's colour (v1's PadColour1-4) on its first LED
        st(f"PadLed{i}Style", "PadLedStyle", colFill=f"PadColour{i}")

    # Knobs, faders, meters.
    st("HardKnobStyle", "BaseKnobStyle", colFill="KnobRing", colFillHighlight="Readout", colAltFill="KnobRing", colAltBorder="HwFace")
    st("HardKnobBodyStyle", "BasePanelStyle", colFill="KnobFace", colBorder="HwFrame", optDrawBorder="true", realCornerRounding="100")
    st("HardKnobSmallStyle", "BaseKnobStyle", colFill="KnobRing", colFillHighlight="HwDarkText", colAltFill="KnobRing", colAltBorder="HwFace")
    st("LcdKnobStyle", "BaseKnobStyle")
    st("LcdKnobBodyStyle", "BasePanelStyle", colFill="LcdLight", colBorder="InkSoft", optDrawBorder="true", realCornerRounding="100")
    st("SpinnerStyle", "BaseKnobStyle", colFill="Transparent", colFillHighlight="InkSoft")
    st("LcdFaderStyle", "BaseSliderStyle", enumHandleShape="Rect")
    st("LcdPanStyle", "BaseSliderStyle", enumHandleShape="Rect", realHandleLength="10")
    st("LcdScrollBarStyle", "BaseSliderStyle", colFill="LcdLight", colAltFill="InkMid")
    st("LcdMeterStyle", "BaseMeterStyle")
    st("StripMeterStyle", "BaseMeterStyle", colFill="ReadoutDim", colAltFill="Readout", colAltBorder="Readout")

    # Readouts on the status strips.
    st("ReadoutStyle", "BaseLabelStyle", colText="Readout", enumTextHorizontalAlign="center", font=text_font(11, True))
    st("ReadoutMenuStyle", "BaseButtonStyle", colFill="Transparent", colBorder="Transparent", colText="Readout", colAltFill="ReadoutDim",
       colAltBorder="Transparent", colAltText="Readout", font=text_font(9, True), enumTextHorizontalAlign="left")
    st("ReadoutButtonStyle", "BaseButtonStyle", colFill="Transparent", colBorder="ReadoutDim", colText="Readout",
       colAltFill="ReadoutDim", colAltBorder="Readout", colAltText="Readout", font=text_font(9, True))

    # Modal dialogs.
    st("ModalButtonStyle", "BaseButtonStyle", colFill="ModalGroup", colAltFill="ModalOn", colBorder="Ink", colAltBorder="Ink",
       colAltText="Ink", realCornerRounding="3")
    st("ModalFieldStyle", "BaseTextBoxStyle", colFill="ModalField", colBorder="Ink")
    st("ModalProgressStyle", "BaseStyle", optDrawBorder="true", colFill="ModalField", colBorder="Ink", colAltFill="InkMid", colAltBorder="Ink")
    st("CheckBoxStyle", "BaseStyle", colFill="LcdLight", colBorder="Ink", colAltFill="LcdLight", colAltBorder="Ink",
       colTick="Ink", realCornerRounding="2", realTickInset="3", realTickLineWidth="2")

    # The LCD content v1 drew in code (step grid, song parts, browser tree
    # and lists), after v1's colour table (the survey's palette section);
    # the preview example (crates/einst/examples/v1skins.rs) builds it.
    st("TimelineStyle", "BasePanelStyle", colFill="LcdLight", colBorder="InkMid", optDrawBorder="true")
    st("TimelineLabelStyle", "BaseLabelStyle", font=text_font(9, True))
    st("TimelineTickStyle", "BasePanelStyle", colFill="InkMid")
    st("GridBackStyle", "BasePanelStyle", colFill="LcdLight", colBorder="InkMid", optDrawBorder="true")
    st("GridShadeStyle", "BasePanelStyle", colFill="LcdMid")
    st("GridLineStyle", "BasePanelStyle", colFill="LcdDim")
    st("GridRowSelectedStyle", "BasePanelStyle", colFill="RowSelected")
    st("GridPositionStyle", "BasePanelStyle", colFill="PlayPosition")
    st("StepOnStyle", "BasePanelStyle", colFill="StepOn", colBorder="Ink", optDrawBorder="true", realCornerRounding="1")
    st("SongLaneStyle", "BasePanelStyle", colFill="LcdLight")
    st("SongLaneAltStyle", "BasePanelStyle", colFill="LaneAlt")
    st("SongLaneSelectedStyle", "BasePanelStyle", colFill="RowSelected")
    st("SongPartBodyStyle", "BaseLabelStyle", colFill="LcdLight", colBorder="Ink", optDrawBorder="true", realCornerRounding="2",
       font=text_font(10, True), enumTextHorizontalAlign="center")
    st("SongPartSelectedBodyStyle", "SongPartBodyStyle", colFill="PartSelected")
    st("SongPartFootStyle", "BasePanelStyle", colFill="Ink")
    st("TrackNameStyle", "BaseLabelStyle", colFill="LcdLight", colBorder="InkSoft", optDrawBorder="true", font=text_font(10), realTextOffset="3")
    st("TrackArrowStyle", "LcdFlatButtonStyle", colText="InkMid", font=text_font(9))
    st("TrackNumberStyle", "BaseLabelStyle", font=text_font(24), enumTextHorizontalAlign="center")
    st("BrowserCrumbStyle", "BaseButtonStyle", colFill="Crumb", colBorder="InkMid", realCornerRounding="3", font=text_font(10))
    st("BrowserTreeHeadStyle", "BaseLabelStyle", font=text_font(10), realTextOffset="4")
    st("BrowserTreeItemStyle", "BaseLabelStyle", font=text_font(10), realTextOffset="22")
    st("BrowserTreeSelectedStyle", "BrowserTreeItemStyle", colFill="BrowserSel")
    st("BrowserTreeIconStyle", "GlyphLabelStyle", colText="InkMid", font=icon_font(12))
    st("BrowserParentStyle", "BaseButtonStyle", colFill="LcdLight", colBorder="InkSoft", realCornerRounding="0", font=text_font(10))
    st("BrowserListStyle", "BaseTableStyle", font=text_font(10), colFill="LcdShade", colBorder="InkSoft", colRow="LcdLight",
       colRowAlt="LcdMid", colSelection="BrowserSel", colHeaderFill="LcdLight", colHeaderDivider="InkMid",
       colColumnDivider="InkSoft", optAltRowColours="true", optColumnDividers="true", optRowDividers="true", colRowDivider="InkSoft",
       intRowHeight="25", intHeaderHeight="17")
    st("BrowserBadgeStyle", "BaseButtonStyle", colFill="LcdLight", colBorder="InkMid", realCornerRounding="2", font=text_font(7, True))
    st("BrowserScrollStyle", "BasePanelStyle", colFill="Ink", realCornerRounding="2")

    # Tooltips and menus (read by dreist-gui by these names).
    st("TooltipStyle", "BaseStyle", optDrawBorder="true", enumDisabledDrawMode="NormalDrawing", colFill="ToolTipPanelColour",
       colBorder="ToolTipBorderColour", colText="ToolTipTextColour")
    st("MenuStyle", "BaseStyle", optDrawBorder="true", optDrawShadow="true", colFill="LcdLight", colBorder="Ink", colAltFill="Amber",
       colDivider="InkSoft", colSubMenuArrow="Ink", colText="Ink", realCornerRounding="3")
    return S


# Style classes of the theme styles that dreist-gui dispatches on by class.
STYLE_CLASS = {"CheckBoxStyle": "CheckBoxButtonRendererStyle", "MenuStyle": "CtrlOptionMenuStyle", "BrowserListStyle": "CtrlTableViewStyle"}
