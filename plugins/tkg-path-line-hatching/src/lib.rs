#![allow(clippy::drop_non_drop, clippy::question_mark)]

use ae::pf::*;
use after_effects as ae;
use std::env;
use utils::ToPixel;

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
#[allow(non_camel_case_types)]
enum Params {
    RenderMode,
    Algorithm,
    Angle,
    DensityThresh,
    BaseCount,
    GroupLine_Start,
    Color,
    Thickness,
    RefLayer,
    RefMode,
    GroupLine_End,
    GroupU_Start,
    Align_U,
    Bias_U,
    Ease_U,
    Offset_U,
    GroupU_End,
    GroupV_Start,
    Align_V,
    Bias_V,
    Ease_V,
    Offset_V,
    GroupV_End,
}

#[derive(Default)]
struct Plugin {
    my_id: ae::aegp::PluginId,
}

ae::define_effect!(Plugin, (), Params);

const PLUGIN_DESCRIPTION: &str = "Procedural path line hatching using UV-like path mapping.";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RenderMode {
    FinalResult,
    UvGrid,
    DistributionMap,
    Assignment,
}

impl RenderMode {
    fn from_popup(value: i32) -> Self {
        match value {
            2 => Self::UvGrid,
            3 => Self::DistributionMap,
            4 => Self::Assignment,
            _ => Self::FinalResult,
        }
    }
}

struct NamedPathSamples {
    u1: Option<Vec<(f32, f32)>>,
    u2: Option<Vec<(f32, f32)>>,
    v1: Option<Vec<(f32, f32)>>,
    v2: Option<Vec<(f32, f32)>>,
}

type PathPoints<'a> = &'a [(f32, f32)];
type PatchBoundaries<'a> = (
    PathPoints<'a>,
    PathPoints<'a>,
    PathPoints<'a>,
    PathPoints<'a>,
);

