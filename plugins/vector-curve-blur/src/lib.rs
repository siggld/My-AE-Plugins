#![allow(
    clippy::drop_non_drop,
    clippy::question_mark,
    clippy::too_many_arguments
)]

use ae::pf::*;
use after_effects as ae;
use std::env;
use std::f32::consts::TAU;
use utils::ToPixel;

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
enum Params {
    ViewMode,
    EnableMasks,
    SwapTangent,
    NormalRange,
    CenterLine,
    NormalFalloff,
    NormalFalloffBias,
    FalloffMode,
    PathBlurAmount,
    SplitTangent,
    NegativeBlurAmount,
    EnableTangentFalloff,
    TangentStartFallOff,
    TangentEndFallOff,
    PathBlurOffset,
    EnableTaper,
    TaperGroupStart,
    StartTaperLength,
    StartTaperCurve,
    EndTaperLength,
    EndTaperCurve,
    TaperGroupEnd,
    FractalAmount,
    FractalScale,
    FractalComplexity,
    Evolution,
    ProfileGroupStart,
    EnableProfileCurve,
    PositiveScale,
    LinkScales,
    NegativeScale,
    NormalSide,
    ProfileGroupEnd,
    TaperSCurve,
    FractalTangentScale,
    FractalTangentOffset,
    AddColorGroupStart,
    AddColorOpacity,
    AddColor,
    AddColorMode,
    AddFractalAmount,
    AddFractalMode,
    AddColorGroupEnd,
    FractalGroupStart,
    FractalGroupEnd,
    TangentFalloffGroupStart,
    TangentFalloffBias,
    TangentFalloffGroupEnd,
    BoxBlurGroupStart,
    FastBoxBlurRadius,
    FastBoxBlurIterations,
    FastBoxBlurRepeatEdge,
    BoxBlurGroupEnd,
}

#[derive(Default)]
struct Plugin {}

ae::define_effect!(Plugin, (), Params);

const PLUGIN_DESCRIPTION: &str = "Path-driven vector blur with taper and slit fractal modulation.";

#[derive(Clone, Copy)]
struct PathSample {
    x: f32,
    y: f32,
    tx: f32,
    ty: f32,
    t_norm: f32,
}

#[derive(Default)]
struct PathData {
    masks: Vec<MaskSamples>,
    profile_curve: Option<ProfileCurve>,
}

