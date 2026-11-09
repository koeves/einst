"""v1 skin XML -> dreist-gui skins and stylesheets.

Reads v1's shipped skins (assets/v1/Resources/**/*.skin, read-only) and
writes the same layout in the syntax dreist-gui reads
(commons/crates/dreist-gui/src: skin.rs, ctrl.rs, style.rs, render.rs):

- rects `[x:0 y:0 w:10 h:10]` -> `0, 0, 10, 10`;
- control classes the engine does not draw -> the nearest one it does
  (CLASSES; the v1 class is kept in `legacyClass` so dreist-gui can take
  them over as it learns them); level strips, pan sliders, needle meters,
  plots, tables and list boxes keep their class (dreist-gui draws them;
  meters and tables with a style of their v1 style class, NATIVE_STYLES);
- style attributes -> dreist-gui's names (STYLE_ATTRS); attributes that
  dreist-gui reads from the control rather than the style
  (`optIsMomentary`, ...) are copied onto every control using the style;
- nested `<VFont vSize= fontName=>` -> a named font of the theme's type
  scale (Liberation Sans standing in for Arial and Verdana), at
  theme.TEXT_SCALE of v1's size;
- every bitmap reference -> a vector look (looks.py);
- identifiers carrying the legacy product name -> neutral names (RENAMES).

Styles from every v1 skin (including the shared style skin) are hoisted
into the theme's Styles.stylesheet: v1 looks styles up globally, while
dreist-gui only registers a skin's inline styles when that skin loads.
Control names, ids, display names and texts are kept: the editor binds
to them.
"""

import os
import re
import xml.etree.ElementTree as ET

import looks
import theme

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
V1 = os.path.join(ROOT, "assets", "v1", "Resources")
LEGACY_RS = os.path.join(ROOT, "commons", "crates", "dreist-formats", "src", "legacy.rs")

# ---------------------------------------------------------------- mapping tables

# v1 control class -> (dreist-gui class, what changes). Classes not listed
# are kept as they are.
CLASSES = {
    "CtrlPanel": ("CtrlPanel", "as is"),
    "CtrlButton": ("CtrlButton", "as is (image states -> fill / border / text colours)"),
    "CtrlButtonGroup": ("CtrlButtonGroup", "as is (radio behaviour)"),
    "CtrlLabel": ("CtrlLabel", "as is"),
    "CtrlKnob": ("CtrlKnob", "as is; a square dial at the top / centre of v1's rect (enumDialAlign; film frames held a scale too)"),
    "CtrlSlider": ("CtrlSlider", "as is (rectangular fader caps: enumHandleShape)"),
    "CtrlTextBox": ("CtrlTextBox", "as is (drawn as a label, edited in place)"),
    "CtrlValueBox": ("CtrlValueBox", "as is (drawn as a label)"),
    "CtrlComboBox": ("CtrlPopupButton", "a popup button: text, arrow and divider"),
    "CtrlSearchTextBox": ("CtrlSearchBox", "search box with magnifier"),
    "VProgressBar": ("CtrlProgressBar", "horizontal bar"),
    "CtrlLevelStrip": ("CtrlLevelStrip", "as is (LED segments: intSegmentSize)"),
    "CtrlPanSlider": ("CtrlPanSlider", "as is (lit from the centre, centre detent)"),
    "CtrlImageView": ("CtrlPanel", "vector panel, or a CtrlLabel for text / icon images (looks.IMAGES)"),
    "CtrlScrollView": ("CtrlPanel", "plain panel; no scrolling"),
    "CtrlScrollViewSlider": ("CtrlScrollBar", "scroll bar (track + thumb)"),
    "CtrlTableView": ("CtrlTableView", "as is: header and rows (code-filled); no scrolling"),
    "CtrlListBox": ("CtrlListBox", "as is: rows (code-filled); no scrolling"),
    "CtrlPlot": ("CtrlPlot", "as is: frame + curve (`points`, code-filled)"),
    "CtrlMeter": ("CtrlMeter", "as is: needle meter (scale arc, range texts, needle)"),
    "Ctrl": ("CtrlPanel", "plain panel"),
}

# Code-drawn panels dreist-gui now draws natively, by overload class:
# (class, style). The overload stays (the editor binds to it).
PANEL_OVERLOADS = {
    "SplitFader": ("CtrlRangeSlider", "LcdRangeStyle"),  # the delay's high / low cut
}

