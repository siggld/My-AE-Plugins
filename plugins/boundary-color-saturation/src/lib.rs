#![allow(clippy::drop_non_drop, clippy::question_mark)]

use ae::pf::*;
use after_effects as ae;
use std::env;
use utils::ToPixel;

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
#[allow(non_camel_case_types)]
enum Params {
    ViewMode,
    Link,
    ColorSet1GroupStart,
    ColorSelectGroupStart,
    ColorA,
    ColorB,
    ColorThreshold,
    ColorSelectGroupEnd,
    Mode,
    Center,
    Angle,
    Amount,
    Offset,
    BlurIterations,
    KeepDivision,
    SssGroupStart,
    EnableSss,
    AdditionalColor,
    SssSpread,
    SssBias,
    EdgeAlongBlur,
    EdgeAcrossBlur,
    SssGroupEnd,
    ColorSet1GroupEnd,
}

#[derive(Default)]
struct Plugin {}

ae::define_effect!(Plugin, (), Params);

const PLUGIN_DESCRIPTION: &str =
    "Creates boundary color extraction and blur control UI for compositing.";

impl AdobePluginGlobal for Plugin {
    fn params_setup(
        &self,
        params: &mut ae::Parameters<Params>,
        _in_data: InData,
        _: OutData,
    ) -> Result<(), Error> {
        params.add(
            Params::ViewMode,
            "View Mode",
            PopupDef::setup(|d| {
                d.set_options(&["Final", "Effect Only"]);
                d.set_default(1);
            }),
        )?;
        params.add(
            Params::Link,
            "Link",
            CheckBoxDef::setup(|d| {
                d.set_default(false);
            }),
        )?;
        params.add_group(
            Params::ColorSet1GroupStart,
            Params::ColorSet1GroupEnd,
            "ColorSet_1",
            true,
            |params| {
                params.add_group(
                    Params::ColorSelectGroupStart,
                    Params::ColorSelectGroupEnd,
                    "Color Select",
                    true,
                    |params| {
                        params.add(
                            Params::ColorA,
                            "Color A",
                            ColorDef::setup(|d| {
                                d.set_default(ae::Pixel8 {
                                    alpha: 255,
                                    red: 255,
                                    green: 255,
                                    blue: 255,
                                });
                            }),
                        )?;
                        params.add(
                            Params::ColorB,
                            "Color B",
                            ColorDef::setup(|d| {
                                d.set_default(ae::Pixel8 {
                                    alpha: 255,
                                    red: 0,
                                    green: 0,
                                    blue: 0,
                                });
                            }),
                        )?;
                        params.add(
                            Params::ColorThreshold,
                            "Color Threshold",
                            FloatSliderDef::setup(|d| {
                                d.set_valid_min(0.0);
                                d.set_valid_max(100.0);
                                d.set_slider_min(0.0);
                                d.set_slider_max(100.0);
                                d.set_default(10.0);
                                d.set_precision(Precision::Tenths);
                                d.set_display_flags(ValueDisplayFlag::PERCENT);
                            }),
                        )?;
                        Ok(())
                    },
                )?;

                params.add_with_flags(
                    Params::Mode,
                    "Mode",
                    PopupDef::setup(|d| {
                        d.set_options(&["Radial", "Directional"]);
                        d.set_default(1);
                    }),
                    ParamFlag::SUPERVISE,
                    ParamUIFlags::NONE,
                )?;

                params.add(
                    Params::Center,
                    "Center",
                    PointDef::setup(|d| {
                        d.set_default_x(50.0);
                        d.set_default_y(50.0);
                    }),
                )?;
                params.add(
                    Params::Angle,
                    "Angle",
                    AngleDef::setup(|d| {
                        d.set_default(0.0);
                    }),
                )?;
                params.add(
                    Params::Amount,
                    "Amount",
                    FloatSliderDef::setup(|d| {
                        d.set_valid_min(0.0);
                        d.set_valid_max(1000.0);
                        d.set_slider_min(0.0);
                        d.set_slider_max(100.0);
                        d.set_default(25.0);
                        d.set_precision(Precision::Tenths);
                    }),
                )?;
                params.add(
                    Params::Offset,
                    "Offset",
                    FloatSliderDef::setup(|d| {
                        d.set_valid_min(-100.0);
                        d.set_valid_max(100.0);
                        d.set_slider_min(-100.0);
                        d.set_slider_max(100.0);
                        d.set_default(0.0);
                        d.set_precision(Precision::Tenths);
                    }),
                )?;
                params.add(
                    Params::BlurIterations,
                    "Blur Iterations",
                    FloatSliderDef::setup(|d| {
                        d.set_valid_min(1.0);
                        d.set_valid_max(512.0);
                        d.set_slider_min(1.0);
                        d.set_slider_max(128.0);
                        d.set_default(16.0);
                        d.set_precision(Precision::Integer);
                    }),
                )?;
                params.add(
                    Params::KeepDivision,
                    "Keep Division",
                    CheckBoxDef::setup(|d| {
                        d.set_default(true);
                    }),
                )?;
                params.add_group(
                    Params::SssGroupStart,
                    Params::SssGroupEnd,
                    "SSS",
                    true,
                    |params| {
                        params.add_with_flags(
                            Params::EnableSss,
                            "Enable SSS",
                            CheckBoxDef::setup(|d| {
                                d.set_default(false);
                            }),
                            ParamFlag::SUPERVISE,
                            ParamUIFlags::NONE,
                        )?;
                        params.add(
                            Params::AdditionalColor,
                            "Additional Color",
                            ColorDef::setup(|d| {
                                d.set_default(ae::Pixel8 {
                                    alpha: 255,
                                    red: 255,
                                    green: 180,
                                    blue: 140,
                                });
                            }),
                        )?;
                        params.add(
                            Params::SssSpread,
                            "Spread",
                            FloatSliderDef::setup(|d| {
                                d.set_valid_min(0.0);
                                d.set_valid_max(100.0);
                                d.set_slider_min(0.0);
                                d.set_slider_max(100.0);
                                d.set_default(25.0);
                                d.set_precision(Precision::Tenths);
                            }),
                        )?;
                        params.add(
                            Params::SssBias,
                            "Bias",
                            FloatSliderDef::setup(|d| {
                                d.set_valid_min(-100.0);
                                d.set_valid_max(100.0);
                                d.set_slider_min(-100.0);
                                d.set_slider_max(100.0);
                                d.set_default(0.0);
                                d.set_precision(Precision::Tenths);
                            }),
                        )?;
                        params.add(
                            Params::EdgeAlongBlur,
                            "Edge Along Blur",
                            FloatSliderDef::setup(|d| {
                                d.set_valid_min(0.0);
                                d.set_valid_max(100.0);
                                d.set_slider_min(0.0);
                                d.set_slider_max(100.0);
                                d.set_default(20.0);
                                d.set_precision(Precision::Tenths);
                            }),
                        )?;
                        params.add(
                            Params::EdgeAcrossBlur,
                            "Edge Across Blur",
                            FloatSliderDef::setup(|d| {
                                d.set_valid_min(0.0);
                                d.set_valid_max(100.0);
                                d.set_slider_min(0.0);
                                d.set_slider_max(100.0);
                                d.set_default(8.0);
                                d.set_precision(Precision::Tenths);
                            }),
                        )?;
                        Ok(())
                    },
                )?;
                Ok(())
            },
        )?;

        Ok(())
    }

