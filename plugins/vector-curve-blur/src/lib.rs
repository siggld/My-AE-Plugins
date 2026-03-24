#![allow(clippy::drop_non_drop, clippy::question_mark)]

use ae::pf::*;
use after_effects as ae;
use std::env;
use utils::ToPixel;

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
enum Params {
    ViewMode,           // 1
    TargetMaskName,     // 2 (Arbitrary: String)
    AllMasks,           // 3
    NormalRange,        // 4
    NormalFalloff,      // 5
    NormalFalloffBias,  // 6
    PathBlurAmount,     // 7
    PathBlurOffset,     // 8
    EnableTaper,        // 9 (SUPERVISE)
    TaperGroupStart,    // 10
    StartTaperLength,   // 11
    StartTaperCurve,    // 12
    EndTaperLength,     // 13
    EndTaperCurve,      // 14
    TaperGroupEnd,      // 15
    FractalAmount,      // 16
    FractalScale,       // 17
    FractalComplexity,  // 18
    Evolution,          // 19
    ProfileGroupStart,  // 20
    EnableProfileCurve, // 21 (SUPERVISE)
    ProfileMaskName,    // 22 (Arbitrary: String)
    PositiveScale,      // 23
    NegativeScale,      // 24
    LinkScales,         // 25 (SUPERVISE)
    InvertCurveX,       // 26
    SwapNormal,         // 27
    ProfileGroupEnd,    // 28
}

#[derive(Default)]
struct Plugin {}

ae::define_effect!(Plugin, (), Params);

const PLUGIN_DESCRIPTION: &str = "Path-driven vector blur with taper and slit fractal modulation.";

#[derive(serde::Serialize, serde::Deserialize, PartialEq, PartialOrd, Clone, Default)]
struct MaskNameArb {
    name: String,
}

impl ae::ArbitraryData<MaskNameArb> for MaskNameArb {
    fn interpolate(&self, other: &Self, v: f64) -> Self {
        if v < 0.5 { self.clone() } else { other.clone() }
    }
}

#[derive(Clone, Copy)]
struct PathSample {
    x: f32,
    y: f32,
    tx: f32,
    ty: f32,
    nx: f32,
    ny: f32,
    t_norm: f32,
}

#[derive(Default)]
struct PathData {
    samples: Vec<PathSample>,
    profile_curve: Option<ProfileCurve>,
}

#[derive(Clone, Copy)]
struct ProfilePoint {
    x_norm: f32,
    y_norm: f32,
}

