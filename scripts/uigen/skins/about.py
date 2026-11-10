"""The About panel (About/)."""

from . import N, skin


@skin("About/AboutPanel.skin")
def about_panel():
    return [
        N("Panel", "CtrlPanel", (0, 0, 654, 500), id=0, style="AboutPanelStyle", kids=[
            N("Panel", None, (0, 0, 654, 262), style="AboutArtStyle"),
            N("Label", "VersionLabel", (2, 264, 250, 15), id=100004, style="AboutLabelStyle", text=""),
            N("Label", "DateLabel", (392, 264, 260, 15), id=100005, style="AboutRightLabel", text=""),
            N("Panel", "AboutView", (2, 282, 650, 215), id=100000),
        ]),
    ]
