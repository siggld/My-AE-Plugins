#![allow(
    clippy::drop_non_drop,
    clippy::question_mark,
    clippy::too_many_arguments
)]

use ae::pf::*;
use ae_ui::{apply_disabled, enable_update_params_ui};
use after_effects as ae;
use std::collections::VecDeque;
use std::env;
use std::f32::consts::TAU;
use utils::ToPixel;

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
enum Params {
    ViewMode,
    EnableMasks,
    AntialiasingQuality,
    NormalSide,
    SwapTangent,
    PathBlurAmount,
    NegativeBlurAmount,
    PathBlurOffset,
    NormalBandGroupStart,
    NormalRange,
    CenterLine,
    NormalFalloff,
    NormalFalloffBias,
    NormalBandGroupEnd,
    TangentFalloffGroupStart,
    EnableTangentFalloff,
    TangentStartFallOff,
    TangentEndFallOff,
    TangentFalloffBias,
    TangentFalloffGroupEnd,
    OffsetEndFade,
    EdgePreserveGroupStart,
    FractalAmount,
    DarkExpandThreshold,
    DarkExpandRadius,
    PreserveMatteEdge,
    EdgeDisplaceMultiplier,
    EdgeBlurMultiplier,
    EdgeGhostMultiplier,
    EdgeGhostAlpha,
    EdgePreserveGroupEnd,
    TaperMode,
    TaperGroupStart,
    StartTaperLength,
    StartTaperCurve,
    EndTaperLength,
    EndTaperCurve,
    TaperGroupEnd,
    FractalScale,
    FractalComplexity,
    Evolution,
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
}

#[derive(Default)]
struct Plugin {}

ae::define_effect!(Plugin, (), Params);

const PLUGIN_DESCRIPTION: &str = "Path-driven vector blur with taper and slit fractal modulation.";
const HIGH_SUPERSAMPLE_OFFSETS: [(f32, f32, f32); 9] = [
    (0.0, 0.0, 2.0),
    (-0.35, 0.0, 1.0),
    (0.35, 0.0, 1.0),
    (0.0, -0.35, 1.0),
    (0.0, 0.35, 1.0),
    (-0.25, -0.25, 0.75),
    (0.25, -0.25, 0.75),
    (-0.25, 0.25, 0.75),
    (0.25, 0.25, 0.75),
];
const LOW_SUPERSAMPLE_OFFSETS: [(f32, f32, f32); 5] = [
    (0.0, 0.0, 2.0),
    (-0.35, 0.0, 1.0),
    (0.35, 0.0, 1.0),
    (0.0, -0.35, 1.0),
    (0.0, 0.35, 1.0),
];
const SINGLE_SAMPLE_OFFSET: [(f32, f32, f32); 1] = [(0.0, 0.0, 1.0)];

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

#[derive(Clone)]
struct ImageBufferF32 {
    width: usize,
    height: usize,
    pixels: Vec<PixelF32>,
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

#[derive(Clone, Copy)]
struct EvalConfig {
    normal_range: f32,
    center_line: f32,
    normal_falloff: f32,
    normal_bias: f32,
    tangent_amount: f32,
    negative_tangent_amount: f32,
    enable_tangent_falloff: bool,
    tangent_start_falloff: f32,
    tangent_end_falloff: f32,
    tangent_falloff_bias: f32,
    enable_taper: bool,
    taper_s_len: f32,
    taper_s_curve: f32,
    taper_e_len: f32,
    taper_e_curve: f32,
    taper_s_curve_enabled: bool,
    enable_profile: bool,
    normal_side: i32,
    swap_tangent: bool,
}

#[derive(Clone, Copy)]
struct DarkExpandConfig {
    threshold: f32,
    radius: usize,
    preserve_matte_edge: bool,
}

#[derive(Clone, Copy)]
struct PixelContribution {
    t_norm: f32,
    tx: f32,
    ty: f32,
    ambiguity: f32,
    tangent_offset: f32,
    effective_range: f32,
    side_u: f32,
    thickness_factor: f32,
    arc_len: f32,
    normal_w: f32,
    edge_falloff: f32,
    total_blend: f32,
}

#[derive(Default)]
struct PixelContributionAccumulator {
    weight_sum: f32,
    t_norm: f32,
    tx: f32,
    ty: f32,
    ambiguity: f32,
    tangent_offset: f32,
    effective_range: f32,
    side_u: f32,
    thickness_factor: f32,
    arc_len: f32,
    normal_w: f32,
    edge_falloff: f32,
    total_blend: f32,
}

impl PixelContributionAccumulator {
    fn add(&mut self, c: &PixelContribution, w: f32) {
        self.weight_sum += w;
        self.t_norm += c.t_norm * w;
        self.tx += c.tx * w;
        self.ty += c.ty * w;
        self.ambiguity += c.ambiguity * w;
        self.tangent_offset += c.tangent_offset * w;
        self.effective_range += c.effective_range * w;
        self.side_u += c.side_u * w;
        self.thickness_factor += c.thickness_factor * w;
        self.arc_len += c.arc_len * w;
        self.normal_w += c.normal_w * w;
        self.edge_falloff += c.edge_falloff * w;
        self.total_blend += c.total_blend * w;
    }