impl AdobePluginGlobal for Plugin {
    fn params_setup(
        &self,
        params: &mut ae::Parameters<Params>,
        _in_data: InData,
        _: OutData,
    ) -> Result<(), Error> {
        params.add(
            Params::RenderMode,
            "描画モード",
            PopupDef::setup(|d| {
                d.set_options(&["Final Result", "UV+Grid", "Distribution map", "Assignment"]);
                d.set_default(1);
            }),
        )?;
        params.add_with_flags(
            Params::Algorithm,
            "Algorithm",
            PopupDef::setup(|d| {
                d.set_options(&["Dynamic Subdivision", "Evenly Spaced"]);
                d.set_default(1);
            }),
            ParamFlag::SUPERVISE,
            ParamUIFlags::NONE,
        )?;
        params.add(Params::Angle, "Angle", AngleDef::setup(|_| {}))?;
        params.add(
            Params::DensityThresh,
            "DensityThresh",
            FloatSliderDef::setup(|d| {
                d.set_valid_min(0.0);
                d.set_valid_max(100.0);
                d.set_slider_min(0.0);
                d.set_slider_max(100.0);
                d.set_default(50.0);
                d.set_precision(Precision::Tenths);
                d.set_display_flags(ValueDisplayFlag::PERCENT);
            }),
        )?;
        params.add(
            Params::BaseCount,
            "BaseCount",
            FloatSliderDef::setup(|d| {
                d.set_valid_min(1.0);
                d.set_valid_max(1024.0);
                d.set_slider_min(1.0);
                d.set_slider_max(128.0);
                d.set_default(24.0);
                d.set_precision(Precision::Integer);
            }),
        )?;

        params.add_group(
            Params::GroupLine_Start,
            Params::GroupLine_End,
            "Line 描画設定",
            true,
            |params| {
                params.add(Params::Color, "Color", ColorDef::setup(|_| {}))?;
                params.add(
                    Params::Thickness,
                    "Thickness",
                    FloatSliderDef::setup(|d| {
                        d.set_valid_min(0.0);
                        d.set_valid_max(200.0);
                        d.set_slider_min(0.0);
                        d.set_slider_max(40.0);
                        d.set_default(2.0);
                        d.set_precision(Precision::Tenths);
                    }),
                )?;
                params.add(Params::RefLayer, "RefLayer", LayerDef::setup(|_| {}))?;
                params.add(
                    Params::RefMode,
                    "RefMode",
                    PopupDef::setup(|d| {
                        d.set_options(&["Alpha", "Lightness", "Luminance"]);
                        d.set_default(1);
                    }),
                )?;
                Ok(())
            },
        )?;

        params.add_group(
            Params::GroupU_Start,
            Params::GroupU_End,
            "U方向 (Normal方向) 分布",
            true,
            |params| {
                params.add(
                    Params::Align_U,
                    "Align_U",
                    PopupDef::setup(|d| {
                        d.set_options(&["Left", "Center", "Right"]);
                        d.set_default(2);
                    }),
                )?;
                params.add(
                    Params::Bias_U,
                    "Bias_U",
                    FloatSliderDef::setup(|d| {
                        d.set_valid_min(0.0);
                        d.set_valid_max(100.0);
                        d.set_slider_min(0.0);
                        d.set_slider_max(100.0);
                        d.set_default(0.0);
                        d.set_precision(Precision::Tenths);
                        d.set_display_flags(ValueDisplayFlag::PERCENT);
                    }),
                )?;
                params.add(
                    Params::Ease_U,
                    "Ease_U",
                    PopupDef::setup(|d| {
                        d.set_options(&["None", "Ease In", "Ease Out", "Ease In-Out"]);
                        d.set_default(1);
                    }),
                )?;
                params.add(Params::Offset_U, "Offset_U", AngleDef::setup(|_| {}))?;
                Ok(())
            },
        )?;

        params.add_group(
            Params::GroupV_Start,
            Params::GroupV_End,
            "V方向 (Tangent方向) 分布",
            true,
            |params| {
                params.add(
                    Params::Align_V,
                    "Align_V",
                    PopupDef::setup(|d| {
                        d.set_options(&["Left", "Center", "Right"]);
                        d.set_default(2);
                    }),
                )?;
                params.add(
                    Params::Bias_V,
                    "Bias_V",
                    FloatSliderDef::setup(|d| {
                        d.set_valid_min(0.0);
                        d.set_valid_max(100.0);
                        d.set_slider_min(0.0);
                        d.set_slider_max(100.0);
                        d.set_default(0.0);
                        d.set_precision(Precision::Tenths);
                        d.set_display_flags(ValueDisplayFlag::PERCENT);
                    }),
                )?;
                params.add(
                    Params::Ease_V,
                    "Ease_V",
                    PopupDef::setup(|d| {
                        d.set_options(&["None", "Ease In", "Ease Out", "Ease In-Out"]);
                        d.set_default(1);
                    }),
                )?;
                params.add(Params::Offset_V, "Offset_V", AngleDef::setup(|_| {}))?;
                Ok(())
            },
        )?;

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
                        "AOD_TKG_PathLineHatching - {version}\r\r{PLUGIN_DESCRIPTION}\rCopyright (c) 2026-{build_year} Aodaruma",
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

                if let Ok(suite) = ae::aegp::suites::Utility::new()
                    && let Ok(id) = suite.register_with_aegp("AOD_TKG_PathLineHatching")
                {
                    self.my_id = id;
                }
            }
            ae::Command::UpdateParamsUi => {
                let mut p = params.cloned();
                let algo_val = p.get(Params::Algorithm)?.as_popup()?.value();

                let is_algo1 = algo_val == 1;

                let mut pd_density = p.get_mut(Params::DensityThresh)?;
                pd_density.set_ui_flag(ae::ParamUIFlags::DISABLED, !is_algo1);
                pd_density.update_param_ui()?;

                let mut pd_base = p.get_mut(Params::BaseCount)?;
                pd_base.set_ui_flag(ae::ParamUIFlags::DISABLED, is_algo1);
                pd_base.update_param_ui()?;
            }
            ae::Command::Render {
                in_layer,
                out_layer,
            } => {
                self.do_render(in_data, in_layer, out_data, out_layer, params, None)?;
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
                }
                if let Some(ref_idx) = params.index(Params::RefLayer) {
                    let _ = extra.callbacks().checkout_layer(
                        ref_idx as i32,
                        1,
                        &req,
                        in_data.current_time(),
                        in_data.time_step(),
                        in_data.time_scale(),
                    );
                }
            }
            ae::Command::SmartRender { extra } => {
                let cb = extra.callbacks();
                let in_layer_opt = cb.checkout_layer_pixels(0)?;
                let out_layer_opt = cb.checkout_output()?;
                let mut ref_layer_opt = None;
                if params.index(Params::RefLayer).is_some() {
                    ref_layer_opt = cb.checkout_layer_pixels(1).ok().flatten();
                }
                let has_ref = ref_layer_opt.is_some();

                if let (Some(in_layer), Some(out_layer)) = (in_layer_opt, out_layer_opt) {
                    self.do_render(
                        in_data,
                        in_layer,
                        out_data,
                        out_layer,
                        params,
                        ref_layer_opt,
                    )?;
                }
                cb.checkin_layer_pixels(0)?;
                if has_ref {
                    let _ = cb.checkin_layer_pixels(1);
                }
            }
            _ => {}
        }
        Ok(())
    }
}