#[derive(Clone)]
struct ProfileCurve {
    points: Vec<ProfilePoint>,
}

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
                d.set_options(&["Final", "Preview Stroke", "Distance Field", "Fractal"]);
                d.set_default(1);
            }),
        )?;
        params.add(
            Params::TargetMaskName,
            "Target Mask Name",
            ArbitraryDef::setup(|d| {
                let _ = d.set_default(MaskNameArb {
                    name: "path".to_string(),
                });
            }),
        )?;
        params.add(
            Params::AllMasks,
            "All Masks",
            CheckBoxDef::setup(|d| {
                d.set_default(false);
            }),
        )?;
        params.add(
            Params::NormalRange,
            "Normal Range",
            FloatSliderDef::setup(|d| {
                d.set_valid_min(0.0);
                d.set_valid_max(4096.0);
                d.set_slider_min(0.0);
                d.set_slider_max(256.0);
                d.set_default(48.0);
                d.set_precision(1);
            }),
        )?;
        params.add(
            Params::NormalFalloff,
            "Normal Falloff",
            FloatSliderDef::setup(|d| {
                d.set_valid_min(0.0);
                d.set_valid_max(100.0);
                d.set_slider_min(0.0);
                d.set_slider_max(100.0);
                d.set_default(100.0);
                d.set_precision(1);
            }),
        )?;
        params.add(
            Params::NormalFalloffBias,
            "Normal Falloff Bias",
            FloatSliderDef::setup(|d| {
                d.set_valid_min(0.01);
                d.set_valid_max(8.0);
                d.set_slider_min(0.25);
                d.set_slider_max(4.0);
                d.set_default(1.0);
                d.set_precision(2);
            }),
        )?;
        params.add(
            Params::PathBlurAmount,
            "Path Blur Amount",
            FloatSliderDef::setup(|d| {
                d.set_valid_min(-1024.0);
                d.set_valid_max(1024.0);
                d.set_slider_min(-200.0);
                d.set_slider_max(200.0);
                d.set_default(36.0);
                d.set_precision(1);
            }),
        )?;
        params.add(
            Params::PathBlurOffset,
            "Path Blur Offset",
            FloatSliderDef::setup(|d| {
                d.set_valid_min(-100.0);
                d.set_valid_max(100.0);
                d.set_slider_min(-20.0);
                d.set_slider_max(20.0);
                d.set_default(0.0);
                d.set_precision(2);
            }),
        )?;

        params.add_with_flags(
            Params::EnableTaper,
            "Enable Taper",
            CheckBoxDef::setup(|d| {
                d.set_default(true);
            }),
            ParamFlag::SUPERVISE,
            ParamUIFlags::NONE,
        )?;
        params.add_group(
            Params::TaperGroupStart,
            Params::TaperGroupEnd,
            "Taper",
            true,
            |params| {
                params.add_with_flags(
                    Params::StartTaperLength,
                    "Start Taper Length",
                    FloatSliderDef::setup(|d| {
                        d.set_valid_min(0.0);
                        d.set_valid_max(100.0);
                        d.set_slider_min(0.0);
                        d.set_slider_max(100.0);
                        d.set_default(12.0);
                        d.set_precision(1);
                    }),
                    ParamFlag::START_COLLAPSED,
                    ParamUIFlags::NONE,
                )?;
                params.add(
                    Params::StartTaperCurve,
                    "Start Taper Curve",
                    FloatSliderDef::setup(|d| {
                        d.set_valid_min(0.1);
                        d.set_valid_max(8.0);
                        d.set_slider_min(0.25);
                        d.set_slider_max(4.0);
                        d.set_default(1.0);
                        d.set_precision(2);
                    }),
                )?;
                params.add(
                    Params::EndTaperLength,
                    "End Taper Length",
                    FloatSliderDef::setup(|d| {
                        d.set_valid_min(0.0);
                        d.set_valid_max(100.0);
                        d.set_slider_min(0.0);
                        d.set_slider_max(100.0);
                        d.set_default(12.0);
                        d.set_precision(1);
                    }),
                )?;
                params.add(
                    Params::EndTaperCurve,
                    "End Taper Curve",
                    FloatSliderDef::setup(|d| {
                        d.set_valid_min(0.1);
                        d.set_valid_max(8.0);
                        d.set_slider_min(0.25);
                        d.set_slider_max(4.0);
                        d.set_default(1.0);
                        d.set_precision(2);
                    }),
                )?;
                Ok(())
            },
        )?;

        params.add(
            Params::FractalAmount,
            "Fractal Amount",
            FloatSliderDef::setup(|d| {
                d.set_valid_min(0.0);
                d.set_valid_max(100.0);
                d.set_slider_min(0.0);
                d.set_slider_max(100.0);
                d.set_default(0.0);
                d.set_precision(1);
            }),
        )?;
        params.add(
            Params::FractalScale,
            "Fractal Scale",
            FloatSliderDef::setup(|d| {
                d.set_valid_min(0.1);
                d.set_valid_max(2048.0);
                d.set_slider_min(1.0);
                d.set_slider_max(256.0);
                d.set_default(28.0);
                d.set_precision(2);
            }),
        )?;
        params.add(
            Params::FractalComplexity,
            "Fractal Complexity",
            FloatSliderDef::setup(|d| {
                d.set_valid_min(1.0);
                d.set_valid_max(8.0);
                d.set_slider_min(1.0);
                d.set_slider_max(8.0);
                d.set_default(4.0);
                d.set_precision(0);
            }),
        )?;
        params.add(
            Params::Evolution,
            "Evolution",
            AngleDef::setup(|d| {
                d.set_default(0.0);
            }),
        )?;
        params.add_group(
            Params::ProfileGroupStart,
            Params::ProfileGroupEnd,
            "Profile",
            true,
            |params| {
                params.add_with_flags(
                    Params::EnableProfileCurve,
                    "Enable Profile Curve",
                    CheckBoxDef::setup(|d| {
                        d.set_default(false);
                    }),
                    ParamFlag::SUPERVISE,
                    ParamUIFlags::NONE,
                )?;
                params.add(
                    Params::ProfileMaskName,
                    "Profile Mask Name",
                    ArbitraryDef::setup(|d| {
                        let _ = d.set_default(MaskNameArb {
                            name: "curve".to_string(),
                        });
                    }),
                )?;
                params.add(
                    Params::PositiveScale,
                    "Positive Scale",
                    FloatSliderDef::setup(|d| {
                        d.set_valid_min(0.0);
                        d.set_valid_max(600.0);
                        d.set_slider_min(0.0);
                        d.set_slider_max(200.0);
                        d.set_default(100.0);
                        d.set_precision(1);
                    }),
                )?;
                params.add(
                    Params::NegativeScale,
                    "Negative Scale",
                    FloatSliderDef::setup(|d| {
                        d.set_valid_min(0.0);
                        d.set_valid_max(600.0);
                        d.set_slider_min(0.0);
                        d.set_slider_max(200.0);
                        d.set_default(100.0);
                        d.set_precision(1);
                    }),
                )?;
                params.add_with_flags(
                    Params::LinkScales,
                    "Link Scales",
                    CheckBoxDef::setup(|d| {
                        d.set_default(true);
                    }),
                    ParamFlag::SUPERVISE,
                    ParamUIFlags::NONE,
                )?;
                params.add(
                    Params::InvertCurveX,
                    "Invert Curve X",
                    PopupDef::setup(|d| {
                        d.set_options(&["None", "Positive", "Negative", "Both"]);
                        d.set_default(1);
                    }),
                )?;
                params.add(
                    Params::SwapNormal,
                    "Swap Normal (+/-)",
                    CheckBoxDef::setup(|d| {
                        d.set_default(false);
                    }),
                )?;
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
                        "AOD_VectorCurveBlur - {version}\r\r{PLUGIN_DESCRIPTION}\rCopyright (c) 2026-{build_year} Aodaruma",
                        version = env!("CARGO_PKG_VERSION"),
                        build_year = env!("BUILD_YEAR")
                    )
                    .as_str(),
                );
            }
            ae::Command::ArbitraryCallback { mut extra } => {
                extra.dispatch::<MaskNameArb, Params>(Params::TargetMaskName)?;
                extra.dispatch::<MaskNameArb, Params>(Params::ProfileMaskName)?;
            }
            ae::Command::GlobalSetup => {
                out_data.set_out_flag2(OutFlags2::SupportsSmartRender, true);
                out_data.set_out_flag2(OutFlags2::SupportsThreadedRendering, true);
                out_data.set_out_flag(OutFlags::SendUpdateParamsUi, true);
            }
            ae::Command::UpdateParamsUi => {
                let enable_taper = params.get(Params::EnableTaper)?.as_checkbox()?.value();
                let enable_profile = params
                    .get(Params::EnableProfileCurve)?
                    .as_checkbox()?
                    .value();
                let link_scales = params.get(Params::LinkScales)?.as_checkbox()?.value();
                let mut p = params.cloned();
                for k in [
                    Params::StartTaperLength,
                    Params::StartTaperCurve,
                    Params::EndTaperLength,
                    Params::EndTaperCurve,
                ] {
                    let mut pd = p.get_mut(k)?;
                    pd.set_ui_flag(ParamUIFlags::DISABLED, !enable_taper);
                    pd.update_param_ui()?;
                }
                for k in [
                    Params::ProfileMaskName,
                    Params::PositiveScale,
                    Params::NegativeScale,
                    Params::LinkScales,
                    Params::InvertCurveX,
                    Params::SwapNormal,
                ] {
                    let mut pd = p.get_mut(k)?;
                    let mut disabled = !enable_profile;
                    if k == Params::NegativeScale && link_scales {
                        disabled = true;
                    }
                    pd.set_ui_flag(ParamUIFlags::DISABLED, disabled);
                    pd.update_param_ui()?;
                }
            }
            ae::Command::Render {
                in_layer,
                out_layer,
            } => {
                self.do_render(in_data, in_layer, out_data, out_layer, params)?;
            }
            ae::Command::SmartPreRender { mut extra } => {
                let req = extra.output_request();
                let in_result = extra.callbacks().checkout_layer(
                    0,
                    0,
                    &req,
                    in_data.current_time(),
                    in_data.time_step(),
                    in_data.time_scale(),
                )?;
                let _ = extra.union_result_rect(in_result.result_rect.into());
                let _ = extra.union_max_result_rect(in_result.max_result_rect.into());
            }
            ae::Command::SmartRender { extra } => {
                let cb = extra.callbacks();
                let in_layer_opt = cb.checkout_layer_pixels(0)?;
                let out_layer_opt = cb.checkout_output()?;
                if let (Some(in_layer), Some(out_layer)) = (in_layer_opt, out_layer_opt) {
                    self.do_render(in_data, in_layer, out_data, out_layer, params)?;
                }
                cb.checkin_layer_pixels(0)?;
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
    ) -> Result<(), Error> {
        let path_data = self.collect_path_samples(in_data, params)?;
        if path_data.samples.is_empty() {
            out_layer.copy_from(&in_layer, None, None)?;
            return Ok(());
        }

        let view_mode = params.get(Params::ViewMode)?.as_popup()?.value();
        let normal_range = params.get(Params::NormalRange)?.as_float_slider()?.value() as f32;
        let normal_falloff = params
            .get(Params::NormalFalloff)?
            .as_float_slider()?
            .value() as f32
            / 100.0;
        let normal_bias = params
            .get(Params::NormalFalloffBias)?
            .as_float_slider()?
            .value() as f32;
        let blur_amount = params
            .get(Params::PathBlurAmount)?
            .as_float_slider()?
            .value() as f32;
        let path_offset = params
            .get(Params::PathBlurOffset)?
            .as_float_slider()?
            .value() as f32;
        let enable_taper = params.get(Params::EnableTaper)?.as_checkbox()?.value();
        let taper_s_len = params
            .get(Params::StartTaperLength)?
            .as_float_slider()?
            .value() as f32
            / 100.0;
        let taper_s_curve = params
            .get(Params::StartTaperCurve)?
            .as_float_slider()?
            .value() as f32;
        let taper_e_len = params
            .get(Params::EndTaperLength)?
            .as_float_slider()?
            .value() as f32
            / 100.0;
        let taper_e_curve = params
            .get(Params::EndTaperCurve)?
            .as_float_slider()?
            .value() as f32;
        let fract_amount = params
            .get(Params::FractalAmount)?
            .as_float_slider()?
            .value() as f32
            / 100.0;
        let fract_scale = params.get(Params::FractalScale)?.as_float_slider()?.value() as f32;
        let fract_complexity = params
            .get(Params::FractalComplexity)?
            .as_float_slider()?
            .value() as i32;
        let evolution = params.get(Params::Evolution)?.as_angle()?.float_value()? as f32;
        let enable_profile = params
            .get(Params::EnableProfileCurve)?
            .as_checkbox()?
            .value();
        let positive_scale = params
            .get(Params::PositiveScale)?
            .as_float_slider()?
            .value() as f32
            / 100.0;
        let mut negative_scale = params
            .get(Params::NegativeScale)?
            .as_float_slider()?
            .value() as f32
            / 100.0;
        let link_scales = params.get(Params::LinkScales)?.as_checkbox()?.value();
        if link_scales {
            negative_scale = positive_scale;
        }
        let invert_curve_x = params.get(Params::InvertCurveX)?.as_popup()?.value();
        let swap_normal = params.get(Params::SwapNormal)?.as_checkbox()?.value();

        let in_world = in_layer.world_type();
        let out_world = out_layer.world_type();
        let in_w = in_layer.width();
        let in_h = in_layer.height();
        let progress_final = out_layer.height() as i32;

        out_layer.iterate(0, progress_final, None, |x, y, mut dst| {
            let xf = x as f32 + 0.5;
            let yf = y as f32 + 0.5;
            let nearest = nearest_sample(&path_data.samples, xf, yf);
            let d_abs = nearest.distance.abs();
            let mut normal_w = if normal_range <= 0.0001 {
                1.0
            } else {
                (1.0 - (d_abs / normal_range)).clamp(0.0, 1.0)
            };
            normal_w = normal_w.powf(normal_bias.max(0.01)) * normal_falloff;

            let mut taper_w = 1.0;
            if enable_taper {
                taper_w *= taper_factor(
                    nearest.t_norm,
                    taper_s_len,
                    taper_s_curve,
                    taper_e_len,
                    taper_e_curve,
                );
            }
            let evo = evolution * 0.05;
            let slit_noise = fbm_1d(
                nearest.distance / fract_scale.max(0.001) + evo,
                fract_complexity,
            );
            let fract_w = 1.0 + (slit_noise - 0.5) * 2.0 * fract_amount;
            let profile_mul = if enable_profile {
                profile_multiplier(
                    path_data.profile_curve.as_ref(),
                    nearest.t_norm,
                    nearest.distance >= 0.0,
                    positive_scale,
                    negative_scale,
                    invert_curve_x,
                    swap_normal,
                )
            } else {
                1.0
            };
            let total_w =
                (normal_w * taper_w * fract_w * nearest.ambiguity * profile_mul).clamp(0.0, 1.0);

            let offset_t = (nearest.t_norm + path_offset * 0.01).rem_euclid(1.0);
            let center = sample_on_path(&path_data.samples, offset_t);
            let mut col = blur_along_tangent(
                &in_layer,
                in_world,
                in_w,
                in_h,
                center.x,
                center.y,
                center.tx,
                center.ty,
                blur_amount,
                total_w,
            );

            if view_mode == 2 {
                let v = total_w;
                col = PixelF32 {
                    red: v,
                    green: v * 0.7,
                    blue: 1.0 - v,
                    alpha: 1.0,
                };
            } else if view_mode == 3 {
                let g = if normal_range <= 0.0001 {
                    1.0
                } else {
                    (d_abs / normal_range).clamp(0.0, 1.0)
                };
                col = PixelF32 {
                    red: g,
                    green: g,
                    blue: g,
                    alpha: 1.0,
                };
            } else if view_mode == 4 {
                let g = slit_noise;
                col = PixelF32 {
                    red: g,
                    green: g,
                    blue: g,
                    alpha: 1.0,
                };
            }

            match out_world {
                ae::aegp::WorldType::U8 => dst.set_from_u8(col.to_pixel8()),
                ae::aegp::WorldType::U15 => dst.set_from_u16(col.to_pixel16()),
                ae::aegp::WorldType::F32 | ae::aegp::WorldType::None => dst.set_from_f32(col),
            }
            Ok(())
        })?;

        Ok(())
    }

    fn collect_path_samples(
        &self,
        in_data: InData,
        params: &mut Parameters<Params>,
    ) -> Result<PathData, Error> {
        if in_data.is_premiere() {
            return Ok(PathData::default());
        }

        let effect_ref = in_data.effect_ref();
        let query = ae::pf::suites::PathQuery::new()?;
        let data_suite = ae::pf::suites::PathData::new()?;
        let path_count = query.num_paths(effect_ref)?;

        let all_masks = params.get(Params::AllMasks)?.as_checkbox()?.value();
        let target = params
            .get(Params::TargetMaskName)?
            .as_arbitrary()?
            .value::<MaskNameArb>()?
            .name
            .to_lowercase();
        let profile_target = params
            .get(Params::ProfileMaskName)?
            .as_arbitrary()?
            .value::<MaskNameArb>()?
            .name
            .to_lowercase();

        let mut out = PathData::default();
        let mut profile_path_pts: Vec<(f32, f32)> = Vec::new();
        for i in 0..path_count {
            let pid = query.path_info(effect_ref, i)?;
            if pid == ae::sys::PF_PathID_NONE as ae::sys::PF_PathID {
                continue;
            }
            let Some(path) = query.checkout_path(
                effect_ref,
                pid,
                in_data.current_time(),
                in_data.time_step(),
                in_data.time_scale(),
            )?
            else {
                continue;
            };
            let name = data_suite
                .path_get_name(effect_ref, pid)
                .unwrap_or_default()
                .to_lowercase();
            let segs = path.num_segments()?;
            if segs <= 0 {
                continue;
            }
            let mut tmp: Vec<(f32, f32, f32, f32)> = Vec::new();
            for seg in 0..segs {
                let mut prep = path.prepare_seg_length(seg, 100)?;
                let seg_len = prep.length()? as f32;
                if seg_len <= 0.0001 {
                    continue;
                }
                let n = 24_i32.max((seg_len / 6.0) as i32);
                for j in 0..=n {
                    let l = seg_len * (j as f32 / n as f32);
                    let (x, y, dx, dy) = prep.eval_deriv1(l as f64)?;
                    let (tx, ty) = normalize2(dx as f32, dy as f32);
                    tmp.push((x as f32, y as f32, tx, ty));
                }
            }
            if tmp.is_empty() {
                continue;
            }
            if name.contains(&profile_target) && profile_path_pts.is_empty() {
                for (x, y, _, _) in &tmp {
                    profile_path_pts.push((*x, *y));
                }
            }
            if !all_masks && !name.contains(&target) {
                continue;
            }
            let last = (tmp.len() - 1) as f32;
            for (idx, (x, y, tx, ty)) in tmp.into_iter().enumerate() {
                out.samples.push(PathSample {
                    x,
                    y,
                    tx,
                    ty,
                    nx: -ty,
                    ny: tx,
                    t_norm: if last <= 0.0 { 0.0 } else { idx as f32 / last },
                });
            }
        }
        if !profile_path_pts.is_empty() {
            out.profile_curve = build_profile_curve(&profile_path_pts);
        }
        Ok(out)
    }
}

