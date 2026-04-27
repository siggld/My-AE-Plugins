#pragma once

#include "CurveEditorModel.h"

#include <cstddef>
#include <vector>

namespace curve_editor {

// Screen-space rectangle where the graph editor is rendered.
// Screen origin is top-left, Y grows downward.
struct EditorViewport {
    float left;
    float top;
    float width;
    float height;
};

struct UiLine {
    Point2D a;  // screen-space
    Point2D b;  // screen-space
};

struct UiDrawData {
    std::vector<UiLine> gridLines;
    std::vector<UiLine> handleLines;
    std::vector<Point2D> curvePolyline;  // screen-space points
    std::vector<Point2D> anchors;        // screen-space points
    std::vector<Point2D> inHandles;      // screen-space points
    std::vector<Point2D> outHandles;     // screen-space points
};

class CustomGraphEditorUiAdapter {
public:
    enum class MouseButton {
        Left,
        Right,
        Middle
    };

    struct MouseEvent {
        float x;
        float y;
        MouseButton button;
        bool shiftDown;
        bool altDown;
    };

    explicit CustomGraphEditorUiAdapter(CurveEditorModel* model);

    void SetViewport(const EditorViewport& viewport);
    void SetGridDivisions(int xDivs, int yDivs);
    void SetCurveSegmentsPerSpan(int segmentsPerSpan);
    void SetPickingRadiusPx(float radiusPx);

    UiDrawData BuildDrawData() const;

    // Input mapping:
    // - Left click+drag on anchor: MoveAnchor
    // - Left click+drag on handle: MoveInHandle / MoveOutHandle
    // - Left click on curve: AddNodeOnCurve and begin anchor drag
    // - Shift+Left click on interior anchor: RemoveNode
    // - Right click on interior anchor: RemoveNode
    bool OnMouseDown(const MouseEvent& e);
    bool OnMouseMove(const MouseEvent& e);
    bool OnMouseUp(const MouseEvent& e);

    Point2D ScreenToNormalized(const Point2D& screen) const;
    Point2D NormalizedToScreen(const Point2D& normalized) const;

private:
    enum class DragTargetType {
        None,
        Anchor,
        InHandle,
        OutHandle
    };

    struct DragTarget {
        DragTargetType type;
        size_t nodeIndex;
    };

    CurveEditorModel* model_;
    EditorViewport viewport_;
    int gridDivX_;
    int gridDivY_;
    int segmentsPerSpan_;
    float pickingRadiusPx_;
    DragTarget activeDrag_;

    float DistanceSquared(const Point2D& a, const Point2D& b) const;
    bool HitTestAnchor(const Point2D& screenPos, size_t* outNodeIndex) const;
    bool HitTestHandle(const Point2D& screenPos, bool outHandle, size_t* outNodeIndex) const;
    bool HitTestCurve(const Point2D& screenPos, float* outNearestNormalizedX) const;
};

}  // namespace curve_editor
