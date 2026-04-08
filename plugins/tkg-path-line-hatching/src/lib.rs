#![allow(clippy::drop_non_drop, clippy::question_mark)]

use ae::pf::*;
use after_effects as ae;
use std::env;
use utils::ToPixel;

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
#[allow(non_camel_case_types)]
enum Params {
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

#[derive(Default, Clone, Copy)]
struct PathUvIds {
    u1: Option<ae::sys::PF_PathID>,
    u2: Option<ae::sys::PF_PathID>,
    v1: Option<ae::sys::PF_PathID>,
    v2: Option<ae::sys::PF_PathID>,
}

impl AdobePluginGlobal for Plugin {
    fn params_setup(
        &self,
        params: &mut ae::Parameters<Params>,
        _in_data: InData,
        _: OutData,
    ) -> Result<(), Error> {
        params.add_with_flags(
            Params::Algorithm,
            "Algorithm",
            PopupDef::setup(|d| {
                d.set_options(&["動的密度補間(隙間埋め)", "均等+両端埋め"]);
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
                        d.set_options(&["アルファ", "明るさ(Lightness)", "輝度(Luminance)"]);
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
                        d.set_options(&["左寄せ", "中央寄せ", "右寄せ"]);
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
                        d.set_options(&["左寄せ", "中央寄せ", "右寄せ"]);
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
                let _ = extra.callbacks().checkout_layer(
                    Params::RefLayer as i32,
                    1,
                    &req,
                    in_data.current_time(),
                    in_data.time_step(),
                    in_data.time_scale(),
                );
            }
            ae::Command::SmartRender { extra } => {
                let cb = extra.callbacks();
                let in_layer_opt = cb.checkout_layer_pixels(0)?;
                let out_layer_opt = cb.checkout_output()?;
                let ref_layer_opt = cb.checkout_layer_pixels(1).ok().flatten();
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
                    cb.checkin_layer_pixels(1)?;
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
        let uv_ids = self.collect_uv_path_ids(in_data)?;

        // TODO: PF_PathGetName で U_1/U_2/V_1/V_2 を厳密に引く処理に置き換える。
        // TODO: PathDataSuite 相当の API で曲線評価を行い、UV 空間を構成する。
        // TODO: in_pixels.iter().zip(out_pixels.iter_mut()) でバッファ走査する。
        let _line_color = params.get(Params::Color)?.as_color()?.value();
        let _line_thickness = params.get(Params::Thickness)?.as_float_slider()?.value() as f32;
        let _angle = params.get(Params::Angle)?.as_angle()?.value() as f32;
        let _offset_u = params.get(Params::Offset_U)?.as_angle()?.value() as f32;
        let _offset_v = params.get(Params::Offset_V)?.as_angle()?.value() as f32;
        let _ref_mode = params.get(Params::RefMode)?.as_popup()?.value();

        let in_world_type = in_layer.world_type();
        let out_world_type = out_layer.world_type();
        let progress_final = out_layer.height() as i32;

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

            let _ = draw_hatching_stub(x as f32, y as f32, uv_ids);
            let out_px = src;

            if let Some(ref_layer) = ref_layer.as_ref() {
                let _modulation = sample_ref_layer_stub(ref_layer, x as usize, y as usize);
                // TODO: RefMode(alpha/lightness/luminance) で線の太さまたは alpha を変調する。
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

    fn collect_uv_path_ids(&self, in_data: InData) -> Result<PathUvIds, Error> {
        if in_data.is_premiere() {
            return Ok(PathUvIds::default());
        }

        let effect_ref = in_data.effect_ref();
        let query = ae::pf::suites::PathQuery::new()?;
        let path_count = query.num_paths(effect_ref)?;
        let mut ids = PathUvIds::default();
        let mut u1 = None;
        let mut u2 = None;
        let mut v1 = None;
        let mut v2 = None;

        for i in 0..path_count {
            let pid = query.path_info(effect_ref, i)?;
            if pid == ae::sys::PF_PathID_NONE as ae::sys::PF_PathID {
                continue;
            }
            let Some(_path_data) = query.checkout_path(
                effect_ref,
                pid,
                in_data.current_time(),
                in_data.time_step(),
                in_data.time_scale(),
            )?
            else {
                continue;
            };

            // TODO: PF_PathGetName で名前取得して U_1/U_2/V_1/V_2 に割り当てる。
            // 現段階では順序ベースで最低限の ID だけ確保する。
            if u1.is_none() {
                u1 = Some(pid);
            } else if u2.is_none() {
                u2 = Some(pid);
            } else if v1.is_none() {
                v1 = Some(pid);
            } else if v2.is_none() {
                v2 = Some(pid);
            }
        }

        ids.u1 = u1;
        ids.u2 = u2;
        ids.v1 = v1;
        ids.v2 = v2;
        Ok(ids)
    }
}

fn draw_hatching_stub(_x: f32, _y: f32, _uv: PathUvIds) -> f32 {
    // TODO: UV空間、Angle、Offset_U/V、Bias_U/V、Ease_U/V を使った線分布を実装する。
    0.0
}

fn sample_ref_layer_stub(layer: &Layer, x: usize, y: usize) -> f32 {
    let px = *layer.as_pixel32(
        x.min(layer.width().saturating_sub(1)),
        y.min(layer.height().saturating_sub(1)),
    );
    // TODO: RefMode に応じて alpha/lightness/luminance を返す。
    px.alpha
}
