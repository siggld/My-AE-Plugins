#include "CurveEditorModel.h"

#include <algorithm>
#include <cmath>

namespace curve_editor {

// --- Point2D free functions ----------------------------------------------

Point2D operator+(const Point2D& a, const Point2D& b) {
    return Point2D(a.x + b.x, a.y + b.y);
}

Point2D operator-(const Point2D& a, const Point2D& b) {
    return Point2D(a.x - b.x, a.y - b.y);
}

Point2D operator*(const Point2D& a, float s) {
    return Point2D(a.x * s, a.y * s);
}

float Clamp01(float v) {
    if (v < 0.0f) return 0.0f;
    if (v > 1.0f) return 1.0f;
    return v;
}

Point2D Lerp(const Point2D& a, const Point2D& b, float t) {
    return Point2D(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t);
}

Point2D CubicBezierPoint(const Point2D& p0,
                         const Point2D& p1,
                         const Point2D& p2,
                         const Point2D& p3,
                         float t) {
    const float u = 1.0f - t;
    const float b0 = u * u * u;
    const float b1 = 3.0f * u * u * t;
    const float b2 = 3.0f * u * t * t;
    const float b3 = t * t * t;
    return Point2D(b0 * p0.x + b1 * p1.x + b2 * p2.x + b3 * p3.x,
                   b0 * p0.y + b1 * p1.y + b2 * p2.y + b3 * p3.y);
}

float CubicBezierX(const Point2D& p0,
                   const Point2D& p1,
                   const Point2D& p2,
                   const Point2D& p3,
                   float t) {
    const float u = 1.0f - t;
    return u * u * u * p0.x + 3.0f * u * u * t * p1.x
           + 3.0f * u * t * t * p2.x + t * t * t * p3.x;
}

float CubicBezierY(const Point2D& p0,
                   const Point2D& p1,
                   const Point2D& p2,
                   const Point2D& p3,
                   float t) {
    const float u = 1.0f - t;
    return u * u * u * p0.y + 3.0f * u * u * t * p1.y
           + 3.0f * u * t * t * p2.y + t * t * t * p3.y;
}

float CubicBezierDerivativeX(const Point2D& p0,
                             const Point2D& p1,
                             const Point2D& p2,
                             const Point2D& p3,
                             float t) {
    const float u = 1.0f - t;
    return 3.0f * u * u * (p1.x - p0.x)
           + 6.0f * u * t * (p2.x - p1.x)
           + 3.0f * t * t * (p3.x - p2.x);
}

// --- BezierNode -----------------------------------------------------------

BezierNode::BezierNode() : anchor(), inHandle(), outHandle() {}

BezierNode::BezierNode(const Point2D& a, const Point2D& in, const Point2D& out)
    : anchor(a), inHandle(in), outHandle(out) {}

// --- CurveEditorModel -----------------------------------------------------

CurveEditorModel::CurveEditorModel() {
    ResetToDefault();
}

void CurveEditorModel::ResetToDefault() {
    // Linear default: anchors at (0,0) and (1,1) with handles at the 1/3
    // points. This yields a cubic bezier identical to y=x.
    nodes_.clear();
    nodes_.emplace_back(Point2D(0.0f, 0.0f),
                        Point2D(0.0f, 0.0f),
                        Point2D(1.0f / 3.0f, 1.0f / 3.0f));
    nodes_.emplace_back(Point2D(1.0f, 1.0f),
                        Point2D(2.0f / 3.0f, 2.0f / 3.0f),
                        Point2D(1.0f, 1.0f));
}

size_t CurveEditorModel::NodeCount() const {
    return nodes_.size();
}

const BezierNode& CurveEditorModel::GetNode(size_t index) const {
    return nodes_[index];
}

const std::vector<BezierNode>& CurveEditorModel::GetNodes() const {
    return nodes_;
}

size_t CurveEditorModel::FindSegmentIndex(float x) const {
    // nodes_.size() >= 2 by invariant.
    if (x <= nodes_.front().anchor.x) return 0;
    if (x >= nodes_.back().anchor.x) return nodes_.size() - 2;
    for (size_t i = 0; i + 1 < nodes_.size(); ++i) {
        if (x <= nodes_[i + 1].anchor.x) {
            return i;
        }
    }
    return nodes_.size() - 2;
}

float CurveEditorModel::SolveTForX(const Point2D& p0,
                                   const Point2D& p1,
                                   const Point2D& p2,
                                   const Point2D& p3,
                                   float targetX) {
    // Normalize targetX into [p0.x, p3.x] to pick a sensible initial guess.
    const float span = p3.x - p0.x;
    float t = (span > 1e-8f) ? (targetX - p0.x) / span : 0.5f;
    t = Clamp01(t);

    // Newton-Raphson: fast when the derivative is well-behaved.
    for (int i = 0; i < 8; ++i) {
        const float xt = CubicBezierX(p0, p1, p2, p3, t) - targetX;
        if (std::fabs(xt) < 1e-6f) return t;
        const float dxt = CubicBezierDerivativeX(p0, p1, p2, p3, t);
        if (std::fabs(dxt) < 1e-6f) break;
        t -= xt / dxt;
        t = Clamp01(t);
    }

    // Bisection fallback. Segment is X-monotonic by class invariant, so a
    // bracket always exists.
    float lo = 0.0f;
    float hi = 1.0f;
    for (int i = 0; i < 32; ++i) {
        const float mid = 0.5f * (lo + hi);
        const float xm = CubicBezierX(p0, p1, p2, p3, mid);
        if (xm < targetX) {
            lo = mid;
        } else {
            hi = mid;
        }
        if (hi - lo < 1e-6f) return 0.5f * (lo + hi);
    }
    return 0.5f * (lo + hi);
}

float CurveEditorModel::EvaluateY(float x) const {
    x = Clamp01(x);
    if (x <= nodes_.front().anchor.x) return nodes_.front().anchor.y;
    if (x >= nodes_.back().anchor.x) return nodes_.back().anchor.y;

    const size_t seg = FindSegmentIndex(x);
    const BezierNode& a = nodes_[seg];
    const BezierNode& b = nodes_[seg + 1];
    const Point2D p0 = a.anchor;
    const Point2D p1 = a.outHandle;
    const Point2D p2 = b.inHandle;
    const Point2D p3 = b.anchor;

    const float t = SolveTForX(p0, p1, p2, p3, x);
    return CubicBezierY(p0, p1, p2, p3, t);
}

size_t CurveEditorModel::AddNodeOnCurve(float x) {
    x = Clamp01(x);
    // Clamp x strictly between the end anchors so we always split an
    // existing interior segment.
    const float minX = nodes_.front().anchor.x;
    const float maxX = nodes_.back().anchor.x;
    if (x <= minX) x = minX + 1e-4f;
    if (x >= maxX) x = maxX - 1e-4f;

    const size_t seg = FindSegmentIndex(x);
    BezierNode& a = nodes_[seg];
    BezierNode& b = nodes_[seg + 1];

    const Point2D p0 = a.anchor;
    const Point2D p1 = a.outHandle;
    const Point2D p2 = b.inHandle;
    const Point2D p3 = b.anchor;

    const float t = SolveTForX(p0, p1, p2, p3, x);

    // De Casteljau subdivision at t:
    //   q0 = lerp(p0, p1, t)
    //   q1 = lerp(p1, p2, t)
    //   q2 = lerp(p2, p3, t)
    //   r0 = lerp(q0, q1, t)
    //   r1 = lerp(q1, q2, t)
    //   s  = lerp(r0, r1, t)  == point on curve
    const Point2D q0 = Lerp(p0, p1, t);
    const Point2D q1 = Lerp(p1, p2, t);
    const Point2D q2 = Lerp(p2, p3, t);
    const Point2D r0 = Lerp(q0, q1, t);
    const Point2D r1 = Lerp(q1, q2, t);
    const Point2D s  = Lerp(r0, r1, t);

    // Adjust existing handles so the two new segments still match the
    // original shape.
    a.outHandle = q0;
    b.inHandle = q2;

    BezierNode inserted(s, r0, r1);
    nodes_.insert(nodes_.begin() + static_cast<std::ptrdiff_t>(seg + 1), inserted);

    return seg + 1;
}

bool CurveEditorModel::RemoveNode(size_t index) {
    // End anchors are protected.
    if (index == 0 || index + 1 >= nodes_.size()) return false;
    nodes_.erase(nodes_.begin() + static_cast<std::ptrdiff_t>(index));
    return true;
}

void CurveEditorModel::MoveAnchor(size_t index, const Point2D& target) {
    if (index >= nodes_.size()) return;

    Point2D next = target;
    next.x = Clamp01(next.x);
    next.y = Clamp01(next.y);

    // Lock end anchors on the X axis. Y is still movable.
    if (index == 0) {
        next.x = nodes_.front().anchor.x;
    } else if (index + 1 == nodes_.size()) {
        next.x = nodes_.back().anchor.x;
    } else {
        const float lo = nodes_[index - 1].anchor.x;
        const float hi = nodes_[index + 1].anchor.x;
        const float pad = 1e-4f;
        if (next.x <= lo) next.x = lo + pad;
        if (next.x >= hi) next.x = hi - pad;
    }

    const Point2D delta = next - nodes_[index].anchor;
    nodes_[index].anchor = next;
    // Drag handles along with the anchor so their relative offset stays
    // stable; the caller can still move them independently afterwards.
    nodes_[index].inHandle = nodes_[index].inHandle + delta;
    nodes_[index].outHandle = nodes_[index].outHandle + delta;

    EnforceMonotonicityAround(index);
}

void CurveEditorModel::MoveInHandle(size_t index, const Point2D& target) {
    if (index >= nodes_.size()) return;

    Point2D next = target;
    next.x = Clamp01(next.x);
    next.y = Clamp01(next.y);

    // inHandle belongs to the segment [index-1, index]. Keep the handle's X
    // inside that segment so the segment stays X-monotonic.
    if (index == 0) {
        next = nodes_[0].anchor;
    } else {
        const float lo = nodes_[index - 1].anchor.x;
        const float hi = nodes_[index].anchor.x;
        if (next.x < lo) next.x = lo;
        if (next.x > hi) next.x = hi;
    }

    nodes_[index].inHandle = next;
    EnforceMonotonicityAround(index);
}

void CurveEditorModel::MoveOutHandle(size_t index, const Point2D& target) {
    if (index >= nodes_.size()) return;

    Point2D next = target;
    next.x = Clamp01(next.x);
    next.y = Clamp01(next.y);

    // outHandle belongs to the segment [index, index+1].
    if (index + 1 >= nodes_.size()) {
        next = nodes_.back().anchor;
    } else {
        const float lo = nodes_[index].anchor.x;
        const float hi = nodes_[index + 1].anchor.x;
        if (next.x < lo) next.x = lo;
        if (next.x > hi) next.x = hi;
    }

    nodes_[index].outHandle = next;
    EnforceMonotonicityAround(index);
}

void CurveEditorModel::EnforceMonotonicityAround(size_t index) {
    // For every segment touching `index`, clamp the control polygon so that
    //   P0.x <= P1.x <= P2.x <= P3.x
    // which is a sufficient condition for x(t) to be monotonic on [0,1].
    auto fix = [&](size_t left) {
        if (left + 1 >= nodes_.size()) return;
        BezierNode& a = nodes_[left];
        BezierNode& b = nodes_[left + 1];
        if (a.outHandle.x < a.anchor.x) a.outHandle.x = a.anchor.x;
        if (a.outHandle.x > b.anchor.x) a.outHandle.x = b.anchor.x;
        if (b.inHandle.x < a.anchor.x) b.inHandle.x = a.anchor.x;
        if (b.inHandle.x > b.anchor.x) b.inHandle.x = b.anchor.x;
        if (a.outHandle.x > b.inHandle.x) {
            const float mid = 0.5f * (a.outHandle.x + b.inHandle.x);
            a.outHandle.x = mid;
            b.inHandle.x = mid;
        }
    };
    if (index > 0) fix(index - 1);
    if (index + 1 < nodes_.size()) fix(index);
}

// --- Drawing data ---------------------------------------------------------

std::vector<CurveEditorModel::GridLine>
CurveEditorModel::GetGridLines(int xDivs, int yDivs) const {
    std::vector<GridLine> lines;
    if (xDivs < 1) xDivs = 1;
    if (yDivs < 1) yDivs = 1;
    lines.reserve(static_cast<size_t>(xDivs + yDivs + 2) * 2);

    for (int i = 0; i <= xDivs; ++i) {
        const float x = static_cast<float>(i) / static_cast<float>(xDivs);
        lines.push_back({Point2D(x, 0.0f), Point2D(x, 1.0f)});
    }
    for (int j = 0; j <= yDivs; ++j) {
        const float y = static_cast<float>(j) / static_cast<float>(yDivs);
        lines.push_back({Point2D(0.0f, y), Point2D(1.0f, y)});
    }
    return lines;
}

std::vector<CurveEditorModel::HandleLink>
CurveEditorModel::GetHandleLinks() const {
    std::vector<HandleLink> links;
    links.reserve(nodes_.size() * 2);
    for (size_t i = 0; i < nodes_.size(); ++i) {
        // Skip degenerate handles for the end anchors (they coincide with
        // the anchor itself and would render as zero-length lines).
        if (i > 0) {
            links.push_back({i, nodes_[i].anchor, nodes_[i].inHandle, false});
        }
        if (i + 1 < nodes_.size()) {
            links.push_back({i, nodes_[i].anchor, nodes_[i].outHandle, true});
        }
    }
    return links;
}

std::vector<Point2D>
CurveEditorModel::BuildCurvePolyline(int segmentsPerSpan) const {
    if (segmentsPerSpan < 1) segmentsPerSpan = 1;

    std::vector<Point2D> pts;
    const size_t spans = nodes_.size() - 1;
    pts.reserve(spans * static_cast<size_t>(segmentsPerSpan) + 1);
    pts.push_back(nodes_.front().anchor);

    for (size_t i = 0; i < spans; ++i) {
        const BezierNode& a = nodes_[i];
        const BezierNode& b = nodes_[i + 1];
        const Point2D p0 = a.anchor;
        const Point2D p1 = a.outHandle;
        const Point2D p2 = b.inHandle;
        const Point2D p3 = b.anchor;
        for (int k = 1; k <= segmentsPerSpan; ++k) {
            const float t = static_cast<float>(k) / static_cast<float>(segmentsPerSpan);
            pts.push_back(CubicBezierPoint(p0, p1, p2, p3, t));
        }
    }
    return pts;
}

}  // namespace curve_editor