# Plain `Ctrl`s by overload class: None drops the control.
CTRL_OVERLOADS = {
    "OverlayImage": None,  # the glass/shine bitmap over the window
    "TapTempoBox": ("CtrlButton", "ReadoutMenuStyle", ""),  # v1 drew no art there: a click area left of the tempo
}

# Controls v1 left unstyled (code-drawn) that need a vector style to be
# seen: (class, overloadClass) -> style.
UNSTYLED = {
    ("CtrlValueBox", "TempoBox"): "ReadoutStyle",
    ("VProgressBar", ""): "ModalProgressStyle",
    ("CtrlScrollView", ""): "BasePanelStyle",
    ("CtrlButtonGroup", ""): "BasePanelStyle",
    ("CtrlPanel", "EditorPad"): "PadSlotStyle",
}

# Rects v1's layout got wrong, by (skin, control name) -> x, y, w, h.
RECT_FIXES = {
    # "-> Pad" (x 719) and "-> Pads" (x 749) overlapped by 20 px: both 50
    # wide as the row's other buttons, paired 2 px apart right after
    # Slice like the row's other pairs, ending at the row's right edge.
    ("Sampler/SamplerPanel.skin", "SendToPadButton"): [711, 19, 50, 17],
    ("Sampler/SamplerPanel.skin", "SendAllToPadsButton"): [762, 19, 50, 17],
    # The song track numbers drew 3.5 px left of v1's (v1-song.png: digit
    # centres at x 339.5 in the 48 px header; Liberation Sans' digits sit
    # further left in the advance than Arial's).
    ("Song/SongTrackHeaderPanel.skin", "TrackNumberLabel"): [17, 8, 20, 24],
}

# Controls added to a skin's top panel: rel -> controls. The slicer's
# zoom scrollbar (the manual's "Zoom in/out and scrollbar" on the
# Slicer page): under the wave, left of the zoom buttons.
EXTRA = {
    "Preview/PreviewPanel.skin": lambda: [
        E("CtrlPanel", {"name": "PreviewZoomSlider", "rect": "8, 327, 772, 12", "overloadClass": "WaveDisplayZoomSlider",
                        "displayName": "Drag to scroll, drag an edge to zoom", "style": "WaveDisplayZoomSliderStyle"}),
    ],
    # The song lanes' horizontal scroll bar (v1-song.png: under the lanes,
    # left of the zoom buttons; v1's scroll view drew it), drawn by the
    # editor (`SongHScroll`).
    "Song/SongPanel.skin": lambda: [
        E("CtrlScrollBar", {"name": "HorizontalScrollBar", "rect": "52, 380, 734, 11", "overloadClass": "SongHScroll",
                            "style": "SongScrollStyle"}),
    ],
}

# Control texts by name: the debug frame-rate label stays empty until code
# fills it.
CTRL_TEXT = {"FPSLabel": ""}


def upper_caption(t):
    """v1's image captions: upper case, "Pad / Layers" spaced, "Load..."
    menus with a drop mark."""
    return t.upper().replace("/", " / ").replace("...", "▼")


# Other unstyled controls (dreist-gui would give them its white default).
UNSTYLED_CLASS = {"CtrlLabel": "BaseLabelStyle", "CtrlButton": "LcdButtonStyle", "CtrlImageView": "BaseImageStyle"}

# Code-drawn panels the editor paints over, with the shape v1's bitmaps
# gave them (looks.py decor entries).
PANEL_DECOR = {
    # 52 px wide pad frames in a 56 px slot, with the row of eight status
    # LEDs along the bottom (the first one shows the pad's colour).
    # (measured on v1's screens: 50 x 43 from the slot's left edge)
    "EditorPad": [("inside", 0, 0, 50, 42, "PadStyle")] + [("inside", 7 + 5 * i, 34, 3, 3, "PadLedStyle") for i in range(8)],
}

# Decor by (skin, control name), as the looks' decor. The engine mixer's
# MIX / SEND / SETUP tabs share one dark spine down their right edge (v1's
# tab bitmaps: 34 px faces, then 10 px of ink the full height; behind the
# group, as the group's children are its buttons).
NAMED_DECOR = {
    ("MIxer/EngineMixerPanel.skin", "ModeGroup"): [("behind", 34, 0, 12, 0, "LcdSideTabSpineStyle")],
}

# Labels grow to fit their font (v1 drew label text unclipped; dreist-gui
# clips to the rect): this many pixels above the font size, and centred
# labels this much wider on each side.
LABEL_LEAD = 3
LABEL_WIDEN = 8
# Labels that keep v1's rect: a scene's name fits its trigger and shortens
# in the middle (v1's shortening label).
LABEL_KEEP = {("Scene/Scene.skin", "NameLabel")}
# Right-aligned labels (captions left of their control) reach this much
# further left.
LABEL_REACH = 60

