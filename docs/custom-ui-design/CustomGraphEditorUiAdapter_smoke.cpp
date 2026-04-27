#include "CustomGraphEditorUiAdapter.h"

#include <cassert>

namespace curve_editor {

// Minimal UI integration smoke checks.
// This function is intentionally lightweight and can be called from any
// host-specific test harness when wiring the Custom UI.
void RunCustomGraphEditorUiAdapterSmokeTest() {
    CurveEditorModel model;
    CustomGraphEditorUiAdapter ui(&model);
    ui.SetViewport({10.0f, 20.0f, 300.0f, 180.0f});
    ui.SetGridDivisions(4, 4);
    ui.SetCurveSegmentsPerSpan(24);
    ui.SetPickingRadiusPx(10.0f);

    // 1) redraw data generation
    const UiDrawData draw0 = ui.BuildDrawData();
    assert(!draw0.gridLines.empty());
    assert(!draw0.curvePolyline.empty());
    assert(model.NodeCount() == 2);

    // 2) add node by clicking on the curve around x=0.5
    const Point2D curveMidScreen = ui.NormalizedToScreen(Point2D(0.5f, model.EvaluateY(0.5f)));
    const CustomGraphEditorUiAdapter::MouseEvent addDown{
        curveMidScreen.x,
        curveMidScreen.y,
        CustomGraphEditorUiAdapter::MouseButton::Left,
        false,
        false
    };
    const bool didAdd = ui.OnMouseDown(addDown);
    ui.OnMouseUp(addDown);
    assert(didAdd);
    assert(model.NodeCount() == 3);

    // 3) move the inserted anchor (index 1)
    const Point2D anchorScreen = ui.NormalizedToScreen(model.GetNode(1).anchor);
    CustomGraphEditorUiAdapter::MouseEvent anchorDown{
        anchorScreen.x,
        anchorScreen.y,
        CustomGraphEditorUiAdapter::MouseButton::Left,
        false,
        false
    };
    assert(ui.OnMouseDown(anchorDown));
    const Point2D movedTarget = ui.NormalizedToScreen(Point2D(0.55f, 0.70f));
    CustomGraphEditorUiAdapter::MouseEvent anchorMove{
        movedTarget.x,
        movedTarget.y,
        CustomGraphEditorUiAdapter::MouseButton::Left,
        false,
        false
    };
    assert(ui.OnMouseMove(anchorMove));
    ui.OnMouseUp(anchorMove);

    // 4) move one handle of the inserted node
    const Point2D inHandleScreen = ui.NormalizedToScreen(model.GetNode(1).inHandle);
    CustomGraphEditorUiAdapter::MouseEvent handleDown{
        inHandleScreen.x,
        inHandleScreen.y,
        CustomGraphEditorUiAdapter::MouseButton::Left,
        false,
        false
    };
    assert(ui.OnMouseDown(handleDown));
    const Point2D handleTarget = ui.NormalizedToScreen(Point2D(0.52f, 0.62f));
    CustomGraphEditorUiAdapter::MouseEvent handleMove{
        handleTarget.x,
        handleTarget.y,
        CustomGraphEditorUiAdapter::MouseButton::Left,
        false,
        false
    };
    assert(ui.OnMouseMove(handleMove));
    ui.OnMouseUp(handleMove);

    // 5) redraw still returns valid curve data
    const UiDrawData draw1 = ui.BuildDrawData();
    assert(draw1.curvePolyline.size() >= draw0.curvePolyline.size());
}

}  // namespace curve_editor
