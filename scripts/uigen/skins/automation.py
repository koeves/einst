"""Automation: the host and auto-map panels (Automation/)."""

from . import N, skin


@skin("Automation/AutoMapPanel.skin")
def auto_map_panel():
    return [
        N("Panel", "CtrlPanel", (0, 0, 295, 598), id=0, style="EmptyPanelStyle", kids=[
            N("Panel", "MapControls", (0, 0, 295, 21), id=0, style="EmptyPanelStyle", kids=[
                N("Button", "Clear", (5, 2, 19, 17), displayName="Clear all mappings", icon="cross", id=100016,
                  optIsMomentary=True, overloadClass="IconButton", style="StandardClearButtonStyle", text=""),
                N("Button", "Lock", (25, 2, 19, 17), displayName="Lock map", icon="padlock", id=100017, optIsMomentary=False,
                  overloadClass="IconButton", style="StandardLockButtonStyle", text=""),
                N("Button", "Select", (138, 2, 50, 17), displayName="Automatically select bindings when recieving MIDI",
                  id=10000, optIsMomentary=False, style="LEDLatchingButtonStyle", text="Auto-sel"),
                N("Button", "SendNotesButton", (190, 2, 50, 17), id=100000, optIsMomentary=False,
                  style="LEDLatchingButtonStyle", text="Send Note"),
                N("Button", "SendCCButton", (242, 2, 50, 17), id=100001, optIsMomentary=False, style="LEDLatchingButtonStyle",
                  text="Send CC"),
            ]),
            N("TableView", "TableView", (0, 21, 295, 577), id=100001, style="GlobalTableStyleAlwaysVisibleHSlider", kids=[
                N("Panel", "ScrollView", (0, 15, 295, 562), id=0, legacyClass="CtrlScrollView", style="BasePanelStyle"),
            ]),
        ]),
    ]


@skin("Automation/HostPanel.skin")
def host_panel():
    return [
        N("Panel", "CtrlPanel", (0, 0, 295, 210), id=0, style="EmptyPanelStyle", kids=[
            N("Label", "CtrlLabel", (-8, 0, 311, 15), id=100001, style="StandardLabelCenteredStyle", text="Host parameters"),
            N("Button", "Param0", (21, 18, 50, 17), id=100002, optIsMomentary=True, style="LEDMomentaryButtonStyle", text=1),
            N("Button", "Param1", (88, 18, 50, 17), id=100010, optIsMomentary=True, style="LEDMomentaryButtonStyle", text=2),
            N("Button", "Param2", (155, 18, 50, 17), id=100004, optIsMomentary=True, style="LEDMomentaryButtonStyle", text=3),
            N("Button", "Param3", (222, 18, 50, 17), id=100005, optIsMomentary=True, style="LEDMomentaryButtonStyle", text=4),
            N("Button", "Param4", (21, 41, 50, 17), id=100006, optIsMomentary=True, style="LEDMomentaryButtonStyle", text=5),
            N("Button", "Param5", (88, 41, 50, 17), id=100007, optIsMomentary=True, style="LEDMomentaryButtonStyle", text=6),
            N("Button", "Param6", (155, 41, 50, 17), id=100008, optIsMomentary=True, style="LEDMomentaryButtonStyle", text=7),
            N("Button", "Param7", (222, 41, 50, 17), id=100009, optIsMomentary=True, style="LEDMomentaryButtonStyle", text=8),
            N("Button", "Param8", (21, 65, 50, 17), id=100002, optIsMomentary=True, style="LEDMomentaryButtonStyle", text=9),
            N("Button", "Param9", (88, 65, 50, 17), id=100010, optIsMomentary=True, style="LEDMomentaryButtonStyle", text=10),
            N("Button", "Param10", (155, 65, 50, 17), id=100004, optIsMomentary=True, style="LEDMomentaryButtonStyle", text=11),
            N("Button", "Param11", (222, 65, 50, 17), id=100005, optIsMomentary=True, style="LEDMomentaryButtonStyle", text=12),
            N("Button", "Param12", (21, 88, 50, 17), id=100006, optIsMomentary=True, style="LEDMomentaryButtonStyle", text=13),
            N("Button", "Param13", (88, 88, 50, 17), id=100007, optIsMomentary=True, style="LEDMomentaryButtonStyle", text=14),
            N("Button", "Param14", (155, 88, 50, 17), id=100008, optIsMomentary=True, style="LEDMomentaryButtonStyle", text=15),
            N("Button", "Param15", (222, 88, 50, 17), id=100009, optIsMomentary=True, style="LEDMomentaryButtonStyle", text=16),
            N("Button", "Param16", (21, 110, 50, 17), id=100002, optIsMomentary=True, style="LEDMomentaryButtonStyle", text=17),
            N("Button", "Param17", (88, 110, 50, 17), id=100010, optIsMomentary=True, style="LEDMomentaryButtonStyle", text=18),
            N("Button", "Param18", (155, 110, 50, 17), id=100004, optIsMomentary=True, style="LEDMomentaryButtonStyle", text=19),
            N("Button", "Param19", (222, 110, 50, 17), id=100005, optIsMomentary=True, style="LEDMomentaryButtonStyle", text=20),
            N("Button", "Param20", (21, 133, 50, 17), id=100006, optIsMomentary=True, style="LEDMomentaryButtonStyle", text=21),
            N("Button", "Param21", (88, 133, 50, 17), id=100007, optIsMomentary=True, style="LEDMomentaryButtonStyle", text=22),
            N("Button", "Param22", (155, 133, 50, 17), id=100008, optIsMomentary=True, style="LEDMomentaryButtonStyle", text=23),
            N("Button", "Param23", (222, 133, 50, 17), id=100009, optIsMomentary=True, style="LEDMomentaryButtonStyle", text=24),
            N("Button", "Param24", (21, 156, 50, 17), id=100002, optIsMomentary=True, style="LEDMomentaryButtonStyle", text=25),
            N("Button", "Param25", (88, 156, 50, 17), id=100010, optIsMomentary=True, style="LEDMomentaryButtonStyle", text=26),
            N("Button", "Param26", (155, 156, 50, 17), id=100004, optIsMomentary=True, style="LEDMomentaryButtonStyle", text=27),
            N("Button", "Param27", (222, 156, 50, 17), id=100005, optIsMomentary=True, style="LEDMomentaryButtonStyle", text=28),
            N("Button", "Param28", (21, 179, 50, 17), id=100006, optIsMomentary=True, style="LEDMomentaryButtonStyle", text=29),
            N("Button", "Param29", (88, 179, 50, 17), id=100007, optIsMomentary=True, style="LEDMomentaryButtonStyle", text=30),
            N("Button", "Param30", (155, 179, 50, 17), id=100008, optIsMomentary=True, style="LEDMomentaryButtonStyle", text=31),
            N("Button", "Param31", (222, 179, 50, 17), id=100009, optIsMomentary=True, style="LEDMomentaryButtonStyle", text=32),
        ]),
    ]