    fn handle_command(
        &mut self,
        cmd: ae::Command,
        _in_data: InData,
        mut out_data: OutData,
        params: &mut ae::Parameters<Params>,
    ) -> Result<(), ae::Error> {
        match cmd {
            ae::Command::About => {
                out_data.set_return_msg(
                    format!(
                        "TKG_BoundaryColorSaturation - {version}\r\r{PLUGIN_DESCRIPTION}\rCopyright (c) 2026-{build_year} Aodaruma",
                        version = env!("CARGO_PKG_VERSION"),
                        build_year = env!("BUILD_YEAR")
                    )
                    .as_str(),
                );
            }
            ae::Command::GlobalSetup => {
                out_data.set_out_flag2(ae::OutFlags2::SupportsThreadedRendering, true);
                out_data.set_out_flag2(ae::OutFlags2::ParamGroupStartCollapsedFlag, true);
                out_data.set_out_flag(ae::OutFlags::SendUpdateParamsUi, true);
            }
            ae::Command::UpdateParamsUi => {
                let mut p = params.cloned();
                let mode = p.get(Params::Mode)?.as_popup()?.value();
                let is_radial = mode == 1;
                let is_directional = mode == 2;
                let enable_sss = p.get(Params::EnableSss)?.as_checkbox()?.value();

                let mut pd_center = p.get_mut(Params::Center)?;
                let center_disabled_now = pd_center.ui_flags().contains(ae::ParamUIFlags::DISABLED);
                if center_disabled_now != is_directional {
                    pd_center.set_flag(ae::ParamFlag::START_COLLAPSED, true);
                    pd_center.set_ui_flag(ae::ParamUIFlags::DISABLED, is_directional);
                    pd_center.update_param_ui()?;
                }

                let mut pd_angle = p.get_mut(Params::Angle)?;
                let angle_disabled_now = pd_angle.ui_flags().contains(ae::ParamUIFlags::DISABLED);
                if angle_disabled_now != is_radial {
                    pd_angle.set_flag(ae::ParamFlag::START_COLLAPSED, true);
                    pd_angle.set_ui_flag(ae::ParamUIFlags::DISABLED, is_radial);
                    pd_angle.update_param_ui()?;
                }

                let mut pd_additional_color = p.get_mut(Params::AdditionalColor)?;
                let additional_color_disabled_now = pd_additional_color
                    .ui_flags()
                    .contains(ae::ParamUIFlags::DISABLED);
                if additional_color_disabled_now == enable_sss {
                    pd_additional_color.set_flag(ae::ParamFlag::START_COLLAPSED, true);
                    pd_additional_color.set_ui_flag(ae::ParamUIFlags::DISABLED, !enable_sss);
                    pd_additional_color.update_param_ui()?;
                }

                let mut pd_sss_spread = p.get_mut(Params::SssSpread)?;
                let sss_spread_disabled_now = pd_sss_spread
                    .ui_flags()
                    .contains(ae::ParamUIFlags::DISABLED);
                if sss_spread_disabled_now == enable_sss {
                    pd_sss_spread.set_flag(ae::ParamFlag::START_COLLAPSED, true);
                    pd_sss_spread.set_ui_flag(ae::ParamUIFlags::DISABLED, !enable_sss);
                    pd_sss_spread.update_param_ui()?;
                }

                let mut pd_sss_bias = p.get_mut(Params::SssBias)?;
                let sss_bias_disabled_now =
                    pd_sss_bias.ui_flags().contains(ae::ParamUIFlags::DISABLED);
                if sss_bias_disabled_now == enable_sss {
                    pd_sss_bias.set_flag(ae::ParamFlag::START_COLLAPSED, true);
                    pd_sss_bias.set_ui_flag(ae::ParamUIFlags::DISABLED, !enable_sss);
                    pd_sss_bias.update_param_ui()?;
                }

                let mut pd_edge_along_blur = p.get_mut(Params::EdgeAlongBlur)?;
                let edge_along_blur_disabled_now = pd_edge_along_blur
                    .ui_flags()
                    .contains(ae::ParamUIFlags::DISABLED);
                if edge_along_blur_disabled_now == enable_sss {
                    pd_edge_along_blur.set_flag(ae::ParamFlag::START_COLLAPSED, true);
                    pd_edge_along_blur.set_ui_flag(ae::ParamUIFlags::DISABLED, !enable_sss);
                    pd_edge_along_blur.update_param_ui()?;
                }

                let mut pd_edge_across_blur = p.get_mut(Params::EdgeAcrossBlur)?;
                let edge_across_blur_disabled_now = pd_edge_across_blur
                    .ui_flags()
                    .contains(ae::ParamUIFlags::DISABLED);
                if edge_across_blur_disabled_now == enable_sss {
                    pd_edge_across_blur.set_flag(ae::ParamFlag::START_COLLAPSED, true);
                    pd_edge_across_blur.set_ui_flag(ae::ParamUIFlags::DISABLED, !enable_sss);
                    pd_edge_across_blur.update_param_ui()?;
                }
            }
            ae::Command::Render {
                in_layer,
                out_layer,
            } => {
                self.do_render(_in_data, in_layer, out_layer, params)?;
            }
            _ => {}
        }
        Ok(())
    }
}

