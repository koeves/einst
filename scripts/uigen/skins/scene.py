"""Scenes (Scene/)."""

from . import N, skin


@skin("Scene/Scene.skin")
def scene():
    return [
        N("Panel", "ScenePanel", (5, 4, 105, 75), id=0, style="SceneNoBorderPanelStyleOdd", kids=[
            N("Panel", "TriggerImageView", (10, 8, 83, 38), id=100008, legacyClass="CtrlImageView",
              style="TriggerImageViewStyleImageStyle"),
            N("Button", "SelectButton", (10, 49, 40, 17), displayName="Select Scene", id=100001, optIsMomentary=True,
              overloadClass="FlashingButton", style="SmallSceneButtonStyle", text="G#-2"),
            N("Button", "SnapshotButton", (54, 49, 19, 17), displayName="Snapshot to Scene", icon="camera", iconSize=9,
              id=100001, optIsMomentary=True, overloadClass="IconButton", style="SceneSnapshotButtonStyle", text=""),
            N("Button", "ClearButton", (74, 49, 19, 17), displayName="Clear Scene", icon="trash", iconSize=12, id=100001,
              optIsMomentary=True, overloadClass="IconButton", style="SceneClearButtonStyle", text=""),
            N("Panel", "OverlayImageView", (10, 8, 83, 38), displayName="Trigger Scene", id=100008, legacyClass="CtrlImageView",
              style="BaseImageStyle", zorder=2),
            N("Label", "NameLabel", (10, 10, 83, 15), id=100005, overloadClass="AutoShorteningLabel",
              style="SceneNoteLabelCenteredStyle", text="", zorder=3),
        ]),
    ]