impl Plugin {
    fn do_render(
        &self,
        in_data: InData,
        in_layer: Layer,
        _out_data: OutData,
        mut out_layer: Layer,
        params: &mut Parameters<Params>,
        ref_layer: Option<Layer>,
    ) -> Result<(), Error> {
        let render_mode =
            RenderMode::from_popup(params.get(Params::RenderMode)?.as_popup()?.value());
        let _line_color = params.get(Params::Color)?.as_color()?.value();
        let _line_thickness = params.get(Params::Thickness)?.as_float_slider()?.value() as f32;
        let _angle = params.get(Params::Angle)?.as_angle()?.value();
        let _offset_u = params.get(Params::Offset_U)?.as_angle()?.value();
        let _offset_v = params.get(Params::Offset_V)?.as_angle()?.value();
        let _ref_mode = params.get(Params::RefMode)?.as_popup()?.value();

        let in_world_type = in_layer.world_type();
        let out_world_type = out_layer.world_type();
        let progress_final = out_layer.height() as i32;
        let width = out_layer.width() as i32;
        let height = out_layer.height() as i32;

        let named_paths = self.collect_named_path_samples(in_data)?;
        let mut overlay: std::collections::HashMap<(i32, i32), PixelF32> =
            std::collections::HashMap::new();
        if let Some(paths) = named_paths.as_ref() {
            match render_mode {
                RenderMode::Assignment => {
                    draw_named_endpoint(
                        &mut overlay,
                        paths.u1.as_deref(),
                        "U1",
                        PixelF32 {
                            alpha: 1.0,
                            red: 1.0,
                            green: 0.0,
                            blue: 0.0,
                        },
                        width,
                        height,
                    );
                    draw_named_endpoint(
                        &mut overlay,
                        paths.u2.as_deref(),
                        "U2",
                        PixelF32 {
                            alpha: 1.0,
                            red: 0.0,
                            green: 1.0,
                            blue: 0.0,
                        },
                        width,
                        height,
                    );
                    draw_named_endpoint(
                        &mut overlay,
                        paths.v1.as_deref(),
                        "V1",
                        PixelF32 {
                            alpha: 1.0,
                            red: 0.0,
                            green: 0.0,
                            blue: 1.0,
                        },
                        width,
                        height,
                    );
                    draw_named_endpoint(
                        &mut overlay,
                        paths.v2.as_deref(),
                        "V2",
                        PixelF32 {
                            alpha: 1.0,
                            red: 1.0,
                            green: 1.0,
                            blue: 0.0,
                        },
                        width,
                        height,
                    );
                }
                RenderMode::UvGrid => {
                    let steps = 300_i32;
                    for iy in 0..=steps {
                        let v = iy as f32 / steps as f32;
                        for ix in 0..=steps {
                            let u = ix as f32 / steps as f32;
                            let Some((gx, gy)) = calculate_coons_patch(
                                u,
                                v,
                                paths.u1.as_deref(),
                                paths.u2.as_deref(),
                                paths.v1.as_deref(),
                                paths.v2.as_deref(),
                            ) else {
                                continue;
                            };
                            let mut color = PixelF32 {
                                alpha: 1.0,
                                red: u.clamp(0.0, 1.0),
                                green: v.clamp(0.0, 1.0),
                                blue: 0.0,
                            };
                            if (u % 0.1) < 0.02 || (v % 0.1) < 0.02 {
                                color = PixelF32 {
                                    alpha: 1.0,
                                    red: 1.0,
                                    green: 1.0,
                                    blue: 1.0,
                                };
                            }
                            draw_square_marker(&mut overlay, (gx, gy), 1, color, width, height);
                        }
                    }
                }
                RenderMode::FinalResult | RenderMode::DistributionMap => {
                    // Final Result / Distribution map は現時点では入力画像コピーをそのまま返す。
                }
            }
        }

        // 常に最初に in_layer を out_layer へコピーし、その後オーバーレイを上書きする。
        out_layer.iterate(0, progress_final, None, |x, y, mut dst| {
            let src = match in_world_type {
                ae::aegp::WorldType::U8 => in_layer.as_pixel8(x as usize, y as usize).to_pixel32(),
                ae::aegp::WorldType::U15 => {
                    in_layer.as_pixel16(x as usize, y as usize).to_pixel32()
                }
                ae::aegp::WorldType::F32 | ae::aegp::WorldType::None => {
                    *in_layer.as_pixel32(x as usize, y as usize)
                }
            };

            let mut out_px = src;

            if let Some(ref_layer) = ref_layer.as_ref() {
                let _modulation = sample_ref_layer_stub(ref_layer, x as usize, y as usize);
                // TODO: RefMode(alpha/lightness/luminance) で線の太さまたは alpha を変調する。
            }
            if let Some(overlay_px) = overlay.get(&(x, y)) {
                out_px = *overlay_px;
            }

            match out_world_type {
                ae::aegp::WorldType::U8 => dst.set_from_u8(out_px.to_pixel8()),
                ae::aegp::WorldType::U15 => dst.set_from_u16(out_px.to_pixel16()),
                ae::aegp::WorldType::F32 | ae::aegp::WorldType::None => dst.set_from_f32(out_px),
            }
            Ok(())
        })?;

        Ok(())
    }

