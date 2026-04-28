#![allow(clippy::drop_non_drop, clippy::question_mark)]

use ae::pf::*;
use after_effects as ae;
use std::env;

const UI_BOX_WIDTH: u16 = 320;
const UI_BOX_HEIGHT: u16 = 180;

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
enum Params {
    EnableGuiTest,
    AnchorPoint,
    GraphEditor,
}

#[derive(Default)]
struct Plugin {}

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

        params.add_with_flags(
            Params::AnchorPoint,
            "Anchor (debug)",
            PointDef::setup(|d| {
                d.set_default_x(0.5);
                d.set_default_y(0.5);
            }),
            ParamFlag::SUPERVISE,
            ParamUIFlags::NONE,
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
        params: &mut ae::Parameters<Params>,
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
                    Self::draw_graph_editor(params, &mut extra)?;
                }
                ae::Event::Click(_) => {
                    extra.set_send_drag(true);
                    extra.set_event_out_flags(ae::EventOutFlags::HANDLED_EVENT);
                }
                ae::Event::Drag(_) => {
                    Self::update_anchor_from_mouse(params, &mut extra)?;
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
    fn update_anchor_from_mouse(
        params: &mut ae::Parameters<Params>,
        extra: &mut ae::EventExtra,
    ) -> Result<(), ae::Error> {
        if extra.effect_area() != ae::EffectArea::Control {
            return Ok(());
        }
        let frame = extra.current_frame();
        let w = (frame.right - frame.left).max(1) as f32;
        let h = (frame.bottom - frame.top).max(1) as f32;
        let p = extra.screen_point();
        let nx = ((p.h - frame.left) as f32 / w).clamp(0.0, 1.0);
        let ny = (1.0 - ((p.v - frame.top) as f32 / h)).clamp(0.0, 1.0);
        let mut anchor = params.get_mut(Params::AnchorPoint)?;
        anchor.as_point_mut()?.set_value((nx, ny));
        extra.set_event_out_flags(
            ae::EventOutFlags::HANDLED_EVENT
                | ae::EventOutFlags::ALWAYS_UPDATE
                | ae::EventOutFlags::UPDATE_NOW,
        );
        Ok(())
    }

    fn draw_graph_editor(
        params: &mut ae::Parameters<Params>,
        extra: &mut ae::EventExtra,
    ) -> Result<(), ae::Error> {
        if extra.effect_area() != ae::EffectArea::Control {
            return Ok(());
        }

        let drawbot = extra.context_handle().drawing_reference()?;
        let supplier = drawbot.supplier()?;
        let surface = drawbot.surface()?;
        let frame = extra.current_frame();
        let left = frame.left as f32 + 0.5;
        let top = frame.top as f32 + 0.5;
        let width = (frame.right - frame.left).max(1) as f32;
        let height = (frame.bottom - frame.top).max(1) as f32;

        let bg = ae::drawbot::ColorRgba {
            red: 0.12,
            green: 0.12,
            blue: 0.12,
            alpha: 1.0,
        };
        let grid = ae::drawbot::ColorRgba {
            red: 0.28,
            green: 0.28,
            blue: 0.28,
            alpha: 1.0,
        };
        let curve = ae::drawbot::ColorRgba {
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

        surface.paint_rect(
            &bg,
            &ae::drawbot::RectF32 {
                left,
                top,
                width,
                height,
            },
        )?;

        let grid_pen = supplier.new_pen(&grid, 1.0)?;
        for i in 0..=4 {
            let t = i as f32 / 4.0;
            let x = left + width * t;
            let y = top + height * t;

            let mut v = supplier.new_path()?;
            v.move_to(x, top)?;
            v.line_to(x, top + height)?;
            surface.stroke_path(&grid_pen, &v)?;

            let mut h = supplier.new_path()?;
            h.move_to(left, y)?;
            h.line_to(left + width, y)?;
            surface.stroke_path(&grid_pen, &h)?;
        }

        let anchor = params.get(Params::AnchorPoint)?.as_point()?.value();
        let to_screen = |nx: f32, ny: f32| -> (f32, f32) {
            let sx = left + nx * width;
            let sy = top + (1.0 - ny) * height;
            (sx, sy)
        };

        let p0 = to_screen(0.0, 0.0);
        let p1 = to_screen(anchor.0, anchor.1);
        let p2 = to_screen(1.0, 1.0);

        let curve_pen = supplier.new_pen(&curve, 2.0)?;
        let mut curve_path = supplier.new_path()?;
        curve_path.move_to(p0.0, p0.1)?;
        curve_path.line_to(p1.0, p1.1)?;
        curve_path.line_to(p2.0, p2.1)?;
        surface.stroke_path(&curve_pen, &curve_path)?;

        let anchor_brush = supplier.new_brush(&anchor_color)?;
        let mut anchor_shape = supplier.new_path()?;
        anchor_shape.add_arc(&ae::drawbot::PointF32 { x: p1.0, y: p1.1 }, 4.0, 0.0, 360.0)?;
        surface.fill_path(&anchor_brush, &anchor_shape, ae::drawbot::FillType::Winding)?;

        extra.set_event_out_flags(ae::EventOutFlags::HANDLED_EVENT);
        Ok(())
    }
}