#[derive(Clone, Copy)]
struct Nearest {
    distance: f32,
    t_norm: f32,
    tx: f32,
    ty: f32,
    ambiguity: f32,
}

fn nearest_sample(samples: &[PathSample], x: f32, y: f32) -> Nearest {
    let mut best = f32::MAX;
    let mut second = f32::MAX;
    let mut best_s = samples[0];
    let mut second_s = samples[0];
    let mut out = Nearest {
        distance: 0.0,
        t_norm: 0.0,
        tx: 1.0,
        ty: 0.0,
        ambiguity: 1.0,
    };
    for s in samples {
        let dx = x - s.x;
        let dy = y - s.y;
        let d2 = dx * dx + dy * dy;
        if d2 < best {
            second = best;
            second_s = best_s;
            best = d2;
            best_s = *s;
        } else if d2 < second {
            second = d2;
            second_s = *s;
        }
    }
    let dx = x - best_s.x;
    let dy = y - best_s.y;
    out.distance = dx * best_s.nx + dy * best_s.ny;
    out.t_norm = best_s.t_norm;
    out.tx = best_s.tx;
    out.ty = best_s.ty;

    // 近接する別枝（ヘヤピンや鋭角折り返し）の場合にブラー強度を抑える。
    if second.is_finite() && best.is_finite() {
        let near_ratio = (best / (second + 1e-6)).clamp(0.0, 1.0);
        let tangent_sim = (best_s.tx * second_s.tx + best_s.ty * second_s.ty).abs();
        let branch_conflict = (1.0 - tangent_sim).clamp(0.0, 1.0);
        out.ambiguity = (0.35 + 0.65 * (1.0 - near_ratio * branch_conflict)).clamp(0.35, 1.0);
    }
    out
}