    fn collect_named_path_samples(
        &self,
        in_data: InData,
    ) -> Result<Option<NamedPathSamples>, Error> {
        if in_data.is_premiere() {
            return Ok(None);
        }

        let effect_ref = in_data.effect_ref();
        let query = ae::pf::suites::PathQuery::new()?;
        let path_count = query.num_paths(effect_ref)?;
        let mut u1_id = None;
        let mut u2_id = None;
        let mut v1_id = None;
        let mut v2_id = None;

        for i in 0..path_count {
            let pid = query.path_info(effect_ref, i)?;
            if pid == ae::sys::PF_PathID_NONE as ae::sys::PF_PathID {
                continue;
            }
            let Some(path_outline) = query.checkout_path(
                effect_ref,
                pid,
                in_data.current_time(),
                in_data.time_step(),
                in_data.time_scale(),
            )?
            else {
                continue;
            };
            let name = path_outline.name()?;
            match name.as_str() {
                "U_1" => u1_id = Some(pid),
                "U_2" => u2_id = Some(pid),
                "V_1" => v1_id = Some(pid),
                "V_2" => v2_id = Some(pid),
                _ => {}
            }
        }

        if u1_id.is_none() && u2_id.is_none() && v1_id.is_none() && v2_id.is_none() {
            return Ok(None);
        }

        let fetch_samples =
            |pid: Option<ae::sys::PF_PathID>| -> Result<Option<Vec<(f32, f32)>>, Error> {
                let Some(pid) = pid else {
                    return Ok(None);
                };
                let Some(path) = query.checkout_path(
                    effect_ref,
                    pid,
                    in_data.current_time(),
                    in_data.time_step(),
                    in_data.time_scale(),
                )?
                else {
                    return Ok(None);
                };
                let points = sample_path_points(&path)?;
                if points.is_empty() {
                    return Ok(None);
                }
                Ok(Some(points))
            };

        Ok(Some(NamedPathSamples {
            u1: fetch_samples(u1_id)?,
            u2: fetch_samples(u2_id)?,
            v1: fetch_samples(v1_id)?,
            v2: fetch_samples(v2_id)?,
        }))
    }
}

