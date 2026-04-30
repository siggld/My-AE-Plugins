#![allow(clippy::drop_non_drop, clippy::question_mark)]

mod curve_editor;
mod ui_adapter;

use ae::pf::*;
use after_effects as ae;
use std::env;

use crate::curve_editor::CurveEditorModel;
use crate::ui_adapter::{
    CustomGraphEditorUiAdapter, EditorViewport, MouseButton, MouseEvent, UiDrawData, UiMarker,
};

const UI_BOX_WIDTH: u16 = 420;
const UI_BOX_HEIGHT: u16 = 430;
const GRID_SIZE_SMALL: f32 = 280.0;
const GRID_SIZE_LARGE: f32 = 350.0;
const TOOLBAR_HEIGHT: f32 = 40.0;
const TOOLBAR_MARGIN: f32 = 6.0;
const TOOL_BUTTON_SIZE: f32 = 24.0;

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
enum Params {
    EnableGuiTest,
    GraphEditor,
}

struct Plugin {
    model: CurveEditorModel,
    adapter: CustomGraphEditorUiAdapter,
    magnet_snap: bool,
    grid_size_px: f32,
}

impl Default for Plugin {
    fn default() -> Self {
        Self {
            model: CurveEditorModel::new(),
            adapter: CustomGraphEditorUiAdapter::new(),
            magnet_snap: false,
            grid_size_px: GRID_SIZE_LARGE,
        }
    }
}

ae::define_effect!(Plugin, (), Params);

const PLUGIN_DESCRIPTION: &str =
    "Provides a reusable test plugin for validating custom GUI behaviors in AE.";

impl AdobePluginGlobal for Plugin {
    fn params_setup(
        &self,
        params: &mut ae::Parameters<Params>,
        in_data: InData,
        _: OutData,
    ) -> Result<(), Error> {
        params.add(
            Params::EnableGuiTest,
            "Enable GUI Test",
            CheckBoxDef::setup(|d| {
                d.set_default(true);
            }),
        )?;

        params.add_customized(
            Params::GraphEditor,
            "Graph Editor",
            NullDef::new(),
            |param: &mut ae::ParamDef| {
                param.set_flags(ParamFlag::SUPERVISE);
                param.set_ui_flags(ParamUIFlags::CONTROL | ParamUIFlags::DO_NOT_ERASE_CONTROL);
                param.set_ui_width(UI_BOX_WIDTH);
                param.set_ui_height(UI_BOX_HEIGHT);
                -1
            },
        )?;

        in_data
            .interact()
            .register_ui(CustomUIInfo::new().events(ae::CustomEventFlags::EFFECT))?;

        Ok(())
    }

