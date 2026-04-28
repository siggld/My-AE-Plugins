#![allow(clippy::drop_non_drop, clippy::question_mark)]

mod curve_editor;
mod ui_adapter;

use ae::pf::*;
use after_effects as ae;
use std::env;

use crate::curve_editor::CurveEditorModel;
use crate::ui_adapter::{
    CustomGraphEditorUiAdapter, EditorViewport, MouseButton, MouseEvent, UiDrawData,
};

const UI_BOX_WIDTH: u16 = 320;
const UI_BOX_HEIGHT: u16 = 180;

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
enum Params {
    EnableGuiTest,
    GraphEditor,
}

#[derive(Default)]
struct Plugin {
    model: CurveEditorModel,
    adapter: CustomGraphEditorUiAdapter,
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
                _ => {}
            },
            ae::Command::UpdateParamsUi => {}
            _ => {}
        }
        Ok(())
    }
}

impl Plugin {
    fn sync_viewport(&mut self, extra: &ae::EventExtra) {
        let frame = extra.current_frame();
        let viewport = EditorViewport {
            left: frame.left as f32,
            top: frame.top as f32,
            width: (frame.right - frame.left).max(1) as f32,
            height: (frame.bottom - frame.top).max(1) as f32,
        };
        self.adapter.set_viewport(viewport);
    }

    fn handle_click(&mut self, extra: &mut ae::EventExtra) -> Result<(), ae::Error> {
        if extra.effect_area() != ae::EffectArea::Control {
            return Ok(());
        }

        self.sync_viewport(extra);
        extra.set_send_drag(true);

        let p = extra.screen_point();
        let shift_down = extra.modifiers().contains(Modifiers::SHIFT_KEY);

        self.adapter.on_mouse_down(
            &mut self.model,
            MouseEvent {
                x: p.h as f32,
                y: p.v as f32,
                button: MouseButton::Left,
                shift_down,
            },
        );

        extra.set_event_out_flags(
            ae::EventOutFlags::HANDLED_EVENT
                | ae::EventOutFlags::ALWAYS_UPDATE
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
        let shift_down = extra.modifiers().contains(Modifiers::SHIFT_KEY);
        let last = extra.last_time();
        let mouse = MouseEvent {
            x: p.h as f32,
            y: p.v as f32,
            button: MouseButton::Left,
            shift_down,
        };

        if last {
            self.adapter.on_mouse_up(mouse);
        } else {
            self.adapter.on_mouse_move(&mut self.model, mouse);
        }

        extra.set_event_out_flags(
            ae::EventOutFlags::HANDLED_EVENT
                | ae::EventOutFlags::ALWAYS_UPDATE
                | ae::EventOutFlags::UPDATE_NOW,
        );
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
        let bg_rect = ae::drawbot::RectF32 {
            left: frame.left as f32 + 0.5,
            top: frame.top as f32 + 0.5,
            width: (frame.right - frame.left).max(1) as f32,
            height: (frame.bottom - frame.top).max(1) as f32,
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

        let draw: UiDrawData = self.adapter.build_draw_data(&self.model);

        let grid_pen = supplier.new_pen(&grid_color, 1.0)?;
        for line in &draw.grid_lines {
            let mut path = supplier.new_path()?;
            path.move_to(line.a.x, line.a.y)?;
            path.line_to(line.b.x, line.b.y)?;
            surface.stroke_path(&grid_pen, &path)?;
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

        let in_handle_brush = supplier.new_brush(&in_handle_color)?;
        for p in &draw.in_handles {
            let mut shape = supplier.new_path()?;
            shape.add_arc(&ae::drawbot::PointF32 { x: p.x, y: p.y }, 3.0, 0.0, 360.0)?;
            surface.fill_path(&in_handle_brush, &shape, ae::drawbot::FillType::Winding)?;
        }

        let out_handle_brush = supplier.new_brush(&out_handle_color)?;
        for p in &draw.out_handles {
            let mut shape = supplier.new_path()?;
            shape.add_arc(&ae::drawbot::PointF32 { x: p.x, y: p.y }, 3.0, 0.0, 360.0)?;
            surface.fill_path(&out_handle_brush, &shape, ae::drawbot::FillType::Winding)?;
        }

        let anchor_brush = supplier.new_brush(&anchor_color)?;
        for p in &draw.anchors {
            let mut shape = supplier.new_path()?;
            shape.add_arc(&ae::drawbot::PointF32 { x: p.x, y: p.y }, 4.0, 0.0, 360.0)?;
            surface.fill_path(&anchor_brush, &shape, ae::drawbot::FillType::Winding)?;
        }

        extra.set_event_out_flags(ae::EventOutFlags::HANDLED_EVENT);
        Ok(())
    }
}