    fn finish(self) -> Option<PixelContribution> {
        if self.weight_sum <= 1e-6 {
            return None;
        }
        let inv_w = 1.0 / self.weight_sum;
        let (tx, ty) = normalize2(self.tx * inv_w, self.ty * inv_w);
        Some(PixelContribution {
            t_norm: (self.t_norm * inv_w).clamp(0.0, 1.0),
            tx,
            ty,
            ambiguity: (self.ambiguity * inv_w).clamp(0.0, 1.0),
            tangent_offset: self.tangent_offset * inv_w,
            effective_range: (self.effective_range * inv_w).max(0.001),
            side_u: (self.side_u * inv_w).clamp(0.0, 1.0),
            thickness_factor: (self.thickness_factor * inv_w).clamp(0.0, 1.0),
            arc_len: (self.arc_len * inv_w).max(1.0),
            normal_w: (self.normal_w * inv_w).clamp(0.0, 1.0),
            edge_falloff: (self.edge_falloff * inv_w).clamp(0.0, 1.0),
            total_blend: (self.total_blend * inv_w).clamp(0.0, 1.0),
        })
    }
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
                d.set_options(&["Final", "NormalMat", "TangentMat", "Fractal", "Taper"]);
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
        params.add(
            Params::AntialiasingQuality,
            "Antialiasing Quality",
            PopupDef::setup(|d| {
                d.set_options(&["Non", "Low", "High"]);
                d.set_default(3);
            }),
        )?;
        params.add(
            Params::NormalSide,
            "Normal Side",
            PopupDef::setup(|d| {
                d.set_options(&["Positive", "Negative"]);
                d.set_default(1);
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
            Params::PathBlurAmount,
            "TangentAmount(+)",
            FloatSliderDef::setup(|d| {
                d.set_valid_min(0.0);
                d.set_valid_max(1024.0);
                d.set_slider_min(0.0);
                d.set_slider_max(200.0);
                d.set_default(36.0);
                d.set_precision(1);
            }),
        )?;
        params.add(
            Params::NegativeBlurAmount,
            "TangentAmount(-)",
            FloatSliderDef::setup(|d| {
                d.set_valid_min(0.0);
                d.set_valid_max(1024.0);
                d.set_slider_min(0.0);
                d.set_slider_max(200.0);
                d.set_default(0.0);
                d.set_precision(1);
            }),
        )?;
        params.add(
            Params::PathBlurOffset,
            "TangentOffset",
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
            Params::NormalBandGroupStart,
            Params::NormalBandGroupEnd,
            "NormalControls",
            true,
            |params| {
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
                        d.set_default(50.0);
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
                Ok(())
            },
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
            Params::OffsetEndFade,
            "Subtraction Alpha",
            FloatSliderDef::setup(|d| {
                d.set_valid_min(0.0);
                d.set_valid_max(100.0);
                d.set_slider_min(0.0);
                d.set_slider_max(100.0);
                d.set_default(0.0);
                d.set_precision(1);
            }),
        )?;
        params.add_group(
            Params::EdgePreserveGroupStart,
            Params::EdgePreserveGroupEnd,
            "Master Intensity",
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
                        d.set_default(100.0);
                        d.set_precision(1);
                    }),
                )?;
                params.add(
                    Params::DarkExpandThreshold,
                    "Dark Expand Threshold",
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
                    Params::DarkExpandRadius,
                    "Dark Expand Radius",
                    FloatSliderDef::setup(|d| {
                        d.set_valid_min(0.0);
                        d.set_valid_max(128.0);
                        d.set_slider_min(0.0);
                        d.set_slider_max(32.0);
                        d.set_default(0.0);
                        d.set_precision(2);
                    }),
                )?;
                params.add(
                    Params::PreserveMatteEdge,
                    "Preserve Matte Edge",
                    CheckBoxDef::setup(|d| {
                        d.set_default(true);
                    }),
                )?;
                params.add(
                    Params::EdgeDisplaceMultiplier,
                    "Displace Multiplier",
                    FloatSliderDef::setup(|d| {
                        d.set_valid_min(0.0);
                        d.set_valid_max(400.0);
                        d.set_slider_min(0.0);
                        d.set_slider_max(200.0);
                        d.set_default(100.0);
                        d.set_precision(1);
                    }),
                )?;
                params.add(
                    Params::EdgeBlurMultiplier,
                    "Blur Multiplier",
                    FloatSliderDef::setup(|d| {
                        d.set_valid_min(0.0);
                        d.set_valid_max(400.0);
                        d.set_slider_min(0.0);
                        d.set_slider_max(200.0);
                        d.set_default(10.0);
                        d.set_precision(1);
                    }),
                )?;
                params.add(
                    Params::EdgeGhostMultiplier,
                    "Ghost Multiplier",
                    FloatSliderDef::setup(|d| {
                        d.set_valid_min(0.0);
                        d.set_valid_max(400.0);
                        d.set_slider_min(0.0);
                        d.set_slider_max(200.0);
                        d.set_default(100.0);
                        d.set_precision(1);
                    }),
                )?;
                params.add(
                    Params::EdgeGhostAlpha,
                    "Ghost Alpha",
                    FloatSliderDef::setup(|d| {
                        d.set_valid_min(0.0);
                        d.set_valid_max(100.0);
                        d.set_slider_min(0.0);
                        d.set_slider_max(100.0);
                        d.set_default(0.0);
                        d.set_precision(1);
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
                    Params::FractalScale,
                    "Fractal Scale",
                    FloatSliderDef::setup(|d| {
                        d.set_valid_min(0.1);
                        d.set_valid_max(2048.0);
                        d.set_slider_min(1.0);
                        d.set_slider_max(256.0);
                        d.set_default(15.0);
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
                        d.set_default(5.0);
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
                        d.set_default(15.0);
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
                        d.set_default(100.0);
                        d.set_precision(1);
                    }),
                )?;
                params.add(
                    Params::AddColor,
                    "Color",
                    ColorDef::setup(|d| {
                        d.set_default(ae::Pixel8 {
                            alpha: 255,
                            red: 128,
                            green: 128,
                            blue: 128,
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
                        d.set_default(4);
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
                        d.set_default(4);
                    }),
                )?;
                Ok(())
            },
        )?;

        params.add_with_flags(
            Params::TaperMode,
            "Taper Mode",
            PopupDef::setup(|d| {
                d.set_options(&["Non", "SimpleTaper", "ProfileTaper(Curve)"]);
                d.set_default(1);
            }),
            ParamFlag::SUPERVISE,
            ParamUIFlags::NONE,
        )?;

        params.add_group(
            Params::TaperGroupStart,
            Params::TaperGroupEnd,
            "Simple Taper",
            true,
            |params| {
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
                enable_update_params_ui(&mut out_data);
                out_data.set_out_flag(OutFlags::NonParamVary, true);
            }
            ae::Command::UpdateParamsUi => {
                let enable_tangent_falloff = params
                    .get(Params::EnableTangentFalloff)?
                    .as_checkbox()?
                    .value();
                let taper_mode = params.get(Params::TaperMode)?.as_popup()?.value();
                let enable_taper = taper_mode == 2;
                let tangent_rules = [
                    Params::TangentStartFallOff,
                    Params::TangentEndFallOff,
                    Params::TangentFalloffBias,
                ]
                .map(|param| (param, enable_tangent_falloff));
                apply_disabled(params, &tangent_rules)?;

                let taper_rules = [
                    Params::StartTaperLength,
                    Params::StartTaperCurve,
                    Params::EndTaperLength,
                    Params::EndTaperCurve,
                    Params::TaperSCurve,
                ]
                .map(|param| (param, enable_taper));
                apply_disabled(params, &taper_rules)?;
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
        let tangent_amount = params
            .get(Params::PathBlurAmount)?
            .as_float_slider()?
            .value() as f32;
        let negative_tangent_amount = params
            .get(Params::NegativeBlurAmount)?
            .as_float_slider()?
            .value() as f32;
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
        let taper_mode = params.get(Params::TaperMode)?.as_popup()?.value();
        let enable_taper = taper_mode == 2;
        let enable_profile = taper_mode == 3;
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
        let dark_expand_threshold = params
            .get(Params::DarkExpandThreshold)?
            .as_float_slider()?
            .value() as f32
            / 100.0;
        let dark_expand_radius = params
            .get(Params::DarkExpandRadius)?
            .as_float_slider()?
            .value() as f32;
        let preserve_matte_edge = params
            .get(Params::PreserveMatteEdge)?
            .as_checkbox()?
            .value();
        let fract_scale = params.get(Params::FractalScale)?.as_float_slider()?.value() as f32;
        let fract_complexity = params
            .get(Params::FractalComplexity)?
            .as_float_slider()?
            .value() as f32
            / 100.0;
        let evolution = params.get(Params::Evolution)?.as_angle()?.float_value()? as f32;
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
        let offset_end_fade = params
            .get(Params::OffsetEndFade)?
            .as_float_slider()?
            .value() as f32
            / 100.0;
        let antialiasing_quality = params.get(Params::AntialiasingQuality)?.as_popup()?.value();
        let edge_displace_multiplier = params
            .get(Params::EdgeDisplaceMultiplier)?
            .as_float_slider()?
            .value() as f32
            / 100.0;
        let edge_blur_multiplier = params
            .get(Params::EdgeBlurMultiplier)?
            .as_float_slider()?
            .value() as f32
            / 100.0;
        let edge_ghost_multiplier = params
            .get(Params::EdgeGhostMultiplier)?
            .as_float_slider()?
            .value() as f32
            / 100.0;
        let edge_ghost_alpha = params
            .get(Params::EdgeGhostAlpha)?
            .as_float_slider()?
            .value() as f32
            / 100.0;
        let sample_mode = (antialiasing_quality - 1).clamp(0, 2);

        let in_world = in_layer.world_type();
        let out_world = out_layer.world_type();
        let in_w = in_layer.width();
        let in_h = in_layer.height();
        let original_buffer = ImageBufferF32::from_layer(&in_layer, in_world, in_w, in_h);
        let progress_final = out_layer.height() as i32;
        let eval_cfg = EvalConfig {
            normal_range,
            center_line,
            normal_falloff,
            normal_bias,
            tangent_amount,
            negative_tangent_amount,
            enable_tangent_falloff,
            tangent_start_falloff,
            tangent_end_falloff,
            tangent_falloff_bias,
            enable_taper,
            taper_s_len,
            taper_s_curve,
            taper_e_len,
            taper_e_curve,
            taper_s_curve_enabled,
            enable_profile,
            normal_side,
            swap_tangent,
        };
        let dark_expand_cfg = if dark_expand_threshold > 0.0 && dark_expand_radius > 0.0 {
            Some(DarkExpandConfig {
                threshold: dark_expand_threshold.clamp(0.0, 1.0),
                radius: dark_expand_radius.ceil().max(1.0) as usize,
                preserve_matte_edge,
            })
        } else {
            None
        };
        let prefilled_source = dark_expand_cfg
            .as_ref()
            .map(|cfg| build_dark_expand_prefill(&original_buffer, &path_data, &eval_cfg, cfg));
        let source_buffer = prefilled_source.as_ref().unwrap_or(&original_buffer);

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
            let original = original_buffer.pixel_at(x as usize, y as usize);
            let contrib = if sample_mode == 2 {
                let mut accum = PixelContributionAccumulator::default();
                for (ox, oy, w) in HIGH_SUPERSAMPLE_OFFSETS {
                    if let Some(c) =
                        eval_pixel_contribution(&path_data, xf + ox, yf + oy, &eval_cfg)
                    {
                        accum.add(&c, w);
                    }
                }
                accum.finish()
            } else {
                eval_pixel_contribution(&path_data, xf, yf, &eval_cfg)
            };

            let Some(contrib) = contrib else {
                set_dst!(dst, original);
                return Ok(());
            };

            let normal_mat = (contrib.normal_w * contrib.ambiguity).clamp(0.0, 1.0);
            let tangent_mat = (contrib.edge_falloff * contrib.ambiguity).clamp(0.0, 1.0);
            let effect_mat = contrib.total_blend.clamp(0.0, 1.0);
            let end_fade_factor =
                1.0 - offset_end_fade.clamp(0.0, 1.0) * (1.0 - tangent_mat.clamp(0.0, 1.0));
            let effect_strength = (effect_mat * end_fade_factor).clamp(0.0, 1.0);
            let arc_len = contrib.arc_len.max(1.0);

            let evo = evolution * 0.05;
            let tangent_pos = contrib.t_norm * arc_len + contrib.tangent_offset;
            let fract_iso = (arc_len / contrib.effective_range.max(1.0))
                .sqrt()
                .clamp(0.25, 4.0);
            let centered_normal_distance = (contrib.side_u - center_line) * contrib.effective_range;
            let fract_x = tangent_pos / fract_scale.max(0.1) / fract_tangent_scale.max(0.01)
                + fract_tangent_offset;
            let fract_y = centered_normal_distance / fract_scale.max(0.1) * fract_iso;
            let fract_val = fractal_value_with_quality(
                fract_x,
                fract_y,
                fract_scale,
                fract_tangent_scale,
                fract_iso,
                fract_complexity,
                evo,
                sample_mode,
            );
            let fract_mask = ((1.0 - fract_amount) + fract_val * fract_amount).clamp(0.0, 1.0);

            if view_mode == 2 {
                let vis = gray_pixel(normal_mat);
                set_dst!(dst, vis);
                return Ok(());
            } else if view_mode == 3 {
                let vis = gray_pixel(tangent_mat);
                set_dst!(dst, vis);
                return Ok(());
            } else if view_mode == 4 {
                let vis = PixelF32 {
                    red: fract_mask,
                    green: fract_mask,
                    blue: fract_mask,
                    alpha: 1.0,
                };
                let col = lerp_pixel(&original, &vis, normal_mat);
                set_dst!(dst, col);
                return Ok(());
            } else if view_mode == 5 {
                let vis = PixelF32 {
                    red: contrib.thickness_factor,
                    green: contrib.thickness_factor,
                    blue: contrib.thickness_factor,
                    alpha: 1.0,
                };
                let col = lerp_pixel(&original, &vis, normal_mat);
                set_dst!(dst, col);
                return Ok(());
            }

            let (blur_tx, blur_ty) = if swap_tangent {
                (-contrib.tx, -contrib.ty)
            } else {
                (contrib.tx, contrib.ty)
            };
            let displace_scale = (fract_mask * edge_displace_multiplier * effect_strength).max(0.0);
            let blur_scale = (fract_mask * edge_blur_multiplier * effect_strength).max(0.0);
            let ghost_scale =
                (fract_mask * edge_ghost_multiplier * effect_strength).clamp(0.0, 1.0);
            let post_offset = path_offset * effect_strength;
            let cur_pos_amt = tangent_amount * blur_scale;
            let cur_neg_amt = negative_tangent_amount * blur_scale;
            let base_result = blur_along_tangent(&TangentBlurParams {
                source: source_buffer,
                center_x: xf,
                center_y: yf,
                tangent_x: blur_tx,
                tangent_y: blur_ty,
                positive_amount: cur_pos_amt,
                negative_amount: cur_neg_amt,
                positive_displace_amount: tangent_amount * displace_scale,
                negative_displace_amount: negative_tangent_amount * displace_scale,
                post_offset_amount: post_offset,
                sample_mode,
            });
            let mut col = base_result;
            let blend_strength = effect_strength;

            if ghost_scale > 0.001 && edge_ghost_alpha > 0.001 {
                let ghost_result = blur_along_tangent(&TangentBlurParams {
                    source: source_buffer,
                    center_x: xf,
                    center_y: yf,
                    tangent_x: blur_tx,
                    tangent_y: blur_ty,
                    positive_amount: tangent_amount * ghost_scale,
                    negative_amount: negative_tangent_amount * ghost_scale,
                    positive_displace_amount: tangent_amount * ghost_scale,
                    negative_displace_amount: negative_tangent_amount * ghost_scale,
                    post_offset_amount: post_offset,
                    sample_mode,
                });
                col = lerp_pixel(
                    &col,
                    &ghost_result,
                    (edge_ghost_alpha * effect_strength).clamp(0.0, 1.0),
                );
            }

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
                    add_color_opacity * blend_strength,
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
        let taper_mode = params.get(Params::TaperMode)?.as_popup()?.value();
        let enable_profile = taper_mode == 3;

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
fn eval_pixel_contribution(
    path_data: &PathData,
    x: f32,
    y: f32,
    cfg: &EvalConfig,
) -> Option<PixelContribution> {
    let mut chosen: Option<(Nearest, f32, f32, f32, f32, f32, f32)> = None;
    let mut best_effect_blend = 0.0_f32;

    for mask in &path_data.masks {
        if mask.samples.is_empty() {
            continue;
        }

        let nearest = nearest_sample(&mask.samples, x, y);
        let taper_thickness = if cfg.enable_taper {
            taper_factor(
                nearest.t_norm,
                cfg.taper_s_len,
                cfg.taper_s_curve,
                cfg.taper_e_len,
                cfg.taper_e_curve,
                cfg.taper_s_curve_enabled,
            )
        } else {
            1.0
        };
        let profile_thickness = if cfg.enable_profile {
            profile_multiplier(
                path_data.profile_curve.as_ref(),
                nearest.t_norm,
                cfg.swap_tangent,
            )
        } else {
            1.0
        };
        let thickness_factor = (taper_thickness * profile_thickness).clamp(0.0, 1.0);
        let Some(side_u_full) =
            selected_normal_side_u(nearest.distance, cfg.normal_range, cfg.normal_side)
        else {
            continue;
        };
        let Some((side_u, band_width_u)) =
            remap_side_u_around_centerline(side_u_full, thickness_factor, cfg.center_line)
        else {
            continue;
        };
        let effective_range = (cfg.normal_range * band_width_u).max(0.001);

        let arc_len = mask.arc_len.max(1.0);
        let edge_zone_start = if cfg.enable_tangent_falloff {
            cfg.tangent_start_falloff.clamp(0.0, 1.0)
        } else {
            (cfg.negative_tangent_amount / arc_len).clamp(0.01, 0.5)
        };
        let edge_zone_end = if cfg.enable_tangent_falloff {
            cfg.tangent_end_falloff.clamp(0.0, 1.0)
        } else {
            (cfg.tangent_amount / arc_len).clamp(0.01, 0.5)
        };

        let at_start = nearest.best_t_norm < 0.01 && nearest.best_tangent_offset < 0.0;
        let at_end = nearest.best_t_norm > 0.99 && nearest.best_tangent_offset > 0.0;
        if at_start || at_end {
            continue;
        }

        let normal_w =
            normal_band_weight(side_u, cfg.center_line, cfg.normal_falloff, cfg.normal_bias);
        if normal_w < 0.001 {
            continue;
        }
        let edge_falloff = edge_fade_asymmetric(
            nearest.t_norm,
            edge_zone_start,
            edge_zone_end,
            cfg.tangent_falloff_bias,
        );
        if edge_falloff < 0.01 {
            continue;
        }

        let blend_i = (normal_w * edge_falloff * nearest.ambiguity).clamp(0.0, 1.0);
        if blend_i > best_effect_blend {
            chosen = Some((
                nearest,
                arc_len,
                effective_range,
                normal_w,
                edge_falloff,
                side_u,
                thickness_factor,
            ));
            best_effect_blend = blend_i;
        }
        if best_effect_blend >= 1.0 - 1e-6 {
            break;
        }
    }

    chosen.map(
        |(nearest, arc_len, effective_range, normal_w, edge_falloff, side_u, thickness_factor)| {
            PixelContribution {
                t_norm: nearest.t_norm,
                tx: nearest.tx,
                ty: nearest.ty,
                ambiguity: nearest.ambiguity,
                tangent_offset: nearest.tangent_offset,
                effective_range,
                side_u,
                thickness_factor,
                arc_len,
                normal_w,
                edge_falloff,
                total_blend: best_effect_blend.min(1.0),
            }
        },
    )
}

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
    source: &'a ImageBufferF32,
    center_x: f32,
    center_y: f32,
    tangent_x: f32,
    tangent_y: f32,
    positive_amount: f32,
    negative_amount: f32,
    positive_displace_amount: f32,
    negative_displace_amount: f32,
    post_offset_amount: f32,
    sample_mode: i32,
}

fn blur_along_tangent(p: &TangentBlurParams<'_>) -> PixelF32 {
    let pos_r = p.positive_amount.max(0.0);
    let neg_r = p.negative_amount.max(0.0);
    let total = pos_r + neg_r;
    let pos_disp = p.positive_displace_amount.max(0.0);
    let neg_disp = p.negative_displace_amount.max(0.0);

    let offset_center_x = p.center_x + p.tangent_x * p.post_offset_amount;
    let offset_center_y = p.center_y + p.tangent_y * p.post_offset_amount;
    let center_shift = (pos_disp - neg_disp) * 0.5;
    let blur_center_x = offset_center_x + p.tangent_x * center_shift;
    let blur_center_y = offset_center_y + p.tangent_y * center_shift;
    let center_px =
        sample_buffer_with_quality(p.source, offset_center_x, offset_center_y, p.sample_mode);
    let pos_px = if pos_disp > 0.001 {
        sample_buffer_with_quality(
            p.source,
            offset_center_x + p.tangent_x * pos_disp,
            offset_center_y + p.tangent_y * pos_disp,
            p.sample_mode,
        )
    } else {
        center_px
    };
    let neg_px = if neg_disp > 0.001 {
        sample_buffer_with_quality(
            p.source,
            offset_center_x - p.tangent_x * neg_disp,
            offset_center_y - p.tangent_y * neg_disp,
            p.sample_mode,
        )
    } else {
        center_px
    };
    let displaced = match (pos_disp > 0.001, neg_disp > 0.001) {
        (true, true) => {
            let mix = pos_disp / (pos_disp + neg_disp).max(1e-6);
            lerp_pixel(&neg_px, &pos_px, mix)
        }
        (true, false) => pos_px,
        (false, true) => neg_px,
        (false, false) => center_px,
    };

    let taps = if total < 0.25 {
        1
    } else {
        ((total.ceil() as i32) * 4 + 1).max(5)
    };
    let mut blur_sum = PixelF32 {
        alpha: 0.0,
        red: 0.0,
        green: 0.0,
        blue: 0.0,
    };
    let mut blur_wsum = 0.0_f32;

    let pos_sigma = (pos_r / 3.0).max(0.001);
    let neg_sigma = (neg_r / 3.0).max(0.001);

    for i in 0..taps {
        let offset = if taps <= 1 {
            0.0
        } else {
            let t = i as f32 / (taps - 1) as f32;
            -neg_r + t * total
        };
        let sigma = if offset < 0.0 { neg_sigma } else { pos_sigma };
        let gaussian_w = if total < 0.25 {
            1.0
        } else {
            (-0.5 * (offset / sigma).powi(2)).exp()
        };
        let sx = blur_center_x + p.tangent_x * offset;
        let sy = blur_center_y + p.tangent_y * offset;
        let px = sample_buffer_with_quality(p.source, sx, sy, p.sample_mode);
        let detail_w = (1.15 - color_distance(&px, &displaced) * 0.65).clamp(0.35, 1.25);
        let center_w = if total < 0.25 {
            1.0
        } else {
            (1.0 - (offset.abs() / total.max(1.0)) * 0.35).clamp(0.65, 1.0)
        };
        let blur_w = gaussian_w * detail_w * center_w;
        let premult_px = to_premultiplied(px);
        blur_sum.alpha += premult_px.alpha * blur_w;
        blur_sum.red += premult_px.red * blur_w;
        blur_sum.green += premult_px.green * blur_w;
        blur_sum.blue += premult_px.blue * blur_w;
        blur_wsum += blur_w;
        if total < 0.25 {
            break;
        }
    }

    if blur_wsum > 0.0 {
        from_premultiplied(PixelF32 {
            alpha: blur_sum.alpha / blur_wsum,
            red: blur_sum.red / blur_wsum,
            green: blur_sum.green / blur_wsum,
            blue: blur_sum.blue / blur_wsum,
        })
    } else {
        displaced
    }
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

fn remap_side_u_around_centerline(
    side_u_full: f32,
    thickness_factor: f32,
    center_line: f32,
) -> Option<(f32, f32)> {
    let thickness = thickness_factor.clamp(0.0, 1.0);
    if thickness <= 1e-4 {
        return None;
    }

    let center = center_line.clamp(0.0, 1.0);
    let band_start = center * (1.0 - thickness);
    let band_end = center + (1.0 - center) * thickness;
    if side_u_full < band_start || side_u_full > band_end {
        return None;
    }

    let band_width = (band_end - band_start).max(1e-4);
    let local_u = ((side_u_full - band_start) / band_width).clamp(0.0, 1.0);
    Some((local_u, band_width))
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

fn profile_multiplier(curve: Option<&ProfileCurve>, t_norm: f32, swap_tangent: bool) -> f32 {
    let t = if swap_tangent { 1.0 - t_norm } else { t_norm };
    sample_profile_y(curve, t.clamp(0.0, 1.0)).clamp(0.0, 1.0)
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

fn gray_pixel(v: f32) -> PixelF32 {
    let g = v.clamp(0.0, 1.0);
    PixelF32 {
        alpha: 1.0,
        red: g,
        green: g,
        blue: g,
    }
}

fn color_distance(a: &PixelF32, b: &PixelF32) -> f32 {
    let dr = a.red - b.red;
    let dg = a.green - b.green;
    let db = a.blue - b.blue;
    let da = a.alpha - b.alpha;
    ((dr * dr + dg * dg + db * db + da * da).sqrt() * 0.5).clamp(0.0, 1.0)
}

fn to_premultiplied(pixel: PixelF32) -> PixelF32 {
    let a = pixel.alpha.clamp(0.0, 1.0);
    PixelF32 {
        alpha: a,
        red: pixel.red.clamp(0.0, 1.0) * a,
        green: pixel.green.clamp(0.0, 1.0) * a,
        blue: pixel.blue.clamp(0.0, 1.0) * a,
    }
}

fn from_premultiplied(pixel: PixelF32) -> PixelF32 {
    let a = pixel.alpha.clamp(0.0, 1.0);
    if a <= 1e-6 {
        return PixelF32 {
            alpha: 0.0,
            red: 0.0,
            green: 0.0,
            blue: 0.0,
        };
    }
    let inv_a = 1.0 / a;
    PixelF32 {
        alpha: a,
        red: (pixel.red * inv_a).clamp(0.0, 1.0),
        green: (pixel.green * inv_a).clamp(0.0, 1.0),
        blue: (pixel.blue * inv_a).clamp(0.0, 1.0),
    }
}

impl ImageBufferF32 {
    fn from_layer(
        layer: &Layer,
        world_type: ae::aegp::WorldType,
        width: usize,
        height: usize,
    ) -> Self {
        let mut pixels = Vec::with_capacity(width * height);
        for y in 0..height {
            for x in 0..width {
                pixels.push(read_pixel_f32(layer, world_type, x, y));
            }
        }
        Self {
            width,
            height,
            pixels,
        }
    }

    fn pixel_at(&self, x: usize, y: usize) -> PixelF32 {
        self.pixels[y * self.width + x]
    }

    fn set_pixel(&mut self, x: usize, y: usize, pixel: PixelF32) {
        self.pixels[y * self.width + x] = pixel;
    }
}

fn luminance(pixel: &PixelF32) -> f32 {
    (pixel.red * 0.2126 + pixel.green * 0.7152 + pixel.blue * 0.0722).clamp(0.0, 1.0)
}

fn premult_luminance(pixel: &PixelF32) -> f32 {
    let p = to_premultiplied(*pixel);
    luminance(&p)
}

#[derive(Clone, Copy)]
struct DarkExpandCandidate {
    luma: f32,
    pixel: PixelF32,
}

fn build_dark_expand_prefill(
    source: &ImageBufferF32,
    path_data: &PathData,
    eval_cfg: &EvalConfig,
    dark_cfg: &DarkExpandConfig,
) -> ImageBufferF32 {
    let matte_mask = build_effect_mask(source.width, source.height, path_data, eval_cfg);
    let mut out = source.clone();
    let radius = dark_cfg.radius;
    let edge_radius = radius.min(2);
    let edge_offsets = circular_offsets(edge_radius);
    let mut premult_luma = vec![0.0_f32; source.width * source.height];
    let mut candidates = vec![invalid_dark_expand_candidate(); source.width * source.height];
    for y in 0..source.height {
        for x in 0..source.width {
            let idx = y * source.width + x;
            let pixel = source.pixel_at(x, y);
            premult_luma[idx] = premult_luminance(&pixel);
            if matte_mask[idx] && pixel.alpha > 1e-4 {
                candidates[idx] = DarkExpandCandidate {
                    luma: premult_luma[idx],
                    pixel,
                };
            }
        }
    }
    let best_candidates = max_filter_candidates(&candidates, source.width, source.height, radius);
    let mut targets: Vec<(usize, usize)> = Vec::new();
    for y in 0..source.height {
        for x in 0..source.width {
            let idx = y * source.width + x;
            if matte_mask[idx] && premult_luma[idx] <= dark_cfg.threshold {
                targets.push((x, y));
            }
        }
    }
    if targets.is_empty() {
        return out;
    }
    let mut replaced: Vec<(usize, usize, PixelF32)> = Vec::new();
    for (x, y) in targets {
        if dark_cfg.preserve_matte_edge
            && touches_matte_edge_with_offsets(
                &matte_mask,
                source.width,
                source.height,
                x,
                y,
                &edge_offsets,
            )
        {
            continue;
        }
        let idx = y * source.width + x;
        let current_luma = premult_luma[idx];
        let best = best_candidates[idx];

        if best.luma > current_luma + 1e-6 {
            out.set_pixel(x, y, best.pixel);
            replaced.push((x, y, best.pixel));
        }
    }
    if replaced.is_empty() {
        return out;
    }

    let mut extension_candidates =
        vec![invalid_dark_expand_candidate(); source.width * source.height];
    for (x, y, fill) in replaced {
        let idx = y * source.width + x;
        extension_candidates[idx] = DarkExpandCandidate {
            luma: premult_luminance(&fill),
            pixel: fill,
        };
    }
    let extension_map =
        max_filter_candidates(&extension_candidates, source.width, source.height, radius);
    for y in 0..source.height {
        for x in 0..source.width {
            let idx = y * source.width + x;
            if matte_mask[idx] || extension_map[idx].luma < 0.0 {
                continue;
            }
            let fill = extension_map[idx].pixel;
            let mut outer = out.pixel_at(x, y);
            outer.red = fill.red;
            outer.green = fill.green;
            outer.blue = fill.blue;
            out.set_pixel(x, y, outer);
        }
    }

    out
}

fn invalid_dark_expand_candidate() -> DarkExpandCandidate {
    DarkExpandCandidate {
        luma: -1.0,
        pixel: PixelF32 {
            alpha: 0.0,
            red: 0.0,
            green: 0.0,
            blue: 0.0,
        },
    }
}

fn max_filter_candidates(
    input: &[DarkExpandCandidate],
    width: usize,
    height: usize,
    radius: usize,
) -> Vec<DarkExpandCandidate> {
    if width == 0 || height == 0 || radius == 0 {
        return input.to_vec();
    }

    let mut horizontal = vec![invalid_dark_expand_candidate(); width * height];
    for y in 0..height {
        let mut deque: VecDeque<usize> = VecDeque::new();
        let mut right = 0_usize;
        for x in 0..width {
            let max_right = (x + radius).min(width - 1);
            while right <= max_right {
                let idx = y * width + right;
                while deque
                    .back()
                    .map(|&back| input[y * width + back].luma <= input[idx].luma)
                    .unwrap_or(false)
                {
                    deque.pop_back();
                }
                deque.push_back(right);
                right += 1;
            }

            let min_left = x.saturating_sub(radius);
            while deque
                .front()
                .map(|&front| front < min_left)
                .unwrap_or(false)
            {
                deque.pop_front();
            }

            if let Some(&best_x) = deque.front() {
                horizontal[y * width + x] = input[y * width + best_x];
            }
        }
    }

    let mut output = vec![invalid_dark_expand_candidate(); width * height];
    for x in 0..width {
        let mut deque: VecDeque<usize> = VecDeque::new();
        let mut bottom = 0_usize;
        for y in 0..height {
            let max_bottom = (y + radius).min(height - 1);
            while bottom <= max_bottom {
                let idx = bottom * width + x;
                while deque
                    .back()
                    .map(|&back| horizontal[back * width + x].luma <= horizontal[idx].luma)
                    .unwrap_or(false)
                {
                    deque.pop_back();
                }
                deque.push_back(bottom);
                bottom += 1;
            }

            let min_top = y.saturating_sub(radius);
            while deque.front().map(|&front| front < min_top).unwrap_or(false) {
                deque.pop_front();
            }

            if let Some(&best_y) = deque.front() {
                output[y * width + x] = horizontal[best_y * width + x];
            }
        }
    }

    output
}

fn build_effect_mask(
    width: usize,
    height: usize,
    path_data: &PathData,
    eval_cfg: &EvalConfig,
) -> Vec<bool> {
    let mut mask = vec![false; width * height];
    for y in 0..height {
        for x in 0..width {
            let inside =
                eval_pixel_contribution(path_data, x as f32 + 0.5, y as f32 + 0.5, eval_cfg)
                    .map(|c| c.total_blend > 0.001)
                    .unwrap_or(false);
            mask[y * width + x] = inside;
        }
    }
    mask
}

fn touches_matte_edge_with_offsets(
    matte_mask: &[bool],
    width: usize,
    height: usize,
    x: usize,
    y: usize,
    offsets: &[(isize, isize)],
) -> bool {
    for (dx, dy) in offsets {
        let nx = x as isize + dx;
        let ny = y as isize + dy;
        if nx < 0 || ny < 0 || nx >= width as isize || ny >= height as isize {
            continue;
        }
        if !matte_mask[ny as usize * width + nx as usize] {
            return true;
        }
    }
    false
}

fn circular_offsets(radius: usize) -> Vec<(isize, isize)> {
    if radius == 0 {
        return vec![(0, 0)];
    }
    let radius_i = radius as isize;
    let radius_sq = (radius * radius) as isize;
    let mut out: Vec<(isize, isize)> = Vec::new();
    for dy in -radius_i..=radius_i {
        for dx in -radius_i..=radius_i {
            if dx == 0 && dy == 0 {
                continue;
            }
            if dx * dx + dy * dy <= radius_sq {
                out.push((dx, dy));
            }
        }
    }
    out
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

fn sample_buffer_bilinear(source: &ImageBufferF32, x: f32, y: f32) -> PixelF32 {
    if source.width == 0 || source.height == 0 {
        return PixelF32 {
            alpha: 0.0,
            red: 0.0,
            green: 0.0,
            blue: 0.0,
        };
    }
    let fx = x.clamp(0.0, (source.width.saturating_sub(1)) as f32);
    let fy = y.clamp(0.0, (source.height.saturating_sub(1)) as f32);
    let x0 = fx.floor() as usize;
    let y0 = fy.floor() as usize;
    let x1 = (x0 + 1).min(source.width.saturating_sub(1));
    let y1 = (y0 + 1).min(source.height.saturating_sub(1));
    let tx = fx - x0 as f32;
    let ty = fy - y0 as f32;

    let p00 = to_premultiplied(source.pixel_at(x0, y0));
    let p10 = to_premultiplied(source.pixel_at(x1, y0));
    let p01 = to_premultiplied(source.pixel_at(x0, y1));
    let p11 = to_premultiplied(source.pixel_at(x1, y1));

    let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t;
    let premult = PixelF32 {
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
    };
    from_premultiplied(premult)
}

fn sample_buffer_point(source: &ImageBufferF32, x: f32, y: f32) -> PixelF32 {
    if source.width == 0 || source.height == 0 {
        return PixelF32 {
            alpha: 0.0,
            red: 0.0,
            green: 0.0,
            blue: 0.0,
        };
    }

    let xi = x
        .round()
        .clamp(0.0, (source.width.saturating_sub(1)) as f32) as usize;
    let yi = y
        .round()
        .clamp(0.0, (source.height.saturating_sub(1)) as f32) as usize;
    source.pixel_at(xi, yi)
}

fn sample_buffer_with_quality(
    source: &ImageBufferF32,
    x: f32,
    y: f32,
    sample_mode: i32,
) -> PixelF32 {
    match sample_mode {
        0 => sample_buffer_point(source, x, y),
        2 => {
            let mut sum = PixelF32 {
                alpha: 0.0,
                red: 0.0,
                green: 0.0,
                blue: 0.0,
            };
            let mut wsum = 0.0_f32;
            for (ox, oy, w) in HIGH_SUPERSAMPLE_OFFSETS {
                let px = to_premultiplied(sample_buffer_bilinear(source, x + ox, y + oy));
                sum.alpha += px.alpha * w;
                sum.red += px.red * w;
                sum.green += px.green * w;
                sum.blue += px.blue * w;
                wsum += w;
            }
            let premult = PixelF32 {
                alpha: sum.alpha / wsum.max(1e-6),
                red: sum.red / wsum.max(1e-6),
                green: sum.green / wsum.max(1e-6),
                blue: sum.blue / wsum.max(1e-6),
            };
            from_premultiplied(premult)
        }
        _ => sample_buffer_bilinear(source, x, y),
    }
}

fn fractal_value_with_quality(
    fract_x: f32,
    fract_y: f32,
    fract_scale: f32,
    fract_tangent_scale: f32,
    fract_iso: f32,
    fract_complexity: f32,
    evo: f32,
    sample_mode: i32,
) -> f32 {
    let x_step = 0.5 / fract_scale.max(0.1) / fract_tangent_scale.max(0.01);
    let y_step = 0.5 / fract_scale.max(0.1) * fract_iso;
    let offsets: &[(f32, f32, f32)] = match sample_mode {
        2 => &HIGH_SUPERSAMPLE_OFFSETS,
        1 => &LOW_SUPERSAMPLE_OFFSETS,
        _ => &SINGLE_SAMPLE_OFFSET,
    };

    let mut sum = 0.0_f32;
    let mut wsum = 0.0_f32;
    for (ox, oy, w) in offsets {
        sum += voronoi_2d(
            fract_x + ox * x_step,
            fract_y + oy * y_step,
            fract_complexity,
            evo,
        ) * *w;
        wsum += *w;
    }
    (sum / wsum.max(1e-6)).clamp(0.0, 1.0)
}

fn read_pixel_f32(layer: &Layer, world_type: ae::aegp::WorldType, x: usize, y: usize) -> PixelF32 {
    match world_type {
        ae::aegp::WorldType::U8 => layer.as_pixel8(x, y).to_pixel32(),
        ae::aegp::WorldType::U15 => layer.as_pixel16(x, y).to_pixel32(),
        ae::aegp::WorldType::F32 | ae::aegp::WorldType::None => *layer.as_pixel32(x, y),
    }
}