    fn handle_command(
        &mut self,
        cmd: ae::Command,
        in_data: InData,
        mut out_data: OutData,
        _params: &mut ae::Parameters<Params>,
    ) -> Result<(), ae::Error> {
        match cmd {
            ae::Command::About => {
                out_data.set_return_msg(
                    format!(
                        "TKG_CustomGUITest - {version}\r\r{PLUGIN_DESCRIPTION}\rCopyright (c) 2026-{build_year} Aodaruma",
                        version = env!("CARGO_PKG_VERSION"),
                        build_year = env!("BUILD_YEAR")
                    )
                    .as_str(),
                );
            }
            ae::Command::GlobalSetup => {
                out_data.set_out_flag2(ae::OutFlags2::SupportsThreadedRendering, true);
                out_data.set_out_flag2(ae::OutFlags2::SupportsSmartRender, true);
                out_data.set_out_flag2(ae::OutFlags2::ParamGroupStartCollapsedFlag, true);
                out_data.set_out_flag(ae::OutFlags::SendUpdateParamsUi, true);
                out_data.set_out_flag(ae::OutFlags::CustomUi, true);
            }
            ae::Command::SmartPreRender { mut extra } => {
                let req = extra.output_request();
                if let Ok(in_result) = extra.callbacks().checkout_layer(
                    0,
                    0,
                    &req,
                    in_data.current_time(),
                    in_data.time_step(),
                    in_data.time_scale(),
                ) {
                    let _ = extra.union_result_rect(in_result.result_rect.into());
                    let _ = extra.union_max_result_rect(in_result.max_result_rect.into());
                } else {
                    return Err(Error::InterruptCancel);
                }
            }
            ae::Command::SmartRender { extra } => {
                let cb = extra.callbacks();
                let in_layer_opt = cb.checkout_layer_pixels(0)?;
                let out_layer_opt = cb.checkout_output()?;
                if let (Some(in_layer), Some(mut out_layer)) = (in_layer_opt, out_layer_opt) {
                    out_layer.copy_from(&in_layer, None, None)?;
                }
                cb.checkin_layer_pixels(0)?;
            }
            ae::Command::Render {
                in_layer,
                mut out_layer,
            } => {
                out_layer.copy_from(&in_layer, None, None)?;
            }
            ae::Command::Event { mut extra } => match extra.event() {
                ae::Event::Draw(_) => {
                    self.handle_draw(&mut extra)?;
                }
                ae::Event::Click(_) => {
                    self.handle_click(&mut extra)?;
                }
                ae::Event::Drag(_) => {
                    self.handle_drag(&mut extra)?;
                }
                ae::Event::Keydown(key_event) => {
                    self.handle_keydown(&mut extra, key_event)?;
                }
                _ => {}
            },
            ae::Command::UpdateParamsUi => {}
            _ => {}
        }
        Ok(())
    }
}

impl Plugin {
    fn fixed_aspect_viewport(frame: ae::Rect, preferred_side: f32) -> EditorViewport {
        let area_w = (frame.right - frame.left).max(1) as f32;
        let area_h = (frame.bottom - frame.top).max(1) as f32;
        let avail_h = (area_h - TOOLBAR_HEIGHT - TOOLBAR_MARGIN * 2.0).max(1.0);
        let side = preferred_side.min(area_w).min(avail_h).max(1.0);
        EditorViewport {
            left: frame.left as f32 + (area_w - side) * 0.5,
            top: frame.top as f32 + TOOLBAR_MARGIN,
            width: side,
            height: side,
        }
    }

    fn toolbar_button_rects(
        _frame: ae::Rect,
        viewport: &EditorViewport,
    ) -> [(f32, f32, f32, f32); 3] {
        let toolbar_top = viewport.top + viewport.height + TOOLBAR_MARGIN;
        let total_w = TOOL_BUTTON_SIZE * 3.0 + TOOLBAR_MARGIN * 2.0;
        let start_x = viewport.left + (viewport.width - total_w) * 0.5;
        [
            (start_x, toolbar_top, TOOL_BUTTON_SIZE, TOOL_BUTTON_SIZE),
            (
                start_x + TOOL_BUTTON_SIZE + TOOLBAR_MARGIN,
                toolbar_top,
                TOOL_BUTTON_SIZE,
                TOOL_BUTTON_SIZE,
            ),
            (
                start_x + (TOOL_BUTTON_SIZE + TOOLBAR_MARGIN) * 2.0,
                toolbar_top,
                TOOL_BUTTON_SIZE,
                TOOL_BUTTON_SIZE,
            ),
        ]
    }

    fn hit_button(x: f32, y: f32, rect: (f32, f32, f32, f32)) -> bool {
        x >= rect.0 && x <= rect.0 + rect.2 && y >= rect.1 && y <= rect.1 + rect.3
    }

    fn sync_viewport(&mut self, extra: &ae::EventExtra) {
        let frame = extra.current_frame();
        let viewport = Self::fixed_aspect_viewport(frame, self.grid_size_px);
        self.adapter.set_viewport(viewport);
        self.adapter.set_snap_enabled(self.magnet_snap);
    }