# v1 style attribute -> (dreist-gui attribute or None, how). `ctrl:` means
# the value goes onto the controls using the style.
STYLE_ATTRS = {
    "intRoundedRectRadius": ("realCornerRounding", "same value"),
    "colFill": ("colFill", "colour"),
    "colBorder": ("colBorder", "colour; a visible border sets optDrawBorder"),
    "colText": ("colText", "colour"),
    "colTextOff": ("colText", "colour (button off)"),
    "colTextOn": ("colAltText", "colour (button on)"),
    "colFillOff": ("colFill", "colour (button off)"),
    "colFillOn": ("colAltFill", "colour (button on)"),
    "colBackground": ("colFill", "colour (text boxes)"),
    "colSlider": ("colFillHighlight", "colour (fader / scroll thumb; also colAltFill)"),
    "centeredText": ("enumTextHorizontalAlign", "true -> center (v1 text is left-aligned by default)"),
    "rightAlignText": ("enumTextHorizontalAlign", "true -> right"),
    "optTextLeftJustify": ("enumTextHorizontalAlign", "true -> left"),
    "textOffset": ("realTextOffset", "same value"),
    "noDefaultDrawing": ("colFill", "true -> transparent, no border"),
    "optMomentary": ("ctrl:optIsMomentary", "v1 default true"),
    "optSendOnMouseDown": ("ctrl:optSendOnMouseDown", "same value"),
    "optSingleClickEdit": ("ctrl:optSingleClickEdit", "same value"),
    "optDrawText": ("optDrawText", "false kept only on styles without a look (a look's text is the label)"),
    "colTextDisabled": ("colTextDisabled", "colour (text of disabled controls)"),
    "optFlipDirection": ("ctrl:optFlipDirection", "per control by orientation (see Converter.ctrl)"),
    "faderHeadSize": ("realHandleLength", "same value (rectangular caps)"),
    # Tables and list boxes.
    "optHeader": ("optHeader", "same value"),
    "optAltRowColours": ("optAltRowColours", "same value"),
    "optColumnDividers": ("optColumnDividers", "same value"),
    "optRowDividers": ("optRowDividers", "same value"),
    "colSelect": ("colSelection", "colour"),
    "colRow": ("colRow", "colour"),
    "colRowAlt": ("colRowAlt", "colour"),
    "colHeaderBackground": ("colHeaderFill", "colour"),
    "colHeaderText": ("colHeaderText", "colour"),
    "colHeaderDivider": ("colHeaderDivider", "colour"),
    "colColumnDivider": ("colColumnDivider", "colour"),
    "colColumnText": ("colText", "colour (cell text)"),
    # Needle meters and plots.
    "units": ("textUnits", "same value"),
    "leftRangeText": ("textLeftRange", "same value (-oo -> -∞)"),
    "rightRangeText": ("textRightRange", "same value (-oo -> -∞)"),
    "colUnits": ("colText", "colour"),
    "pivotX": ("realPivotX", "same value"),
    "pivotY": ("realPivotY", "same value"),
    "angleStart": ("realAngleStart", "same value"),
    "angleSize": ("realAngleSize", "same value"),
    "radius": ("realRadius", "same value"),
    "lineWidth": ("realLineWidth", "same value"),
    "colLine": ("colLine", "colour"),
    "plotRectInset": ("realPlotInset", "same value"),
    # Value boxes a press drags (v1's style flag 0x80; dreist-gui's App).
    "dragable": ("dragable", "same value"),
}

# v1 style classes dreist-gui needs as the style's element class to draw the
# control natively (other converted styles are plain CtrlStyles).
NATIVE_STYLES = {"CtrlMeterStyle", "CtrlTableViewStyle", "CtrlListBoxStyle"}

# Attributes with no dreist-gui equivalent yet, dropped (kept here so a
# new v1 attribute is noticed: the generator fails on unknown ones).
DROPPED = {
    "name", "imUp", "imDown", "imDisabled", "imBackground", "imKnobNormal", "imBackingStrip", "imAltBackingStrip",
    "imKnobOff", "imKnobOn", "imOff", "imOn", "optSmartTile", "intSmartTileEdgeSize", "optNoGraphics", "clippingRect",
    "optEditable", "optSendValueWhenFocusLost", "optAlphaDisabled", "optGreyOutDisabled", "optNoBackground",
    "fineScale", "incDecButtonMode", "snapSteps", "optScaleKnob", "optHistogram",
    "optNoClickOutsideKnobMove", "optNoClickOutsideKnobPage", "optIsScrollbar", "optXFineBehaviour",
    "optShiftFineBehaviour", "optStripInverted", "macroValueScaler", "microValueScaler", "longDim",
    "optBorder", "optBackground", "optMouseHandle", "alwaysShowVSlider", "alwaysHideHSlider", "optVSlider",
    "optHSlider", "colHeaderSelBackground", "optNotEditable", "optNoSelection", "unitMarkings",
}

