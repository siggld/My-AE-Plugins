#pragma once

// Platform-independent data model for an AE Custom UI bezier curve editor.
// No AE SDK / Drawbot dependencies. UI code is expected to consume the
// drawing primitives returned by this model and render them itself.

#include <cstddef>
#include <vector>

namespace curve_editor {

struct Point2D {
    float x;
    float y;

    Point2D() : x(0.0f), y(0.0f) {}
    Point2D(float ix, float iy) : x(ix), y(iy) {}
};

Point2D operator+(const Point2D& a, const Point2D& b);
Point2D operator-(const Point2D& a, const Point2D& b);
Point2D operator*(const Point2D& a, float s);

float Clamp01(float v);
Point2D Lerp(const Point2D& a, const Point2D& b, float t);

// Cubic Bezier helpers. Control points are in absolute normalized coords.
Point2D CubicBezierPoint(const Point2D& p0,
                         const Point2D& p1,
                         const Point2D& p2,
                         const Point2D& p3,
                         float t);

float CubicBezierX(const Point2D& p0,
                   const Point2D& p1,
                   const Point2D& p2,
                   const Point2D& p3,
                   float t);

float CubicBezierY(const Point2D& p0,
                   const Point2D& p1,
                   const Point2D& p2,
                   const Point2D& p3,
                   float t);

float CubicBezierDerivativeX(const Point2D& p0,
                             const Point2D& p1,
                             const Point2D& p2,
                             const Point2D& p3,
                             float t);

// Single anchor with its in/out handles. Handles are stored in absolute
// normalized coordinates (same space as the anchor), not as offsets.
class BezierNode {
public:
    Point2D anchor;
    Point2D inHandle;
    Point2D outHandle;

    BezierNode();
    BezierNode(const Point2D& a, const Point2D& in, const Point2D& out);
};

// Owns an ordered list of BezierNodes describing a curve that is monotonic
// in X. Node[0] sits at x=0 and Node[last] sits at x=1 in the default state.
// Invariants maintained by mutating APIs:
//   - nodes_.size() >= 2
//   - nodes are sorted by anchor.x (strictly ascending for interior nodes)
//   - each segment's control polygon is X-monotonic:
//       P0.x <= P1.x <= P2.x <= P3.x
//   - all coordinates are clamped to [0, 1]
class CurveEditorModel {
public:
    struct GridLine {
        Point2D a;
        Point2D b;
    };

    struct HandleLink {
        size_t nodeIndex;
        Point2D anchor;
        Point2D handle;
        bool isOutHandle;
    };

    CurveEditorModel();

    // Restore the two-anchor default (0,0) -> (1,1) with linear handles.
    void ResetToDefault();

    size_t NodeCount() const;
    const BezierNode& GetNode(size_t index) const;
    const std::vector<BezierNode>& GetNodes() const;

    // Insert a new anchor on the existing curve at the given x. The curve
    // shape is preserved via De Casteljau subdivision of the containing
    // segment. Returns the index of the inserted node.
    size_t AddNodeOnCurve(float x);

    // Remove an interior node. End nodes (index 0 and last) are protected.
    // Returns true if a removal happened.
    bool RemoveNode(size_t index);

    // Mutators. Inputs are clamped to preserve the class invariants.
    void MoveAnchor(size_t index, const Point2D& target);
    void MoveInHandle(size_t index, const Point2D& target);
    void MoveOutHandle(size_t index, const Point2D& target);

    // Evaluate the curve's Y at a given X. Assumes (and enforces) X-monotonicity.
    // x is clamped into the domain spanned by the first and last anchors.
    float EvaluateY(float x) const;

    // Drawing data. All returned coordinates live in the normalized [0,1]^2 space.
    std::vector<GridLine> GetGridLines(int xDivs, int yDivs) const;
    std::vector<HandleLink> GetHandleLinks() const;
    std::vector<Point2D> BuildCurvePolyline(int segmentsPerSpan) const;

private:
    std::vector<BezierNode> nodes_;

    // Returns the segment index i such that
    //   nodes_[i].anchor.x <= x <= nodes_[i+1].anchor.x
    // Clamps to a valid segment if x is outside the domain.
    size_t FindSegmentIndex(float x) const;

    // Solve x(t) = targetX on a single cubic segment using Newton-Raphson
    // with a bisection fallback. Assumes segment is X-monotonic.
    static float SolveTForX(const Point2D& p0,
                            const Point2D& p1,
                            const Point2D& p2,
                            const Point2D& p3,
                            float targetX);

    // Re-apply the X-monotone control-polygon constraint on segments that
    // touch a given node index.
    void EnforceMonotonicityAround(size_t index);
};

}  // namespace curve_editor
