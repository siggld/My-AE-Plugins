// UI adapter for the AE Custom UI bezier curve editor.
// Ported from docs/custom-ui-design/CustomGraphEditorUiAdapter.{h,cpp}.
//
// Bridges screen-space pixel events to the normalized [0,1]^2 graph space
// owned by `CurveEditorModel`. UI code is expected to call `build_draw_data`
// on Draw and `on_mouse_*` on input events.

#![allow(dead_code)]

use super::curve_editor::{CurveEditorModel, Point2D, clamp01};

const MIN_VIEWPORT_SIZE: f32 = 1.0;

#[derive(Clone, Copy, Debug)]
pub struct EditorViewport {
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
}

impl Default for EditorViewport {
    fn default() -> Self {
        Self {
            left: 0.0,
            top: 0.0,
            width: 100.0,
            height: 100.0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct UiLine {
    pub a: Point2D,
    pub b: Point2D,
}

#[derive(Clone, Copy, Debug)]
pub struct UiMarker {
    pub center: Point2D,
    pub selected: bool,
}

#[derive(Clone, Debug, Default)]
pub struct UiDrawData {
    pub grid_lines: Vec<UiLine>,
    pub handle_lines: Vec<UiLine>,
    pub curve_polyline: Vec<Point2D>,
    pub anchors: Vec<UiMarker>,
    pub in_handles: Vec<UiMarker>,
    pub out_handles: Vec<UiMarker>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Clone, Copy, Debug)]
pub struct MouseEvent {
    pub x: f32,
    pub y: f32,
    pub button: MouseButton,
    pub shift_down: bool,
    pub alt_down: bool,
    pub ctrl_down: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DragTargetType {
    None,
    Anchor,
    InHandle,
    OutHandle,
}

#[derive(Clone, Copy, Debug)]
struct DragTarget {
    kind: DragTargetType,
    node_index: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum HandleDragMode {
    LinkedHorizontal,
    Single,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Selection {
    Anchor(usize),
    InHandle(usize),
    OutHandle(usize),
}

pub struct CustomGraphEditorUiAdapter {
    viewport: EditorViewport,
    grid_div_x: i32,
    grid_div_y: i32,
    segments_per_span: i32,
    picking_radius_px: f32,
    active_drag: DragTarget,
    selected: Option<Selection>,
    handle_drag_mode: Option<HandleDragMode>,
    handle_enabled: Vec<bool>,
    snap_enabled: bool,
    snap_threshold_px: f32,
    linked_ratio: f32,
}

impl Default for CustomGraphEditorUiAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl CustomGraphEditorUiAdapter {
    pub fn new() -> Self {
        Self {
            viewport: EditorViewport::default(),
            grid_div_x: 4,
            grid_div_y: 4,
            segments_per_span: 32,
            picking_radius_px: 8.0,
            active_drag: DragTarget {
                kind: DragTargetType::None,
                node_index: 0,
            },
            selected: None,
            handle_drag_mode: None,
            handle_enabled: Vec::new(),
            snap_enabled: false,
            snap_threshold_px: 10.0,
            linked_ratio: 1.0,
        }
    }

    pub fn set_snap_enabled(&mut self, enabled: bool) {
        self.snap_enabled = enabled;
    }

    fn sync_node_state(&mut self, model: &CurveEditorModel) {
        let n = model.node_count();
        if self.handle_enabled.len() != n {
            self.handle_enabled.resize(n, false);
        }
    }

    fn apply_handles_for_anchor(&mut self, model: &mut CurveEditorModel, index: usize) {
        if index >= model.node_count() {
            return;
        }
        let anchor = model.get_node(index).anchor;
        let min_norm_x = (18.0 / self.viewport.width.max(1.0)).min(0.25);
        if index > 0 {
            let prev = model.get_node(index - 1).anchor;
            let span_l = (anchor.x - prev.x).max(1e-4);
            let len = (span_l / 2.0).max(min_norm_x).min(span_l - 1e-5);
            let in_target = Point2D::new(anchor.x - len, anchor.y);
            model.move_in_handle(index, in_target);
        }
        if index + 1 < model.node_count() {
            let next = model.get_node(index + 1).anchor;
            let span_r = (next.x - anchor.x).max(1e-4);
            let len = (span_r / 2.0).max(min_norm_x).min(span_r - 1e-5);
            let out_target = Point2D::new(anchor.x + len, anchor.y);
            model.move_out_handle(index, out_target);
        }
        self.handle_enabled[index] = true;
    }

    fn remove_handles_for_anchor(&mut self, model: &mut CurveEditorModel, index: usize) {
        if index >= model.node_count() {
            return;
        }
        let anchor = model.get_node(index).anchor;
        let linear_lerp = |a: Point2D, b: Point2D, t: f32| -> Point2D {
            Point2D::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t)
        };
        if index > 0 {
            let prev = model.get_node(index - 1).anchor;
            model.move_in_handle(index, linear_lerp(prev, anchor, 2.0 / 3.0));
        }
        if index + 1 < model.node_count() {
            let next = model.get_node(index + 1).anchor;
            model.move_out_handle(index, linear_lerp(anchor, next, 1.0 / 3.0));
        }
        self.handle_enabled[index] = false;
    }

    pub fn set_viewport(&mut self, viewport: EditorViewport) {
        self.viewport = viewport;
        if self.viewport.width < MIN_VIEWPORT_SIZE {
            self.viewport.width = MIN_VIEWPORT_SIZE;
        }
        if self.viewport.height < MIN_VIEWPORT_SIZE {
            self.viewport.height = MIN_VIEWPORT_SIZE;
        }
    }

    pub fn set_grid_divisions(&mut self, x_divs: i32, y_divs: i32) {
        self.grid_div_x = x_divs.max(1);
        self.grid_div_y = y_divs.max(1);
    }

    pub fn set_curve_segments_per_span(&mut self, segments_per_span: i32) {
        self.segments_per_span = segments_per_span.max(1);
    }

    pub fn set_picking_radius_px(&mut self, radius_px: f32) {
        self.picking_radius_px = radius_px.max(1.0);
    }

    pub fn build_draw_data(&self, model: &CurveEditorModel) -> UiDrawData {
        let mut draw = UiDrawData::default();

        let grid = model.get_grid_lines(self.grid_div_x, self.grid_div_y);
        draw.grid_lines.reserve(grid.len());
        for g in &grid {
            draw.grid_lines.push(UiLine {
                a: self.normalized_to_screen(g.a),
                b: self.normalized_to_screen(g.b),
            });
        }

        let links = model.get_handle_links();
        draw.handle_lines.reserve(links.len());
        for h in &links {
            if !self
                .handle_enabled
                .get(h.node_index)
                .copied()
                .unwrap_or(false)
            {
                continue;
            }
            draw.handle_lines.push(UiLine {
                a: self.normalized_to_screen(h.anchor),
                b: self.normalized_to_screen(h.handle),
            });
        }

        let nodes = model.get_nodes();
        draw.anchors.reserve(nodes.len());
        draw.in_handles.reserve(nodes.len());
        draw.out_handles.reserve(nodes.len());
        for (i, n) in nodes.iter().enumerate() {
            draw.anchors.push(UiMarker {
                center: self.normalized_to_screen(n.anchor),
                selected: self.selected == Some(Selection::Anchor(i)),
            });
            if i > 0 {
                if !self.handle_enabled.get(i).copied().unwrap_or(false) {
                    continue;
                }
                draw.in_handles.push(UiMarker {
                    center: self.normalized_to_screen(n.in_handle),
                    selected: self.selected == Some(Selection::InHandle(i)),
                });
            }
            if i + 1 < nodes.len() {
                if !self.handle_enabled.get(i).copied().unwrap_or(false) {
                    continue;
                }
                draw.out_handles.push(UiMarker {
                    center: self.normalized_to_screen(n.out_handle),
                    selected: self.selected == Some(Selection::OutHandle(i)),
                });
            }
        }

        let curve = model.build_curve_polyline(self.segments_per_span);
        draw.curve_polyline.reserve(curve.len());
        for p in &curve {
            draw.curve_polyline.push(self.normalized_to_screen(*p));
        }

        draw
    }

    // Returns true if the event was consumed.
    pub fn on_mouse_down(&mut self, model: &mut CurveEditorModel, e: MouseEvent) -> bool {
        self.sync_node_state(model);
        if e.button != MouseButton::Left && e.button != MouseButton::Right {
            return false;
        }

        let screen = Point2D::new(e.x, e.y);

        if let Some(node_index) = self.hit_test_anchor(model, screen) {
            let is_interior = node_index > 0 && node_index + 1 < model.node_count();
            if (e.button == MouseButton::Left || e.button == MouseButton::Right) && e.alt_down {
                if self.handle_enabled[node_index] {
                    self.remove_handles_for_anchor(model, node_index);
                    self.active_drag = DragTarget {
                        kind: DragTargetType::None,
                        node_index: 0,
                    };
                } else {
                    self.apply_handles_for_anchor(model, node_index);
                    self.active_drag = DragTarget {
                        kind: DragTargetType::Anchor,
                        node_index,
                    };
                }
                self.selected = Some(Selection::Anchor(node_index));
                self.handle_drag_mode = None;
                return true;
            }
            if (e.button == MouseButton::Right || e.shift_down) && is_interior {
                model.remove_node(node_index);
                self.sync_node_state(model);
                self.active_drag = DragTarget {
                    kind: DragTargetType::None,
                    node_index: 0,
                };
                self.selected = None;
                self.handle_drag_mode = None;
                return true;
            }

            if e.button == MouseButton::Left {
                self.active_drag = DragTarget {
                    kind: DragTargetType::Anchor,
                    node_index,
                };
                self.selected = Some(Selection::Anchor(node_index));
                self.handle_drag_mode = None;
                self.sync_linear_handles_if_disabled(model, node_index);
                return true;
            }
        }

        if e.button == MouseButton::Left {
            if let Some(node_index) = self.hit_test_handle(model, screen, true) {
                self.active_drag = DragTarget {
                    kind: DragTargetType::OutHandle,
                    node_index,
                };
                self.selected = Some(Selection::OutHandle(node_index));
                self.handle_drag_mode = Some(if e.ctrl_down {
                    HandleDragMode::Single
                } else {
                    HandleDragMode::LinkedHorizontal
                });
                self.capture_link_ratio(model, node_index);
                return true;
            }
            if let Some(node_index) = self.hit_test_handle(model, screen, false) {
                self.active_drag = DragTarget {
                    kind: DragTargetType::InHandle,
                    node_index,
                };
                self.selected = Some(Selection::InHandle(node_index));
                self.handle_drag_mode = Some(if e.ctrl_down {
                    HandleDragMode::Single
                } else {
                    HandleDragMode::LinkedHorizontal
                });
                self.capture_link_ratio(model, node_index);
                return true;
            }
            if let Some(new_node_x) = self.hit_test_curve(model, screen) {
                let inserted = model.add_node_on_curve(new_node_x);
                self.sync_node_state(model);
                self.remove_handles_for_anchor(model, inserted);
                self.active_drag = DragTarget {
                    kind: DragTargetType::Anchor,
                    node_index: inserted,
                };
                self.selected = Some(Selection::Anchor(inserted));
                self.handle_drag_mode = None;
                return true;
            }
        }

        self.active_drag = DragTarget {
            kind: DragTargetType::None,
            node_index: 0,
        };
        self.selected = None;
        self.handle_drag_mode = None;
        false
    }

    pub fn on_mouse_move(&mut self, model: &mut CurveEditorModel, e: MouseEvent) -> bool {
        self.sync_node_state(model);
        if self.active_drag.kind == DragTargetType::None {
            return false;
        }
        let mut normalized = self.screen_to_normalized(Point2D::new(e.x, e.y));
        if self.snap_enabled {
            normalized = self.snap_to_grid(normalized);
        }
        match self.active_drag.kind {
            DragTargetType::Anchor => {
                if e.alt_down
                    && self.active_drag.node_index > 0
                    && self.active_drag.node_index + 1 < model.node_count()
                {
                    // Alt+drag on anchor extends in/out handles in the drag direction.
                    let index = self.active_drag.node_index;
                    let anchor = model.get_node(index).anchor;
                    let dx = normalized.x - anchor.x;
                    let dy = normalized.y - anchor.y;
                    self.handle_enabled[index] = true;
                    model.move_out_handle(index, Point2D::new(anchor.x + dx, anchor.y + dy));
                    model.move_in_handle(index, Point2D::new(anchor.x - dx, anchor.y - dy));
                    return true;
                }
                model.move_anchor(self.active_drag.node_index, normalized);
                self.sync_linear_handles_if_disabled(model, self.active_drag.node_index);
                true
            }
            DragTargetType::InHandle => {
                let index = self.active_drag.node_index;
                self.handle_enabled[index] = true;
                match self
                    .handle_drag_mode
                    .unwrap_or(HandleDragMode::LinkedHorizontal)
                {
                    HandleDragMode::Single => {
                        model.move_in_handle(index, normalized);
                    }
                    HandleDragMode::LinkedHorizontal => {
                        let anchor = model.get_node(index).anchor;
                        let v = Point2D::new(normalized.x - anchor.x, normalized.y - anchor.y);
                        let out = Point2D::new(
                            anchor.x - v.x * self.linked_ratio,
                            anchor.y - v.y * self.linked_ratio,
                        );
                        model.move_in_handle(index, normalized);
                        model.move_out_handle(index, out);
                    }
                }
                true
            }
            DragTargetType::OutHandle => {
                let index = self.active_drag.node_index;
                self.handle_enabled[index] = true;
                match self
                    .handle_drag_mode
                    .unwrap_or(HandleDragMode::LinkedHorizontal)
                {
                    HandleDragMode::Single => {
                        model.move_out_handle(index, normalized);
                    }
                    HandleDragMode::LinkedHorizontal => {
                        let anchor = model.get_node(index).anchor;
                        let v = Point2D::new(normalized.x - anchor.x, normalized.y - anchor.y);
                        let inv_ratio = if self.linked_ratio.abs() > 1e-6 {
                            1.0 / self.linked_ratio
                        } else {
                            1.0
                        };
                        let inn =
                            Point2D::new(anchor.x - v.x * inv_ratio, anchor.y - v.y * inv_ratio);
                        model.move_out_handle(index, normalized);
                        model.move_in_handle(index, inn);
                    }
                }
                true
            }
            DragTargetType::None => false,
        }
    }

    pub fn on_mouse_up(&mut self, _e: MouseEvent) -> bool {
        let had_drag = self.active_drag.kind != DragTargetType::None;
        self.active_drag = DragTarget {
            kind: DragTargetType::None,
            node_index: 0,
        };
        self.handle_drag_mode = None;
        had_drag
    }

    pub fn delete_selected(&mut self, model: &mut CurveEditorModel) -> bool {
        self.sync_node_state(model);
        if let Some(Selection::Anchor(index)) = self.selected
            && model.remove_node(index)
        {
            self.sync_node_state(model);
            self.selected = None;
            self.active_drag = DragTarget {
                kind: DragTargetType::None,
                node_index: 0,
            };
            self.handle_drag_mode = None;
            return true;
        }
        false
    }

    pub fn screen_to_normalized(&self, screen: Point2D) -> Point2D {
        let nx = (screen.x - self.viewport.left) / self.viewport.width;
        let sy = (screen.y - self.viewport.top) / self.viewport.height;
        let ny = 1.0 - sy;
        Point2D::new(clamp01(nx), clamp01(ny))
    }

    pub fn normalized_to_screen(&self, normalized: Point2D) -> Point2D {
        let nx = clamp01(normalized.x);
        let ny = clamp01(normalized.y);
        let sx = self.viewport.left + nx * self.viewport.width;
        let sy = self.viewport.top + (1.0 - ny) * self.viewport.height;
        Point2D::new(sx, sy)
    }

    fn distance_squared(a: Point2D, b: Point2D) -> f32 {
        let dx = a.x - b.x;
        let dy = a.y - b.y;
        dx * dx + dy * dy
    }

    fn hit_test_anchor(&self, model: &CurveEditorModel, screen_pos: Point2D) -> Option<usize> {
        let r2 = self.picking_radius_px * self.picking_radius_px;
        let nodes = model.get_nodes();
        for (i, n) in nodes.iter().enumerate() {
            let p = self.normalized_to_screen(n.anchor);
            if Self::distance_squared(p, screen_pos) <= r2 {
                return Some(i);
            }
        }
        None
    }

    fn hit_test_handle(
        &self,
        model: &CurveEditorModel,
        screen_pos: Point2D,
        out_handle: bool,
    ) -> Option<usize> {
        let r2 = self.picking_radius_px * self.picking_radius_px;
        let nodes = model.get_nodes();
        for (i, n) in nodes.iter().enumerate() {
            if out_handle {
                if i + 1 >= nodes.len() {
                    continue;
                }
                let p = self.normalized_to_screen(n.out_handle);
                if Self::distance_squared(p, screen_pos) <= r2 {
                    return Some(i);
                }
            } else {
                if i == 0 {
                    continue;
                }
                let p = self.normalized_to_screen(n.in_handle);
                if Self::distance_squared(p, screen_pos) <= r2 {
                    return Some(i);
                }
            }
        }
        None
    }

    fn hit_test_curve(&self, model: &CurveEditorModel, screen_pos: Point2D) -> Option<f32> {
        let curve = model.build_curve_polyline(self.segments_per_span);
        if curve.len() < 2 {
            return None;
        }

        let threshold = self.picking_radius_px;
        let mut best_distance = threshold;
        let mut best_x: f32 = -1.0;

        for i in 0..curve.len() - 1 {
            let a = self.normalized_to_screen(curve[i]);
            let b = self.normalized_to_screen(curve[i + 1]);
            let ab = b - a;
            let ap = screen_pos - a;
            let ab_len2 = ab.x * ab.x + ab.y * ab.y;
            if ab_len2 <= 1e-8 {
                continue;
            }
            let mut t = (ap.x * ab.x + ap.y * ab.y) / ab_len2;
            t = t.clamp(0.0, 1.0);
            let q = a + ab * t;
            let d = Self::distance_squared(q, screen_pos).sqrt();
            if d <= best_distance {
                best_distance = d;
                let n0 = curve[i];
                let n1 = curve[i + 1];
                best_x = n0.x + (n1.x - n0.x) * t;
            }
        }

        if best_x < 0.0 {
            return None;
        }
        Some(clamp01(best_x))
    }

    fn snap_to_grid(&self, p: Point2D) -> Point2D {
        let major_step_x = 1.0 / self.grid_div_x.max(1) as f32;
        let major_step_y = 1.0 / self.grid_div_y.max(1) as f32;
        let fine_step_x = major_step_x / 9.0;
        let fine_step_y = major_step_y / 9.0;

        let snap_axis = |value: f32, step: f32, px_scale: f32| -> f32 {
            let snapped = (value / step).round() * step;
            let threshold_norm = self.snap_threshold_px / px_scale.max(1.0);
            if (snapped - value).abs() <= threshold_norm {
                clamp01(snapped)
            } else {
                value
            }
        };

        Point2D::new(
            snap_axis(p.x, fine_step_x, self.viewport.width),
            snap_axis(p.y, fine_step_y, self.viewport.height),
        )
    }

    fn capture_link_ratio(&mut self, model: &CurveEditorModel, index: usize) {
        if index >= model.node_count() {
            self.linked_ratio = 1.0;
            return;
        }
        let n = model.get_node(index);
        let in_v = Point2D::new(n.in_handle.x - n.anchor.x, n.in_handle.y - n.anchor.y);
        let out_v = Point2D::new(n.out_handle.x - n.anchor.x, n.out_handle.y - n.anchor.y);
        let in_len = (in_v.x * in_v.x + in_v.y * in_v.y).sqrt();
        let out_len = (out_v.x * out_v.x + out_v.y * out_v.y).sqrt();
        self.linked_ratio = if in_len > 1e-6 && out_len > 1e-6 {
            out_len / in_len
        } else {
            1.0
        };
    }

    fn sync_linear_handles_if_disabled(&mut self, model: &mut CurveEditorModel, index: usize) {
        if index < self.handle_enabled.len() && !self.handle_enabled[index] {
            self.remove_handles_for_anchor(model, index);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::curve_editor::CurveEditorModel;

    fn approx(a: f32, b: f32, eps: f32) -> bool {
        (a - b).abs() <= eps
    }

    fn make_adapter() -> CustomGraphEditorUiAdapter {
        let mut a = CustomGraphEditorUiAdapter::new();
        a.set_viewport(EditorViewport {
            left: 10.0,
            top: 20.0,
            width: 200.0,
            height: 100.0,
        });
        a
    }

    #[test]
    fn screen_normalized_round_trip() {
        let a = make_adapter();
        for &(nx, ny) in &[(0.0_f32, 0.0_f32), (0.25, 0.5), (0.7, 0.9), (1.0, 1.0)] {
            let s = a.normalized_to_screen(Point2D::new(nx, ny));
            let n = a.screen_to_normalized(s);
            assert!(
                approx(n.x, nx, 1e-4) && approx(n.y, ny, 1e-4),
                "round trip failed for ({nx},{ny}) -> screen {:?} -> normalized {:?}",
                s,
                n
            );
        }
    }

    #[test]
    fn normalized_to_screen_orientation() {
        let a = make_adapter();
        // Bottom-left of graph maps to bottom-left screen (y grows downward).
        let bl = a.normalized_to_screen(Point2D::new(0.0, 0.0));
        let tr = a.normalized_to_screen(Point2D::new(1.0, 1.0));
        assert!(approx(bl.x, 10.0, 1e-4));
        assert!(approx(bl.y, 120.0, 1e-4));
        assert!(approx(tr.x, 210.0, 1e-4));
        assert!(approx(tr.y, 20.0, 1e-4));
    }

    #[test]
    fn hit_test_anchor_finds_correct_node() {
        let a = make_adapter();
        let m = CurveEditorModel::new();
        let target = a.normalized_to_screen(Point2D::new(1.0, 1.0));
        let near = Point2D::new(target.x + 1.0, target.y - 2.0);
        let idx = a.hit_test_anchor(&m, near).expect("hit");
        assert_eq!(idx, 1);

        let far = Point2D::new(target.x + 50.0, target.y);
        assert!(a.hit_test_anchor(&m, far).is_none());
    }

    #[test]
    fn click_on_curve_inserts_node_and_drags_it() {
        let mut a = make_adapter();
        let mut m = CurveEditorModel::new();
        // The default curve is y=x; the screen midpoint of the curve is at
        // roughly the center of the viewport.
        let mid = a.normalized_to_screen(Point2D::new(0.5, 0.5));
        let consumed = a.on_mouse_down(
            &mut m,
            MouseEvent {
                x: mid.x,
                y: mid.y,
                button: MouseButton::Left,
                shift_down: false,
                alt_down: false,
                ctrl_down: false,
            },
        );
        assert!(consumed);
        assert_eq!(m.node_count(), 3);

        // Now drag to a new normalized position.
        let drag_to = a.normalized_to_screen(Point2D::new(0.6, 0.2));
        a.on_mouse_move(
            &mut m,
            MouseEvent {
                x: drag_to.x,
                y: drag_to.y,
                button: MouseButton::Left,
                shift_down: false,
                alt_down: false,
                ctrl_down: false,
            },
        );
        let n1 = m.get_node(1).anchor;
        assert!(approx(n1.x, 0.6, 1e-3));
        assert!(approx(n1.y, 0.2, 1e-3));

        a.on_mouse_up(MouseEvent {
            x: drag_to.x,
            y: drag_to.y,
            button: MouseButton::Left,
            shift_down: false,
            alt_down: false,
            ctrl_down: false,
        });
    }

    #[test]
    fn shift_left_on_interior_anchor_removes_it() {
        let mut a = make_adapter();
        let mut m = CurveEditorModel::new();
        m.add_node_on_curve(0.5);
        assert_eq!(m.node_count(), 3);
        let target = a.normalized_to_screen(m.get_node(1).anchor);
        let consumed = a.on_mouse_down(
            &mut m,
            MouseEvent {
                x: target.x,
                y: target.y,
                button: MouseButton::Left,
                shift_down: true,
                alt_down: false,
                ctrl_down: false,
            },
        );
        assert!(consumed);
        assert_eq!(m.node_count(), 2);
    }

    #[test]
    fn shift_left_on_end_anchor_does_not_remove() {
        let mut a = make_adapter();
        let mut m = CurveEditorModel::new();
        let target = a.normalized_to_screen(Point2D::new(0.0, 0.0));
        a.on_mouse_down(
            &mut m,
            MouseEvent {
                x: target.x,
                y: target.y,
                button: MouseButton::Left,
                shift_down: true,
                alt_down: false,
                ctrl_down: false,
            },
        );
        assert_eq!(m.node_count(), 2);
    }

    #[test]
    fn alt_click_anchor_toggles_handles() {
        let mut a = make_adapter();
        let mut m = CurveEditorModel::new();
        m.add_node_on_curve(0.5);
        let p = a.normalized_to_screen(m.get_node(1).anchor);

        a.on_mouse_down(
            &mut m,
            MouseEvent {
                x: p.x,
                y: p.y,
                button: MouseButton::Left,
                shift_down: false,
                alt_down: true,
                ctrl_down: false,
            },
        );
        let n = m.get_node(1);
        assert!(
            !approx(n.in_handle.x, n.anchor.x, 1e-4) || !approx(n.out_handle.x, n.anchor.x, 1e-4)
        );

        a.on_mouse_down(
            &mut m,
            MouseEvent {
                x: p.x,
                y: p.y,
                button: MouseButton::Left,
                shift_down: false,
                alt_down: true,
                ctrl_down: false,
            },
        );
        let n = m.get_node(1);
        let prev = m.get_node(0).anchor;
        let next = m.get_node(2).anchor;
        assert!(approx(
            n.in_handle.x,
            n.anchor.x - (n.anchor.x - prev.x) / 3.0,
            1e-4
        ));
        assert!(approx(
            n.out_handle.x,
            n.anchor.x + (next.x - n.anchor.x) / 3.0,
            1e-4
        ));
    }

    #[test]
    fn ctrl_drag_handle_edits_single_handle() {
        let mut a = make_adapter();
        let mut m = CurveEditorModel::new();
        m.add_node_on_curve(0.5);
        let p = a.normalized_to_screen(m.get_node(1).anchor);
        a.on_mouse_down(
            &mut m,
            MouseEvent {
                x: p.x,
                y: p.y,
                button: MouseButton::Left,
                shift_down: false,
                alt_down: true,
                ctrl_down: false,
            },
        );
        let out_screen = a.normalized_to_screen(m.get_node(1).out_handle);
        let before_in_x = m.get_node(1).in_handle.x;
        a.on_mouse_down(
            &mut m,
            MouseEvent {
                x: out_screen.x,
                y: out_screen.y,
                button: MouseButton::Left,
                shift_down: false,
                alt_down: false,
                ctrl_down: true,
            },
        );
        let drag = a.normalized_to_screen(Point2D::new(0.8, 0.9));
        a.on_mouse_move(
            &mut m,
            MouseEvent {
                x: drag.x,
                y: drag.y,
                button: MouseButton::Left,
                shift_down: false,
                alt_down: false,
                ctrl_down: true,
            },
        );
        assert!(!approx(
            m.get_node(1).out_handle.x,
            m.get_node(1).anchor.x,
            1e-4
        ));
        assert!(approx(m.get_node(1).in_handle.x, before_in_x, 1e-4));
    }
}