# v1 style class -> theme base style.
STYLE_BASES = {
    "CtrlPanelStyle": "BasePanelStyle",
    "CtrlButtonStyle": "LcdButtonStyle",
    "CtrlLabelStyle": "BaseLabelStyle",
    "CtrlTextBoxStyle": "BaseTextBoxStyle",
    "CtrlKnobStyle": "LcdKnobStyle",
    "CtrlSliderStyle": "BaseSliderStyle",
    "CtrlPanSliderStyle": "LcdFaderStyle",
    "CtrlImageViewStyle": "BaseImageStyle",
    "CtrlScrollViewStyle": "BasePanelStyle",
    "CtrlTreeViewStyle": "BasePanelStyle",
    "CtrlTableViewStyle": "BaseTableStyle",
    "CtrlListBoxStyle": "BaseTableStyle",
    "CtrlPlotStyle": "LcdPlotStyle",
    "CtrlMeterStyle": "LcdPlotStyle",
    "CtrlIndicatorStyle": "IndicatorStyle",
}

# Control attributes copied as they are.
CTRL_KEEP = ["name", "id", "displayName", "text", "zorder", "overloadClass", "style"]
CTRL_DROPPED = {"image"}

# v1 folders renamed in the output (v1's `MIxer` only resolved on
# case-insensitive file systems).
FOLDERS = {"MIxer": "Mixer"}


def legacy_names():
    """The legacy product and vendor names, read from dreist-formats'
    legacy table (the one place they are spelled), to rename and to check
    the output against."""
    text = open(LEGACY_RS, encoding="utf-8").read()
    product = re.search(r'ROOT_ELEMENT: &str = "([^"]+)"', text).group(1)
    vendor = re.search(r'REVERB_TYPE: &str = "([^" ]+)', text).group(1)
    return product, vendor


PRODUCT, VENDOR = legacy_names()

# Identifiers of the form <product><suffix> -> neutral names.
RENAMES = {"TabbedPanel": "TabbedPanel", "ButtonStyle": "OptionsButtonStyle", " options": "Options"}


def neutral(v):
    out = re.sub(re.escape(PRODUCT) + "(" + "|".join(re.escape(k) for k in RENAMES) + ")", lambda m: RENAMES[m.group(1)], v)
    if PRODUCT.lower() in out.lower():
        raise SystemExit(f"uigen: no neutral name for {v!r}")
    return out


# ---------------------------------------------------------------- values


def parse_rect(s):
    m = re.fullmatch(r"\s*\[x:(-?\d+) y:(-?\d+) w:(-?\d+) h:(-?\d+)\]\s*", s)
    if not m:
        raise SystemExit(f"uigen: bad rect {s!r}")
    return [int(v) for v in m.groups()]


def parse_colour(s):
    m = re.fullmatch(r"\s*\[r:(\d+) g:(\d+) b:(\d+) a:(\d+)\]\s*", s)
    if not m:
        raise SystemExit(f"uigen: bad colour {s!r}")
    return tuple(int(v) for v in m.groups())


# Palette entries by value: v1 literals that are palette colours are
# written by name.
PALETTE = {}
for _n, _c in theme.COLOURS.items():
    PALETTE.setdefault(tuple(_c) + ((255,) if len(_c) == 3 else ()), _n)


def colour(s):
    c = parse_colour(s)
    if c[3] == 0:
        return "Transparent"
    return PALETTE.get(c) or theme.colour_value(c)


def truthy(v):
    return v.strip().lower() in ("true", "1")


class Fonts:
    """The text fonts the converted styles name (on top of the theme's)."""

    def __init__(self):
        self.used = set(theme.TEXT_SIZES)

    def name(self, vfont):
        size = max(7, int(round(float(vfont.get("vSize", "12")) * theme.TEXT_SCALE)))
        bold = (vfont.get("weight") or "").lower() == "bold"
        self.used.add((size, bold))
        return theme.text_font(size, bold)


# ---------------------------------------------------------------- elements