fn sample_ref_layer_stub(layer: &Layer, x: usize, y: usize) -> f32 {
    let px = *layer.as_pixel32(
        x.min(layer.width().saturating_sub(1)),
        y.min(layer.height().saturating_sub(1)),
    );
    // TODO: RefMode に応じて alpha/lightness/luminance を返す。
    px.alpha
}

fn sample_path_points(path: &PathOutline) -> Result<Vec<(f32, f32)>, Error> {
    let mut points = Vec::new();
    let samples = 200_i32;
    for i in 0..=samples {
        let ratio = i as f64 / samples as f64;
        points.push(evaluate_path_at_ratio(path, ratio)?);
    }
    Ok(points)
}

fn evaluate_path_at_ratio(path: &PathOutline, ratio: f64) -> Result<(f32, f32), Error> {
    let num_segments = path.num_segments()?;
    if num_segments <= 0 {
        return Ok((0.0, 0.0));
    }

    let ratio = ratio.clamp(0.0, 1.0);
    let mut total_length = 0.0_f64;
    let mut seg_lengths = Vec::with_capacity(num_segments as usize);
    for seg in 0..num_segments {
        let mut prep = path.prepare_seg_length(seg, 100)?;
        let len = prep.length().unwrap_or(0.0);
        seg_lengths.push(len);
        total_length += len;
    }
    if total_length <= 1.0e-6 {
        return Ok((0.0, 0.0));
    }

    let target = total_length * ratio;
    let mut current = 0.0_f64;
    for seg in 0..num_segments {
        let seg_len = seg_lengths[seg as usize];
        if current + seg_len >= target || seg == num_segments - 1 {
            let local_len = (target - current).clamp(0.0, seg_len.max(0.0));
            let mut prep = path.prepare_seg_length(seg, 100)?;
            let (x, y) = prep.eval(local_len).unwrap_or((0.0, 0.0));
            return Ok((x as f32, y as f32));
        }
        current += seg_len;
    }
    Ok((0.0, 0.0))
}

fn sample_polyline_t(points: &[(f32, f32)], t: f32) -> (f32, f32) {
    if points.is_empty() {
        return (0.0, 0.0);
    }
    let idx = ((points.len().saturating_sub(1)) as f32 * t.clamp(0.0, 1.0)).round() as usize;
    points[idx.min(points.len() - 1)]
}