    fn handle_click(&mut self, extra: &mut ae::EventExtra) -> Result<(), ae::Error> {
        if extra.effect_area() != ae::EffectArea::Control {
            return Ok(());
        }

        self.sync_viewport(extra);
        extra.set_send_drag(true);

        let p = extra.screen_point();
        let mouse_x = p.h as f32;
        let mouse_y = p.v as f32;
        let frame = extra.current_frame();
        let viewport = Self::fixed_aspect_viewport(frame, self.grid_size_px);
        let [mag_rect, small_rect, large_rect] = Self::toolbar_button_rects(frame, &viewport);
        if Self::hit_button(mouse_x, mouse_y, mag_rect) {
            self.magnet_snap = !self.magnet_snap;
            self.adapter.set_snap_enabled(self.magnet_snap);
            extra.set_event_out_flags(
                ae::EventOutFlags::HANDLED_EVENT
                    | ae::EventOutFlags::NEVER_UPDATE
                    | ae::EventOutFlags::UPDATE_NOW,
            );
            return Ok(());
        }
        if Self::hit_button(mouse_x, mouse_y, small_rect) {
            self.grid_size_px = GRID_SIZE_SMALL;
            self.sync_viewport(extra);
            extra.set_event_out_flags(
                ae::EventOutFlags::HANDLED_EVENT
                    | ae::EventOutFlags::NEVER_UPDATE
                    | ae::EventOutFlags::UPDATE_NOW,
            );
            return Ok(());
        }
        if Self::hit_button(mouse_x, mouse_y, large_rect) {
            self.grid_size_px = GRID_SIZE_LARGE;
            self.sync_viewport(extra);
            extra.set_event_out_flags(
                ae::EventOutFlags::HANDLED_EVENT
                    | ae::EventOutFlags::NEVER_UPDATE
                    | ae::EventOutFlags::UPDATE_NOW,
            );
            return Ok(());
        }
        let modifiers = extra.modifiers();
        let shift_down = modifiers.contains(Modifiers::SHIFT_KEY);
        let alt_down = modifiers.contains(Modifiers::OPT_ALT_KEY);
        let ctrl_down = modifiers.contains(Modifiers::CMD_CTRL_KEY);

        self.adapter.on_mouse_down(
            &mut self.model,
            MouseEvent {
                x: mouse_x,
                y: mouse_y,
                button: MouseButton::Left,
                shift_down,
                alt_down,
                ctrl_down,
            },
        );

        extra.set_event_out_flags(
            ae::EventOutFlags::HANDLED_EVENT
                | ae::EventOutFlags::NEVER_UPDATE
                | ae::EventOutFlags::UPDATE_NOW,
        );
        Ok(())
    }

    fn handle_drag(&mut self, extra: &mut ae::EventExtra) -> Result<(), ae::Error> {
        if extra.effect_area() != ae::EffectArea::Control {
            return Ok(());
        }

        self.sync_viewport(extra);

        let p = extra.screen_point();
        let modifiers = extra.modifiers();
        let shift_down = modifiers.contains(Modifiers::SHIFT_KEY);
        let alt_down = modifiers.contains(Modifiers::OPT_ALT_KEY);
        let ctrl_down = modifiers.contains(Modifiers::CMD_CTRL_KEY);
        let last = extra.last_time();
        let mouse = MouseEvent {
            x: p.h as f32,
            y: p.v as f32,
            button: MouseButton::Left,
            shift_down,
            alt_down,
            ctrl_down,
        };

        if last {
            self.adapter.on_mouse_up(mouse);
        } else {
            self.adapter.on_mouse_move(&mut self.model, mouse);
        }

        extra.set_event_out_flags(
            ae::EventOutFlags::HANDLED_EVENT
                | ae::EventOutFlags::NEVER_UPDATE
                | ae::EventOutFlags::UPDATE_NOW,
        );
        Ok(())
    }