class E:
    def __init__(self, cls, attrs=None, kids=None, comment=None):
        self.cls, self.attrs, self.kids, self.comment = cls, dict(attrs or {}), list(kids or []), comment


def esc(v):
    return (v.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;").replace('"', "&quot;")
            .replace("\n", "&#10;").replace("\t", "&#9;"))


FIRST = ["name", "rect"]


def write(e, depth=1):
    pad = "    " * depth
    out = []
    if e.comment:
        out.append(f"{pad}<!-- {e.comment} -->")
    keys = [k for k in FIRST if k in e.attrs] + sorted(k for k in e.attrs if k not in FIRST)
    a = "".join(f' {k}="{esc(str(e.attrs[k]))}"' for k in keys)
    if e.kids:
        out.append(f"{pad}<{e.cls}{a}>")
        for k in e.kids:
            out.extend(write(k, depth + 1))
        out.append(f"{pad}</{e.cls}>")
    else:
        out.append(f"{pad}<{e.cls}{a} />")
    return out


def document(elements, header):
    lines = [f"<!-- {header} -->", "<root>"]
    for e in elements:
        lines.extend(write(e))
    lines.append("</root>")
    return ("\n".join(lines) + "\n").encode("utf-8")


# ---------------------------------------------------------------- conversion


class Converter:
    def __init__(self):
        self.fonts = Fonts()
        self.styles = {}  # name -> (source, E)
        self.order = []  # (source, name)
        self.info = {}  # style name -> look applied to controls
        self.extra = {}  # synthesized look styles (image views), name -> E
        self.class_use = {}  # (v1 class, new class) -> count
        self.dropped_ctrls = []
        self.unmatched = []  # bitmap references no look rule matched
        self.decor = 0
        self.skins = {}  # output rel path -> bytes
        self.theme_names = {n for n, _ in theme.base_styles()}
        self.local = {}  # (skin, v1 style name) -> that skin's renamed copy

    # -- styles

    def style(self, el, source):
        name = neutral(el.get("name"))
        a = dict(el.attrib)
        images = {k: v for k, v in a.items() if k.startswith("im")}
        out = {}
        ctrl = {}
        flip = False
        draw_text = True
        for k, v in a.items():
            if k == "name" or k in DROPPED:
                continue
            if k not in STYLE_ATTRS:
                raise SystemExit(f"uigen: {source}: {name}: unknown style attribute {k}")
            dest, _ = STYLE_ATTRS[k]
            if dest is None:
                continue
            if k == "optFlipDirection":
                flip = truthy(v)
            elif k == "optDrawText":
                draw_text = truthy(v)
            elif dest.startswith("ctrl:"):
                ctrl[dest[5:]] = "true" if truthy(v) else "false"
            elif k == "noDefaultDrawing":
                if truthy(v):
                    out["colFill"] = "Transparent"
                    out["optDrawBorder"] = "false"
            elif k in ("centeredText", "rightAlignText", "optTextLeftJustify"):
                if truthy(v):
                    out[dest] = {"centeredText": "center", "rightAlignText": "right", "optTextLeftJustify": "left"}[k]
            elif k in ("leftRangeText", "rightRangeText"):
                out[dest] = v.replace("-oo", "-∞")
            elif dest.startswith("col"):
                if images and el.tag != "CtrlTextBoxStyle":
                    continue  # the look replaces the bitmap's colours
                out[dest] = colour(v)
                if k == "colSlider":
                    out["colAltFill"] = colour(v)
                if k == "colBorder" and out[dest] != "Transparent" and "noDefaultDrawing" not in out:
                    out["optDrawBorder"] = "true"
            else:
                out[dest] = v
        if out.get("colFill") == "Transparent" and a.get("noDefaultDrawing", "").strip() == "true":
            out["optDrawBorder"] = "false"
        for f in el.findall("VFont"):
            out["font"] = self.fonts.name(f)
        if el.tag == "CtrlButtonStyle" and "optIsMomentary" not in ctrl:
            ctrl["optIsMomentary"] = "true"
        look, fit = self.look_for_style(el.tag, name, images, a)
        if not draw_text and not look:
            out["optDrawText"] = "false"
        if fit:
            out["enumDialAlign"] = "Top" if fit == "top" else "Center"
        parent = STYLE_BASES.get(el.tag, "BaseStyle")
        cls = el.tag if el.tag in NATIVE_STYLES else "CtrlStyle"
        if look:
            parent = look["parent"]
            if look["text"] and any("\ue000" <= ch <= "\uf8ff" for ch in look["text"]) and "font" not in look["attrs"]:
                out.pop("font", None)  # the glyph needs the parent's icon font
            out.update(look["attrs"])
            ctrl.update(look["ctrl"])
            if parent == "CheckBoxStyle":
                cls, parent = "CheckBoxButtonRendererStyle", "CheckBoxStyle"
        attrs = {"name": name, "parentStyle0": parent}
        attrs.update(out)
        attrs.update(theme.STYLE_OVERRIDES.get(name, {}))
        info = {"ctrl": ctrl, "look": look, "fit": fit, "class": el.tag, "flip": flip}
        if name in self.theme_names:
            self.info[name] = info
            return  # the theme's own style of that name wins (e.g. LogoStyle)
        e = E(cls, attrs)
        if name in self.styles:
            if write(self.styles[name][1]) == write(e):
                return
            # A skin redefining a shared name: its own copy, renamed for
            # that skin's controls.
            local = name + re.sub(r"[^A-Za-z0-9]", "", os.path.splitext(os.path.basename(source))[0])
            self.local[(source, name)] = local
            name = e.attrs["name"] = local
        self.info[name] = info
        self.styles[name] = (source, e)
        self.order.append((source, name))

    def look_for_style(self, cls, name, images, a):
        """(look, knob fit) for a v1 style, from its bitmaps."""
        if name in looks.BY_NAME:
            return looks.BY_NAME[name], None
        if cls == "CtrlButtonStyle":
            if "imUp" in images:
                r = looks.match(looks.BUTTONS, looks.stem(images["imUp"]))
                if r:
                    return r[1], None
                self.unmatched.append((name, images["imUp"]))
                return None, None
            if truthy(a.get("optNoGraphics", "false")):
                return looks.L("LcdFlatButtonStyle", text=""), None
            if name.startswith("Modal"):
                return looks.L("ModalButtonStyle"), None
            return None, None
        if cls == "CtrlPanelStyle" and "imBackground" in images:
            r = looks.match(looks.PANELS, looks.stem(images["imBackground"]))
            if r:
                return r[1], None
            self.unmatched.append((name, images["imBackground"]))
        if cls == "CtrlTextBoxStyle":
            if name.startswith("Modal"):
                return looks.L("ModalFieldStyle"), None
            if "imBackground" in images:
                return looks.match(looks.TEXT_BOXES, looks.stem(images["imBackground"]))[1], None
            if a.get("colBackground", "").endswith("a:0]"):
                return looks.L("BaseLabelStyle"), None
        if cls == "CtrlKnobStyle":
            r = looks.match(looks.KNOBS, looks.stem(images.get("imKnobNormal", "")))
            if r:
                return r[1], r[2]
        if cls in ("CtrlSliderStyle", "CtrlPanSliderStyle") and "imBackingStrip" in images:
            return looks.match(looks.SLIDERS, looks.stem(images["imBackingStrip"]))[1], None
        if cls == "CtrlImageViewStyle" and "imBackground" in images:
            r = looks.match(looks.IMAGES, looks.stem(images["imBackground"]))
            if r:
                return r[1], None
        if cls == "CtrlMeterStyle":
            return looks.METER, None
        if cls == "CtrlPlotStyle":
            return looks.PLOT, None
        return None, None

    # -- controls

    def image_style(self, look, key):
        """The style an image view's look draws with."""
        if not look["attrs"]:
            return look["parent"]
        name = re.sub(r"[^A-Za-z0-9]", "", key) + "ImageStyle"
        if name not in self.extra:
            self.extra[name] = E("CtrlStyle", dict(name=name, parentStyle0=look["parent"], **look["attrs"]))
        return name

    def ctrl(self, el, source):
        """The converted control: a list (decor siblings first)."""
        v1cls = el.tag
        ov = neutral(el.get("overloadClass", ""))
        if v1cls not in CLASSES:
            raise SystemExit(f"uigen: {source}: unknown control class {v1cls}")
        cls = CLASSES[v1cls][0]
        attrs = {}
        for k, v in el.attrib.items():
            if k in CTRL_KEEP:
                attrs[k] = neutral(v)
            elif k == "rect":
                attrs[k] = list(RECT_FIXES.get((source, el.get("name")), parse_rect(v)))
            elif k not in CTRL_DROPPED:
                raise SystemExit(f"uigen: {source}: unknown control attribute {k}")
        if v1cls == "Ctrl" and ov in CTRL_OVERLOADS:
            spec = CTRL_OVERLOADS[ov]
            if spec is None:
                self.dropped_ctrls.append((source, attrs.get("name"), ov))
                return []
            cls, attrs["style"], attrs["text"] = spec
        if v1cls == "CtrlPanel" and ov in PANEL_OVERLOADS:
            cls, attrs["style"] = PANEL_OVERLOADS[ov]
        if (source, attrs.get("style")) in self.local:
            attrs["style"] = self.local[(source, attrs["style"])]
        if "style" not in attrs:
            st = UNSTYLED.get((v1cls, ov)) or (UNSTYLED_CLASS.get(v1cls) if not ov else None)
            if st:
                attrs["style"] = st
        rect = attrs.get("rect", [0, 0, 0, 0])
        v1_rect = list(rect)
        out_before = []
        kids = []
        info = self.info.get(attrs.get("style", ""))
        look = info["look"] if info else None
        if v1cls == "CtrlImageView":
            img = el.get("image")
            r = looks.match(looks.IMAGES, looks.stem(img)) if img else None
            if r:
                look = r[1]
                attrs["style"] = self.image_style(look, looks.stem(img))
            elif attrs.get("style") in looks.IMAGE_STYLE_LOOKS:
                look = looks.IMAGE_STYLE_LOOKS[attrs["style"]]
                attrs["style"] = self.image_style(look, attrs["style"])
            elif img:
                self.unmatched.append((attrs.get("name"), img))
            if look and look["cls"]:
                cls = look["cls"]
            if look:
                attrs.update(look["ctrl"])
        if look:
            if look["text"] is not None:
                attrs["text"] = look["text"]
            elif look["upper"] and attrs.get("text"):
                attrs["text"] = upper_caption(attrs["text"])
            if look["inset"]:
                l, t, r, b = look["inset"]
                rect = [rect[0] + l, rect[1] + t, rect[2] - l - r, rect[3] - t - b]
        attrs["text"] = CTRL_TEXT.get(attrs.get("name"), attrs.get("text"))
        if attrs["text"] is None:
            del attrs["text"]
        if info:
            for k, v in info["ctrl"].items():
                if cls == "CtrlButton" or not k.startswith("optIs"):
                    attrs[k] = v
            if info["fit"]:
                self.knob_body(rect, info, out_before)
            if v1cls in ("CtrlSlider", "CtrlPanSlider"):
                # v1's flip on its vertical faders gives what dreist-gui's
                # vertical sliders do by default (up is more); so flip
                # vertical ones v1 left unflipped and horizontal ones it
                # flipped (inferred from the faders, no evidence yet).
                if info["flip"] != (rect[3] > rect[2]):
                    attrs["optFlipDirection"] = "true"
        if v1cls in ("CtrlKnob",) and info and info["look"] and info["look"]["parent"] in ("HardKnobStyle", "LcdKnobStyle"):
            if attrs.get("style", "").endswith("C"):
                attrs["optCentered"] = "true"
        decor = (look["decor"] if look else []) + PANEL_DECOR.get(ov, []) + NAMED_DECOR.get((source, attrs.get("name")), [])
        if decor:
            for where, x, y, w, h, st in decor:
                # (decor behind a control: on v1's rect, before any inset)
                base = v1_rect if where == "behind" else rect
                w = w if w >= 1 else base[2] + w
                h = h if h >= 1 else base[3] + h
                y = y if y >= 0 else base[3] + y
                d = E("CtrlPanel", {"rect": [x, y, w, h], "style": st})
                if where == "inside":
                    kids.append(d)
                else:
                    d.attrs["rect"] = [base[0] + x, base[1] + y, w, h]
                    out_before.append(d)
                self.decor += 1
        if cls == "CtrlLabel" and v1cls == "CtrlLabel" and (source, attrs.get("name")) not in LABEL_KEEP:
            rect = self.fit_label(rect, attrs.get("style", ""))
        if cls != v1cls:
            attrs["legacyClass"] = v1cls
        self.class_use[(v1cls, cls)] = self.class_use.get((v1cls, cls), 0) + 1
        attrs["rect"] = ", ".join(str(v) for v in rect)
        for d in out_before:
            d.attrs["rect"] = ", ".join(str(v) for v in d.attrs["rect"])
        for d in kids:
            d.attrs["rect"] = ", ".join(str(v) for v in d.attrs["rect"])
        for ch in el:
            if ch.tag.endswith("Style") or ch.tag in ("VColour", "VFont"):
                continue
            kids.extend(self.ctrl(ch, source))
        kids = CHASSIS.get((source, attrs.get("name")), []) + kids
        return out_before + [E(cls, attrs, kids)]

    def font_size(self, style):
        """The text size a converted style draws with."""
        for _ in range(16):
            e = self.styles.get(style, (None, None))[1]
            if e is None:
                attrs = dict(theme.base_styles()).get(style)
                if attrs is None:
                    return 12
            else:
                attrs = e.attrs
            f = attrs.get("font")
            if f:
                return int(re.search(r"\d+", f).group(0))
            style = attrs.get("parentStyle0", "")
        return 12

    def style_attr(self, style, key):
        for _ in range(16):
            e = self.styles.get(style, (None, None))[1]
            attrs = e.attrs if e is not None else dict(theme.base_styles()).get(style)
            if attrs is None:
                return None
            if key in attrs:
                return attrs[key]
            style = attrs.get("parentStyle0", "")
        return None

    def fit_label(self, rect, style):
        x, y, w, h = rect
        need = self.font_size(style) + LABEL_LEAD
        if h < need:
            y -= (need - h) // 2
            h = need
        align = self.style_attr(style, "enumTextHorizontalAlign")
        if align == "center":
            x -= LABEL_WIDEN
            w += 2 * LABEL_WIDEN
        elif align == "right":
            x -= LABEL_REACH
            w += LABEL_REACH
        return [x, y, w, h]

    def knob_body(self, rect, info, decor):
        """The body panel under the square dial dreist-gui draws in a knob's
        rect (enumDialAlign)."""
        x, y, w, h = rect
        s = min(w, h)
        nx = x + (w - s) // 2
        ny = y if info["fit"] == "top" else y + (h - s) // 2
        if s <= 32:
            return
        for style, x0, y0, x1, y1 in looks.KNOB_BODY.get(info["look"]["parent"], []):
            l, t = int(round(s * x0)), int(round(s * y0))
            r, b = int(round(s * x1)), int(round(s * y1))
            decor.append(E("CtrlPanel", {"rect": [nx + l, ny + t, r - l, b - t], "style": style}))
            self.decor += 1

    # -- files

    def run(self):
        files = []
        for dp, dns, fns in os.walk(V1):
            dns.sort()
            for f in sorted(fns):
                if f.endswith(".skin"):
                    files.append(os.path.relpath(os.path.join(dp, f), V1))
        # Shared styles first, then each skin's own.
        files.sort(key=lambda r: (r != "GlobalStyles.skin", r))
        trees = {r: ET.parse(os.path.join(V1, r)).getroot() for r in files}
        self.colours = []
        for rel in files:
            for el in trees[rel]:
                if el.tag == "VColour":
                    self.colours.append((el.get("name"), tuple(int(el.get(k)) for k in ("red", "green", "blue", "alpha"))))
                elif el.tag.endswith("Style"):
                    self.style(el, rel)
        for rel in files:
            if rel == "GlobalStyles.skin":
                continue
            ctrls = []
            for el in trees[rel]:
                if el.tag.endswith("Style") or el.tag in ("VColour", "VFont"):
                    continue
                ctrls.extend(self.ctrl(el, rel))
            ctrls[-1].kids.extend(EXTRA.get(rel, list)())
            out = rel.replace("\\", "/")
            for a, b in FOLDERS.items():
                out = re.sub(rf"^{a}/", f"{b}/", out)
            header = f"Generated by scripts/uigen from the v1 layout ({rel}). Edit the generator, not this file."
            self.skins[out] = document(ctrls, header)


# ---------------------------------------------------------------- chassis art

# The section bars, frames and the transport backing that v1's window
# background bitmap carried, as panels at the start of the moving part
# of the window (MovePanel coordinates).
CHASSIS = {
    ("RootSkin.skin", "MovePanel"): [
        E("CtrlPanel", {"rect": r, "style": s}) for r, s in [
            # The front panel under the hardware controls: brushed, lighter
            # towards the master section (v1's chassis bitmap).
            ("1, 423, 826, 215", "ChassisFrontStyle"),
            ("2, 428, 283, 11", "ChassisBarStyle"),
            ("290, 428, 392, 11", "ChassisBarMidStyle"),
            ("688, 428, 138, 11", "ChassisBarRightStyle"),
            ("290, 500, 392, 11", "ChassisBarMidStyle"),
            ("2, 443, 58, 58", "ChassisFrameStyle"),
            ("2, 504, 58, 56", "ChassisWellStyle"),
            ("688, 437, 138, 160", "ChassisFrameStyle"),
            ("421, 602, 316, 34", "ChassisBackingStyle"),
        ]
    ],
}
