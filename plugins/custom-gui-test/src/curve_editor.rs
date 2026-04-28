// Platform-independent data model for the AE Custom UI bezier curve editor.
// Ported from docs/custom-ui-design/CurveEditorModel.{h,cpp}.
//
// All coordinates live in the normalized [0, 1]^2 graph space. UI code is
// expected to consume the drawing primitives returned by this model and
// render them itself. There are no AE/Drawbot dependencies here.

#![allow(dead_code)]

use std::ops::{Add, Mul, Sub};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point2D {
    pub x: f32,
    pub y: f32,
}

impl Point2D {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl Add for Point2D {
    type Output = Point2D;
    fn add(self, rhs: Point2D) -> Point2D {
        Point2D::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub for Point2D {
    type Output = Point2D;
    fn sub(self, rhs: Point2D) -> Point2D {
        Point2D::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Mul<f32> for Point2D {
    type Output = Point2D;
    fn mul(self, s: f32) -> Point2D {
        Point2D::new(self.x * s, self.y * s)
    }
}

pub fn clamp01(v: f32) -> f32 {
    v.clamp(0.0, 1.0)
}

pub fn lerp(a: Point2D, b: Point2D, t: f32) -> Point2D {
    Point2D::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t)
}

pub fn cubic_bezier_point(p0: Point2D, p1: Point2D, p2: Point2D, p3: Point2D, t: f32) -> Point2D {
    let u = 1.0 - t;
    let b0 = u * u * u;
    let b1 = 3.0 * u * u * t;
    let b2 = 3.0 * u * t * t;
    let b3 = t * t * t;
    Point2D::new(
        b0 * p0.x + b1 * p1.x + b2 * p2.x + b3 * p3.x,
        b0 * p0.y + b1 * p1.y + b2 * p2.y + b3 * p3.y,
    )
}

pub fn cubic_bezier_x(p0: Point2D, p1: Point2D, p2: Point2D, p3: Point2D, t: f32) -> f32 {
    let u = 1.0 - t;
    u * u * u * p0.x + 3.0 * u * u * t * p1.x + 3.0 * u * t * t * p2.x + t * t * t * p3.x
}

pub fn cubic_bezier_y(p0: Point2D, p1: Point2D, p2: Point2D, p3: Point2D, t: f32) -> f32 {
    let u = 1.0 - t;
    u * u * u * p0.y + 3.0 * u * u * t * p1.y + 3.0 * u * t * t * p2.y + t * t * t * p3.y
}

pub fn cubic_bezier_derivative_x(
    p0: Point2D,
    p1: Point2D,
    p2: Point2D,
    p3: Point2D,
    t: f32,
) -> f32 {
    let u = 1.0 - t;
    3.0 * u * u * (p1.x - p0.x) + 6.0 * u * t * (p2.x - p1.x) + 3.0 * t * t * (p3.x - p2.x)
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BezierNode {
    pub anchor: Point2D,
    pub in_handle: Point2D,
    pub out_handle: Point2D,
}

impl BezierNode {
    pub const fn new(anchor: Point2D, in_handle: Point2D, out_handle: Point2D) -> Self {
        Self {
            anchor,
            in_handle,
            out_handle,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct GridLine {
    pub a: Point2D,
    pub b: Point2D,
}

#[derive(Clone, Copy, Debug)]
pub struct HandleLink {
    pub node_index: usize,
    pub anchor: Point2D,
    pub handle: Point2D,
    pub is_out_handle: bool,
}

// Owns an ordered list of BezierNodes describing a curve that is monotonic
// in X. Node[0] sits at x=0 and Node[last] sits at x=1 in the default state.
//
// Invariants maintained by mutating APIs:
//   - nodes.len() >= 2
//   - nodes are sorted by anchor.x (strictly ascending for interior nodes)
//   - each segment's control polygon is X-monotonic:
//       P0.x <= P1.x <= P2.x <= P3.x
//   - all coordinates are clamped to [0, 1]
pub struct CurveEditorModel {
    nodes: Vec<BezierNode>,
}

impl Default for CurveEditorModel {
    fn default() -> Self {
        Self::new()
    }
}

impl CurveEditorModel {
    pub fn new() -> Self {
        let mut m = Self { nodes: Vec::new() };
        m.reset_to_default();
        m
    }

    pub fn reset_to_default(&mut self) {
        self.nodes.clear();
        self.nodes.push(BezierNode::new(
            Point2D::new(0.0, 0.0),
            Point2D::new(0.0, 0.0),
            Point2D::new(1.0 / 3.0, 1.0 / 3.0),
        ));
        self.nodes.push(BezierNode::new(
            Point2D::new(1.0, 1.0),
            Point2D::new(2.0 / 3.0, 2.0 / 3.0),
            Point2D::new(1.0, 1.0),
        ));
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn get_node(&self, index: usize) -> &BezierNode {
        &self.nodes[index]
    }

    pub fn get_nodes(&self) -> &[BezierNode] {
        &self.nodes
    }

    fn find_segment_index(&self, x: f32) -> usize {
        if x <= self.nodes.first().unwrap().anchor.x {
            return 0;
        }
        if x >= self.nodes.last().unwrap().anchor.x {
            return self.nodes.len() - 2;
        }
        for i in 0..self.nodes.len() - 1 {
            if x <= self.nodes[i + 1].anchor.x {
                return i;
            }
        }
        self.nodes.len() - 2
    }

    fn solve_t_for_x(p0: Point2D, p1: Point2D, p2: Point2D, p3: Point2D, target_x: f32) -> f32 {
        let span = p3.x - p0.x;
        let mut t = if span > 1e-8 {
            (target_x - p0.x) / span
        } else {
            0.5
        };
        t = clamp01(t);

        for _ in 0..8 {
            let xt = cubic_bezier_x(p0, p1, p2, p3, t) - target_x;
            if xt.abs() < 1e-6 {
                return t;
            }
            let dxt = cubic_bezier_derivative_x(p0, p1, p2, p3, t);
            if dxt.abs() < 1e-6 {
                break;
            }
            t -= xt / dxt;
            t = clamp01(t);
        }

        let mut lo = 0.0_f32;
        let mut hi = 1.0_f32;
        for _ in 0..32 {
            let mid = 0.5 * (lo + hi);
            let xm = cubic_bezier_x(p0, p1, p2, p3, mid);
            if xm < target_x {
                lo = mid;
            } else {
                hi = mid;
            }
            if hi - lo < 1e-6 {
                return 0.5 * (lo + hi);
            }
        }
        0.5 * (lo + hi)
    }

    pub fn evaluate_y(&self, x: f32) -> f32 {
        let x = clamp01(x);
        let first = self.nodes.first().unwrap();
        let last = self.nodes.last().unwrap();
        if x <= first.anchor.x {
            return first.anchor.y;
        }
        if x >= last.anchor.x {
            return last.anchor.y;
        }

        let seg = self.find_segment_index(x);
        let a = &self.nodes[seg];
        let b = &self.nodes[seg + 1];
        let p0 = a.anchor;
        let p1 = a.out_handle;
        let p2 = b.in_handle;
        let p3 = b.anchor;

        let t = Self::solve_t_for_x(p0, p1, p2, p3, x);
        cubic_bezier_y(p0, p1, p2, p3, t)
    }

    // Insert a new anchor on the existing curve at the given x. The curve
    // shape is preserved via De Casteljau subdivision of the containing
    // segment. Returns the index of the inserted node.
    pub fn add_node_on_curve(&mut self, x: f32) -> usize {
        let mut x = clamp01(x);
        let min_x = self.nodes.first().unwrap().anchor.x;
        let max_x = self.nodes.last().unwrap().anchor.x;
        if x <= min_x {
            x = min_x + 1e-4;
        }
        if x >= max_x {
            x = max_x - 1e-4;
        }

        let seg = self.find_segment_index(x);
        let (p0, p1, p2, p3) = {
            let a = &self.nodes[seg];
            let b = &self.nodes[seg + 1];
            (a.anchor, a.out_handle, b.in_handle, b.anchor)
        };

        let t = Self::solve_t_for_x(p0, p1, p2, p3, x);

        let q0 = lerp(p0, p1, t);
        let q1 = lerp(p1, p2, t);
        let q2 = lerp(p2, p3, t);
        let r0 = lerp(q0, q1, t);
        let r1 = lerp(q1, q2, t);
        let s = lerp(r0, r1, t);

        self.nodes[seg].out_handle = q0;
        self.nodes[seg + 1].in_handle = q2;

        let inserted = BezierNode::new(s, r0, r1);
        self.nodes.insert(seg + 1, inserted);

        seg + 1
    }

    // Remove an interior node. End nodes (index 0 and last) are protected.
    // Returns true if a removal happened.
    pub fn remove_node(&mut self, index: usize) -> bool {
        if index == 0 || index + 1 >= self.nodes.len() {
            return false;
        }
        self.nodes.remove(index);
        true
    }

    pub fn move_anchor(&mut self, index: usize, target: Point2D) -> usize {
        if index >= self.nodes.len() {
            return index;
        }

        let mut next = Point2D::new(clamp01(target.x), clamp01(target.y));

        if index == 0 {
            next.x = self.nodes.first().unwrap().anchor.x;
        } else if index + 1 == self.nodes.len() {
            next.x = self.nodes.last().unwrap().anchor.x;
        }

        let delta = next - self.nodes[index].anchor;
        self.nodes[index].anchor = next;
        self.nodes[index].in_handle = self.nodes[index].in_handle + delta;
        self.nodes[index].out_handle = self.nodes[index].out_handle + delta;

        let mut new_index = index;
        if index > 0 && index + 1 < self.nodes.len() {
            let moved = self.nodes.remove(index);
            let mut insert_at = 1usize;
            while insert_at < self.nodes.len() - 1
                && self.nodes[insert_at].anchor.x < moved.anchor.x
            {
                insert_at += 1;
            }
            self.nodes.insert(insert_at, moved);
            new_index = insert_at;

            // Resolve exact x-ties locally so points can cross without pushing others to ends.
            let pad = 1e-4_f32;
            for i in 1..self.nodes.len() - 1 {
                if self.nodes[i].anchor.x <= self.nodes[i - 1].anchor.x {
                    self.nodes[i].anchor.x = self.nodes[i - 1].anchor.x + pad;
                }
            }
            for i in (1..self.nodes.len() - 1).rev() {
                if self.nodes[i].anchor.x >= self.nodes[i + 1].anchor.x {
                    self.nodes[i].anchor.x = self.nodes[i + 1].anchor.x - pad;
                }
            }
        }
        for i in 0..self.nodes.len() - 1 {
            self.fix_segment(i);
        }
        new_index
    }

    pub fn move_in_handle(&mut self, index: usize, target: Point2D) {
        if index >= self.nodes.len() {
            return;
        }

        let mut next = Point2D::new(clamp01(target.x), clamp01(target.y));

        if index == 0 {
            next = self.nodes[0].anchor;
        } else {
            let lo = self.nodes[index - 1].anchor.x;
            let hi = self.nodes[index].anchor.x;
            if next.x < lo {
                next.x = lo;
            }
            if next.x > hi {
                next.x = hi;
            }
        }

        self.nodes[index].in_handle = next;
        self.enforce_monotonicity_around(index);
    }

    pub fn move_out_handle(&mut self, index: usize, target: Point2D) {
        if index >= self.nodes.len() {
            return;
        }

        let mut next = Point2D::new(clamp01(target.x), clamp01(target.y));

        if index + 1 >= self.nodes.len() {
            next = *self.nodes.last().map(|n| &n.anchor).unwrap();
        } else {
            let lo = self.nodes[index].anchor.x;
            let hi = self.nodes[index + 1].anchor.x;
            if next.x < lo {
                next.x = lo;
            }
            if next.x > hi {
                next.x = hi;
            }
        }

        self.nodes[index].out_handle = next;
        self.enforce_monotonicity_around(index);
    }

    fn enforce_monotonicity_around(&mut self, index: usize) {
        // For every segment touching `index`, clamp the control polygon so that
        //   P0.x <= P1.x <= P2.x <= P3.x
        // which is a sufficient condition for x(t) to be monotonic on [0, 1].
        if index > 0 {
            self.fix_segment(index - 1);
        }
        if index + 1 < self.nodes.len() {
            self.fix_segment(index);
        }
    }

    fn fix_segment(&mut self, left: usize) {
        if left + 1 >= self.nodes.len() {
            return;
        }
        let a_anchor_x = self.nodes[left].anchor.x;
        let b_anchor_x = self.nodes[left + 1].anchor.x;

        if self.nodes[left].out_handle.x < a_anchor_x {
            self.nodes[left].out_handle.x = a_anchor_x;
        }
        if self.nodes[left].out_handle.x > b_anchor_x {
            self.nodes[left].out_handle.x = b_anchor_x;
        }
        if self.nodes[left + 1].in_handle.x < a_anchor_x {
            self.nodes[left + 1].in_handle.x = a_anchor_x;
        }
        if self.nodes[left + 1].in_handle.x > b_anchor_x {
            self.nodes[left + 1].in_handle.x = b_anchor_x;
        }
        if self.nodes[left].out_handle.x > self.nodes[left + 1].in_handle.x {
            let mid = 0.5 * (self.nodes[left].out_handle.x + self.nodes[left + 1].in_handle.x);
            self.nodes[left].out_handle.x = mid;
            self.nodes[left + 1].in_handle.x = mid;
        }
    }

    pub fn get_grid_lines(&self, x_divs: i32, y_divs: i32) -> Vec<GridLine> {
        let x_divs = x_divs.max(1);
        let y_divs = y_divs.max(1);
        let mut lines = Vec::with_capacity((x_divs + y_divs + 2) as usize);
        for i in 0..=x_divs {
            let x = i as f32 / x_divs as f32;
            lines.push(GridLine {
                a: Point2D::new(x, 0.0),
                b: Point2D::new(x, 1.0),
            });
        }
        for j in 0..=y_divs {
            let y = j as f32 / y_divs as f32;
            lines.push(GridLine {
                a: Point2D::new(0.0, y),
                b: Point2D::new(1.0, y),
            });
        }
        lines
    }

    pub fn get_handle_links(&self) -> Vec<HandleLink> {
        let mut links = Vec::with_capacity(self.nodes.len() * 2);
        for i in 0..self.nodes.len() {
            // Skip degenerate handles for the end anchors (they coincide with
            // the anchor itself and would render as zero-length lines).
            if i > 0 {
                links.push(HandleLink {
                    node_index: i,
                    anchor: self.nodes[i].anchor,
                    handle: self.nodes[i].in_handle,
                    is_out_handle: false,
                });
            }
            if i + 1 < self.nodes.len() {
                links.push(HandleLink {
                    node_index: i,
                    anchor: self.nodes[i].anchor,
                    handle: self.nodes[i].out_handle,
                    is_out_handle: true,
                });
            }
        }
        links
    }

    pub fn build_curve_polyline(&self, segments_per_span: i32) -> Vec<Point2D> {
        let segments_per_span = segments_per_span.max(1);
        let spans = self.nodes.len() - 1;
        let mut pts = Vec::with_capacity(spans * segments_per_span as usize + 1);
        pts.push(self.nodes.first().unwrap().anchor);
        for i in 0..spans {
            let a = &self.nodes[i];
            let b = &self.nodes[i + 1];
            let p0 = a.anchor;
            let p1 = a.out_handle;
            let p2 = b.in_handle;
            let p3 = b.anchor;
            for k in 1..=segments_per_span {
                let t = k as f32 / segments_per_span as f32;
                pts.push(cubic_bezier_point(p0, p1, p2, p3, t));
            }
        }
        pts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f32, b: f32, eps: f32) -> bool {
        (a - b).abs() <= eps
    }

    #[test]
    fn default_is_linear() {
        let m = CurveEditorModel::new();
        assert_eq!(m.node_count(), 2);
        assert!(approx(m.evaluate_y(0.0), 0.0, 1e-5));
        assert!(approx(m.evaluate_y(1.0), 1.0, 1e-5));
        assert!(approx(m.evaluate_y(0.5), 0.5, 1e-3));
        assert!(approx(m.evaluate_y(0.25), 0.25, 1e-3));
    }

    #[test]
    fn add_node_preserves_shape() {
        let mut m = CurveEditorModel::new();
        let y_before = m.evaluate_y(0.7);
        let inserted = m.add_node_on_curve(0.5);
        assert_eq!(inserted, 1);
        assert_eq!(m.node_count(), 3);
        let y_after = m.evaluate_y(0.7);
        assert!(approx(y_before, y_after, 1e-3));
        // The inserted anchor sits roughly on the original curve at x=0.5.
        let inserted_node = m.get_node(1);
        assert!(approx(inserted_node.anchor.x, 0.5, 1e-3));
        assert!(approx(inserted_node.anchor.y, 0.5, 1e-3));
    }

    #[test]
    fn end_nodes_are_protected_from_removal() {
        let mut m = CurveEditorModel::new();
        m.add_node_on_curve(0.5);
        let last = m.node_count() - 1;
        assert!(!m.remove_node(0));
        assert!(!m.remove_node(last));
        assert!(m.remove_node(1));
        assert_eq!(m.node_count(), 2);
    }

    #[test]
    fn move_anchor_keeps_x_monotonic() {
        let mut m = CurveEditorModel::new();
        m.add_node_on_curve(0.3);
        m.add_node_on_curve(0.7);
        // Try to drag the middle anchor past its right neighbor.
        m.move_anchor(1, Point2D::new(0.95, 0.2));
        let n0 = m.get_node(0).anchor.x;
        let n1 = m.get_node(1).anchor.x;
        let n2 = m.get_node(2).anchor.x;
        let n3 = m.get_node(3).anchor.x;
        assert!(
            n0 < n1 && n1 < n2 && n2 < n3,
            "x must stay strictly ascending: {n0},{n1},{n2},{n3}"
        );
    }

    #[test]
    fn end_anchors_lock_x_axis() {
        let mut m = CurveEditorModel::new();
        m.move_anchor(0, Point2D::new(0.4, 0.2));
        m.move_anchor(1, Point2D::new(0.4, 0.8));
        assert!(approx(m.get_node(0).anchor.x, 0.0, 1e-5));
        assert!(approx(m.get_node(1).anchor.x, 1.0, 1e-5));
        assert!(approx(m.get_node(0).anchor.y, 0.2, 1e-5));
        assert!(approx(m.get_node(1).anchor.y, 0.8, 1e-5));
    }

    #[test]
    fn move_handle_clamps_to_segment_x() {
        let mut m = CurveEditorModel::new();
        m.add_node_on_curve(0.5);
        // Out handle of node[0] belongs to segment [0, 1]; its X must be
        // clamped to [0, 0.5].
        m.move_out_handle(0, Point2D::new(0.9, 0.5));
        assert!(m.get_node(0).out_handle.x <= 0.5 + 1e-5);
        // In handle of node[1] belongs to segment [0, 1]; its X must be
        // clamped to [0, 0.5].
        m.move_in_handle(1, Point2D::new(0.9, 0.0));
        assert!(m.get_node(1).in_handle.x <= 0.5 + 1e-5);
    }

    #[test]
    fn build_curve_polyline_endpoints_match_anchors() {
        let m = CurveEditorModel::new();
        let pts = m.build_curve_polyline(16);
        assert_eq!(pts.first().unwrap().x, 0.0);
        assert_eq!(pts.last().unwrap().x, 1.0);
    }

    #[test]
    fn grid_lines_count() {
        let m = CurveEditorModel::new();
        let lines = m.get_grid_lines(4, 4);
        assert_eq!(lines.len(), (4 + 4 + 2) as usize);
    }
}