    fn handle_keydown(
        &mut self,
        extra: &mut ae::EventExtra,
        key_event: ae::KeyDownEventInfo,
    ) -> Result<(), ae::Error> {
        let keycode = key_event.as_ref().keycode as u16;
        if keycode == ae::sys::PF_ControlCode_Delete as u16 {
            let _ = self.adapter.delete_selected(&mut self.model);
            // Always consume Delete to avoid AE removing the whole effect.
            extra.set_event_out_flags(
                ae::EventOutFlags::HANDLED_EVENT
                    | ae::EventOutFlags::NEVER_UPDATE
                    | ae::EventOutFlags::UPDATE_NOW,
            );
        }
        Ok(())
    }

    fn handle_draw(&mut self, extra: &mut ae::EventExtra) -> Result<(), ae::Error> {
        if extra.effect_area() != ae::EffectArea::Control {
            return Ok(());
        }

        self.sync_viewport(extra);

        let drawbot = extra.context_handle().drawing_reference()?;
        let supplier = drawbot.supplier()?;
        let surface = drawbot.surface()?;

        let frame = extra.current_frame();
        let viewport = Self::fixed_aspect_viewport(frame, self.grid_size_px);
        let bg_rect = ae::drawbot::RectF32 {
            left: frame.left as f32 + 0.5,
            top: frame.top as f32 + 0.5,
            width: (frame.right - frame.left).max(1) as f32,
            height: (frame.bottom - frame.top).max(1) as f32,
        };
        let editor_rect = ae::drawbot::RectF32 {
            left: viewport.left + 0.5,
            top: viewport.top + 0.5,
            width: viewport.width,
            height: viewport.height,
        };

        let bg = ae::drawbot::ColorRgba {
            red: 0.12,
            green: 0.12,
            blue: 0.12,
            alpha: 1.0,
        };
        let grid_color = ae::drawbot::ColorRgba {
            red: 0.28,
            green: 0.28,
            blue: 0.28,
            alpha: 1.0,
        };
        let handle_line_color = ae::drawbot::ColorRgba {
            red: 0.55,
            green: 0.55,
            blue: 0.55,
            alpha: 1.0,
        };
        let curve_color = ae::drawbot::ColorRgba {
            red: 0.95,
            green: 0.8,
            blue: 0.2,
            alpha: 1.0,
        };
        let anchor_color = ae::drawbot::ColorRgba {
            red: 0.92,
            green: 0.35,
            blue: 0.35,
            alpha: 1.0,
        };
        let in_handle_color = ae::drawbot::ColorRgba {
            red: 0.4,
            green: 0.65,
            blue: 0.95,
            alpha: 1.0,
        };
        let out_handle_color = ae::drawbot::ColorRgba {
            red: 0.45,
            green: 0.85,
            blue: 0.55,
            alpha: 1.0,
        };

        surface.paint_rect(&bg, &bg_rect)?;
        let editor_panel = ae::drawbot::ColorRgba {
            red: 0.1,
            green: 0.1,
            blue: 0.1,
            alpha: 1.0,
        };
        surface.paint_rect(&editor_panel, &editor_rect)?;

        let draw: UiDrawData = self.adapter.build_draw_data(&self.model);

        let grid_pen = supplier.new_pen(&grid_color, 1.0)?;
        let border_pen = supplier.new_pen(&grid_color, 2.0)?;
        let fine_grid_color = ae::drawbot::ColorRgba {
            red: 0.2,
            green: 0.2,
            blue: 0.2,
            alpha: 1.0,
        };
        let fine_grid_pen = supplier.new_pen(&fine_grid_color, 0.5)?;
        // Minor grid: split each major cell into 9 parts.
        let major_div = 4.0_f32;
        for i in 0..=(major_div as i32 * 9) {
            if i % 9 == 0 {
                continue;
            }
            let t = i as f32 / (major_div * 9.0);
            let x = viewport.left + viewport.width * t;
            let y = viewport.top + viewport.height * t;
            let mut v = supplier.new_path()?;
            v.move_to(x, viewport.top)?;
            v.line_to(x, viewport.top + viewport.height)?;
            surface.stroke_path(&fine_grid_pen, &v)?;
            let mut h = supplier.new_path()?;
            h.move_to(viewport.left, y)?;
            h.line_to(viewport.left + viewport.width, y)?;
            surface.stroke_path(&fine_grid_pen, &h)?;
        }

        for line in &draw.grid_lines {
            let mut path = supplier.new_path()?;
            path.move_to(line.a.x, line.a.y)?;
            path.line_to(line.b.x, line.b.y)?;
            let is_outer = (line.a.x - viewport.left).abs() < 0.5
                || (line.a.x - (viewport.left + viewport.width)).abs() < 0.5
                || (line.a.y - viewport.top).abs() < 0.5
                || (line.a.y - (viewport.top + viewport.height)).abs() < 0.5;
            surface.stroke_path(if is_outer { &border_pen } else { &grid_pen }, &path)?;
        }

        let handle_pen = supplier.new_pen(&handle_line_color, 1.0)?;
        for line in &draw.handle_lines {
            let mut path = supplier.new_path()?;
            path.move_to(line.a.x, line.a.y)?;
            path.line_to(line.b.x, line.b.y)?;
            surface.stroke_path(&handle_pen, &path)?;
        }

        if draw.curve_polyline.len() >= 2 {
            let curve_pen = supplier.new_pen(&curve_color, 2.0)?;
            let mut path = supplier.new_path()?;
            let first = draw.curve_polyline[0];
            path.move_to(first.x, first.y)?;
            for p in &draw.curve_polyline[1..] {
                path.line_to(p.x, p.y)?;
            }
            surface.stroke_path(&curve_pen, &path)?;
        }

        Self::draw_marker_squares(&supplier, &surface, &draw.in_handles, &in_handle_color, 6.0)?;
        Self::draw_marker_squares(
            &supplier,
            &surface,
            &draw.out_handles,
            &out_handle_color,
            6.0,
        )?;
        Self::draw_marker_squares(&supplier, &surface, &draw.anchors, &anchor_color, 8.0)?;

        let [mag_rect, small_rect, large_rect] = Self::toolbar_button_rects(frame, &viewport);
        let button_border = ae::drawbot::ColorRgba {
            red: 0.7,
            green: 0.7,
            blue: 0.7,
            alpha: 1.0,
        };
        let button_active = ae::drawbot::ColorRgba {
            red: 0.35,
            green: 0.5,
            blue: 0.8,
            alpha: 1.0,
        };
        let button_idle = ae::drawbot::ColorRgba {
            red: 0.18,
            green: 0.18,
            blue: 0.18,
            alpha: 1.0,
        };
        Self::draw_toolbar_button(
            &supplier,
            &surface,
            mag_rect,
            if self.magnet_snap {
                &button_active
            } else {
                &button_idle
            },
            &button_border,
        )?;
        Self::draw_toolbar_button(
            &supplier,
            &surface,
            small_rect,
            if (self.grid_size_px - GRID_SIZE_SMALL).abs() < 0.5 {
                &button_active
            } else {
                &button_idle
            },
            &button_border,
        )?;
        Self::draw_toolbar_button(
            &supplier,
            &surface,
            large_rect,
            if (self.grid_size_px - GRID_SIZE_LARGE).abs() < 0.5 {
                &button_active
            } else {
                &button_idle
            },
            &button_border,
        )?;
        Self::draw_magnet_icon(&supplier, &surface, mag_rect, &button_border)?;
        Self::draw_square_icon(&surface, small_rect, 0.4, &button_border)?;
        Self::draw_square_icon(&surface, large_rect, 0.65, &button_border)?;

        extra.set_event_out_flags(ae::EventOutFlags::HANDLED_EVENT);
        Ok(())
    }