fn taper_factor(t: f32, s_len: f32, s_curve: f32, e_len: f32, e_curve: f32) -> f32 {
    let mut w = 1.0_f32;
    if s_len > 0.0001 && t < s_len {
        w *= (t / s_len).clamp(0.0, 1.0).powf(s_curve.max(0.1));
    }
    if e_len > 0.0001 && t > 1.0 - e_len {
        let u = ((1.0 - t) / e_len).clamp(0.0, 1.0);
        w *= u.powf(e_curve.max(0.1));
    }
    w
}

fn blur_along_tangent(
    in_layer: &Layer,
    in_world: ae::aegp::WorldType,
    width: usize,
    height: usize,
    center_x: f32,
    center_y: f32,
    tangent_x: f32,
    tangent_y: f32,
    blur_amount: f32,
    amp: f32,
) -> PixelF32 {
    let radius = blur_amount.abs().max(0.5);
    let dir = if blur_amount >= 0.0 { 1.0 } else { -1.0 };
    let taps = (radius / 4.0).ceil() as i32 * 2 + 1;
    let mut sum = PixelF32 {
        alpha: 0.0,
        red: 0.0,
        green: 0.0,
        blue: 0.0,
    };
    let mut wsum = 0.0_f32;
    for i in 0..taps {
        let tt = if taps <= 1 {
            0.0
        } else {
            i as f32 / (taps - 1) as f32
        };
        let centered = (tt - 0.5) * 2.0;
        let profile = 1.0 - centered.abs();
        let step = centered * radius * 0.5 * dir;
        let sx = center_x + tangent_x * step;
        let sy = center_y + tangent_y * step;
        let px = sample_bilinear(in_layer, in_world, width, height, sx, sy);
        let w = profile.max(0.0);
        sum.alpha += px.alpha * w;
        sum.red += px.red * w;
        sum.green += px.green * w;
        sum.blue += px.blue * w;
        wsum += w;
    }
    if wsum > 0.0 {
        sum.alpha /= wsum;
        sum.red /= wsum;
        sum.green /= wsum;
        sum.blue /= wsum;
    }
    let l = PixelF32 {
        alpha: sum.alpha,
        red: sum.red * amp,
        green: sum.green * amp,
        blue: sum.blue * amp,
    };
    l
}

