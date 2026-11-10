"""The pattern tools panel (Tools/)."""

from . import N, skin


@skin("Tools/PatternToolsPanel.skin")
def pattern_tools_panel():
    return [
        N("Panel", "CtrlPanel", (0, 0, 86, 35), id=0, style="EmptyPanelStyle", kids=[
            N("ButtonGroup", "ButtonGroup", (0, 0, 86, 30), id=100001, style="BasePanelStyle", kids=[
                N("Button", "PenButton", (4, 10, 19, 17), displayName="Pen Tool", icon="pencil", id=0, optIsMomentary=False,
                  overloadClass="IconButton", style="PenToolStyle", text=""),
                N("Button", "DeleteButton", (24, 10, 19, 17), displayName="Erase Tool", icon="eraser", id=1,
                  optIsMomentary=False, overloadClass="IconButton", style="DeleteToolStyle", text=""),
                N("Button", "SelectionButton", (44, 10, 19, 17), displayName="Select Tool", icon="select", id=2,
                  optIsMomentary=False, overloadClass="IconButton", style="SelectionToolStyle", text=""),
                N("Button", "MultiButton", (64, 10, 19, 17), displayName="Multi Tool", icon="pencil_plus", id=3,
                  optIsMomentary=False, overloadClass="IconButton", style="MultiToolStyle", text=""),
            ]),
        ]),
    ]
