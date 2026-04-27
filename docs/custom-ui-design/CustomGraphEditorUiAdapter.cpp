#include "CustomGraphEditorUiAdapter.h"

#include <algorithm>
#include <cmath>

namespace curve_editor {

namespace {
constexpr float kMinViewportSize = 1.0f;
}

CustomGraphEditorUiAdapter::CustomGraphEditorUiAdapter(CurveEditorModel* model)
    : model_(model),
      viewport_{0.0f, 0.0f, 100.0f, 100.0f},
      gridDivX_(4),
      gridDivY_(4),
      segmentsPerSpan_(32),
      pickingRadiusPx_(8.0f),
      activeDrag_{DragTargetType::None, 0} {}

void CustomGraphEditorUiAdapter::SetViewport(const EditorViewport& viewport) {
    viewport_ = viewport;
    if (viewport_.width < kMinViewportSize) viewport_.width = kMinViewportSize;
    if (viewport_.height < kMinViewportSize) viewport_.height = kMinViewportSize;
}

void CustomGraphEditorUiAdapter::SetGridDivisions(int xDivs, int yDivs) {
    gridDivX_ = (xDivs < 1) ? 1 : xDivs;
    gridDivY_ = (yDivs < 1) ? 1 : yDivs;
}

void CustomGraphEditorUiAdapter::SetCurveSegmentsPerSpan(int segmentsPerSpan) {
    segmentsPerSpan_ = (segmentsPerSpan < 1) ? 1 : segmentsPerSpan;
}

void CustomGraphEditorUiAdapter::SetPickingRadiusPx(float radiusPx) {
    pickingRadiusPx_ = (radiusPx < 1.0f) ? 1.0f : radiusPx;
}

UiDrawData CustomGraphEditorUiAdapter::BuildDrawData() const {
    UiDrawData draw;
    if (model_ == nullptr) return draw;

    const std::vector<CurveEditorModel::GridLine> grid = model_->GetGridLines(gridDivX_, gridDivY_);
    draw.gridLines.reserve(grid.size());
    for (size_t i = 0; i < grid.size(); ++i) {
        draw.gridLines.push_back(
            {NormalizedToScreen(grid[i].a), NormalizedToScreen(grid[i].b)}
        );
    }

    const std::vector<CurveEditorModel::HandleLink> links = model_->GetHandleLinks();
    draw.handleLines.reserve(links.size());
    for (size_t i = 0; i < links.size(); ++i) {
        draw.handleLines.push_back(
            {NormalizedToScreen(links[i].anchor), NormalizedToScreen(links[i].handle)}
        );
    }

    const std::vector<Point2D>& nodes = model_->GetNodes();
    draw.anchors.reserve(nodes.size());
    draw.inHandles.reserve(nodes.size());
    draw.outHandles.reserve(nodes.size());
    for (size_t i = 0; i < nodes.size(); ++i) {
        draw.anchors.push_back(NormalizedToScreen(nodes[i].anchor));
        if (i > 0) {
            draw.inHandles.push_back(NormalizedToScreen(nodes[i].inHandle));
        }
        if (i + 1 < nodes.size()) {
            draw.outHandles.push_back(NormalizedToScreen(nodes[i].outHandle));
        }
    }

    const std::vector<Point2D> curve = model_->BuildCurvePolyline(segmentsPerSpan_);
    draw.curvePolyline.reserve(curve.size());
    for (size_t i = 0; i < curve.size(); ++i) {
        draw.curvePolyline.push_back(NormalizedToScreen(curve[i]));
    }

    return draw;
}

bool CustomGraphEditorUiAdapter::OnMouseDown(const MouseEvent& e) {
    if (model_ == nullptr) return false;
    if (e.button != MouseButton::Left && e.button != MouseButton::Right) return false;

    const Point2D screen(e.x, e.y);

    size_t nodeIndex = 0;
    if (HitTestAnchor(screen, &nodeIndex)) {
        const bool isInterior = (nodeIndex > 0 && nodeIndex + 1 < model_->NodeCount());
        if ((e.button == MouseButton::Right || e.shiftDown) && isInterior) {
            model_->RemoveNode(nodeIndex);
            activeDrag_ = {DragTargetType::None, 0};
            return true;
        }

        if (e.button == MouseButton::Left) {
            activeDrag_ = {DragTargetType::Anchor, nodeIndex};
            return true;
        }
    }

    if (e.button == MouseButton::Left) {
        if (HitTestHandle(screen, true, &nodeIndex)) {
            activeDrag_ = {DragTargetType::OutHandle, nodeIndex};
            return true;
        }
        if (HitTestHandle(screen, false, &nodeIndex)) {
            activeDrag_ = {DragTargetType::InHandle, nodeIndex};
            return true;
        }

        float newNodeX = 0.0f;
        if (HitTestCurve(screen, &newNodeX)) {
            const size_t inserted = model_->AddNodeOnCurve(newNodeX);
            activeDrag_ = {DragTargetType::Anchor, inserted};
            return true;
        }
    }

    activeDrag_ = {DragTargetType::None, 0};
    return false;
}

bool CustomGraphEditorUiAdapter::OnMouseMove(const MouseEvent& e) {
    if (model_ == nullptr) return false;
    if (activeDrag_.type == DragTargetType::None) return false;

    const Point2D normalized = ScreenToNormalized(Point2D(e.x, e.y));
    switch (activeDrag_.type) {
        case DragTargetType::Anchor:
            model_->MoveAnchor(activeDrag_.nodeIndex, normalized);
            return true;
        case DragTargetType::InHandle:
            model_->MoveInHandle(activeDrag_.nodeIndex, normalized);
            return true;
        case DragTargetType::OutHandle:
            model_->MoveOutHandle(activeDrag_.nodeIndex, normalized);
            return true;
        case DragTargetType::None:
            break;
    }
    return false;
}

bool CustomGraphEditorUiAdapter::OnMouseUp(const MouseEvent& e) {
    (void)e;
    const bool hadDrag = (activeDrag_.type != DragTargetType::None);
    activeDrag_ = {DragTargetType::None, 0};
    return hadDrag;
}

Point2D CustomGraphEditorUiAdapter::ScreenToNormalized(const Point2D& screen) const {
    // Convert screen (origin top-left) to normalized graph space where
    // (0,0) is bottom-left and (1,1) is top-right.
    const float nx = (screen.x - viewport_.left) / viewport_.width;
    const float sy = (screen.y - viewport_.top) / viewport_.height;
    const float ny = 1.0f - sy;
    return Point2D(Clamp01(nx), Clamp01(ny));
}

Point2D CustomGraphEditorUiAdapter::NormalizedToScreen(const Point2D& normalized) const {
    const float nx = Clamp01(normalized.x);
    const float ny = Clamp01(normalized.y);
    const float sx = viewport_.left + nx * viewport_.width;
    const float sy = viewport_.top + (1.0f - ny) * viewport_.height;
    return Point2D(sx, sy);
}

float CustomGraphEditorUiAdapter::DistanceSquared(const Point2D& a, const Point2D& b) const {
    const float dx = a.x - b.x;
    const float dy = a.y - b.y;
    return dx * dx + dy * dy;
}

bool CustomGraphEditorUiAdapter::HitTestAnchor(const Point2D& screenPos, size_t* outNodeIndex) const {
    if (model_ == nullptr) return false;
    const float r2 = pickingRadiusPx_ * pickingRadiusPx_;
    const std::vector<BezierNode>& nodes = model_->GetNodes();
    for (size_t i = 0; i < nodes.size(); ++i) {
        const Point2D p = NormalizedToScreen(nodes[i].anchor);
        if (DistanceSquared(p, screenPos) <= r2) {
            if (outNodeIndex != nullptr) *outNodeIndex = i;
            return true;
        }
    }
    return false;
}

bool CustomGraphEditorUiAdapter::HitTestHandle(const Point2D& screenPos,
                                               bool outHandle,
                                               size_t* outNodeIndex) const {
    if (model_ == nullptr) return false;
    const float r2 = pickingRadiusPx_ * pickingRadiusPx_;
    const std::vector<BezierNode>& nodes = model_->GetNodes();
    for (size_t i = 0; i < nodes.size(); ++i) {
        if (outHandle) {
            if (i + 1 >= nodes.size()) continue;
            const Point2D p = NormalizedToScreen(nodes[i].outHandle);
            if (DistanceSquared(p, screenPos) <= r2) {
                if (outNodeIndex != nullptr) *outNodeIndex = i;
                return true;
            }
        } else {
            if (i == 0) continue;
            const Point2D p = NormalizedToScreen(nodes[i].inHandle);
            if (DistanceSquared(p, screenPos) <= r2) {
                if (outNodeIndex != nullptr) *outNodeIndex = i;
                return true;
            }
        }
    }
    return false;
}

bool CustomGraphEditorUiAdapter::HitTestCurve(const Point2D& screenPos,
                                              float* outNearestNormalizedX) const {
    if (model_ == nullptr) return false;
    const std::vector<Point2D> curve = model_->BuildCurvePolyline(segmentsPerSpan_);
    if (curve.size() < 2) return false;

    const float threshold = pickingRadiusPx_;
    float bestDistance = threshold;
    float bestX = -1.0f;

    for (size_t i = 0; i + 1 < curve.size(); ++i) {
        const Point2D a = NormalizedToScreen(curve[i]);
        const Point2D b = NormalizedToScreen(curve[i + 1]);
        const Point2D ab = b - a;
        const Point2D ap = screenPos - a;
        const float abLen2 = ab.x * ab.x + ab.y * ab.y;
        if (abLen2 <= 1e-8f) continue;

        float t = (ap.x * ab.x + ap.y * ab.y) / abLen2;
        t = std::max(0.0f, std::min(1.0f, t));
        const Point2D q = a + (ab * t);
        const float d = std::sqrt(DistanceSquared(q, screenPos));
        if (d <= bestDistance) {
            bestDistance = d;
            const Point2D n0 = curve[i];
            const Point2D n1 = curve[i + 1];
            bestX = n0.x + (n1.x - n0.x) * t;
        }
    }

    if (bestX < 0.0f) return false;
    if (outNearestNormalizedX != nullptr) *outNearestNormalizedX = Clamp01(bestX);
    return true;
}

}  // namespace curve_editor