fn sample_on_path(samples: &[PathSample], t: f32) -> PathSample {
    if samples.is_empty() {
        return PathSample {
            x: 0.0,
            y: 0.0,
            tx: 1.0,
            ty: 0.0,
            nx: 0.0,
            ny: 1.0,
            t_norm: 0.0,
        };
    }
    let mut best = samples[0];
    let mut best_d = f32::MAX;
    for s in samples {
        let d = (s.t_norm - t).abs();
        if d < best_d {
            best = *s;
            best_d = d;
        }
    }
    best
}

fn sample_bilinear(
    layer: &Layer,
    world_type: ae::aegp::WorldType,
    width: usize,
    height: usize,
    x: f32,
    y: f32,
) -> PixelF32 {
    if width == 0 || height == 0 {
        return PixelF32 {
            alpha: 0.0,
            red: 0.0,
            green: 0.0,
            blue: 0.0,
        };
    }
    let fx = x.clamp(0.0, (width.saturating_sub(1)) as f32);
    let fy = y.clamp(0.0, (height.saturating_sub(1)) as f32);
    let x0 = fx.floor() as usize;
    let y0 = fy.floor() as usize;
    let x1 = (x0 + 1).min(width.saturating_sub(1));
    let y1 = (y0 + 1).min(height.saturating_sub(1));
    let tx = fx - x0 as f32;
    let ty = fy - y0 as f32;

    let p00 = read_pixel_f32(layer, world_type, x0, y0);
    let p10 = read_pixel_f32(layer, world_type, x1, y0);
    let p01 = read_pixel_f32(layer, world_type, x0, y1);
    let p11 = read_pixel_f32(layer, world_type, x1, y1);

    let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t;
    PixelF32 {
        alpha: lerp(
            lerp(p00.alpha, p10.alpha, tx),
            lerp(p01.alpha, p11.alpha, tx),
            ty,
        ),
        red: lerp(lerp(p00.red, p10.red, tx), lerp(p01.red, p11.red, tx), ty),
        green: lerp(
            lerp(p00.green, p10.green, tx),
            lerp(p01.green, p11.green, tx),
            ty,
        ),
        blue: lerp(
            lerp(p00.blue, p10.blue, tx),
            lerp(p01.blue, p11.blue, tx),
            ty,
        ),
    }
}