@skin("Scene/ScenePanel.skin")
def scene_panel():
    return [
        N("Panel", "CtrlPanel", (30, 22, 820, 395), id=0, style="ScenePanelBackgroundStyle", kids=[
            N("Panel", None, (2, 2, 811, 390), id=0, style="EmptyPanelStyle", kids=[
                N("Button", "SnapshotToNextButton", (15, 19, 40, 41), displayName="Snapshot to next free Scene", icon="camera",
                  iconSize=20, id=100028, optIsMomentary=True, overloadClass="IconButton", style="SnapshotToNextButtonStyle",
                  text=""),
                N("Button", "TriggerPrevSceneButton", (186, 19, 40, 41), displayName="Trigger previous Scene", id=100028,
                  optIsMomentary=True, optSendOnMouseDown=True, overloadClass="FlashingButton", style="PrevSceneButtonStyle",
                  text="\ue901"),
                N("Button", "TriggerNextSceneButton", (229, 19, 40, 41), displayName="Trigger next Scene", id=100028,
                  optIsMomentary=True, optSendOnMouseDown=True, overloadClass="FlashingButton", style="NextSceneButtonStyle",
                  text="\ue903"),
                N("Button", "ClearAllButton", (402, 19, 40, 41), displayName="Clear all Scenes", icon="trash", iconSize=26,
                  id=100028, optIsMomentary=True, overloadClass="IconButton", style="SceneClearAllButtonStyle", text=""),
                N("Panel", "ScenesPanel", (18, 66, 420, 300), id=100006, style="EmptyPanelStyle", kids=[
                    N("Panel", "Scene12", (1, 1, 105, 75), id=100002, style="EmptyPanelStyle"),
                    N("Panel", "Scene13", (105, 1, 105, 75), id=100002, style="EmptyPanelStyle"),
                    N("Panel", "Scene14", (209, 1, 105, 75), id=100002, style="EmptyPanelStyle"),
                    N("Panel", "Scene15", (313, 1, 105, 75), id=100002, style="EmptyPanelStyle"),
                    N("Panel", "Scene8", (1, 75, 105, 75), id=100002, style="EmptyPanelStyle"),
                    N("Panel", "Scene9", (105, 75, 105, 75), id=100002, style="EmptyPanelStyle"),
                    N("Panel", "Scene10", (209, 75, 105, 75), id=100002, style="EmptyPanelStyle"),
                    N("Panel", "Scene11", (313, 75, 105, 75), id=100002, style="EmptyPanelStyle"),
                    N("Panel", "Scene4", (1, 149, 105, 75), id=100002, style="EmptyPanelStyle"),
                    N("Panel", "Scene5", (105, 149, 105, 75), id=100002, style="EmptyPanelStyle"),
                    N("Panel", "Scene6", (209, 149, 105, 75), id=100002, style="EmptyPanelStyle"),
                    N("Panel", "Scene7", (313, 149, 105, 75), id=100002, style="EmptyPanelStyle"),
                    N("Panel", "Scene0", (1, 223, 105, 75), id=100002, style="EmptyPanelStyle"),
                    N("Panel", "Scene1", (105, 223, 105, 75), id=100002, style="EmptyPanelStyle"),
                    N("Panel", "Scene2", (209, 223, 105, 75), id=100002, style="EmptyPanelStyle"),
                    N("Panel", "Scene3", (313, 223, 105, 75), id=100002, style="EmptyPanelStyle"),
                ]),
                N("Button", "PageUpButton", (454, 67, 26, 20), displayName="Scroll up one page", icon="page_up", id=100028,
                  optIsMomentary=True, overloadClass="IconButton", style="ScenePageUpButtonStyle", text=""),
                N("Button", "RowUpButton", (454, 88, 26, 20), displayName="Scroll up one row", id=100028, optIsMomentary=True,
                  overloadClass="FlashingButton", style="SceneRowUpButtonStyle", text="\ue900"),
                N("Panel", "ScrollArea", (449, 112, 35, 212), id=100000, kids=[
                    N("Button", "MiniSceneEven", (3, 3, 8, 8), id=100003, optIsMomentary=True, style="MiniSceneEvenButtonStyle"),
                    N("Button", "MiniSceneOdd", (3, 19, 8, 8), id=100003, optIsMomentary=True, style="MiniSceneOddButtonStyle"),
                ]),
                N("Button", "RowDownButton", (454, 328, 26, 20), displayName="Scroll down one row", id=100028,
                  optIsMomentary=True, overloadClass="FlashingButton", style="SceneRowDownButtonStyle", text="\ue902"),
                N("Button", "PageDownButton", (454, 349, 26, 20), displayName="Scroll down one page", icon="page_down",
                  id=100028, optIsMomentary=True, overloadClass="IconButton", style="ScenePageDownButtonStyle", text=""),
                N("Slider", "ViewSlider", (447, 110, 39, 216), id=100004, style="SceneViewSliderStyle", zorder=2),
            ]),
            N("Panel", "CtrlPanel", (504, 18, 301, 254), id=100009, style="EmptyPanelStyle", kids=[
                N("Label", "CtrlLabel", (60, 20, 191, 15), id=100000, style="StandardLabelCenteredStyle", text="Start Mode"),
                N("ButtonGroup", "StartModeButtonGroup", (60, 33, 190, 29), displayName="Select Start Mode", id=100003,
                  style="BasePanelStyle", kids=[
                    N("Button", "ImmediateButton", (8, 4, 60, 17), displayName="Start Scene immediately", id=1,
                      optIsMomentary=False, overloadClass="FlashingButton", style="RadioButtonLeftStyle", text="Immediate"),
                    N("Button", "NextBeatButton", (66, 4, 60, 17), displayName="Start Scene on next beat", id=1,
                      optIsMomentary=False, overloadClass="FlashingButton", style="RadioButtonCenterStyle", text="Next Beat"),
                    N("Button", "NextBarButton", (124, 4, 60, 17), displayName="Start Scene on next bar", id=0,
                      optIsMomentary=False, overloadClass="FlashingButton", style="RadioButtonRightStyle", text="Next Bar"),
                ]),
                N("Label", "CtrlLabel", (60, 64, 191, 15), id=100000, style="StandardLabelCenteredStyle", text="Name"),
                N("TextBox", "NameTextBox", (68, 81, 175, 17), id=100001, optSingleClickEdit=True, style="LEDEditBoxStyle"),
                N("TableView", "DetailsTable", (3, 116, 290, 135), id=100000, style="GlobalTableStyle", kids=[
                    N("Panel", "ScrollView", (0, 15, 290, 120), id=0, legacyClass="CtrlScrollView", style="BasePanelStyle"),
                ]),
            ]),
        ]),
    ]