fn calculate_coons_patch(
    u: f32,
    v: f32,
    path_u1: Option<&[(f32, f32)]>,
    path_u2: Option<&[(f32, f32)]>,
    path_v1: Option<&[(f32, f32)]>,
    path_v2: Option<&[(f32, f32)]>,
) -> Option<(f32, f32)> {
    let virtual_v1;
    let virtual_v2;
    let virtual_u1;
    let virtual_u2;
    let (path_u1, path_u2, path_v1, path_v2): PatchBoundaries<'_> =
        match (path_u1, path_u2, path_v1, path_v2) {
            (Some(u1), Some(u2), Some(v1), Some(v2))
                if !u1.is_empty() && !u2.is_empty() && !v1.is_empty() && !v2.is_empty() =>
            {
                (u1, u2, v1, v2)
            }
            (Some(u1), Some(u2), None, None) if !u1.is_empty() && !u2.is_empty() => {
                virtual_v1 = vec![u1[0], u2[0]];
                virtual_v2 = vec![u1[u1.len() - 1], u2[u2.len() - 1]];
                (u1, u2, virtual_v1.as_slice(), virtual_v2.as_slice())
            }
            (None, None, Some(v1), Some(v2)) if !v1.is_empty() && !v2.is_empty() => {
                virtual_u1 = vec![v1[0], v2[0]];
                virtual_u2 = vec![v1[v1.len() - 1], v2[v2.len() - 1]];
                (virtual_u1.as_slice(), virtual_u2.as_slice(), v1, v2)
            }
            _ => return None,
        };
    let c0 = sample_polyline_t(path_u1, v);
    let c1 = sample_polyline_t(path_u2, v);
    let d0 = sample_polyline_t(path_v1, u);
    let d1 = sample_polyline_t(path_v2, u);

    let p00 = sample_polyline_t(path_u1, 0.0);
    let p10 = sample_polyline_t(path_u1, 1.0);
    let p01 = sample_polyline_t(path_u2, 0.0);
    let p11 = sample_polyline_t(path_u2, 1.0);

    let sx = (1.0 - u) * c0.0 + u * c1.0 + (1.0 - v) * d0.0 + v * d1.0
        - ((1.0 - u) * (1.0 - v) * p00.0
            + u * (1.0 - v) * p10.0
            + (1.0 - u) * v * p01.0
            + u * v * p11.0);
    let sy = (1.0 - u) * c0.1 + u * c1.1 + (1.0 - v) * d0.1 + v * d1.1
        - ((1.0 - u) * (1.0 - v) * p00.1
            + u * (1.0 - v) * p10.1
            + (1.0 - u) * v * p01.1
            + u * v * p11.1);
    Some((sx, sy))
}

fn draw_square_marker(
    overlay: &mut std::collections::HashMap<(i32, i32), PixelF32>,
    center: (f32, f32),
    radius: i32,
    color: PixelF32,
    width: i32,
    height: i32,
) {
    let cx = center.0.round() as i32;
    let cy = center.1.round() as i32;
    for yy in (cy - radius)..=(cy + radius) {
        if yy < 0 || yy >= height {
            continue;
        }
        for xx in (cx - radius)..=(cx + radius) {
            if xx < 0 || xx >= width {
                continue;
            }
            overlay.insert((xx, yy), color);
        }
    }
}

fn draw_named_endpoint(
    overlay: &mut std::collections::HashMap<(i32, i32), PixelF32>,
    path: Option<&[(f32, f32)]>,
    label: &str,
    color: PixelF32,
    width: i32,
    height: i32,
) {
    let Some(path) = path else {
        return;
    };
    if path.is_empty() {
        return;
    }
    let p = path[0];
    draw_square_marker(overlay, p, 2, color, width, height);
    draw_text_3x5(
        overlay,
        p.0.round() as i32 + 4,
        p.1.round() as i32 - 2,
        label,
        color,
        width,
        height,
    );
}

fn draw_text_3x5(
    overlay: &mut std::collections::HashMap<(i32, i32), PixelF32>,
    x: i32,
    y: i32,
    text: &str,
    color: PixelF32,
    width: i32,
    height: i32,
) {
    let mut cursor_x = x;
    for ch in text.chars() {
        draw_char_3x5(overlay, cursor_x, y, ch, color, width, height);
        cursor_x += 4;
    }
}

fn draw_char_3x5(
    overlay: &mut std::collections::HashMap<(i32, i32), PixelF32>,
    x: i32,
    y: i32,
    ch: char,
    color: PixelF32,
    width: i32,
    height: i32,
) {
    let glyph: [&str; 5] = match ch {
        'U' => ["101", "101", "101", "101", "111"],
        'V' => ["101", "101", "101", "101", "010"],
        '1' => ["010", "110", "010", "010", "111"],
        '2' => ["111", "001", "111", "100", "111"],
        _ => ["000", "000", "000", "000", "000"],
    };
    for (yy, row) in glyph.iter().enumerate() {
        for (xx, c) in row.chars().enumerate() {
            if c != '1' {
                continue;
            }
            let px = x + xx as i32;
            let py = y + yy as i32;
            if px < 0 || py < 0 || px >= width || py >= height {
                continue;
            }
            overlay.insert((px, py), color);
        }
    }
}