fn read_pixel_f32(layer: &Layer, world_type: ae::aegp::WorldType, x: usize, y: usize) -> PixelF32 {
    match world_type {
        ae::aegp::WorldType::U8 => layer.as_pixel8(x, y).to_pixel32(),
        ae::aegp::WorldType::U15 => layer.as_pixel16(x, y).to_pixel32(),
        ae::aegp::WorldType::F32 | ae::aegp::WorldType::None => *layer.as_pixel32(x, y),
    }
}

fn normalize2(x: f32, y: f32) -> (f32, f32) {
    let l = (x * x + y * y).sqrt();
    if l <= 1e-6 {
        (1.0, 0.0)
    } else {
        (x / l, y / l)
    }
}

fn hash11(x: f32) -> f32 {
    (x.sin() * 43758.5453).fract().abs()
}

fn fbm_1d(mut x: f32, octaves: i32) -> f32 {
    let mut amp = 0.5_f32;
    let mut sum = 0.0_f32;
    let mut norm = 0.0_f32;
    let oct = octaves.clamp(1, 8);
    for _ in 0..oct {
        sum += hash11(x) * amp;
        norm += amp;
        amp *= 0.5;
        x *= 2.03;
    }
    if norm <= 0.0 {
        0.0
    } else {
        (sum / norm).clamp(0.0, 1.0)
    }
}

