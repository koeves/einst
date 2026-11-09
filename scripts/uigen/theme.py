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
    # v1's LCD knobs: the arc's track and value, the body's ring and edge.
    "KnobLcdTrack": (169, 176, 191),
    "KnobLcdValue": (71, 77, 106),
    "KnobLcdEdge": (57, 65, 95),
    # v1's selected radio segment.
    "RadioOn": (149, 156, 173),
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
    "Chassis": (222, 224, 223),
    "ChassisLow": (196, 198, 197),
    "ChassisEdge": (58, 59, 62),
    "ChassisBar": (139, 144, 145),
    # The section bars and the front panel brighten to the right (v1's
    # chassis is lit from the top right): left, middle and right stops.
    "ChassisBarLeft": (124, 129, 130),
    "ChassisBarLeftEnd": (131, 136, 137),
    "ChassisBarMid": (134, 139, 140),
    "ChassisBarMidEnd": (162, 167, 168),
    "ChassisBarRight": (166, 171, 171),
    "ChassisBarEnd": (192, 196, 196),
    "ChassisFrontLit": (253, 254, 253),
    "ChassisFrontDim": (186, 188, 187),
    "ChassisBacking": (160, 164, 165),
    "ChassisBackingLow": (146, 150, 151),
    "ChassisBackingEdge": (112, 116, 117),
    "ChassisLine": (128, 129, 132),
    "ChassisWell": (128, 129, 133),
    "ChassisText": (236, 236, 236),
    "Bezel": (10, 11, 13),
    "BezelText": (160, 165, 166),
    "BezelTextOn": (196, 220, 232),
    "BezelTextGlow": (30, 104, 200),
    "BezelTabOn": (38, 46, 78),
    # Hardware buttons, keys and pads.
    "HwFace": (230, 231, 233),
    "HwFaceLow": (194, 198, 200),
    "HwFaceOn": (241, 250, 177),
    "HwFaceOnLow": (228, 232, 164),
    "HwFacePressed": (204, 207, 209),
    "HwFacePressedLow": (182, 185, 187),
    "HwGrey": (186, 187, 189),
    "HwGreyLow": (162, 164, 166),
    "HwGreyIcon": (244, 245, 246),
    "HwGreyFaceLow": (142, 144, 146),
    "HwBlueIcon": (40, 72, 150),
    "HwIconDim": (150, 156, 160),
    "MidiPin": (150, 152, 154),
    "HwFrame": (58, 62, 63),
    "HwFrameDark": (52, 54, 58),
    "HwFramePad": (104, 106, 107),
    "HwInk": (62, 64, 68),
    "HwInkOn": (18, 25, 60),
    "HwGreen": (34, 150, 46),
    "HwGreenOn": (60, 200, 70),
    "HwRed": (200, 34, 30),
    "HwRedOn": (240, 60, 50),
    "PadFace": (96, 100, 101),
    "PadFaceLow": (76, 78, 81),
    "PadFaceTop": (84, 85, 87),
    # (v1: pale steel-blue pad names with a soft glow, a near-black frame)
    "PadText": (168, 186, 194),
    "PadTextGlow": (120, 170, 200, 90),
    "PadFrame": (40, 44, 45),
    "EngineFace": (70, 71, 74),
    "EngineFaceOff": (40, 40, 42),
    "EngineRim": (126, 127, 129),
    "EngineDigit": (212, 212, 215),
    "EngineDigitOff": (110, 110, 114),
    # Status strips (black displays in the hardware area).
    "Strip": (10, 13, 16),
    "StripBand": (26, 28, 31),
    "StripFrame": (90, 92, 96),
    "Readout": (112, 175, 240),
    "ReadoutDim": (36, 48, 66),
    "LedOff": (54, 55, 58),
    "HwDark": (93, 93, 94),
    "HwDarkLow": (40, 40, 41),
    "HwDarkFrame": (22, 23, 25),
    "HwDarkText": (92, 94, 98),
    "HwDarkTextLight": (232, 233, 235),
    "HwBlueOn": (46, 84, 122),
    "HwBlueText": (120, 170, 220),
    "SongText": (40, 163, 210),
    "PanicFill": (226, 221, 173),
    "KeyLed": (176, 197, 226),
    "KeyLedOn": (150, 180, 232),
    "KeyLedSelected": (229, 218, 100),
    "KeyFace": (247, 248, 248),
    "KeyFaceLow": (232, 233, 232),
    "KeyFaceSelected": (240, 238, 214),
    "KeyFrame": (140, 142, 141),
    "KeyAmber": (240, 210, 50),
    "KnobFace": (250, 249, 250),
    "KnobFaceLow": (184, 187, 186),
    "KnobRing": (22, 25, 26),
    "KnobRingTop": (128, 128, 126),
    "KnobRingLow": (4, 4, 5),
    "KnobHalo": (238, 238, 237),
    "KnobHaloLow": (220, 221, 220),
    "KnobHaloEdge": (196, 198, 197),
    "KnobShadow": (100, 104, 104),
    "KnobMark": (112, 114, 114),
    "KnobShadowOut": (200, 202, 201, 0),
    "KnobArc": (44, 70, 120),
    "KnobDot": (70, 150, 230),
    "KnobPointer": (84, 86, 88),
    "SmallKnobFace": (66, 67, 66),
    "SmallKnobFaceLow": (26, 28, 27),
    "SmallKnobRim": (105, 109, 110),
    "ReadoutMid": (62, 82, 106),
    "LedMute": (230, 40, 36),
    "LedSolo": (246, 214, 40),
    "LedGreen": (70, 210, 80),
    # LCD state colours.
    "MuteOn": (232, 84, 76),
    "SoloOn": (240, 214, 64),
    "VuTrack": (132, 138, 154),
    "VuLit": (22, 28, 48),
    # LCD content drawn by code (grid, song parts, browser).
    "RowSelected": (229, 225, 200),
    "PlayPosition": (158, 180, 198),
    "GridLine": (186, 193, 205),
    "GridBack": (208, 213, 222),
    "GridDash": (150, 158, 174),
    # (v1: the bar lines solid, 2 px; the grid's left edge and the track
    # column's frame dark)
    "GridBar": (117, 118, 133),
    "GridEdge": (55, 47, 89),
    "TrackColumn": (223, 228, 234),
    "ScrollFrame": (140, 142, 155),
    "TrackColumnEdge": (69, 76, 84),
    # (v1: the shaded beat groups lighter than the others)
    "GridShade": (223, 228, 235),
    "StepOn": (90, 101, 131),
    "StepEdge": (51, 55, 92),
    "FxSlotLit": (204, 210, 220),
    "FxSlotDim": (122, 127, 142),
    # v1's effect panels: the card's edge (ink over the chain's frame, and
    # a mid line inside it), the slot bar items' shading.
    "FxCardEdge": (52, 59, 90),
    "FxSlotLitTop": (181, 185, 196),
    "FxSlotLitLow": (220, 229, 238),
    # Empty song lanes (v1-song.png: odd tracks darker than even ones,
    # both darker than the parts) and the selected track's empty lane.
    "Lane": (195, 202, 212),
    "LaneAlt": (207, 215, 220),
    "LaneSelected": (220, 216, 187),
    "PartSelected": (237, 224, 179),
    # Song lanes (round 4, measured): the navy lane foot and its ticks, the
    # selected part's checker, the dotted bar lines.
    "LaneFoot": (44, 48, 77),
    "LaneFootMark": (200, 206, 224),
    "PartHatch": (217, 205, 157),
    # (pattern ends inside parts: v1's `PartNormalPatternEnd` 1 px on / off)
    "BarDot": (18, 25, 60),
    # Song bar lines through empty lanes and the ruler: every bar, every
    # labelled bar (v1-song.png); the ruler's face and frame.
    "BarLine": (92, 98, 114),
    "BarLineMajor": (49, 55, 74),
    "Ruler": (210, 217, 229),
    "RulerEdge": (38, 45, 60),
    "Crumb": (190, 195, 204),
    # Code-drawn LCD content after v1's colour table (the survey's
    # palette section): wave display, envelopes, velocity split, scenes,
    # song parts being recorded, the area past a track's length.
    # (v1's colour table names a beige wave background, but the display
    # tiles its light `WaveBackgroundTile` over the wave, the grey
    # `MarkerBackgroundTile` over the marker strip and the half-transparent
    # light `DisabledBackgroundTile` outside the start / end markers
    # [re:627879]; the beige shows through that last one)
    "WaveBack": (222, 229, 237),
    "WaveOutside": (211, 214, 217),
    "WaveMarkerFill": (255, 225, 187),
    "WaveLoopMarkerFill": (235, 235, 235),
    "WaveMarkerEdge": (30, 30, 30),
    "EnvHandle": (148, 155, 172, 200),
    "EnvHandleEdge": (93, 100, 125),
    "WaveMarker": (250, 121, 25),
    "WaveLoopMarker": (24, 41, 62),
    "WaveDisabled": (169, 176, 191, 180),
    "WaveTitle": (18, 25, 60, 140),
    "EnvAmpBack": (76, 83, 111, 80),
    "EnvFreeBack": (0, 0, 0, 80),
    "EnvLine": (45, 51, 76),
    "SplitGraph": (76, 83, 111, 200),
    "SplitTick": (18, 25, 60, 102),
    "SplitTickBar": (18, 25, 60, 51),
    # Scene page (`ScenePageBackground`, `SceneBackground`, `Scene*` trigger
    # art, measured): the slate frame, the wells, the slot tiles, the
    # trigger of a used scene and the edges.
    "SceneFrame": (76, 83, 111),
    "SceneWell": (169, 176, 191),
    "SceneTableWell": (199, 206, 217),
    "SceneWellEdge": (113, 120, 142),
    "SceneWellEdgeLow": (128, 135, 156),
    "ScenePanelEdge": (47, 54, 86),
    "ScenePanelEdgeLow": (120, 127, 148),
    "SceneUsed": (169, 176, 191),
    "SceneEdge": (76, 83, 111),
    "SceneMark": (101, 108, 132),
    "PastLength": (169, 176, 191, 140),
    "RecordingPart": (222, 229, 237, 200),
    "RetroPart": (187, 81, 81, 75),
    "RetroEdge": (187, 81, 81),
    # LCD controls (light faces, mid blue-grey edges).
    "LcdFace": (240, 245, 251),
    "LcdFaceLow": (222, 228, 238),
    # (v1's LCD button bitmaps: a dark slate side frame)
    "LcdEdge": (88, 94, 120),
    "LcdKeyFace": (238, 245, 253),
    "LcdKeyEdge": (150, 158, 170),
    "LcdKeyText": (56, 60, 72),
    "LcdOn": (64, 72, 104),
    "LcdOnLow": (18, 25, 60),
    "LcdCap": (92, 98, 114),
    "LcdCapLow": (14, 18, 28),
    "LcdGrip": (222, 228, 238),
    "StripSelected": (69, 74, 100),
    "StripHead": (64, 69, 94),
    "StripHeadEdge": (18, 23, 44),
    "StripSelectedEdge": (18, 25, 60),
    "LcdGroove": (138, 145, 160),
    "StripOdd": (189, 196, 209),
    "StripEven": (158, 166, 181),
    # v1's pad / layer selector (`LayerSelectionBackground*`,
    # `PadSelection*Background`): the rows' edge, the edit panels' group
    # boxes and the inner shadow round the panels' screen.
    "RowEdge": (106, 112, 136),
    "GroupEdge": (94, 101, 126),
    "InsetEdge": (75, 76, 90),
    "DividerEdge": (39, 46, 78),
    "SamplerInsetEdge": (60, 67, 97),
    "BigButtonText": (90, 97, 128),
    "BrowserSel": (158, 170, 186),
    "BrowserTreeSel": (108, 122, 152),
    "BrowserList": (219, 222, 232),
    # File list cells (the reference screenshots): faces, the light lines
    # above and below each row's navy foot.
    "BrowserRow": (211, 218, 227),
    "BrowserRowAlt": (195, 202, 213),
    "BrowserRowLight": (243, 252, 255),
    "BrowserScrollTrack": (199, 208, 209),
    # The path navigator's strip (v1's ParentButtonTileUp / *Stub art).
    "CrumbFace": (222, 229, 238),
    "CrumbTop": (88, 92, 122),
    "CrumbLow": (170, 177, 194),
    # The header's navigation plate and keys (NavigationBackground,
    # Prev* / Next* art).
    "NavPlate": (76, 83, 111),
    "SearchField": (228, 233, 240),
    "SearchEdge": (135, 143, 163),
    "NavKey": (221, 227, 236),
    "NavKeyDown": (169, 176, 191),
    "BrowserHeader": (221, 226, 230),
    "BrowserSplit": (18, 25, 60),
    "BrowserTreeBack": (222, 227, 233),
    "IconLine": (64, 68, 84),
    "IconFace": (204, 209, 218),
    "IconDark": (40, 46, 84),
    # Modal dialogs.
    "ModalFill": (237, 237, 237),
    "ModalGroup": (229, 229, 229),
    "ModalBorder": (137, 137, 137),
    "ModalField": (243, 243, 243),
    "ModalOn": (189, 196, 205),
    "ModalList": (250, 250, 250),
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
    # Disabled controls fade well back, like v1's greyed `imDisabled` art
    # (empty layers, the controls slicing locks).
    st("BaseStyle", realCornerRounding="0", optDrawShadow="false", font=text_font(10), colText="Ink", colAltText="Ink",
       colFill="Transparent", colAltFill="Transparent", colBorder="Transparent", colAltBorder="Transparent",
       realDisabledAlpha="0.4")
    st("BasePanelStyle", "BaseStyle", optDrawBorder="false")
    st("BaseLabelStyle", "BaseStyle", optDrawBorder="false", enumTextHorizontalAlign="left")
    st("BaseTextBoxStyle", "BaseStyle", colFill="LcdFace", colBorder="LcdEdge", realCornerRounding="2", realGradientShade="0.05",
       enumTextHorizontalAlign="left", realTextOffset="3")
    st("BaseButtonStyle", "BaseStyle", colFill="LcdLight", colBorder="Ink", colAltFill="Ink", colAltBorder="Ink",
       colAltText="LcdLight", realCornerRounding="2", enumTextShortenAlgorithm="end")
    st("BaseKnobStyle", "BaseStyle", colFill="InkSoft", colFillHighlight="Ink", colAltFill="LcdLight", colAltBorder="Ink",
       colFace="LcdFace", colFaceGradientTo="LcdFaceLow", realFaceInset="1", optPointer="true", colPointer="Ink",
       realPointerWidth="2", realPointerInset="2")
    st("BaseSliderStyle", "BaseStyle", colFill="InkSoft", colFillHighlight="Ink", colAltFill="LcdLight",
       colAltBorder="Ink", realCornerRounding="2")
    st("BaseImageStyle", "BasePanelStyle")
    st("BaseMeterStyle", "BaseStyle", optDrawBorder="false", colFill="VuTrack", colAltFill="VuLit", colAltBorder="VuLit")
    st("BaseTableStyle", "BaseStyle", colFill="LcdMid", colBorder="Ink", optDrawBorder="true")

    # Chassis and screens.
    st("ChassisStyle", "BasePanelStyle", colFill="Chassis", colFillGradientTo="ChassisLow", colBorder="ChassisEdge", optDrawBorder="true",
       realCornerRounding="5")
    st("ChassisBarStyle", "BasePanelStyle", colFill="ChassisBarLeft", colFillGradientTo="ChassisBarLeftEnd", enumGradientDirection="Horizontal",
       realCornerRounding="1", optBevel="true", colBevelLight="0, 0, 0, 40", colBevelDark="255, 255, 255, 150")
    st("ChassisBarMidStyle", "ChassisBarStyle", colFill="ChassisBarMid", colFillGradientTo="ChassisBarMidEnd")
    st("ChassisBarRightStyle", "ChassisBarStyle", colFill="ChassisBarRight", colFillGradientTo="ChassisBarEnd")
    # (lit from far to the top right: brighter rightwards and a little
    # towards the top; measured along the rows above the section bars and
    # under the pattern keys)
    st("ChassisFrontStyle", "BasePanelStyle", colFill="ChassisFrontLit", colFillGradientTo="ChassisFrontDim", enumGradientDirection="Radial",
       realGradientCentreX="3", realGradientCentreY="-0.3", realGradientInnerRadius="2", realGradientRadius="3.2",
       realNoiseAmount="0.025", realNoiseStreak="40", intNoiseSeed="7")
    st("ChassisFrameStyle", "BasePanelStyle", colBorder="ChassisLine", optDrawBorder="true", realCornerRounding="2")
    st("ChassisWellStyle", "BasePanelStyle", colFill="ChassisWell", realCornerRounding="5")
    st("ChassisBackingStyle", "BasePanelStyle", colFill="ChassisBacking", colFillGradientTo="ChassisBackingLow", colBorder="ChassisBackingEdge",
       optDrawBorder="true", realCornerRounding="5", optBevel="true", colBevelLight="0, 0, 0, 50", colBevelDark="255, 255, 255, 120")
    st("SectionLabelStyle", "BaseLabelStyle", colText="ChassisText", font=text_font(9, True), enumTextHorizontalAlign="right",
       realTextEmbolden="0.6", realLetterSpacing="0.4")
    st("SectionLabelLeftStyle", "SectionLabelStyle", enumTextHorizontalAlign="left")
    st("WordmarkStyle", "BaseLabelStyle", colText="ChassisText", font=text_font(14, True), enumTextHorizontalAlign="left",
       realTextScaleX="0.88", realLetterSpacing="-0.3")
    # (its top edge catches the light: a pale rim under the border)
    st("ScreenBezelStyle", "BasePanelStyle", colFill="Bezel", colBorder="ChassisEdge", optDrawBorder="true", realCornerRounding="4",
       optBevel="true", colBevelLight="210, 212, 214, 190", colBevelDark="0, 0, 0, 0")
    st("ScreenLcdStyle", "BasePanelStyle", colFill="Lcd", realCornerRounding="9", optBevel="true", colBevelLight="0, 0, 0, 110",
       colBevelDark="255, 255, 255, 90")
    # (v1: the screen is shaded under its top and left bezel edges)
    st("ScreenShadeTopStyle", "BasePanelStyle", colFill="0, 0, 0, 80", colFillGradientTo="0, 0, 0, 0", realCornerRounding="9",
       optRoundBottomLeft="false", optRoundBottomRight="false")
    st("ScreenShadeLeftStyle", "BasePanelStyle", colFill="0, 0, 0, 70", colFillGradientTo="0, 0, 0, 0", enumGradientDirection="Horizontal",
       realCornerRounding="9", optRoundTopRight="false", optRoundBottomRight="false")
    st("WellShadowTopStyle", "BasePanelStyle", colFill="0, 0, 0, 100", colFillGradientTo="0, 0, 0, 0", realCornerRounding="9",
       optRoundBottomLeft="false", optRoundBottomRight="false")
    st("WellShadowLeftStyle", "WellShadowTopStyle", enumGradientDirection="Horizontal", optRoundBottomLeft="true",
       optRoundTopRight="false")
    st("StatusStripStyle", "BasePanelStyle", colFill="Strip", colBorder="StripFrame", optDrawBorder="true", realCornerRounding="3")
    st("LogoStyle", "BasePanelStyle", enumDisabledDrawMode="NormalDrawing", colFill="Amber", colAltFill="AmberSoft")
    st("GlyphLabelStyle", "BaseLabelStyle", colText="InkMid", font=icon_font(12), enumTextHorizontalAlign="center")
    st("ChassisGlyphLabelStyle", "GlyphLabelStyle", colText="HwInk", font=icon_font(10))

    # LCD panels.
    st("LcdPanelStyle", "BasePanelStyle", colFill="Lcd", colBorder="InkMid", optDrawBorder="true", realCornerRounding="4")
    st("LcdHeaderStyle", "BasePanelStyle", colFill="LcdDim", colBorder="InkMid", optDrawBorder="true", realCornerRounding="3")
    st("LcdSlotStyle", "BasePanelStyle", colFill="LcdMid", colBorder="InkSoft", optDrawBorder="true", realCornerRounding="3")
    st("LcdRowEvenStyle", "BasePanelStyle", colFill="StripEven", colBorder="InkSoft", optDrawBorder="true", realCornerRounding="2")
    st("LcdRowOddStyle", "LcdRowEvenStyle", colFill="StripOdd")
    st("LcdRowSelectedStyle", "LcdRowEvenStyle", colFill="InkMid", colBorder="Ink", realCornerRounding="3")
    # (v1's layer rows: light / lighter grey in a 1 px grey edge, round 3;
    # the pad item: a 2 px navy frame, grey, navy when selected)
    st("LcdLayerRowEvenStyle", "BasePanelStyle", colFill="LcdDim", colBorder="RowEdge", optDrawBorder="true", realCornerRounding="3")
    st("LcdLayerRowOddStyle", "LcdLayerRowEvenStyle", colFill="LcdMid")
    st("LcdPadRowStyle", "LcdLayerRowEvenStyle", colFill="InkSoft", colBorder="Ink", realBorderWidth="2")
    st("LcdPadRowSelectedStyle", "LcdPadRowStyle", colFill="InkMid")
    st("LcdLayerGlueStyle", "BasePanelStyle", colFill="InkMid")
    # (v1's edit panels: a navy bezel, 2 px dark edge, round a screen with a
    # 2 px inner shadow; grey-headed group boxes; navy divider bands)
    st("LcdBezelStyle", "BasePanelStyle", colFill="InkMid", colBorder="Ink", optDrawBorder="true", realBorderWidth="2", realCornerRounding="4")
    st("LcdInsetStyle", "BasePanelStyle", colFill="Lcd", colBorder="InsetEdge", optDrawBorder="true", realBorderWidth="2")
    st("LcdGroupStyle", "BasePanelStyle", colFill="Lcd", colBorder="GroupEdge", optDrawBorder="true", realCornerRounding="4")
    st("LcdGroupHeadStyle", "LcdGroupStyle", colFill="InkSoft", optRoundBottomLeft="false", optRoundBottomRight="false")
    st("LcdDividerStyle", "BasePanelStyle", colFill="InkMid", colBorder="DividerEdge", optDrawBorder="true")
    st("LcdStripSelectedStyle", "LcdRowEvenStyle", colFill="StripSelected", colBorder="StripSelectedEdge", realBorderWidth="2")
    st("LcdGlueStyle", "BasePanelStyle", colFill="StripHead")
    # (the selected mixer strip's name: light on its dark fill)
    st("StripNameSelectedStyle", "StandardLabelCenteredStyle", colText="LcdMid")
    st("LcdStripHeadStyle", "BasePanelStyle", colFill="StripHead", colBorder="StripHeadEdge", optDrawBorder="true", realBorderWidth="2")
    # (the velocity ruler over the split lanes: a tick at every velocity,
    # labels every 32 on light boxes, italic; drawn by the editor,
    # `VelocityRuler`)
    st("LcdRulerStyle", "BasePanelStyle", colFill="Transparent", realCornerRounding="1", colText="Ink", colTick="Ink", colAltFill="LcdLight",
       font=text_font(7, True), realTextSlant="12")
    st("LcdFieldStyle", "BasePanelStyle", colFill="LcdLight", colBorder="Ink", optDrawBorder="true", realCornerRounding="2")
    st("LcdPlotStyle", "BasePanelStyle", colFill="LcdLight", colBorder="InkMid", optDrawBorder="true", realCornerRounding="2")
    # Effect panels (v1): a card with a 2 px edge and a darker band behind
    # the slot's menus; an empty slot is dark all over.
    st("FxCardStyle", "BasePanelStyle", colFill="Lcd", colBorder="FxCardEdge", optDrawBorder="true", realBorderWidth="2")
    st("FxCardEmptyStyle", "FxCardStyle", colFill="LcdDim")
    st("FxCardHeadStyle", "BasePanelStyle", colFill="LcdDim")

    # LCD buttons.
    st("LcdButtonStyle", "BaseButtonStyle", colFill="LcdLight", colBorder="LcdEdge", colAltFill="LcdOn", colAltBorder="Ink",
       realGradientShade="0.03")
    st("LcdIconButtonStyle", "LcdButtonStyle", font=icon_font(10))
    st("LcdRadioStyle", "LcdButtonStyle", colBorder="InkMid", colFill="LcdLight", realGradientShade="0", colAltFill="RadioOn",
       colAltText="Ink", colAltBorder="InkMid")
    # (mute / solo: a near-white face in a grey frame, shaded at the bottom;
    # heavy dark-grey letters)
    st("LcdMuteButtonStyle", "LcdButtonStyle", colAltFill="MuteOn", colAltText="Ink", font=text_font(10, True), realCornerRounding="3",
       colFill="LcdKeyFace", realGradientShade="0.04", colBorder="LcdKeyEdge", colText="LcdKeyText", realTextEmbolden="0.5",
       realBorderWidth="2",
       optBevel="true", colBevelLight="255, 255, 255, 160", colBevelDark="40, 48, 64, 110")
    st("LcdSoloButtonStyle", "LcdMuteButtonStyle", colAltFill="SoloOn")
    # v1's menu buttons carry a small arrow near the right edge, no divider,
    # so the text keeps most of the width ("Insert", "Pattern" fit in 50 px).
    st("LcdMenuButtonStyle", "LcdButtonStyle", enumTextHorizontalAlign="left", realTextOffset="5",
       realPopupArrowBoxWidth="10", optPopupDivider="false")
    st("LcdBigButtonStyle", "LcdButtonStyle", font=text_font(11, True), realCornerRounding="3")
    # (v1's mixer view tabs: a grey face with ink letters, ink with grey
    # letters when on, merging into the ink spine on their right)
    st("LcdSideTabStyle", "LcdBigButtonStyle", colFill="LcdDim", colText="Ink", colAltFill="Ink", colAltText="LcdDim",
       optDrawBorder="false", realGradientShade="0", font=text_font(10, True), realTextSlant="14", realTextScaleX="0.9", optRoundTopRight="false",
       optRoundBottomRight="false")
    st("LcdSideTabSpineStyle", "BasePanelStyle", colFill="Ink")
    # (v1's round buttons: Phase, Reverse, Find Src / Track 24 x 24, a light
    # disc in a grey ring with a grey icon, navy with a light icon on)
    st("LcdRoundButtonStyle", "LcdButtonStyle", realCornerRounding="100", colFill="Lcd", colBorder="RowEdge", colText="RowEdge",
       realGradientShade="0", colAltFill="Ink", colAltBorder="Ink", colAltText="LcdLight")
    st("LcdPowerButtonStyle", "LcdIconButtonStyle", font=icon_font(14), colAltFill="Amber", colAltText="Ink", realCornerRounding="3")
    # (v1's latching toggles: a disc, ink when on, grey with an ink edge
    # when off, an upright slot through it)
    st("LcdRoundToggleStyle", "LcdButtonStyle", colFill="InkSoft", colBorder="Ink", colAltFill="Ink", colAltBorder="Ink",
       realGradientShade="0", enumGlyph="Bar", colGlyph="LcdLight", colGlyphBorder="Ink")
    # (v1's preset arrows: grey triangles with an ink edge, ink pressed)
    st("LcdTriangleButtonStyle", "LcdButtonStyle", colFill="Transparent", colBorder="Transparent", colAltFill="Transparent",
       colAltBorder="Transparent", optDrawBorder="false", colGlyph="InkSoft", colAltGlyph="Ink", colGlyphBorder="Ink")
    st("LcdFlatButtonStyle", "LcdButtonStyle", colFill="Transparent", colBorder="Transparent", colText="InkMid",
       colAltFill="Transparent", colAltBorder="Transparent", colAltText="Ink")
    st("LcdFlatIconButtonStyle", "LcdFlatButtonStyle", font=icon_font(10))

    # Bezel tabs.
    st("TabButtonStyle", "BaseButtonStyle", colFill="Transparent", colBorder="Transparent", colText="BezelText",
       colAltFill="Transparent", colAltBorder="Transparent", colAltText="BezelTextOn", font=text_font(12, True), realTextScaleX="0.74",
       colAltTextGlow="BezelTextGlow", realTextGlowRadius="2", realTextGlowStrength="2.4", realTextEmbolden="0.7", realLetterSpacing="0.6",
       realCornerRounding="3", optRoundBottomLeft="false", optRoundBottomRight="false")
    st("BezelIconButtonStyle", "TabButtonStyle", font=icon_font(14), realTextOffset="-3", colText="ChassisText", colAltText="ChassisText", realTextScaleX="1",
       realTextEmbolden="0", realLetterSpacing="0")

    # Hardware buttons.
    st("HwButtonStyle", "BaseButtonStyle", colFill="HwFace", colFillGradientTo="HwFaceLow", colBorder="HwFrame", colText="HwInk",
       colAltFill="HwFaceOn", colAltFillGradientTo="HwFaceOnLow", colAltBorder="HwFrameDark", colAltText="HwInkOn",
       realCornerRounding="2", font=text_font(9, True), optBevel="true", colBevelLight="255, 255, 255, 170", colBevelDark="0, 0, 0, 45")
    # Transport buttons: pressed / on is darker, not yellow.
    st("HwIconButtonStyle", "HwButtonStyle", font=icon_font(18), colAltFill="HwFacePressed", colAltFillGradientTo="HwFacePressedLow")
    # (grey group: darker towards the bottom, a heavy frame, no top light)
    st("HwGreyIconButtonStyle", "HwIconButtonStyle", colFill="HwGrey", colFillGradientTo="HwGreyFaceLow", colText="HwGreyIcon",
       colAltText="HwGreyIcon", colBorder="HwFrameDark", realBorderWidth="2", colBevelLight="255, 255, 255, 0", colBevelDark="0, 0, 0, 90")
    st("MidiDiscStyle", "BasePanelStyle", colFill="HwGreyIcon", realCornerRounding="100")
    st("MidiPinStyle", "BasePanelStyle", colFill="MidiPin", realCornerRounding="1.5")
    st("HwSmallIconButtonStyle", "HwButtonStyle", font=icon_font(10))
    st("HwPlayButtonStyle", "HwIconButtonStyle", colText="HwGreen", colAltText="HwGreenOn", font=icon_font(18), colAltFill="HwFace",
       colAltFillGradientTo="HwFaceLow")
    st("HwRecordButtonStyle", "HwIconButtonStyle", colText="HwRed", colAltText="HwRedOn", font=icon_font(16))
    st("HwTinyButtonStyle", "HwButtonStyle", font=text_font(7, True), realCornerRounding="2")
    st("HwDarkTinyButtonStyle", "HwTinyButtonStyle", colFill="HwDarkFrame", colFillGradientTo="HwDarkFrame", colBorder="HwDarkFrame",
       colText="HwDarkText", colAltBorder="HwDarkFrame", realCornerRounding="0", optBevel="false")
    st("HwMuteButtonStyle", "HwDarkTinyButtonStyle", colAltFill="LedMute", colAltFillGradientTo="LedMute", colAltText="HwFace")
    st("HwSoloButtonStyle", "HwDarkTinyButtonStyle", colAltFill="LedSolo", colAltFillGradientTo="LedSolo", colAltText="HwInk")
    st("HwPowerButtonStyle", "HwDarkTinyButtonStyle", font=icon_font(8), colAltFill="HwBlueOn", colAltFillGradientTo="HwBlueOn", colAltText="HwBlueText")
    st("HwDarkButtonStyle", "HwButtonStyle", colFill="HwDark", colFillGradientTo="HwDarkLow", colBorder="HwDarkFrame",
       colText="HwDarkTextLight", realDisabledAlpha="0.45", colAltFill="HwDarkLow", colAltFillGradientTo="HwDarkFrame",
       colAltBorder="HwDarkFrame", colAltText="HwDarkTextLight", realCornerRounding="3", colBevelLight="255, 255, 255, 60")
    st("HwDarkIconButtonStyle", "HwDarkButtonStyle", font=icon_font(8))
    st("HwSongButtonStyle", "HwButtonStyle", colText="SongText", colAltText="SongText", colAltFill="HwFacePressed",
       colAltFillGradientTo="HwFacePressedLow", font=text_font(9, True))
    st("IndicatorStyle", "BaseButtonStyle", colFill="LedOff", colBorder="Strip", colText="HwInk", colAltFill="Readout",
       colAltBorder="Strip", colAltText="Strip", realCornerRounding="1", font=text_font(7, True))
    st("IndicatorLabelStyle", "IndicatorStyle", colFill="Transparent", colBorder="Transparent", colText="ReadoutMid",
       colAltFill="Transparent", colAltBorder="Transparent", colAltText="Readout", enumTextHorizontalAlign="left")
    # (v1's off indicators are blank)
    st("IndicatorMuteStyle", "IndicatorStyle", colFill="Transparent", colAltFill="LedMute", optDrawBorder="false")
    st("IndicatorSoloStyle", "IndicatorStyle", colFill="Transparent", colAltFill="LedSolo", optDrawBorder="false")
    st("IndicatorErrorStyle", "IndicatorStyle", colFill="LedMute")

    # Engine selectors, pattern keys, pads.
    st("EngineButtonStyle", "BaseButtonStyle", colFill="EngineFace", realGradientShade="-0.15", colBorder="EngineRim", colText="EngineDigit",
       colAltFill="EngineFace", colAltBorder="Amber", colAltText="ChassisText", realBorderWidth="2",
       realCornerRounding="13", font=text_font(14, True))
    st("EngineButtonPowerOffStyle", "EngineButtonStyle", colFill="EngineFaceOff", colAltFill="EngineFaceOff",
       colText="EngineDigitOff", colAltText="EngineDigitOff")
    st("PatternKeyStyle", "BaseButtonStyle", colFill="KeyFace", colFillGradientTo="KeyFaceLow", colBorder="KeyFrame", colText="HwInk",
       colAltFill="KeyFace", colAltFillGradientTo="KeyFaceLow", colAltBorder="HwFrameDark", realCornerRounding="3", font=text_font(8),
       realBorderWidth="2", optBevel="true", colBevelLight="255, 255, 255, 220", colBevelDark="0, 0, 0, 0",
       realStripHeight="6", realStripInset="3", realStripSideInset="4", realStripRounding="2", colStripOff="KeyLed", colStripOn="KeyLed")
    st("PatternKeyAllocatedStyle", "PatternKeyStyle", colStripOff="KeyLedOn", colStripOn="KeyLedOn")
    st("PatternKeyEditingStyle", "PatternKeyStyle", colBorder="AmberSoft", colAltBorder="AmberSoft")
    st("PatternKeySelectedStyle", "PatternKeyStyle", colFill="KeyFaceSelected", colFillGradientTo="KeyFaceSelected", colBorder="KeyAmber",
       colAltBorder="KeyAmber", colStripOff="KeyLedSelected", colStripOn="KeyLedSelected")
    st("PadModeIconStyle", "BasePanelStyle", colFill="HwInk", realCornerRounding="0")
    st("PadModeFrameStyle", "BasePanelStyle", colBorder="HwInk", optDrawBorder="true", realCornerRounding="0")
    st("ChassisGlyphBarStyle", "BasePanelStyle", colFill="HwInk")
    st("ChassisGlyphDotStyle", "ChassisGlyphBarStyle", realCornerRounding="100")
    # (the pattern tools' and view modes' icon marks, and Select's dashed
    # square)
    # (light on a selected button, like its text)
    st("ToolMarkStyle", "BasePanelStyle", colFill="InkMid", colParentOnFill="LcdLight")
    st("ToolDashStyle", "BasePanelStyle", colBorder="InkMid", optDrawBorder="true", intDashLength="2", intDashGap="1",
       colParentOnFill="Transparent", colParentOnBorder="LcdLight")
    # (the panic button's warning sign)
    st("PanicFillStyle", "BasePanelStyle", colFill="PanicFill")
    st("PanicEdgeStyle", "BasePanelStyle", colFill="HwInk")
    st("PadSlotStyle", "BasePanelStyle")
    st("PadStyle", "BasePanelStyle", colFill="PadFace", colFillGradientTo="PadFaceLow", colBorder="PadFrame", optDrawBorder="true",
       realCornerRounding="4", realBorderWidth="2", colText="PadText", optBevel="true", colBevelLight="255, 255, 255, 45", colBevelDark="0, 0, 0, 70")
    st("PadSelectedStyle", "PadStyle", colBorder="Amber", realBorderWidth="2")
    st("PadNameStyle", "BaseLabelStyle", colText="PadText", font=text_font(10), enumTextHorizontalAlign="center", colTextGlow="PadTextGlow",
       realTextGlowRadius="1.5", realTextGlowStrength="1")
    st("PadLedStyle", "BasePanelStyle", colFill="PadFrame", realCornerRounding="1.5")  # round dots (v1, round 5)
    for i in range(1, 5):  # a pad's colour (v1's PadColour1-4) on its first LED
        st(f"PadLed{i}Style", "PadLedStyle", colFill=f"PadColour{i}")

    # Knobs, faders, meters.
    # The large knob: a light halo, the dark ring (lit grey at the top) and a
    # face lit from the top left are panels behind it (looks.KNOB_BODY);
    # the knob draws the face and pointer.
    st("HardKnobStyle", "BaseKnobStyle", colFill="Transparent",
       colFace="KnobFace", colFaceGradientTo="KnobFaceLow", enumGradientDirection="Radial", realGradientCentreX="0.42",
       realGradientCentreY="0.18", realGradientRadius="0.95",
       colPointer="KnobPointer", realPointerStart="0.3", realPointerInset="3.3", optPointerDot="false", colAltFill="KnobDot",
       colAltBorder="KnobDot", colFillHighlight="Readout", realFaceInset="2")
    st("HardKnobHaloStyle", "BasePanelStyle", colFill="KnobHalo", colFillGradientTo="KnobHaloLow", enumGradientDirection="Radial",
       realGradientRadius="0.5", realCornerRounding="100")
    st("HardKnobMarkStyle", "BasePanelStyle", colFill="KnobMark", realCornerRounding="100")
    st("HardKnobRingStyle", "BasePanelStyle", colFill="KnobRingTop", colFillGradientTo="KnobRingLow", realCornerRounding="100")
    st("HardKnobShadowStyle", "BasePanelStyle", colFill="KnobShadow", colFillGradientTo="KnobShadowOut", enumGradientDirection="Radial",
       realGradientCentreY="0.45", realGradientRadius="0.5", realCornerRounding="100")
    st("HardKnobBodyStyle", "BasePanelStyle", colFill="KnobFace", colBorder="HwFrame", optDrawBorder="true", realCornerRounding="100")
    st("HardKnobSmallStyle", "BaseKnobStyle", colFill="SmallKnobRim", colFillHighlight="SmallKnobRim", colAltFill="KnobRing",
       colAltBorder="HwFace", colFace="SmallKnobFace", colFaceGradientTo="SmallKnobFaceLow", realFaceInset="0", colPointer="HwFace",
       realPointerWidth="1.5", realPointerInset="1")
    # v1: a uniform dark ring (the value shows by the pointer), light face.
    st("LcdKnobStyle", "BaseKnobStyle", colFill="InkMid", colFillHighlight="InkMid")
    st("LcdKnobLargeStyle", "BaseKnobStyle", colFill="KnobLcdTrack", colFillHighlight="KnobLcdValue", colFace="LcdLight",
       colFaceGradientTo="LcdLight", realFaceInset="3", colFaceBorder="InkSoft", realFaceBorderWidth="3", colPointer="Ink",
       realPointerWidth="2.5", realPointerStart="0.1", realPointerInset="6")
    st("LcdKnobSmallStyle", "BaseKnobStyle", colFill="Transparent", colFillHighlight="Transparent",
       colFace="LcdLight", colFaceGradientTo="LcdLight", realFaceInset="-1", colFaceBorder="InkSoft", realFaceBorderWidth="3",
       colPointer="Ink", realPointerWidth="2", realPointerInset="3")
    st("LcdKnobMicroStyle", "LcdKnobSmallStyle", realFaceBorderWidth="2", realPointerInset="2")
    st("SpinnerFaceStyle", "BaseKnobStyle")
    st("LcdKnobBodyStyle", "BasePanelStyle", colFill="LcdLight", colBorder="InkSoft", optDrawBorder="true", realCornerRounding="100")
    st("SpinnerStyle", "BaseKnobStyle", colFill="Transparent", colFillHighlight="InkSoft", optPointer="false", colFace="Transparent")
    st("LcdFaderStyle", "BaseSliderStyle", enumHandleShape="Rect", colFill="LcdFace", colFillHighlight="Transparent",
       colTrackBorder="LcdEdge", colGroove="LcdGroove", colAltFill="LcdCap", colAltFillGradientTo="LcdCapLow", colAltBorder="LcdCapLow",
       colGrip="LcdGrip", intGripLines="2", enumGripShape="Squares", realGripSize="2", realCornerRounding="3", realHandleInset="2",
       enumGradientDirection="Radial", realGradientRadius="0.62")
    st("LcdPanStyle", "LcdFaderStyle", realHandleLength="9", realHandleInset="1", colBorder="Transparent", intGripLines="1",
       enumGripShape="Along", realHandleRounding="3")
    # (v1's delay cut split fader: a grey pill with an ink edge and a dark
    # line through it, the range light, ▼ / ▲ markers for the two cuts)
    st("LcdRangeStyle", "LcdFaderStyle", enumHandleShape="Triangles", realHandleLength="11", colFill="LcdDim", colFillHighlight="LcdLight",
       colTrackBorder="Ink", colGroove="InkMid", realGrooveWidth="1", realGrooveInset="5", colAltFill="LcdLight", colAltBorder="Ink",
       realCornerRounding="9")
    st("LcdScrollBarStyle", "BaseSliderStyle", colFill="LcdLight", colAltFill="InkMid")
    st("LcdMeterStyle", "BaseMeterStyle")
    # (v1: fine 1 px lines in the readout blue, faint when off)
    st("StripMeterStyle", "BaseMeterStyle", colFill="112, 175, 240, 51", colAltFill="Readout", colAltBorder="Readout")

    # Readouts on the status strips.
    st("ReadoutStyle", "BaseLabelStyle", colText="Readout", enumTextHorizontalAlign="center", font=text_font(10))
    st("ReadoutMenuStyle", "BaseButtonStyle", colFill="Transparent", colBorder="Transparent", colText="Readout", colAltFill="ReadoutDim",
       colAltBorder="Transparent", colAltText="Readout", font=text_font(9, True), enumTextHorizontalAlign="left")
    st("ReadoutButtonStyle", "BaseButtonStyle", colFill="Transparent", colBorder="ReadoutDim", colText="Readout",
       colAltFill="ReadoutDim", colAltBorder="Readout", colAltText="Readout", font=text_font(9, True))

    # Modal dialogs.
    st("ModalButtonStyle", "BaseButtonStyle", colFill="ModalGroup", colAltFill="ModalOn", colBorder="Ink", colAltBorder="Ink",
       colAltText="Ink", realCornerRounding="3")
    st("ModalFieldStyle", "BaseTextBoxStyle", colFill="ModalField", colBorder="Ink")
    st("ModalListStyle", "BaseTableStyle", colFill="ModalList", colBorder="Ink", colText="Ink", colRowAlt="ModalGroup", font=text_font(10))
    st("ModalProgressStyle", "BaseStyle", optDrawBorder="true", colFill="ModalField", colBorder="Ink", colAltFill="InkMid", colAltBorder="Ink")
    st("CheckBoxStyle", "BaseStyle", colFill="LcdLight", colBorder="Ink", colAltFill="LcdLight", colAltBorder="Ink",
       colTick="Ink", realCornerRounding="2", realTickInset="3", realTickLineWidth="2")

    # The LCD content v1 drew in code (step grid, song parts, browser tree
    # and lists), after v1's colour table (the survey's palette section);
    # the preview example (crates/einst/examples/v1skins.rs) builds it.
    st("TimelineStyle", "BasePanelStyle", colFill="LcdLight", colBorder="Ink", optDrawBorder="true")
    st("TimelineLabelStyle", "BaseLabelStyle", font=text_font(9, True))
    st("TimelineTickStyle", "BasePanelStyle", colFill="InkMid")
    # (colAltBorder: the grid's left edge, drawn by the editor)
    st("GridBackStyle", "BasePanelStyle", colFill="GridBack", colBorder="InkMid", colAltBorder="GridEdge", optDrawBorder="true")
    st("GridShadeStyle", "BasePanelStyle", colFill="GridShade")
    st("GridLineStyle", "BasePanelStyle", colFill="GridLine")
    st("GridDashStyle", "GridLineStyle", colFill="GridDash", intDashLength="2", intDashGap="2")
    st("GridBarLineStyle", "GridLineStyle", colFill="GridBar")
    st("TrackColumnStyle", "BasePanelStyle", colFill="TrackColumn", colBorder="TrackColumnEdge", optDrawBorder="true")
    st("GridRowSelectedStyle", "BasePanelStyle", colFill="RowSelected")
    st("GridPositionStyle", "BasePanelStyle", colFill="PlayPosition")
    st("StepOnStyle", "BasePanelStyle", colFill="StepOn", colBorder="StepEdge", optDrawBorder="true", realCornerRounding="0")
    st("SongLaneStyle", "BasePanelStyle", colFill="Lane")
    st("SongLaneAltStyle", "BasePanelStyle", colFill="LaneAlt")
    st("SongLaneSelectedStyle", "BasePanelStyle", colFill="LaneSelected")
    st("SongPartBodyStyle", "BaseLabelStyle", colFill="LcdLight", colBorder="LaneFoot", optDrawBorder="true", realCornerRounding="0",
       font=text_font(10, True), enumTextHorizontalAlign="center")
    st("SongPartSelectedBodyStyle", "SongPartBodyStyle", colFill="PartSelected", colAltFill="PartHatch")
    st("SongPartFootStyle", "BasePanelStyle", colFill="LaneFoot", colText="LaneFootMark")
    st("SongBarDotStyle", "BasePanelStyle", colFill="BarDot")
    # (bar lines: colFill every bar, colAltFill the labelled ones)
    st("SongBarLineStyle", "BasePanelStyle", colFill="BarLine", colAltFill="BarLineMajor")
    st("SongRulerStyle", "BasePanelStyle", colFill="Ruler", colBorder="RulerEdge")
    # The insert marker (a navy head and line on the ruler).
    st("SongMarkerStyle", "BasePanelStyle", colFill="Ink")
    # The scroll bar under the lanes: track, thumb, frame.
    st("SongScrollStyle", "BasePanelStyle", colFill="#D7DFE4", colAltFill="LaneFoot", colBorder="Ink")
    st("TrackNameStyle", "BaseLabelStyle", colFill="LcdLight", colBorder="InkSoft", optDrawBorder="true", font=text_font(10), realTextOffset="3")
    st("TrackArrowStyle", "LcdFlatButtonStyle", colText="InkMid", font=text_font(9))
    st("TrackNumberStyle", "BaseLabelStyle", font=text_font(24), enumTextHorizontalAlign="center")
    st("BrowserCrumbStyle", "BaseButtonStyle", colFill="CrumbFace", realGradientShade="0.1", colBorder="CrumbTop", colAltFill="Crumb",
       colAltBorder="CrumbLow", colSeparator="Ink",
       colAltText="Ink", realCornerRounding="0", font=text_font(10), enumShape="Arrow", realArrowDepth="6")
    st("BrowserTreeHeadStyle", "BaseLabelStyle", font=text_font(10), realTextOffset="7")
    st("BrowserTreeItemStyle", "BaseLabelStyle", font=text_font(10), realTextOffset="35")
    st("BrowserTreeSelectedStyle", "BrowserTreeItemStyle", colFill="BrowserTreeSel", colText="LcdLight")
    st("BrowserTreeIconStyle", "GlyphLabelStyle", colText="InkMid", font=icon_font(12))
    # Browser icons (drawn in code: book, home, folder, desktop, drive,
    # disc, preset page): outline, face, dark parts, detail / label text.
    st("BrowserIconStyle", "BasePanelStyle", colBorder="IconLine", colFill="IconFace", colAltFill="IconDark", colText="Ink",
       font=text_font(7, True))
    st("BrowserIconSelectedStyle", "BrowserIconStyle")
    st("BrowserSplitStyle", "BasePanelStyle", colFill="BrowserSplit", colAltBorder="BrowserSplit")
    # The header's navigation keys: light faces on the navy plate.
    st("BrowserNavKeyStyle", "LcdButtonStyle", colFill="NavKey", colBorder="Transparent", colAltFill="NavKeyDown",
       colAltBorder="Transparent", colText="Ink", colAltText="Ink", realGradientShade="0", realCornerRounding="1", font=icon_font(8))
    st("BrowserTreeBackStyle", "BasePanelStyle", colFill="BrowserTreeBack")
    # The bar between the folder's list and the shortlist (v1's
    # HorizontalDivider art): navy between white edges, a light grip.
    st("BrowserListSplitStyle", "BasePanelStyle", colFill="Ink", colBorder="#FFFFFF", optDrawBorder="true", colAltBorder="LcdDim")
    # The list's column header (v1's BrowserHeader* colours).
    st("BrowserListHeaderStyle", "BasePanelStyle", colFill="BrowserHeader", colBorder="BrowserHeaderDividerColour",
       colText="BrowserHeaderTextColour", font=text_font(10))
    st("BrowserParentStyle", "BaseButtonStyle", colFill="LcdLight", colBorder="InkSoft", realCornerRounding="0", font=text_font(10))
    st("BrowserListStyle", "BaseTableStyle", font=text_font(10), colFill="BrowserList", colBorder="InkSoft", colRow="BrowserRow", colSelectedText="Ink",
       colRowAlt="BrowserRowAlt", colSelection="BrowserSel", colHeaderFill="BrowserHeader", colHeaderDivider="InkMid",
       colColumnDivider="Ink", optAltRowColours="true", optColumnDividers="true", optRowDividers="true", colRowDivider="Ink",
       colRowHighlight="BrowserRowLight",
       intRowHeight="25", intHeaderHeight="17")
    st("BrowserBadgeStyle", "BaseButtonStyle", colFill="LcdLight", colBorder="InkMid", realCornerRounding="2", font=text_font(7, True))
    st("BrowserScrollStyle", "BasePanelStyle", colFill="Ink", colAltFill="BrowserScrollTrack", realCornerRounding="2")

    # Wave displays (layer page, sampler, slicer): background, the wave
    # (normal / outside the markers / inside the loop), markers, the
    # timeline strip and the envelopes drawn over the wave.
    st("WaveDisplayStyle", "BasePanelStyle", colFill="WaveBack", colAltFill="WaveBack", colCenterLine="Ink", colLine="Ink",
       colLineDisabled="WaveDisabled", colLineLooping="InkMid", colStartMarker="WaveMarker", colEndMarker="WaveMarker",
       colLoopStartMarker="WaveLoopMarker", colLoopEndMarker="WaveLoopMarker", colTimelineBackground="LcdDim",
       colTimelineText="Ink", colTimelineMarkers="Ink", colAmpEnvelopeBack="EnvAmpBack", colFreeEnvelopeBack="EnvFreeBack",
       colEnvelopeLine="EnvLine", colEnvelopeHandle="EnvHandle", colEnvelopeHandleBorder="EnvHandleEdge",
       colEnvelopeGrab="InkMid", colOutsideFill="WaveOutside", colMarkerStrip="InkSoft", colMarkerFill="WaveMarkerFill",
       colLoopMarkerFill="WaveLoopMarkerFill", colMarkerEdge="WaveMarkerEdge", colTitleFill="WaveTitle", colText="LcdLight",
       font=text_font(9, True), fontTimeline=text_font(8), fontNoAudio=text_font(20), colNoAudioText="Ink")
    # (track, thumb, its edge lines and grips, the grips' centres)
    st("WaveDisplayZoomSliderStyle", "BasePanelStyle", colTrack="LcdDim", colFill="InkSoft", colAltFill="LcdFace", colBorder="Ink")
    # Velocity split lanes: the split's area, the dragged handle, tick
    # lines (every eighth lighter), the handles.
    st("VSplitStyle", "BasePanelStyle", colFill="SplitGraph", colAltFill="AmberSoft", colBorder="SplitTick", colAltBorder="SplitTickBar",
       colHandle="Ink")
    # Scene slots' trigger area by state (v1's `Scene{Empty,Full,Cued,
    # Active}` art): the field, its edge and corner rounding; a cued scene
    # shows a row of dots (colText), the playing one a play mark (colAltFill
    # in colAltBorder).
    st("SceneTriggerEmptyStyle", "BasePanelStyle", colFill="LcdLight", colBorder="SceneEdge", realCornerRounding="2",
       colText="SceneMark", colAltFill="SceneEdge", colAltBorder="Ink")
    st("SceneTriggerUsedStyle", "SceneTriggerEmptyStyle", colFill="SceneUsed")
    st("SceneTriggerCuedStyle", "SceneTriggerUsedStyle")
    st("SceneTriggerActiveStyle", "SceneTriggerUsedStyle")
    # The scene page's art: the slate frame (colFill, outline colBorder),
    # the light panels, the slot and scroll wells and the table's well.
    st("ScenePageBackStyle", "BasePanelStyle", colFill="SceneFrame", colBorder="Ink", colPanel="Lcd", colPanelEdge="ScenePanelEdge",
       colPanelEdgeLow="ScenePanelEdgeLow", colWell="SceneWell", colTableWell="SceneTableWell", colWellEdge="SceneWellEdge",
       colWellEdgeLow="SceneWellEdgeLow")
    # A slot's tile in the well (`SceneBackground`: 93 x 67 at 6, 4).
    st("SceneTileStyle", "BasePanelStyle", colFill="LcdMid", colBorder="SceneEdge", realCornerRounding="3")
    # The effect chain's slot bar: one item per slot, the shown ones lit.
    # v1 (round 5): the shown slots light blue-grey, the others a darker
    # slate, both with a dark edge.
    st("FxSlotItemStyle", "BasePanelStyle", colFill="FxSlotDim", colBorder="Ink", colText="Ink", font=text_font(9))
    st("FxSlotItemLitStyle", "FxSlotItemStyle", colFill="FxSlotLitTop", colFillGradientTo="FxSlotLitLow")
    # A layer row in a multiple selection (the primary row stays amber).
    st("SelectionButtonMultiStyle", "LcdRowSelectedStyle", colFill="RowSelected", realCornerRounding="0")
    # Pattern grid: past a track's length, its length marker (amber while
    # dragged) and the selected steps.
    st("GridPastLengthStyle", "BasePanelStyle", colFill="PastLength")
    st("PatternTrackMarkerStyle", "BasePanelStyle", colFill="InkMid", colAltFill="Amber")
    st("StepSelectionStyle", "BasePanelStyle", colFill="AmberSoft")
    # Song page: parts being recorded and retro parts, the loop range.
    st("SongPartRecordingStyle", "BasePanelStyle", colFill="RecordingPart", colBorder="Ink")
    st("SongPartRetroStyle", "BasePanelStyle", colFill="RetroPart", colBorder="RetroEdge")
    # (v1's `LoopHorizontal` / `LoopGrab`: a navy line on the ruler between
    # two upright pairs)
    st("SongLoopStyle", "BasePanelStyle", colFill="Ink")

    # Tooltips and menus (read by dreist-gui by these names).
    st("TooltipStyle", "BaseStyle", optDrawBorder="true", enumDisabledDrawMode="NormalDrawing", colFill="ToolTipPanelColour",
       colBorder="ToolTipBorderColour", colText="ToolTipTextColour")
    st("MenuStyle", "BaseStyle", optDrawBorder="true", optDrawShadow="true", colFill="LcdLight", colBorder="Ink", colAltFill="Amber",
       colDivider="InkSoft", colSubMenuArrow="Ink", colText="Ink", realCornerRounding="3")
    return S