impl Plugin {
    fn do_render(
        &self,
        _in_data: InData,
        in_layer: Layer,
        mut out_layer: Layer,
        params: &mut Parameters<Params>,
    ) -> Result<(), Error> {
        let width = out_layer.width() as usize;
        let height = out_layer.height() as usize;
        if width == 0 || height == 0 {
            return Ok(());
        }

        let color_a = params.get(Params::ColorA)?.as_color()?.value().to_pixel32();
        let color_b = params.get(Params::ColorB)?.as_color()?.value().to_pixel32();
        let threshold = (params
            .get(Params::ColorThreshold)?
            .as_float_slider()?
            .value() as f32
            / 100.0)
            .clamp(0.0, 1.0);
        let mode = params.get(Params::Mode)?.as_popup()?.value();
        let center = params.get(Params::Center)?.as_point()?.value();
        let angle = params.get(Params::Angle)?.as_angle()?.value() as f32;
        let amount = params.get(Params::Amount)?.as_float_slider()?.value() as f32;
        let offset = params.get(Params::Offset)?.as_float_slider()?.value() as f32;
        let keep_division = params.get(Params::KeepDivision)?.as_checkbox()?.value();
        let view_mode = params.get(Params::ViewMode)?.as_popup()?.value();
        let enable_sss = params.get(Params::EnableSss)?.as_checkbox()?.value();
        let sss_color = params
            .get(Params::AdditionalColor)?
            .as_color()?
            .value()
            .to_pixel32();
        let sss_spread = (params.get(Params::SssSpread)?.as_float_slider()?.value() as f32 / 100.0)
            .clamp(0.0, 1.0);

        let in_world_type = in_layer.world_type();
        let out_world_type = out_layer.world_type();
        let mut source: Vec<ae::PixelF32> = Vec::with_capacity(width * height);
        for y in 0..height {
            for x in 0..width {
                source.push(read_pixel_f32(&in_layer, in_world_type, x, y));
            }
        }

        let mut mask_a = vec![0.0f32; width * height];
        let mut mask_b = vec![0.0f32; width * height];
        for y in 0..height {
            for x in 0..width {
                let idx = y * width + x;
                let px = source[idx];
                if keep_division && px.alpha <= 0.0 {
                    continue;
                }
                let da = color_distance(px, color_a);
                let db = color_distance(px, color_b);
                mask_a[idx] = to_membership(da, threshold);
                mask_b[idx] = to_membership(db, threshold);
            }
        }

        let radius = amount.max(0.0).round() as i32;
        let (dir_x, dir_y) = if mode == 2 {
            let rad = angle.to_radians();
            (rad.cos(), -rad.sin())
        } else {
            (0.0, 0.0)
        };
        let center_x = center.0 as f32 / 100.0 * (width as f32 - 1.0);
        let center_y = center.1 as f32 / 100.0 * (height as f32 - 1.0);

        let mut boundary = vec![0.0f32; width * height];
        for y in 0..height {
            for x in 0..width {
                let idx = y * width + x;
                let (vx, vy) = if mode == 2 {
                    (dir_x, dir_y)
                } else {
                    let dx = x as f32 - center_x;
                    let dy = y as f32 - center_y;
                    let len = (dx * dx + dy * dy).sqrt().max(1.0e-6);
                    (dx / len, dy / len)
                };
                let mut sum_a = 0.0;
                let mut sum_b = 0.0;
                let mut count = 0.0;
                for step in -radius..=radius {
                    let sx = x as f32 + vx * (step as f32 + offset);
                    let sy = y as f32 + vy * (step as f32 + offset);
                    if sx < 0.0 || sy < 0.0 || sx >= width as f32 || sy >= height as f32 {
                        continue;
                    }
                    let si = sy.round() as usize * width + sx.round() as usize;
                    if keep_division && source[si].alpha <= 0.0 {
                        continue;
                    }
                    sum_a += mask_a[si];
                    sum_b += mask_b[si];
                    count += 1.0;
                }
                if count > 0.0 {
                    let ba = sum_a / count;
                    let bb = sum_b / count;
                    boundary[idx] = (ba.min(bb) * 2.0).clamp(0.0, 1.0);
                }
            }
        }

        out_layer.iterate(0, out_layer.height() as i32, None, |x, y, mut dst| {
            let idx = y as usize * width + x as usize;
            let src = source[idx];
            let b = boundary[idx];
            let mut out = if view_mode == 2 {
                ae::PixelF32 {
                    red: b,
                    green: b,
                    blue: b,
                    alpha: src.alpha,
                }
            } else {
                let mut color = src;
                if enable_sss {
                    color.red = (color.red + sss_color.red * b * sss_spread).clamp(0.0, 1.0);
                    color.green = (color.green + sss_color.green * b * sss_spread).clamp(0.0, 1.0);
                    color.blue = (color.blue + sss_color.blue * b * sss_spread).clamp(0.0, 1.0);
                }
                color
            };
            out.alpha = src.alpha;

            match out_world_type {
                ae::aegp::WorldType::U8 => dst.set_from_u8(out.to_pixel8()),
                ae::aegp::WorldType::U15 => dst.set_from_u16(out.to_pixel16()),
                ae::aegp::WorldType::F32 | ae::aegp::WorldType::None => dst.set_from_f32(out),
            }
            Ok(())
        })?;

        Ok(())
    }
}

fn read_pixel_f32(
    layer: &Layer,
    world_type: ae::aegp::WorldType,
    x: usize,
    y: usize,
) -> ae::PixelF32 {
    match world_type {
        ae::aegp::WorldType::U8 => layer.as_pixel8(x, y).to_pixel32(),
        ae::aegp::WorldType::U15 => layer.as_pixel16(x, y).to_pixel32(),
        ae::aegp::WorldType::F32 | ae::aegp::WorldType::None => *layer.as_pixel32(x, y),
    }
}

fn color_distance(a: ae::PixelF32, b: ae::PixelF32) -> f32 {
    let dr = a.red - b.red;
    let dg = a.green - b.green;
    let db = a.blue - b.blue;
    (dr * dr + dg * dg + db * db).sqrt() / 1.732_050_8
}

fn to_membership(distance: f32, threshold: f32) -> f32 {
    if threshold <= 0.0 {
        return if distance <= 0.0 { 1.0 } else { 0.0 };
    }
    (1.0 - distance / threshold).clamp(0.0, 1.0)
}