fn build_profile_curve(points: &[(f32, f32)]) -> Option<ProfileCurve> {
    if points.len() < 2 {
        return None;
    }
    let (sx, sy) = points[0];
    let (ex, ey) = points[points.len() - 1];
    let dx = ex - sx;
    let mut out: Vec<ProfilePoint> = Vec::with_capacity(points.len());

    let y_top = sy.min(ey);
    let y_bottom = sy.max(ey);
    let y_span = (y_bottom - y_top).abs().max(1e-4);
    let x_span = dx.abs();

    for (idx, (x, y)) in points.iter().enumerate() {
        let x_norm = if x_span <= 1e-4 {
            idx as f32 / (points.len() - 1) as f32
        } else {
            ((x - sx) / dx).clamp(0.0, 1.0)
        };
        let y_norm = ((y_bottom - *y) / y_span).clamp(0.0, 1.0);
        out.push(ProfilePoint { x_norm, y_norm });
    }
    Some(ProfileCurve { points: out })
}

fn sample_profile_y(curve: Option<&ProfileCurve>, t: f32) -> f32 {
    let Some(curve) = curve else {
        return 1.0;
    };
    if curve.points.is_empty() {
        return 1.0;
    }
    let mut best = curve.points[0];
    let mut best_d = f32::MAX;
    for p in &curve.points {
        let d = (p.x_norm - t).abs();
        if d < best_d {
            best_d = d;
            best = *p;
        }
    }
    best.y_norm
}

fn profile_multiplier(
    curve: Option<&ProfileCurve>,
    t_norm: f32,
    is_positive_side: bool,
    positive_scale: f32,
    negative_scale: f32,
    invert_mode: i32,
    swap_normal: bool,
) -> f32 {
    let mut use_positive = is_positive_side;
    if swap_normal {
        use_positive = !use_positive;
    }
    let invert_this_side = match invert_mode {
        2 => use_positive,
        3 => !use_positive,
        4 => true,
        _ => false,
    };
    let t = if invert_this_side {
        1.0 - t_norm
    } else {
        t_norm
    };
    let base = sample_profile_y(curve, t.clamp(0.0, 1.0));
    if use_positive {
        (base * positive_scale).max(0.0)
    } else {
        (base * negative_scale).max(0.0)
    }
}