    fn draw_toolbar_button(
        supplier: &ae::drawbot::Supplier,
        surface: &ae::drawbot::Surface,
        rect: (f32, f32, f32, f32),
        fill: &ae::drawbot::ColorRgba,
        border: &ae::drawbot::ColorRgba,
    ) -> Result<(), ae::Error> {
        surface.paint_rect(
            fill,
            &ae::drawbot::RectF32 {
                left: rect.0,
                top: rect.1,
                width: rect.2,
                height: rect.3,
            },
        )?;
        let pen = supplier.new_pen(border, 1.0)?;
        let mut path = supplier.new_path()?;
        path.move_to(rect.0, rect.1)?;
        path.line_to(rect.0 + rect.2, rect.1)?;
        path.line_to(rect.0 + rect.2, rect.1 + rect.3)?;
        path.line_to(rect.0, rect.1 + rect.3)?;
        path.line_to(rect.0, rect.1)?;
        surface.stroke_path(&pen, &path)?;
        Ok(())
    }

    fn draw_square_icon(
        surface: &ae::drawbot::Surface,
        rect: (f32, f32, f32, f32),
        scale: f32,
        color: &ae::drawbot::ColorRgba,
    ) -> Result<(), ae::Error> {
        let s = rect.2.min(rect.3) * scale;
        let x = rect.0 + (rect.2 - s) * 0.5;
        let y = rect.1 + (rect.3 - s) * 0.5;
        surface.paint_rect(
            color,
            &ae::drawbot::RectF32 {
                left: x,
                top: y,
                width: s,
                height: s,
            },
        )?;
        Ok(())
    }