#[derive(Default)]
struct MaskSamples {
    samples: Vec<PathSample>,
    arc_len: f32,
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
        params.add_with_flags(
            Params::ViewMode,
            "View Mode",
            PopupDef::setup(|d| {
                d.set_options(&["Final", "Preview Stroke", "Distance Field", "Fractal"]);
                d.set_default(1);
            }),
            ParamFlag::SUPERVISE,
            ParamUIFlags::NONE,
        )?;
        params.add(
            Params::EnableMasks,
            "Use All Paths (path / path_[n])",
            CheckBoxDef::setup(|d| {
                d.set_default(true);
            }),
        )?;
        params.add_with_flags(
            Params::SwapTangent,
            "Swap Tangent (+/-)",
            CheckBoxDef::setup(|d| {
                d.set_default(false);
            }),
            ParamFlag::SUPERVISE,
            ParamUIFlags::NONE,
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
            Params::CenterLine,
            "CenterLine (%)",
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
            Params::NormalFalloff,
            "Normal Falloff",
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
            Params::NormalFalloffBias,
            "Normal Falloff Bias",
            FloatSliderDef::setup(|d| {
                d.set_valid_min(0.0);
                d.set_valid_max(300.0);
                d.set_slider_min(0.0);
                d.set_slider_max(300.0);
                d.set_default(0.0);
                d.set_precision(1);
            }),
        )?;
        params.add(
            Params::FalloffMode,
            "Falloff Mode",
            PopupDef::setup(|d| {
                d.set_options(&["Opacity", "Blur Amount"]);
                d.set_default(1);
            }),
        )?;
        params.add(
            Params::PathBlurAmount,
            "Path Blur Amount",
            FloatSliderDef::setup(|d| {
                d.set_valid_min(0.0);
                d.set_valid_max(1024.0);
                d.set_slider_min(0.0);
                d.set_slider_max(200.0);
                d.set_default(36.0);
                d.set_precision(1);
            }),
        )?;
        params.add_with_flags(
            Params::SplitTangent,
            "Split Tangent Direction",
            CheckBoxDef::setup(|d| {
                d.set_default(false);
            }),
            ParamFlag::SUPERVISE,
            ParamUIFlags::NONE,
        )?;
        params.add(
            Params::NegativeBlurAmount,
            "Negative Blur Amount",
            FloatSliderDef::setup(|d| {
                d.set_valid_min(0.0);
                d.set_valid_max(1024.0);
                d.set_slider_min(0.0);
                d.set_slider_max(200.0);
                d.set_default(36.0);
                d.set_precision(1);
            }),
        )?;
        params.add_group(
            Params::TangentFalloffGroupStart,
            Params::TangentFalloffGroupEnd,
            "Tangent Falloff",
            true,
            |params| {
                params.add_with_flags(
                    Params::EnableTangentFalloff,
                    "Enable Tangent Falloff",
                    CheckBoxDef::setup(|d| {
                        d.set_default(false);
                    }),
                    ParamFlag::SUPERVISE,
                    ParamUIFlags::NONE,
                )?;
                params.add(
                    Params::TangentStartFallOff,
                    "Tangent Start FallOff",
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
                    Params::TangentEndFallOff,
                    "Tangent End FallOff",
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
                    Params::TangentFalloffBias,
                    "Tangent Falloff Bias",
                    FloatSliderDef::setup(|d| {
                        d.set_valid_min(0.0);
                        d.set_valid_max(300.0);
                        d.set_slider_min(0.0);
                        d.set_slider_max(300.0);
                        d.set_default(0.0);
                        d.set_precision(1);
                    }),
                )?;
                Ok(())
            },
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
        params.add_group(
            Params::BoxBlurGroupStart,
            Params::BoxBlurGroupEnd,
            "Fast Box Blur",
            true,
            |params| {
                params.add_with_flags(
                    Params::FastBoxBlurRadius,
                    "Radius",
                    FloatSliderDef::setup(|d| {
                        d.set_valid_min(0.0);
                        d.set_valid_max(100.0);
                        d.set_slider_min(0.0);
                        d.set_slider_max(32.0);
                        d.set_default(0.0);
                        d.set_precision(0);
                    }),
                    ParamFlag::SUPERVISE,
                    ParamUIFlags::NONE,
                )?;
                params.add(
                    Params::FastBoxBlurIterations,
                    "Iterations",
                    FloatSliderDef::setup(|d| {
                        d.set_valid_min(1.0);
                        d.set_valid_max(10.0);
                        d.set_slider_min(1.0);
                        d.set_slider_max(10.0);
                        d.set_default(4.0);
                        d.set_precision(0);
                    }),
                )?;
                params.add(
                    Params::FastBoxBlurRepeatEdge,
                    "Repeat Edge Pixels",
                    CheckBoxDef::setup(|d| {
                        d.set_default(true);
                    }),
                )?;
                Ok(())
            },
        )?;

        params.add_group(
            Params::FractalGroupStart,
            Params::FractalGroupEnd,
            "Fractal",
            true,
            |params| {
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
                    Params::FractalTangentScale,
                    "Fractal Tangent Scale",
                    FloatSliderDef::setup(|d| {
                        d.set_valid_min(0.01);
                        d.set_valid_max(100.0);
                        d.set_slider_min(0.1);
                        d.set_slider_max(10.0);
                        d.set_default(1.0);
                        d.set_precision(2);
                    }),
                )?;
                params.add(
                    Params::FractalTangentOffset,
                    "Fractal Tangent Offset",
                    FloatSliderDef::setup(|d| {
                        d.set_valid_min(-1000.0);
                        d.set_valid_max(1000.0);
                        d.set_slider_min(-100.0);
                        d.set_slider_max(100.0);
                        d.set_default(0.0);
                        d.set_precision(1);
                    }),
                )?;
                params.add(
                    Params::FractalComplexity,
                    "Fractal Complexity",
                    FloatSliderDef::setup(|d| {
                        d.set_valid_min(0.0);
                        d.set_valid_max(100.0);
                        d.set_slider_min(0.0);
                        d.set_slider_max(100.0);
                        d.set_default(50.0);
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
                Ok(())
            },
        )?;

        params.add_group(
            Params::AddColorGroupStart,
            Params::AddColorGroupEnd,
            "Add Color",
            true,
            |params| {
                params.add(
                    Params::AddColorOpacity,
                    "Opacity",
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
                    Params::AddColor,
                    "Color",
                    ColorDef::setup(|d| {
                        d.set_default(ae::Pixel8 {
                            alpha: 255,
                            red: 140,
                            green: 140,
                            blue: 140,
                        });
                    }),
                )?;
                params.add(
                    Params::AddColorMode,
                    "Mode",
                    PopupDef::setup(|d| {
                        d.set_options(&[
                            "Normal",
                            "Multiply",
                            "Screen",
                            "Overlay",
                            "Add",
                            "Soft Light",
                            "Hard Light",
                            "Color Dodge",
                            "Color Burn",
                        ]);
                        d.set_default(1);
                    }),
                )?;
                params.add(
                    Params::AddFractalAmount,
                    "Add Fractal Amount",
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
                    Params::AddFractalMode,
                    "Add Fractal Mode",
                    PopupDef::setup(|d| {
                        d.set_options(&[
                            "Normal",
                            "Multiply",
                            "Screen",
                            "Overlay",
                            "Add",
                            "Soft Light",
                            "Hard Light",
                            "Color Dodge",
                            "Color Burn",
                        ]);
                        d.set_default(1);
                    }),
                )?;
                Ok(())
            },
        )?;

        params.add(
            Params::NormalSide,
            "Normal Side",
            PopupDef::setup(|d| {
                d.set_options(&["Positive", "Negative"]);
                d.set_default(1);
            }),
        )?;

        params.add_group(
            Params::TaperGroupStart,
            Params::TaperGroupEnd,
            "Simple Taper",
            true,
            |params| {
                params.add_with_flags(
                    Params::EnableTaper,
                    "Enable Taper",
                    CheckBoxDef::setup(|d| {
                        d.set_default(false);
                    }),
                    ParamFlag::SUPERVISE,
                    ParamUIFlags::NONE,
                )?;
                params.add(
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
                )?;
                params.add(
                    Params::StartTaperCurve,
                    "Start Taper Curve",
                    FloatSliderDef::setup(|d| {
                        d.set_valid_min(0.1);
                        d.set_valid_max(8.0);
                        d.set_slider_min(0.25);
                        d.set_slider_max(4.0);
                        d.set_default(0.5);
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
                        d.set_default(0.5);
                        d.set_precision(2);
                    }),
                )?;
                params.add_with_flags(
                    Params::TaperSCurve,
                    "Taper S-Curve",
                    CheckBoxDef::setup(|d| {
                        d.set_default(false);
                    }),
                    ParamFlag::SUPERVISE,
                    ParamUIFlags::NONE,
                )?;
                Ok(())
            },
        )?;

        params.add_group(
            Params::ProfileGroupStart,
            Params::ProfileGroupEnd,
            "Profile Taper (Curve)",
            true,
            |params| {
                params.add_with_flags(
                    Params::EnableProfileCurve,
                    "Enable Profile Curve (Curve)",
                    CheckBoxDef::setup(|d| {
                        d.set_default(false);
                    }),
                    ParamFlag::SUPERVISE,
                    ParamUIFlags::NONE,
                )?;
                params.add(
                    Params::PositiveScale,
                    "Positive Scale",
                    FloatSliderDef::setup(|d| {
                        d.set_valid_min(0.0);
                        d.set_valid_max(600.0);
                        d.set_slider_min(0.0);
                        d.set_slider_max(200.0);
                        d.set_default(10.0);
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
            ae::Command::GlobalSetup => {
                out_data.set_out_flag2(OutFlags2::SupportsSmartRender, true);
                out_data.set_out_flag2(OutFlags2::SupportsThreadedRendering, true);
                out_data.set_out_flag(OutFlags::SendUpdateParamsUi, true);
                out_data.set_out_flag(OutFlags::NonParamVary, true);
            }
            ae::Command::UpdateParamsUi => {
                let split_tangent = params.get(Params::SplitTangent)?.as_checkbox()?.value();
                let enable_tangent_falloff = params
                    .get(Params::EnableTangentFalloff)?
                    .as_checkbox()?
                    .value();
                let enable_taper = params.get(Params::EnableTaper)?.as_checkbox()?.value();
                let enable_profile = params
                    .get(Params::EnableProfileCurve)?
                    .as_checkbox()?
                    .value();
                let link_scales = params.get(Params::LinkScales)?.as_checkbox()?.value();
                let box_blur_radius = params
                    .get(Params::FastBoxBlurRadius)?
                    .as_float_slider()?
                    .value() as f32;
                let mut p = params.cloned();

                {
                    let mut pd = p.get_mut(Params::NegativeBlurAmount)?;
                    pd.set_ui_flag(ParamUIFlags::DISABLED, !split_tangent);
                    pd.update_param_ui()?;
                }
                for k in [
                    Params::TangentStartFallOff,
                    Params::TangentEndFallOff,
                    Params::TangentFalloffBias,
                ] {
                    let mut pd = p.get_mut(k)?;
                    pd.set_ui_flag(ParamUIFlags::DISABLED, !enable_tangent_falloff);
                    pd.update_param_ui()?;
                }
                for k in [Params::FastBoxBlurIterations, Params::FastBoxBlurRepeatEdge] {
                    let mut pd = p.get_mut(k)?;
                    pd.set_ui_flag(ParamUIFlags::DISABLED, box_blur_radius < 0.5);
                    pd.update_param_ui()?;
                }
                for k in [
                    Params::StartTaperLength,
                    Params::StartTaperCurve,
                    Params::EndTaperLength,
                    Params::EndTaperCurve,
                    Params::TaperSCurve,
                ] {
                    let mut pd = p.get_mut(k)?;
                    pd.set_ui_flag(ParamUIFlags::DISABLED, !enable_taper);
                    pd.update_param_ui()?;
                }
                for k in [
                    Params::PositiveScale,
                    Params::LinkScales,
                    Params::NegativeScale,
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

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------
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
        if path_data.masks.is_empty() {
            out_layer.copy_from(&in_layer, None, None)?;
            return Ok(());
        }

        let view_mode = params.get(Params::ViewMode)?.as_popup()?.value();
        let normal_range = params.get(Params::NormalRange)?.as_float_slider()?.value() as f32;
        let center_line = params.get(Params::CenterLine)?.as_float_slider()?.value() as f32 / 100.0;
        let normal_falloff = params
            .get(Params::NormalFalloff)?
            .as_float_slider()?
            .value() as f32
            / 100.0;
        let normal_bias = params
            .get(Params::NormalFalloffBias)?
            .as_float_slider()?
            .value() as f32;
        let falloff_mode = params.get(Params::FalloffMode)?.as_popup()?.value();
        let blur_amount = params
            .get(Params::PathBlurAmount)?
            .as_float_slider()?
            .value() as f32;
        let split_tangent = params.get(Params::SplitTangent)?.as_checkbox()?.value();
        let neg_blur_amount = if split_tangent {
            params
                .get(Params::NegativeBlurAmount)?
                .as_float_slider()?
                .value() as f32
        } else {
            blur_amount
        };
        let enable_tangent_falloff = params
            .get(Params::EnableTangentFalloff)?
            .as_checkbox()?
            .value();
        let tangent_start_falloff = params
            .get(Params::TangentStartFallOff)?
            .as_float_slider()?
            .value() as f32
            / 100.0;
        let tangent_end_falloff = params
            .get(Params::TangentEndFallOff)?
            .as_float_slider()?
            .value() as f32
            / 100.0;
        let tangent_falloff_bias = if enable_tangent_falloff {
            params
                .get(Params::TangentFalloffBias)?
                .as_float_slider()?
                .value() as f32
        } else {
            0.0
        };
        let path_offset = params
            .get(Params::PathBlurOffset)?
            .as_float_slider()?
            .value() as f32;
        let box_blur_radius = params
            .get(Params::FastBoxBlurRadius)?
            .as_float_slider()?
            .value()
            .round()
            .max(0.0) as usize;
        let box_blur_iterations = params
            .get(Params::FastBoxBlurIterations)?
            .as_float_slider()?
            .value()
            .round()
            .clamp(1.0, 10.0) as usize;
        let box_blur_repeat_edge = params
            .get(Params::FastBoxBlurRepeatEdge)?
            .as_checkbox()?
            .value();
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
        let taper_s_curve_enabled = params.get(Params::TaperSCurve)?.as_checkbox()?.value();
        let fract_amount = params
            .get(Params::FractalAmount)?
            .as_float_slider()?
            .value() as f32
            / 100.0;
        let fract_scale = params.get(Params::FractalScale)?.as_float_slider()?.value() as f32;
        let fract_complexity = params
            .get(Params::FractalComplexity)?
            .as_float_slider()?
            .value() as f32
            / 100.0;
        let evolution = params.get(Params::Evolution)?.as_angle()?.float_value()? as f32;
        let enable_profile = params
            .get(Params::EnableProfileCurve)?
            .as_checkbox()?
            .value();
        let positive_scale = params
            .get(Params::PositiveScale)?
            .as_float_slider()?
            .value() as f32
            / 10.0;
        let link_scales = params.get(Params::LinkScales)?.as_checkbox()?.value();
        let mut negative_scale = params
            .get(Params::NegativeScale)?
            .as_float_slider()?
            .value() as f32
            / 10.0;
        if link_scales {
            negative_scale = positive_scale;
        }
        let normal_side = params.get(Params::NormalSide)?.as_popup()?.value();
        let swap_tangent = params.get(Params::SwapTangent)?.as_checkbox()?.value();
        let fract_tangent_scale = params
            .get(Params::FractalTangentScale)?
            .as_float_slider()?
            .value() as f32;
        let fract_tangent_offset = params
            .get(Params::FractalTangentOffset)?
            .as_float_slider()?
            .value() as f32;
        let add_color_raw = params.get(Params::AddColor)?.as_color()?.value();
        let add_color_f32 = PixelF32 {
            alpha: 1.0,
            red: add_color_raw.red as f32 / ae::MAX_CHANNEL8 as f32,
            green: add_color_raw.green as f32 / ae::MAX_CHANNEL8 as f32,
            blue: add_color_raw.blue as f32 / ae::MAX_CHANNEL8 as f32,
        };
        let add_color_mode = params.get(Params::AddColorMode)?.as_popup()?.value();
        let add_color_opacity = params
            .get(Params::AddColorOpacity)?
            .as_float_slider()?
            .value() as f32
            / 100.0;
        let add_fract_amount = params
            .get(Params::AddFractalAmount)?
            .as_float_slider()?
            .value() as f32
            / 100.0;
        let add_fract_mode = params.get(Params::AddFractalMode)?.as_popup()?.value();

        let in_world = in_layer.world_type();
        let out_world = out_layer.world_type();
        let in_w = in_layer.width();
        let in_h = in_layer.height();
        let progress_final = out_layer.height() as i32;
        let smoothed_normal_buffer = if box_blur_radius > 0 && box_blur_iterations > 0 {
            let mut values = compute_max_normal_buffer(
                &path_data,
                in_w,
                in_h,
                normal_range,
                center_line,
                normal_falloff,
                normal_bias,
                enable_taper,
                taper_s_len,
                taper_s_curve,
                taper_e_len,
                taper_e_curve,
                taper_s_curve_enabled,
                enable_profile,
                positive_scale,
                negative_scale,
                swap_tangent,
                normal_side,
            );
            box_blur_channel_in_place(
                &mut values,
                in_w,
                in_h,
                box_blur_radius,
                box_blur_iterations,
                box_blur_repeat_edge,
            );
            Some(values)
        } else {
            None
        };

        macro_rules! set_dst {
            ($dst:expr, $col:expr) => {
                match out_world {
                    ae::aegp::WorldType::U8 => $dst.set_from_u8($col.to_pixel8()),
                    ae::aegp::WorldType::U15 => $dst.set_from_u16($col.to_pixel16()),
                    ae::aegp::WorldType::F32 | ae::aegp::WorldType::None => $dst.set_from_f32($col),
                }
            };
        }

        out_layer.iterate(0, progress_final, None, |x, y, mut dst| {
            let xf = x as f32 + 0.5;
            let yf = y as f32 + 0.5;
            let original = read_pixel_f32(&in_layer, in_world, x as usize, y as usize);
            let mut chosen: Option<(&MaskSamples, Nearest, f32, f32, f32)> = None;
            let mut best_normal_w = 0.0_f32;
            let mut max_blend = 0.0_f32;
            let mut max_preview_blend = 0.0_f32;

            for mask in &path_data.masks {
                if mask.samples.is_empty() {
                    continue;
                }

                let nearest = nearest_sample(&mask.samples, xf, yf);
                let taper_thickness = if enable_taper {
                    taper_factor(
                        nearest.t_norm,
                        taper_s_len,
                        taper_s_curve,
                        taper_e_len,
                        taper_e_curve,
                        taper_s_curve_enabled,
                    )
                } else {
                    1.0
                };
                let profile_thickness = if enable_profile {
                    profile_multiplier(
                        path_data.profile_curve.as_ref(),
                        nearest.t_norm,
                        nearest.distance >= 0.0,
                        positive_scale,
                        negative_scale,
                        swap_tangent,
                    )
                } else {
                    1.0
                };
                let effective_range = normal_range * taper_thickness * profile_thickness;
                let Some(side_u) =
                    selected_normal_side_u(nearest.distance, effective_range, normal_side)
                else {
                    continue;
                };

                let arc_len = mask.arc_len.max(1.0);
                let edge_zone_start = if enable_tangent_falloff {
                    tangent_start_falloff.clamp(0.0, 1.0)
                } else if split_tangent {
                    (neg_blur_amount / arc_len).clamp(0.01, 0.5)
                } else {
                    (blur_amount / arc_len).clamp(0.01, 0.5)
                };
                let edge_zone_end = if enable_tangent_falloff {
                    tangent_end_falloff.clamp(0.0, 1.0)
                } else if split_tangent {
                    (blur_amount / arc_len).clamp(0.01, 0.5)
                } else {
                    edge_zone_start
                };
                let at_start = nearest.best_t_norm < 0.01 && nearest.best_tangent_offset < 0.0;
                let at_end = nearest.best_t_norm > 0.99 && nearest.best_tangent_offset > 0.0;
                if at_start || at_end {
                    continue;
                }

                let normal_w = normal_band_weight(side_u, center_line, normal_falloff, normal_bias);
                if normal_w < 0.001 {
                    continue;
                }
                let edge_falloff = edge_fade_asymmetric(
                    nearest.t_norm,
                    edge_zone_start,
                    edge_zone_end,
                    tangent_falloff_bias,
                );
                if edge_falloff < 0.01 {
                    continue;
                }

                let edge_opacity_i = if falloff_mode == 2 { 1.0 } else { edge_falloff };
                let blend_i = (normal_w * edge_opacity_i * nearest.ambiguity).clamp(0.0, 1.0);
                let preview_blend_i = (normal_w * edge_falloff * nearest.ambiguity).clamp(0.0, 1.0);
                max_blend = max_blend.max(blend_i);
                max_preview_blend = max_preview_blend.max(preview_blend_i);

                if normal_w > best_normal_w {
                    chosen = Some((mask, nearest, effective_range, normal_w, edge_falloff));
                    best_normal_w = normal_w;
                }
                if max_blend >= 1.0 - 1e-6 && best_normal_w >= 1.0 - 1e-6 {
                    break;
                }
            }

            let Some((mask, nearest, effective_range, normal_w, edge_falloff)) = chosen else {
                set_dst!(dst, original);
                return Ok(());
            };

            let combined_blend = max_blend.clamp(0.0, 1.0);
            let preview_blend = max_preview_blend.clamp(0.0, 1.0);
            let d_abs = nearest.distance.abs();
            let arc_len = mask.arc_len.max(1.0);

            let evo = evolution * 0.05;
            let tangent_pos = nearest.t_norm * arc_len + nearest.tangent_offset;
            let fract_iso = (arc_len / effective_range.max(1.0)).sqrt().clamp(0.25, 4.0);
            let fract_x = tangent_pos / fract_scale.max(0.1) / fract_tangent_scale.max(0.01)
                + fract_tangent_offset;
            let fract_y = nearest.distance / fract_scale.max(0.1) * fract_iso;
            let fract_val = voronoi_2d(fract_x, fract_y, fract_complexity, evo);
            let fract_w = 1.0 + (fract_val - 0.5) * 2.0 * fract_amount;
            let total_blend = combined_blend;

            if view_mode == 2 {
                let vis = PixelF32 {
                    red: preview_blend,
                    green: preview_blend * 0.7,
                    blue: 1.0 - preview_blend,
                    alpha: 1.0,
                };
                let col = lerp_pixel(&original, &vis, preview_blend);
                set_dst!(dst, col);
                return Ok(());
            } else if view_mode == 3 {
                let g = (d_abs / effective_range.max(0.001)).clamp(0.0, 1.0);
                let vis = PixelF32 {
                    red: g,
                    green: g,
                    blue: g,
                    alpha: 1.0,
                };
                let col = lerp_pixel(&original, &vis, preview_blend);
                set_dst!(dst, col);
                return Ok(());
            } else if view_mode == 4 {
                let vis = PixelF32 {
                    red: fract_val,
                    green: fract_val,
                    blue: fract_val,
                    alpha: 1.0,
                };
                let col = lerp_pixel(&original, &vis, preview_blend);
                set_dst!(dst, col);
                return Ok(());
            }

            let (blur_tx, blur_ty) = if swap_tangent {
                (-nearest.tx, -nearest.ty)
            } else {
                (nearest.tx, nearest.ty)
            };
            let smoothed_normal_w = smoothed_normal_buffer
                .as_ref()
                .map(|values| values[y as usize * in_w + x as usize])
                .unwrap_or(normal_w)
                .clamp(0.0, 1.0);
            let ox = xf + blur_tx * path_offset * edge_falloff * smoothed_normal_w;
            let oy = yf + blur_ty * path_offset * edge_falloff * smoothed_normal_w;

            let (mut col, blend_strength) = if falloff_mode == 2 {
                let cur_pos_amt = blur_amount * edge_falloff * smoothed_normal_w * fract_w;
                let cur_neg_amt = neg_blur_amount * edge_falloff * smoothed_normal_w * fract_w;
                let blurred = blur_along_tangent(&TangentBlurParams {
                    layer: &in_layer,
                    world: in_world,
                    width: in_w,
                    height: in_h,
                    center_x: ox,
                    center_y: oy,
                    tangent_x: blur_tx,
                    tangent_y: blur_ty,
                    positive_amount: cur_pos_amt,
                    negative_amount: cur_neg_amt,
                });
                let opacity = total_blend;
                (lerp_pixel(&original, &blurred, opacity), opacity)
            } else {
                let cur_pos_amt = blur_amount * edge_falloff * fract_w;
                let cur_neg_amt = neg_blur_amount * edge_falloff * fract_w;
                let blurred = blur_along_tangent(&TangentBlurParams {
                    layer: &in_layer,
                    world: in_world,
                    width: in_w,
                    height: in_h,
                    center_x: ox,
                    center_y: oy,
                    tangent_x: blur_tx,
                    tangent_y: blur_ty,
                    positive_amount: cur_pos_amt,
                    negative_amount: cur_neg_amt,
                });
                (lerp_pixel(&original, &blurred, total_blend), total_blend)
            };

            if add_color_opacity > 0.001 && blend_strength > 0.001 {
                let mut tinted = add_color_f32;
                if add_fract_amount > 0.001 {
                    let fract_col = PixelF32 {
                        alpha: 1.0,
                        red: fract_val,
                        green: fract_val,
                        blue: fract_val,
                    };
                    tinted = blend_pixel(&tinted, &fract_col, add_fract_mode, add_fract_amount);
                }
                col = blend_pixel(
                    &col,
                    &tinted,
                    add_color_mode,
                    add_color_opacity * blend_strength * smoothed_normal_w,
                );
            }

            set_dst!(dst, col);
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
        let path_count = query.num_paths(effect_ref)?;

        let enable_masks = params.get(Params::EnableMasks)?.as_checkbox()?.value();
        let enable_profile = params
            .get(Params::EnableProfileCurve)?
            .as_checkbox()?
            .value();

        let mut out = PathData::default();
        let mut profile_path_pts: Vec<(f32, f32)> = Vec::new();
        let mut blur_mask_count = 0_usize;

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
                // Item 2: higher sampling density (seg_len / 2.0)
                let n = 24_i32.max((seg_len / 2.0) as i32);
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

            let path_name = path.name()?;
            let path_name = path_name.trim();

            if enable_profile && profile_path_pts.is_empty() && is_profile_curve_name(path_name) {
                for &(x, y, _, _) in &tmp {
                    profile_path_pts.push((x, y));
                }
                continue;
            }

            if !is_blur_path_name(path_name) {
                continue;
            }

            if !enable_masks && blur_mask_count > 0 {
                continue;
            }

            let last = (tmp.len() - 1) as f32;
            let mut mask_samples = MaskSamples::default();
            for (idx, (x, y, tx, ty)) in tmp.into_iter().enumerate() {
                mask_samples.samples.push(PathSample {
                    x,
                    y,
                    tx,
                    ty,
                    t_norm: if last <= 0.0 { 0.0 } else { idx as f32 / last },
                });
            }
            let smooth_radius = (mask_samples.samples.len() / 20).clamp(2, 16);
            smooth_tangents(&mut mask_samples.samples, smooth_radius);
            mask_samples.arc_len = compute_arc_length(&mask_samples.samples);
            out.masks.push(mask_samples);
            blur_mask_count += 1;
        }

        if !profile_path_pts.is_empty() {
            out.profile_curve = build_profile_curve(&profile_path_pts);
        }
        Ok(out)
    }
}

// ---------------------------------------------------------------------------
// Item 3: tangent vector smoothing
// ---------------------------------------------------------------------------
fn smooth_tangents(samples: &mut [PathSample], radius: usize) {
    if samples.len() < 3 || radius == 0 {
        return;
    }
    let orig: Vec<(f32, f32)> = samples.iter().map(|s| (s.tx, s.ty)).collect();
    for i in 0..samples.len() {
        let lo = i.saturating_sub(radius);
        let hi = (i + radius).min(samples.len() - 1);
        let (mut sx, mut sy) = (0.0_f32, 0.0_f32);
        for item in orig.iter().take(hi + 1).skip(lo) {
            sx += item.0;
            sy += item.1;
        }
        let (ntx, nty) = normalize2(sx, sy);
        samples[i].tx = ntx;
        samples[i].ty = nty;
    }
}

// ---------------------------------------------------------------------------
// Nearest-sample search with normal blending
// ---------------------------------------------------------------------------
#[derive(Clone, Copy)]
struct Nearest {
    distance: f32,
    t_norm: f32,
    tx: f32,
    ty: f32,
    ambiguity: f32,
    tangent_offset: f32,
    best_t_norm: f32,
    best_tangent_offset: f32,
}

fn nearest_sample(samples: &[PathSample], x: f32, y: f32) -> Nearest {
    let mut best_d2 = f32::MAX;
    let mut second_d2 = f32::MAX;
    let mut best_s = samples[0];
    let mut second_s = samples[0];

    for s in samples {
        let dx = x - s.x;
        let dy = y - s.y;
        let d2 = dx * dx + dy * dy;
        if d2 < best_d2 {
            second_d2 = best_d2;
            second_s = best_s;
            best_d2 = d2;
            best_s = *s;
        } else if d2 < second_d2 {
            second_d2 = d2;
            second_s = *s;
        }
    }

    let w = (best_d2 / (best_d2 + second_d2 + 1e-6)).clamp(0.0, 1.0);
    let blended_tx = best_s.tx * (1.0 - w) + second_s.tx * w;
    let blended_ty = best_s.ty * (1.0 - w) + second_s.ty * w;
    let (tx, ty) = normalize2(blended_tx, blended_ty);
    let nx = -ty;
    let ny = tx;

    let dx = x - best_s.x;
    let dy = y - best_s.y;
    let signed_dist = dx * nx + dy * ny;
    let tang_off = dx * tx + dy * ty;
    let best_tang_off = dx * best_s.tx + dy * best_s.ty;

    let blended_t = best_s.t_norm * (1.0 - w) + second_s.t_norm * w;

    let mut ambiguity = 1.0_f32;
    if second_d2.is_finite() && best_d2.is_finite() {
        let near_ratio = (best_d2 / (second_d2 + 1e-6)).clamp(0.0, 1.0);
        let tangent_sim = (best_s.tx * second_s.tx + best_s.ty * second_s.ty).abs();
        let branch_conflict = (1.0 - tangent_sim).clamp(0.0, 1.0);
        ambiguity = (0.35 + 0.65 * (1.0 - near_ratio * branch_conflict)).clamp(0.35, 1.0);
    }

    Nearest {
        distance: signed_dist,
        t_norm: blended_t,
        tx,
        ty,
        ambiguity,
        tangent_offset: tang_off,
        best_t_norm: best_s.t_norm,
        best_tangent_offset: best_tang_off,
    }
}

// ---------------------------------------------------------------------------
// Tangent blur with Gaussian kernel and asymmetric support (items 2, 2c)
// ---------------------------------------------------------------------------
struct TangentBlurParams<'a> {
    layer: &'a Layer,
    world: ae::aegp::WorldType,
    width: usize,
    height: usize,
    center_x: f32,
    center_y: f32,
    tangent_x: f32,
    tangent_y: f32,
    positive_amount: f32,
    negative_amount: f32,
}

fn blur_along_tangent(p: &TangentBlurParams<'_>) -> PixelF32 {
    let pos_r = p.positive_amount.max(0.0);
    let neg_r = p.negative_amount.max(0.0);
    let total = pos_r + neg_r;

    if total < 0.5 {
        return sample_bilinear(p.layer, p.world, p.width, p.height, p.center_x, p.center_y);
    }

    let taps = ((total / 4.0).ceil() as i32 * 2 + 1).max(3);
    let mut sum = PixelF32 {
        alpha: 0.0,
        red: 0.0,
        green: 0.0,
        blue: 0.0,
    };
    let mut wsum = 0.0_f32;

    let pos_sigma = (pos_r / 3.0).max(0.001);
    let neg_sigma = (neg_r / 3.0).max(0.001);

    for i in 0..taps {
        let t = i as f32 / (taps - 1) as f32;
        let offset = -neg_r + t * total;

        // Gaussian kernel per side
        let sigma = if offset < 0.0 { neg_sigma } else { pos_sigma };
        let w = (-0.5 * (offset / sigma).powi(2)).exp();
        if w < 1e-6 {
            continue;
        }

        let sx = p.center_x + p.tangent_x * offset;
        let sy = p.center_y + p.tangent_y * offset;
        let px = sample_bilinear(p.layer, p.world, p.width, p.height, sx, sy);
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
    sum
}

// ---------------------------------------------------------------------------
// Taper: controls NormalRange thickness at path endpoints
// ---------------------------------------------------------------------------
fn taper_factor(
    t: f32,
    s_len: f32,
    s_curve: f32,
    e_len: f32,
    e_curve: f32,
    s_curve_enabled: bool,
) -> f32 {
    let mut w = 1.0_f32;
    if s_len > 0.0001 && t < s_len {
        let u = (t / s_len).clamp(0.0, 1.0);
        w *= if s_curve_enabled {
            s_curve_power(u, s_curve.max(0.1))
        } else {
            u.powf(s_curve.max(0.1))
        };
    }
    if e_len > 0.0001 && t > 1.0 - e_len {
        let u = ((1.0 - t) / e_len).clamp(0.0, 1.0);
        w *= if s_curve_enabled {
            s_curve_power(u, e_curve.max(0.1))
        } else {
            u.powf(e_curve.max(0.1))
        };
    }
    w
}

fn s_curve_power(u: f32, curve: f32) -> f32 {
    if u < 0.5 {
        0.5 * (2.0 * u).powf(curve)
    } else {
        1.0 - 0.5 * (2.0 * (1.0 - u)).powf(curve)
    }
}

// ---------------------------------------------------------------------------
// Edge fade: smooth blur falloff at path start/end
// ---------------------------------------------------------------------------
fn edge_fade_asymmetric(t_norm: f32, zone_start: f32, zone_end: f32, bias: f32) -> f32 {
    let zs = zone_start.max(1e-6);
    let ze = zone_end.max(1e-6);
    let curve_pow = 1.0 + bias.max(0.0) / 100.0;

    let start_w = if t_norm <= 0.0 {
        0.0
    } else if t_norm < zs {
        (t_norm / zs).clamp(0.0, 1.0).powf(curve_pow)
    } else {
        1.0
    };

    let end_t = 1.0 - t_norm;
    let end_w = if end_t <= 0.0 {
        0.0
    } else if end_t < ze {
        (end_t / ze).clamp(0.0, 1.0).powf(curve_pow)
    } else {
        1.0
    };

    start_w.min(end_w)
}

fn selected_normal_side_u(distance: f32, effective_range: f32, normal_side: i32) -> Option<f32> {
    if effective_range < 0.001 {
        return None;
    }

    let side_matches = match normal_side {
        1 => distance >= -1e-4,
        2 => distance <= 1e-4,
        _ => true,
    };
    if !side_matches {
        return None;
    }

    let side_u = distance.abs() / effective_range;
    if side_u > 1.0 {
        return None;
    }

    Some(side_u.clamp(0.0, 1.0))
}

fn edge_to_center_weight(progress: f32, falloff: f32, bias: f32) -> f32 {
    let zone = falloff.clamp(0.0, 1.0);
    if zone <= 1e-4 {
        return 1.0;
    }

    let t = (progress / zone).clamp(0.0, 1.0);
    t.powf(1.0 + bias.max(0.0) / 100.0)
}

fn normal_band_weight(side_u: f32, center_line: f32, falloff: f32, bias: f32) -> f32 {
    if falloff <= 1e-4 {
        return 1.0;
    }

    let center = center_line.clamp(0.0, 1.0);
    if center <= 1e-4 {
        return edge_to_center_weight(1.0 - side_u, falloff, bias);
    }
    if center >= 1.0 - 1e-4 {
        return edge_to_center_weight(side_u, falloff, bias);
    }

    let from_path = side_u / center.max(1e-4);
    let from_outer = (1.0 - side_u) / (1.0 - center).max(1e-4);
    edge_to_center_weight(from_path, falloff, bias)
        .min(edge_to_center_weight(from_outer, falloff, bias))
}

fn is_blur_path_name(name: &str) -> bool {
    let lower = name.trim().to_ascii_lowercase();
    if lower == "path" {
        return true;
    }

    let Some(suffix) = lower.strip_prefix("path_") else {
        return false;
    };
    !suffix.is_empty() && suffix.chars().all(|ch| ch.is_ascii_digit())
}

fn is_profile_curve_name(name: &str) -> bool {
    name.trim().eq_ignore_ascii_case("curve")
}

fn compute_max_normal_buffer(
    path_data: &PathData,
    width: usize,
    height: usize,
    normal_range: f32,
    center_line: f32,
    normal_falloff: f32,
    normal_bias: f32,
    enable_taper: bool,
    taper_s_len: f32,
    taper_s_curve: f32,
    taper_e_len: f32,
    taper_e_curve: f32,
    taper_s_curve_enabled: bool,
    enable_profile: bool,
    positive_scale: f32,
    negative_scale: f32,
    swap_tangent: bool,
    normal_side: i32,
) -> Vec<f32> {
    let mut values = vec![0.0_f32; width * height];

    for y in 0..height {
        let yf = y as f32 + 0.5;
        for x in 0..width {
            let xf = x as f32 + 0.5;
            let mut max_normal_w = 0.0_f32;

            for mask in &path_data.masks {
                if mask.samples.is_empty() {
                    continue;
                }

                let nearest = nearest_sample(&mask.samples, xf, yf);
                let taper_thickness = if enable_taper {
                    taper_factor(
                        nearest.t_norm,
                        taper_s_len,
                        taper_s_curve,
                        taper_e_len,
                        taper_e_curve,
                        taper_s_curve_enabled,
                    )
                } else {
                    1.0
                };
                let profile_thickness = if enable_profile {
                    profile_multiplier(
                        path_data.profile_curve.as_ref(),
                        nearest.t_norm,
                        nearest.distance >= 0.0,
                        positive_scale,
                        negative_scale,
                        swap_tangent,
                    )
                } else {
                    1.0
                };
                let effective_range = normal_range * taper_thickness * profile_thickness;
                let Some(side_u) =
                    selected_normal_side_u(nearest.distance, effective_range, normal_side)
                else {
                    continue;
                };

                let at_start = nearest.best_t_norm < 0.01 && nearest.best_tangent_offset < 0.0;
                let at_end = nearest.best_t_norm > 0.99 && nearest.best_tangent_offset > 0.0;
                if at_start || at_end {
                    continue;
                }

                let normal_w = normal_band_weight(side_u, center_line, normal_falloff, normal_bias);
                max_normal_w = max_normal_w.max(normal_w.clamp(0.0, 1.0));
                if max_normal_w >= 1.0 - 1e-6 {
                    break;
                }
            }

            values[y * width + x] = max_normal_w;
        }
    }

    values
}

fn box_blur_channel_in_place(
    values: &mut [f32],
    width: usize,
    height: usize,
    radius: usize,
    iterations: usize,
    repeat_edge: bool,
) {
    if radius == 0 || iterations == 0 || values.is_empty() {
        return;
    }

    let mut tmp = vec![0.0_f32; values.len()];
    for _ in 0..iterations {
        box_blur_horizontal_runsum(values, &mut tmp, width, height, radius, repeat_edge);
        box_blur_vertical_runsum(&tmp, values, width, height, radius, repeat_edge);
    }
}

fn box_blur_horizontal_runsum(
    src: &[f32],
    dst: &mut [f32],
    width: usize,
    height: usize,
    radius: usize,
    repeat_edge: bool,
) {
    if width == 0 {
        return;
    }

    let radius = radius as isize;
    let full_count = (radius * 2 + 1) as f32;
    let mut prefix = vec![0.0_f32; width + 1];

    for y in 0..height {
        let row = y * width;
        prefix[0] = 0.0;
        for x in 0..width {
            prefix[x + 1] = prefix[x] + src[row + x];
        }

        for x in 0..width {
            let left = x as isize - radius;
            let right = x as isize + radius;
            let lo = left.max(0) as usize;
            let hi = right.min(width as isize - 1) as usize;
            let mut sum = prefix[hi + 1] - prefix[lo];
            let count = if repeat_edge {
                if left < 0 {
                    sum += src[row] * (-left) as f32;
                }
                if right >= width as isize {
                    sum += src[row + width - 1] * (right - width as isize + 1) as f32;
                }
                full_count
            } else {
                (hi - lo + 1) as f32
            };
            dst[row + x] = sum / count.max(1.0);
        }
    }
}

fn box_blur_vertical_runsum(
    src: &[f32],
    dst: &mut [f32],
    width: usize,
    height: usize,
    radius: usize,
    repeat_edge: bool,
) {
    if height == 0 {
        return;
    }

    let radius = radius as isize;
    let full_count = (radius * 2 + 1) as f32;
    let mut prefix = vec![0.0_f32; height + 1];

    for x in 0..width {
        prefix[0] = 0.0;
        for y in 0..height {
            prefix[y + 1] = prefix[y] + src[y * width + x];
        }

        for y in 0..height {
            let top = y as isize - radius;
            let bottom = y as isize + radius;
            let lo = top.max(0) as usize;
            let hi = bottom.min(height as isize - 1) as usize;
            let mut sum = prefix[hi + 1] - prefix[lo];
            let count = if repeat_edge {
                if top < 0 {
                    sum += src[x] * (-top) as f32;
                }
                if bottom >= height as isize {
                    sum += src[(height - 1) * width + x] * (bottom - height as isize + 1) as f32;
                }
                full_count
            } else {
                (hi - lo + 1) as f32
            };
            dst[y * width + x] = sum / count.max(1.0);
        }
    }
}

// ---------------------------------------------------------------------------
// 2D Voronoi cellular noise
// ---------------------------------------------------------------------------
fn hash21(ix: i32, iy: i32) -> (f32, f32) {
    let n = ix.wrapping_mul(1597).wrapping_add(iy.wrapping_mul(51749));
    let a = n.wrapping_mul(n).wrapping_mul(15731).wrapping_add(789221);
    let b = a.wrapping_mul(a).wrapping_add(1376312589);
    (
        (a as f32 / 2147483648.0).fract().abs(),
        (b as f32 / 2147483648.0).fract().abs(),
    )
}

fn voronoi_2d(x: f32, y: f32, sharpness: f32, evo: f32) -> f32 {
    let ix = x.floor() as i32;
    let iy = y.floor() as i32;
    let fx = x - ix as f32;
    let fy = y - iy as f32;
    let mut min_d = f32::MAX;
    for j in -1..=1_i32 {
        for i in -1..=1_i32 {
            let (hx, hy) = hash21(ix + i, iy + j);
            let cx = hx + 0.5 * (evo + hx * TAU).sin();
            let cy = hy + 0.5 * (evo * 1.3 + hy * TAU).cos();
            let dx = fx - i as f32 - cx;
            let dy = fy - j as f32 - cy;
            let d = (dx * dx + dy * dy).sqrt();
            if d < min_d {
                min_d = d;
            }
        }
    }
    let contrast = 1.0 + sharpness * 4.0;
    (min_d * contrast).clamp(0.0, 1.0)
}

// ---------------------------------------------------------------------------
// Profile curve: returns thickness multiplier
// ---------------------------------------------------------------------------
fn build_profile_curve(points: &[(f32, f32)]) -> Option<ProfileCurve> {
    if points.len() < 2 {
        return None;
    }
    let (sx, sy) = points[0];
    let (ex, ey) = points[points.len() - 1];
    let dx = ex - sx;
    let y_top = sy.min(ey);
    let y_bottom = sy.max(ey);
    let y_span = (y_bottom - y_top).abs().max(1e-4);
    let x_span = dx.abs();

    let mut out: Vec<ProfilePoint> = Vec::with_capacity(points.len());
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
    let Some(curve) = curve else { return 1.0 };
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
    swap_tangent: bool,
) -> f32 {
    let t = if swap_tangent { 1.0 - t_norm } else { t_norm };
    let base = sample_profile_y(curve, t.clamp(0.0, 1.0));

    if is_positive_side {
        (base * positive_scale).max(0.0)
    } else {
        (base * negative_scale).max(0.0)
    }
}

// ---------------------------------------------------------------------------
// Utility functions
// ---------------------------------------------------------------------------
fn compute_arc_length(samples: &[PathSample]) -> f32 {
    let mut len = 0.0_f32;
    for w in samples.windows(2) {
        let dx = w[1].x - w[0].x;
        let dy = w[1].y - w[0].y;
        len += (dx * dx + dy * dy).sqrt();
    }
    len
}

fn normalize2(x: f32, y: f32) -> (f32, f32) {
    let l = (x * x + y * y).sqrt();
    if l <= 1e-6 {
        (1.0, 0.0)
    } else {
        (x / l, y / l)
    }
}

fn lerp_pixel(a: &PixelF32, b: &PixelF32, t: f32) -> PixelF32 {
    let s = 1.0 - t;
    PixelF32 {
        alpha: a.alpha * s + b.alpha * t,
        red: a.red * s + b.red * t,
        green: a.green * s + b.green * t,
        blue: a.blue * s + b.blue * t,
    }
}

fn blend_channel(base: f32, blend: f32, mode: i32) -> f32 {
    match mode {
        2 => base * blend,
        3 => 1.0 - (1.0 - base) * (1.0 - blend),
        4 => {
            if base < 0.5 {
                2.0 * base * blend
            } else {
                1.0 - 2.0 * (1.0 - base) * (1.0 - blend)
            }
        }
        5 => (base + blend).min(1.0),
        6 => {
            if blend <= 0.5 {
                base - (1.0 - 2.0 * blend) * base * (1.0 - base)
            } else {
                let d = if base <= 0.25 {
                    ((16.0 * base - 12.0) * base + 4.0) * base
                } else {
                    base.sqrt()
                };
                base + (2.0 * blend - 1.0) * (d - base)
            }
        }
        7 => {
            if blend < 0.5 {
                2.0 * base * blend
            } else {
                1.0 - 2.0 * (1.0 - base) * (1.0 - blend)
            }
        }
        8 => {
            if blend < 1.0 {
                (base / (1.0 - blend)).min(1.0)
            } else {
                1.0
            }
        }
        9 => {
            if blend > 0.0 {
                (1.0 - ((1.0 - base) / blend).min(1.0)).max(0.0)
            } else {
                0.0
            }
        }
        _ => blend,
    }
}

fn blend_pixel(base: &PixelF32, blend: &PixelF32, mode: i32, opacity: f32) -> PixelF32 {
    let r = blend_channel(base.red, blend.red, mode);
    let g = blend_channel(base.green, blend.green, mode);
    let b = blend_channel(base.blue, blend.blue, mode);
    let s = 1.0 - opacity;
    PixelF32 {
        alpha: base.alpha,
        red: (base.red * s + r * opacity).clamp(0.0, 1.0),
        green: (base.green * s + g * opacity).clamp(0.0, 1.0),
        blue: (base.blue * s + b * opacity).clamp(0.0, 1.0),
    }
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
