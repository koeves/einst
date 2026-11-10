"""The pattern selectors and the convertor (Patterns/)."""

from . import N, skin


@skin("Patterns/PatternConvertor.skin")
def pattern_convertor():
    return [
        N("Panel", "CtrlPanel", (50, 34, 164, 141), id=0, style="ModalPanelStyle", kids=[
            N("Panel", "CtrlPanel", (8, 9, 150, 97), id=0, style="ModalPanelCollectionStyle", kids=[
                N("Label", "CtrlLabel", (23, 15, 75, 17), id=100003, style="ModalLabelStyle", text="Number of steps"),
                N("TextBox", "NumStepsBox", (99, 15, 40, 17), id=100001, optSingleClickEdit=True, style="ModalEditBoxStyle"),
                N("Label", "CtrlLabel", (53, 39, 45, 17), id=100005, style="ModalLabelStyle", text="Grid Type"),
                N("PopupButton", "RezCombo", (99, 39, 40, 17), id=100004, legacyClass="CtrlComboBox", optSingleClickEdit=True,
                  style="ModalComboBoxStyle"),
                N("Label", "CtrlLabel", (8, 62, 90, 17), id=100009, style="ModalLabelStyle", text="Apply to all patterns"),
                N("Button", "ApplyToAll", (99, 62, 14, 15), id=100008, optIsMomentary=False, renderer="CheckBoxButtonRenderer",
                  style="ModalCheckButtonStyle", text=""),
            ]),
            N("Button", "CancelButton", (36, 118, 50, 17), id=100007, optIsMomentary=True, style="ModalMomentaryButtonStyle",
              text="Cancel"),
            N("Button", "ApplyButton", (90, 118, 50, 17), id=100006, optIsMomentary=True, style="ModalMomentaryButtonStyle",
              text="Apply"),
        ]),
    ]


@skin("Patterns/PatternSelectors.skin")
def pattern_selectors():
    return [
        N("Panel", "CtrlPanel", (0, 0, 340, 85), id=0, style="EmptyPanelStyle", kids=[
            N("Button", "Key1", (12, 1, 21, 36), id=100003, optIsMomentary=False, overloadClass="PatternSelector",
              style="SelectorStyleNotAllocated"),
            N("Button", "Key3", (36, 1, 21, 36), id=100004, optIsMomentary=False, overloadClass="PatternSelector",
              style="SelectorStyleNotAllocated"),
            N("Button", "Key6", (86, 1, 21, 36), id=100005, optIsMomentary=False, overloadClass="PatternSelector",
              style="SelectorStyleNotAllocated"),
            N("Button", "Key8", (110, 1, 21, 36), id=100006, optIsMomentary=False, overloadClass="PatternSelector",
              style="SelectorStyleNotAllocated"),
            N("Button", "Key10", (134, 1, 21, 36), id=100007, optIsMomentary=False, overloadClass="PatternSelector",
              style="SelectorStyleNotAllocated"),
            N("Button", "Key13", (181, 1, 21, 36), id=100008, optIsMomentary=False, overloadClass="PatternSelector",
              style="SelectorStyleNotAllocated"),
            N("Button", "Key15", (205, 1, 21, 36), id=100009, optIsMomentary=False, overloadClass="PatternSelector",
              style="SelectorStyleNotAllocated"),
            N("Button", "Key18", (254, 1, 21, 36), id=100010, optIsMomentary=False, overloadClass="PatternSelector",
              style="SelectorStyleNotAllocated"),
            N("Button", "Key20", (278, 1, 21, 36), id=100011, optIsMomentary=False, overloadClass="PatternSelector",
              style="SelectorStyleNotAllocated"),
            N("Button", "Key22", (302, 1, 21, 36), id=100012, optIsMomentary=False, overloadClass="PatternSelector",
              style="SelectorStyleNotAllocated"),
            N("Button", "Key0", (1, 44, 21, 36), id=100000, optIsMomentary=False, overloadClass="PatternSelector",
              style="SelectorStyleNotAllocated"),
            N("Button", "Key2", (25, 44, 21, 36), id=100001, optIsMomentary=False, overloadClass="PatternSelector",
              style="SelectorStyleNotAllocated"),
            N("Button", "Key4", (49, 44, 21, 36), id=100000, optIsMomentary=False, overloadClass="PatternSelector",
              style="SelectorStyleNotAllocated"),
            N("Button", "Key5", (73, 44, 21, 36), id=100001, optIsMomentary=False, overloadClass="PatternSelector",
              style="SelectorStyleNotAllocated"),
            N("Button", "Key7", (97, 44, 21, 36), id=100000, optIsMomentary=False, overloadClass="PatternSelector",
              style="SelectorStyleNotAllocated"),
            N("Button", "Key9", (121, 44, 21, 36), id=100001, optIsMomentary=False, overloadClass="PatternSelector",
              style="SelectorStyleNotAllocated"),
            N("Button", "Key11", (145, 44, 21, 36), id=100000, optIsMomentary=False, overloadClass="PatternSelector",
              style="SelectorStyleNotAllocated"),
            N("Button", "Key12", (169, 44, 21, 36), id=100001, optIsMomentary=False, overloadClass="PatternSelector",
              style="SelectorStyleNotAllocated"),
            N("Button", "Key14", (193, 44, 21, 36), id=100000, optIsMomentary=False, overloadClass="PatternSelector",
              style="SelectorStyleNotAllocated"),
            N("Button", "Key16", (217, 44, 21, 36), id=100001, optIsMomentary=False, overloadClass="PatternSelector",
              style="SelectorStyleNotAllocated"),
            N("Button", "Key17", (241, 44, 21, 36), id=100000, optIsMomentary=False, overloadClass="PatternSelector",
              style="SelectorStyleNotAllocated"),
            N("Button", "Key19", (265, 44, 21, 36), id=100001, optIsMomentary=False, overloadClass="PatternSelector",
              style="SelectorStyleNotAllocated"),
            N("Button", "Key21", (289, 44, 21, 36), id=100000, optIsMomentary=False, overloadClass="PatternSelector",
              style="SelectorStyleNotAllocated"),
            N("Button", "Key23", (313, 44, 21, 36), id=100001, optIsMomentary=False, overloadClass="PatternSelector",
              style="SelectorStyleNotAllocated"),
        ]),
    ]
