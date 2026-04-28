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
        }
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
                draw.in_handles.push(UiMarker {
                    center: self.normalized_to_screen(n.in_handle),
                    selected: self.selected == Some(Selection::InHandle(i)),
                });
            }
            if i + 1 < nodes.len() {
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
        if e.button != MouseButton::Left && e.button != MouseButton::Right {
            return false;
        }

        let screen = Point2D::new(e.x, e.y);

        if let Some(node_index) = self.hit_test_anchor(model, screen) {
            let is_interior = node_index > 0 && node_index + 1 < model.node_count();
            if (e.button == MouseButton::Right || e.shift_down) && is_interior {
                model.remove_node(node_index);
                self.active_drag = DragTarget {
                    kind: DragTargetType::None,
                    node_index: 0,
                };
                self.selected = None;
                return true;
            }

            if e.button == MouseButton::Left {
                self.active_drag = DragTarget {
                    kind: DragTargetType::Anchor,
                    node_index,
                };
                self.selected = Some(Selection::Anchor(node_index));
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
                return true;
            }
            if let Some(node_index) = self.hit_test_handle(model, screen, false) {
                self.active_drag = DragTarget {
                    kind: DragTargetType::InHandle,
                    node_index,
                };
                self.selected = Some(Selection::InHandle(node_index));
                return true;
            }
            if let Some(new_node_x) = self.hit_test_curve(model, screen) {
                let inserted = model.add_node_on_curve(new_node_x);
                self.active_drag = DragTarget {
                    kind: DragTargetType::Anchor,
                    node_index: inserted,
                };
                self.selected = Some(Selection::Anchor(inserted));
                return true;
            }
        }

        self.active_drag = DragTarget {
            kind: DragTargetType::None,
            node_index: 0,
        };
        self.selected = None;
        false
    }

    pub fn on_mouse_move(&mut self, model: &mut CurveEditorModel, e: MouseEvent) -> bool {
        if self.active_drag.kind == DragTargetType::None {
            return false;
        }
        let normalized = self.screen_to_normalized(Point2D::new(e.x, e.y));
        match self.active_drag.kind {
            DragTargetType::Anchor => {
                model.move_anchor(self.active_drag.node_index, normalized);
                true
            }
            DragTargetType::InHandle => {
                model.move_in_handle(self.active_drag.node_index, normalized);
                true
            }
            DragTargetType::OutHandle => {
                model.move_out_handle(self.active_drag.node_index, normalized);
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
        had_drag
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
            },
        );
        assert_eq!(m.node_count(), 2);
    }
}