    fn draw_magnet_icon(
        supplier: &ae::drawbot::Supplier,
        surface: &ae::drawbot::Surface,
        rect: (f32, f32, f32, f32),
        color: &ae::drawbot::ColorRgba,
    ) -> Result<(), ae::Error> {
        let pen = supplier.new_pen(color, 2.0)?;
        let cx = rect.0 + rect.2 * 0.5;
        let cy = rect.1 + rect.3 * 0.5;
        let w = rect.2 * 0.42;
        let h = rect.3 * 0.45;
        let mut path = supplier.new_path()?;
        path.move_to(cx - w * 0.5, cy - h * 0.5)?;
        path.line_to(cx - w * 0.5, cy + h * 0.2)?;
        path.add_arc(
            &ae::drawbot::PointF32 {
                x: cx,
                y: cy + h * 0.2,
            },
            w * 0.5,
            180.0,
            360.0,
        )?;
        path.line_to(cx + w * 0.5, cy - h * 0.5)?;
        surface.stroke_path(&pen, &path)?;
        Ok(())
    }

    fn draw_marker_squares(
        supplier: &ae::drawbot::Supplier,
        surface: &ae::drawbot::Surface,
        markers: &[UiMarker],
        color: &ae::drawbot::ColorRgba,
        size: f32,
    ) -> Result<(), ae::Error> {
        let pen = supplier.new_pen(color, 1.0)?;
        let brush = supplier.new_brush(color)?;
        let half = size * 0.5;
        for marker in markers {
            let left = marker.center.x - half;
            let top = marker.center.y - half;
            let rect = ae::drawbot::RectF32 {
                left,
                top,
                width: size,
                height: size,
            };
            if marker.selected {
                surface.paint_rect(color, &rect)?;
            }
            let mut path = supplier.new_path()?;
            path.move_to(left, top)?;
            path.line_to(left + size, top)?;
            path.line_to(left + size, top + size)?;
            path.line_to(left, top + size)?;
            path.line_to(left, top)?;
            surface.stroke_path(&pen, &path)?;
            if marker.selected {
                let mut center_dot = supplier.new_path()?;
                center_dot.add_arc(
                    &ae::drawbot::PointF32 {
                        x: marker.center.x,
                        y: marker.center.y,
                    },
                    1.0,
                    0.0,
                    360.0,
                )?;
                surface.fill_path(&brush, &center_dot, ae::drawbot::FillType::Winding)?;
            }
        }
        Ok(())
    }
}