# Attributes set on converted v1 styles (v1 left them to code or to a
# bitmap): style -> attrs.
STYLE_OVERRIDES = {
    # The table header band (v1's header bitmap).
    "GlobalTableStyle": {"colHeaderFill": "LcdDim"},
    "GlobalTableStyleAlwaysVisibleHSlider": {"colHeaderFill": "LcdDim"},
    # The browser's search box (v1: optNoBackground, on the SearchBox art;
    # the options button left of it carries the magnifier).
    # The list's view options button: a bare navy ▼ (the reference
    # screenshots).
    "BrowserHeaderViewOptionsButtonStyle": {"colText": "Ink", "colAltText": "Ink", "font": text_font(9)},
    "SearchBoxStyle": {"colFill": "Transparent", "colBorder": "Transparent", "realGradientShade": "0", "optSearchIcon": "false"},
    # The slicer's screen (v1's background bitmap).
    "PreviewPanelBackgroundStyle": {"colFill": "Lcd"},
    # The scene mini-map's cells: square and flat (ScenePanel.skin).
    "MiniSceneEvenButtonStyle": {"realCornerRounding": "0", "realGradientShade": "0"},
    "MiniSceneOddButtonStyle": {"realCornerRounding": "0", "realGradientShade": "0"},
    # A scene's name shortens in the middle (v1's shortening label).
    "SceneNoteLabelCenteredStyle": {"enumTextShortenAlgorithm": "middle"},
}


# Style classes of the theme styles that dreist-gui dispatches on by class.
STYLE_CLASS = {"CheckBoxStyle": "CheckBoxButtonRendererStyle", "MenuStyle": "CtrlOptionMenuStyle", "BrowserListStyle": "CtrlTableViewStyle"}
