#![allow(clippy::drop_non_drop, clippy::question_mark, dead_code)]

use after_effects as ae;
use std::env;

use ae::pf::*;
use utils::ToPixel;

// ---------------------------------------------------------------------------
// Output (always visible)

// ---------------------------------------------------------------------------
const EXTRACTION_SETS: usize = 8;
const MERGE_ISLAND_SETS: usize = 32;
const GRADIENT_SETS: usize = 32;

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
enum Params {
    // 0. Output Settings
    OutputMode,

    // 1. Color Extraction group
    ColorExtGroupStart,
    ColorExtGroupEnd,
    InvertExtraction,
    ExtractionCount,
    // TargetColor[0..EXTRACTION_SETS], ColorRange[0..EXTRACTION_SETS]
    TargetColor0,
    ColorRange0,
    TargetColor1,
    ColorRange1,
    TargetColor2,
    ColorRange2,
    TargetColor3,
    ColorRange3,
    TargetColor4,
    ColorRange4,
    TargetColor5,
    ColorRange5,
    TargetColor6,
    ColorRange6,
    TargetColor7,
    ColorRange7,
    ChokeSpread,

    // 2. Island Tracking & Temp Colors group
    IslandTrackGroupStart,
    IslandTrackGroupEnd,
    TrackingPath,
    ShowTempColors,
    MergeIslandCount,
    // SourceTempColor[n], TargetTempColor[n] x MERGE_ISLAND_SETS
    SourceTempColor0,
    TargetTempColor0,
    SourceTempColor1,
    TargetTempColor1,
    SourceTempColor2,
    TargetTempColor2,
    SourceTempColor3,
    TargetTempColor3,
    SourceTempColor4,
    TargetTempColor4,
    SourceTempColor5,
    TargetTempColor5,
    SourceTempColor6,
    TargetTempColor6,
    SourceTempColor7,
    TargetTempColor7,
    SourceTempColor8,
    TargetTempColor8,
    SourceTempColor9,
    TargetTempColor9,
    SourceTempColor10,
    TargetTempColor10,
    SourceTempColor11,
    TargetTempColor11,
    SourceTempColor12,
    TargetTempColor12,
    SourceTempColor13,
    TargetTempColor13,
    SourceTempColor14,
    TargetTempColor14,
    SourceTempColor15,
    TargetTempColor15,
    SourceTempColor16,
    TargetTempColor16,
    SourceTempColor17,
    TargetTempColor17,
    SourceTempColor18,
    TargetTempColor18,
    SourceTempColor19,
    TargetTempColor19,
    SourceTempColor20,
    TargetTempColor20,
    SourceTempColor21,
    TargetTempColor21,
    SourceTempColor22,
    TargetTempColor22,
    SourceTempColor23,
    TargetTempColor23,
    SourceTempColor24,
    TargetTempColor24,
    SourceTempColor25,
    TargetTempColor25,
    SourceTempColor26,
    TargetTempColor26,
    SourceTempColor27,
    TargetTempColor27,
    SourceTempColor28,
    TargetTempColor28,
    SourceTempColor29,
    TargetTempColor29,
    SourceTempColor30,
    TargetTempColor30,
    SourceTempColor31,
    TargetTempColor31,

    // 3. Gradient Render group
    GradientGroupStart,
    GradientGroupEnd,
    GradientSettingsCount,
    MasterAngle,
    // GradType, StartColor, EndColor, InvertGradient, Bias, Offset, NoiseAmount x GRADIENT_SETS
    GradType0,
    StartColor0,
    EndColor0,
    InvertGradient0,
    Bias0,
    Offset0,
    NoiseAmount0,
    GradType1,
    StartColor1,
    EndColor1,
    InvertGradient1,
    Bias1,
    Offset1,
    NoiseAmount1,
    GradType2,
    StartColor2,
    EndColor2,
    InvertGradient2,
    Bias2,
    Offset2,
    NoiseAmount2,
    GradType3,
    StartColor3,
    EndColor3,
    InvertGradient3,
    Bias3,
    Offset3,
    NoiseAmount3,
    GradType4,
    StartColor4,
    EndColor4,
    InvertGradient4,
    Bias4,
    Offset4,
    NoiseAmount4,
    GradType5,
    StartColor5,
    EndColor5,
    InvertGradient5,
    Bias5,
    Offset5,
    NoiseAmount5,
    GradType6,
    StartColor6,
    EndColor6,
    InvertGradient6,
    Bias6,
    Offset6,
    NoiseAmount6,
    GradType7,
    StartColor7,
    EndColor7,
    InvertGradient7,
    Bias7,
    Offset7,
    NoiseAmount7,
    GradType8,
    StartColor8,
    EndColor8,
    InvertGradient8,
    Bias8,
    Offset8,
    NoiseAmount8,
    GradType9,
    StartColor9,
    EndColor9,
    InvertGradient9,
    Bias9,
    Offset9,
    NoiseAmount9,
    GradType10,
    StartColor10,
    EndColor10,
    InvertGradient10,
    Bias10,
    Offset10,
    NoiseAmount10,
    GradType11,
    StartColor11,
    EndColor11,
    InvertGradient11,
    Bias11,
    Offset11,
    NoiseAmount11,
    GradType12,
    StartColor12,
    EndColor12,
    InvertGradient12,
    Bias12,
    Offset12,
    NoiseAmount12,
    GradType13,
    StartColor13,
    EndColor13,
    InvertGradient13,
    Bias13,
    Offset13,
    NoiseAmount13,
    GradType14,
    StartColor14,
    EndColor14,
    InvertGradient14,
    Bias14,
    Offset14,
    NoiseAmount14,
    GradType15,
    StartColor15,
    EndColor15,
    InvertGradient15,
    Bias15,
    Offset15,
    NoiseAmount15,
    GradType16,
    StartColor16,
    EndColor16,
    InvertGradient16,
    Bias16,
    Offset16,
    NoiseAmount16,
    GradType17,
    StartColor17,
    EndColor17,
    InvertGradient17,
    Bias17,
    Offset17,
    NoiseAmount17,
    GradType18,
    StartColor18,
    EndColor18,
    InvertGradient18,
    Bias18,
    Offset18,
    NoiseAmount18,
    GradType19,
    StartColor19,
    EndColor19,
    InvertGradient19,
    Bias19,
    Offset19,
    NoiseAmount19,
    GradType20,
    StartColor20,
    EndColor20,
    InvertGradient20,
    Bias20,
    Offset20,
    NoiseAmount20,
    GradType21,
    StartColor21,
    EndColor21,
    InvertGradient21,
    Bias21,
    Offset21,
    NoiseAmount21,
    GradType22,
    StartColor22,
    EndColor22,
    InvertGradient22,
    Bias22,
    Offset22,
    NoiseAmount22,
    GradType23,
    StartColor23,
    EndColor23,
    InvertGradient23,
    Bias23,
    Offset23,
    NoiseAmount23,
    GradType24,
    StartColor24,
    EndColor24,
    InvertGradient24,
    Bias24,
    Offset24,
    NoiseAmount24,
    GradType25,
    StartColor25,
    EndColor25,
    InvertGradient25,
    Bias25,
    Offset25,
    NoiseAmount25,
    GradType26,
    StartColor26,
    EndColor26,
    InvertGradient26,
    Bias26,
    Offset26,
    NoiseAmount26,
    GradType27,
    StartColor27,
    EndColor27,
    InvertGradient27,
    Bias27,
    Offset27,
    NoiseAmount27,
    GradType28,
    StartColor28,
    EndColor28,
    InvertGradient28,
    Bias28,
    Offset28,
    NoiseAmount28,
    GradType29,
    StartColor29,
    EndColor29,
    InvertGradient29,
    Bias29,
    Offset29,
    NoiseAmount29,
    GradType30,
    StartColor30,
    EndColor30,
    InvertGradient30,
    Bias30,
    Offset30,
    NoiseAmount30,
    GradType31,
    StartColor31,
    EndColor31,
    InvertGradient31,
    Bias31,
    Offset31,
    NoiseAmount31,
}

#[derive(Default)]
struct Plugin {}

ae::define_effect!(Plugin, (), Params);

const PLUGIN_DESCRIPTION: &str =
    "Tracks colored regions as islands and applies per-island gradients or temp colors.";

// Param arrays for dynamic UI visibility (UpdateParamsUi)
const EXTRACTION_TARGET_COLORS: [Params; EXTRACTION_SETS] = [
    Params::TargetColor0,
    Params::TargetColor1,
    Params::TargetColor2,
    Params::TargetColor3,
    Params::TargetColor4,
    Params::TargetColor5,
    Params::TargetColor6,
    Params::TargetColor7,
];
const EXTRACTION_COLOR_RANGES: [Params; EXTRACTION_SETS] = [
    Params::ColorRange0,
    Params::ColorRange1,
    Params::ColorRange2,
    Params::ColorRange3,
    Params::ColorRange4,
    Params::ColorRange5,
    Params::ColorRange6,
    Params::ColorRange7,
];
const MERGE_SOURCE_TEMP: [Params; MERGE_ISLAND_SETS] = [
    Params::SourceTempColor0,
    Params::SourceTempColor1,
    Params::SourceTempColor2,
    Params::SourceTempColor3,
    Params::SourceTempColor4,
    Params::SourceTempColor5,
    Params::SourceTempColor6,
    Params::SourceTempColor7,
    Params::SourceTempColor8,
    Params::SourceTempColor9,
    Params::SourceTempColor10,
    Params::SourceTempColor11,
    Params::SourceTempColor12,
    Params::SourceTempColor13,
    Params::SourceTempColor14,
    Params::SourceTempColor15,
    Params::SourceTempColor16,
    Params::SourceTempColor17,
    Params::SourceTempColor18,
    Params::SourceTempColor19,
    Params::SourceTempColor20,
    Params::SourceTempColor21,
    Params::SourceTempColor22,
    Params::SourceTempColor23,
    Params::SourceTempColor24,
    Params::SourceTempColor25,
    Params::SourceTempColor26,
    Params::SourceTempColor27,
    Params::SourceTempColor28,
    Params::SourceTempColor29,
    Params::SourceTempColor30,
    Params::SourceTempColor31,
];
const MERGE_TARGET_TEMP: [Params; MERGE_ISLAND_SETS] = [
    Params::TargetTempColor0,
    Params::TargetTempColor1,
    Params::TargetTempColor2,
    Params::TargetTempColor3,
    Params::TargetTempColor4,
    Params::TargetTempColor5,
    Params::TargetTempColor6,
    Params::TargetTempColor7,
    Params::TargetTempColor8,
    Params::TargetTempColor9,
    Params::TargetTempColor10,
    Params::TargetTempColor11,
    Params::TargetTempColor12,
    Params::TargetTempColor13,
    Params::TargetTempColor14,
    Params::TargetTempColor15,
    Params::TargetTempColor16,
    Params::TargetTempColor17,
    Params::TargetTempColor18,
    Params::TargetTempColor19,
    Params::TargetTempColor20,
    Params::TargetTempColor21,
    Params::TargetTempColor22,
    Params::TargetTempColor23,
    Params::TargetTempColor24,
    Params::TargetTempColor25,
    Params::TargetTempColor26,
    Params::TargetTempColor27,
    Params::TargetTempColor28,
    Params::TargetTempColor29,
    Params::TargetTempColor30,
    Params::TargetTempColor31,
];
const GRADIENT_GRAD_TYPE: [Params; GRADIENT_SETS] = [
    Params::GradType0,
    Params::GradType1,
    Params::GradType2,
    Params::GradType3,
    Params::GradType4,
    Params::GradType5,
    Params::GradType6,
    Params::GradType7,
    Params::GradType8,
    Params::GradType9,
    Params::GradType10,
    Params::GradType11,
    Params::GradType12,
    Params::GradType13,
    Params::GradType14,
    Params::GradType15,
    Params::GradType16,
    Params::GradType17,
    Params::GradType18,
    Params::GradType19,
    Params::GradType20,
    Params::GradType21,
    Params::GradType22,
    Params::GradType23,
    Params::GradType24,
    Params::GradType25,
    Params::GradType26,
    Params::GradType27,
    Params::GradType28,
    Params::GradType29,
    Params::GradType30,
    Params::GradType31,
];
const GRADIENT_START_COLOR: [Params; GRADIENT_SETS] = [
    Params::StartColor0,
    Params::StartColor1,
    Params::StartColor2,
    Params::StartColor3,
    Params::StartColor4,
    Params::StartColor5,
    Params::StartColor6,
    Params::StartColor7,
    Params::StartColor8,
    Params::StartColor9,
    Params::StartColor10,
    Params::StartColor11,
    Params::StartColor12,
    Params::StartColor13,
    Params::StartColor14,
    Params::StartColor15,
    Params::StartColor16,
    Params::StartColor17,
    Params::StartColor18,
    Params::StartColor19,
    Params::StartColor20,
    Params::StartColor21,
    Params::StartColor22,
    Params::StartColor23,
    Params::StartColor24,
    Params::StartColor25,
    Params::StartColor26,
    Params::StartColor27,
    Params::StartColor28,
    Params::StartColor29,
    Params::StartColor30,
    Params::StartColor31,
];
const GRADIENT_END_COLOR: [Params; GRADIENT_SETS] = [
    Params::EndColor0,
    Params::EndColor1,
    Params::EndColor2,
    Params::EndColor3,
    Params::EndColor4,
    Params::EndColor5,
    Params::EndColor6,
    Params::EndColor7,
    Params::EndColor8,
    Params::EndColor9,
    Params::EndColor10,
    Params::EndColor11,
    Params::EndColor12,
    Params::EndColor13,
    Params::EndColor14,
    Params::EndColor15,
    Params::EndColor16,
    Params::EndColor17,
    Params::EndColor18,
    Params::EndColor19,
    Params::EndColor20,
    Params::EndColor21,
    Params::EndColor22,
    Params::EndColor23,
    Params::EndColor24,
    Params::EndColor25,
    Params::EndColor26,
    Params::EndColor27,
    Params::EndColor28,
    Params::EndColor29,
    Params::EndColor30,
    Params::EndColor31,
];
const GRADIENT_INVERT: [Params; GRADIENT_SETS] = [
    Params::InvertGradient0,
    Params::InvertGradient1,
    Params::InvertGradient2,
    Params::InvertGradient3,
    Params::InvertGradient4,
    Params::InvertGradient5,
    Params::InvertGradient6,
    Params::InvertGradient7,
    Params::InvertGradient8,
    Params::InvertGradient9,
    Params::InvertGradient10,
    Params::InvertGradient11,
    Params::InvertGradient12,
    Params::InvertGradient13,
    Params::InvertGradient14,
    Params::InvertGradient15,
    Params::InvertGradient16,
    Params::InvertGradient17,
    Params::InvertGradient18,
    Params::InvertGradient19,
    Params::InvertGradient20,
    Params::InvertGradient21,
    Params::InvertGradient22,
    Params::InvertGradient23,
    Params::InvertGradient24,
    Params::InvertGradient25,
    Params::InvertGradient26,
    Params::InvertGradient27,
    Params::InvertGradient28,
    Params::InvertGradient29,
    Params::InvertGradient30,
    Params::InvertGradient31,
];
const GRADIENT_BIAS: [Params; GRADIENT_SETS] = [
    Params::Bias0,
    Params::Bias1,
    Params::Bias2,
    Params::Bias3,
    Params::Bias4,
    Params::Bias5,
    Params::Bias6,
    Params::Bias7,
    Params::Bias8,
    Params::Bias9,
    Params::Bias10,
    Params::Bias11,
    Params::Bias12,
    Params::Bias13,
    Params::Bias14,
    Params::Bias15,
    Params::Bias16,
    Params::Bias17,
    Params::Bias18,
    Params::Bias19,
    Params::Bias20,
    Params::Bias21,
    Params::Bias22,
    Params::Bias23,
    Params::Bias24,
    Params::Bias25,
    Params::Bias26,
    Params::Bias27,
    Params::Bias28,
    Params::Bias29,
    Params::Bias30,
    Params::Bias31,
];
const GRADIENT_OFFSET: [Params; GRADIENT_SETS] = [
    Params::Offset0,
    Params::Offset1,
    Params::Offset2,
    Params::Offset3,
    Params::Offset4,
    Params::Offset5,
    Params::Offset6,
    Params::Offset7,
    Params::Offset8,
    Params::Offset9,
    Params::Offset10,
    Params::Offset11,
    Params::Offset12,
    Params::Offset13,
    Params::Offset14,
    Params::Offset15,
    Params::Offset16,
    Params::Offset17,
    Params::Offset18,
    Params::Offset19,
    Params::Offset20,
    Params::Offset21,
    Params::Offset22,
    Params::Offset23,
    Params::Offset24,
    Params::Offset25,
    Params::Offset26,
    Params::Offset27,
    Params::Offset28,
    Params::Offset29,
    Params::Offset30,
    Params::Offset31,
];
const GRADIENT_NOISE_AMOUNT: [Params; GRADIENT_SETS] = [
    Params::NoiseAmount0,
    Params::NoiseAmount1,
    Params::NoiseAmount2,
    Params::NoiseAmount3,
    Params::NoiseAmount4,
    Params::NoiseAmount5,
    Params::NoiseAmount6,
    Params::NoiseAmount7,
    Params::NoiseAmount8,
    Params::NoiseAmount9,
    Params::NoiseAmount10,
    Params::NoiseAmount11,
    Params::NoiseAmount12,
    Params::NoiseAmount13,
    Params::NoiseAmount14,
    Params::NoiseAmount15,
    Params::NoiseAmount16,
    Params::NoiseAmount17,
    Params::NoiseAmount18,
    Params::NoiseAmount19,
    Params::NoiseAmount20,
    Params::NoiseAmount21,
    Params::NoiseAmount22,
    Params::NoiseAmount23,
    Params::NoiseAmount24,
    Params::NoiseAmount25,
    Params::NoiseAmount26,
    Params::NoiseAmount27,
    Params::NoiseAmount28,
    Params::NoiseAmount29,
    Params::NoiseAmount30,
    Params::NoiseAmount31,
];

fn set_param_visibility(
    in_data: InData,
    params: &ae::Parameters<Params>,
    param_type: Params,
    visible: bool,
) -> Result<(), ae::Error> {
    let index = match params.map.get(&param_type) {
        Some(info) => info.index as i32,
        None => return Ok(()),
    };
    let expected = params.raw_param_type(param_type);
    let mut param_def = ae::ParamDef::checkout(
        in_data,
        index,
        in_data.current_time(),
        in_data.time_step(),
        in_data.time_scale(),
        expected,
    )?;
    let raw = param_def.as_mut();
    let inv = ae::ParamUIFlags::INVISIBLE.bits();
    if visible {
        raw.ui_flags &= !inv;
    } else {
        raw.ui_flags |= inv;
    }
    param_def.update_param_ui()?;
    Ok(())
}

fn update_params_ui_visibility(
    in_data: InData,
    params: &mut ae::Parameters<Params>,
) -> Result<(), ae::Error> {
    // Extraction: TargetColor[i] and ColorRange[i] are always shown/hidden as a pair per index i.
    let extraction_count: usize = params
        .get(Params::ExtractionCount)
        .ok()
        .and_then(|p| p.as_popup().ok().map(|pd| pd.value() as usize))
        .unwrap_or(1)
        .clamp(1, EXTRACTION_SETS);
    for i in 0..EXTRACTION_SETS {
        let visible = i < extraction_count;
        set_param_visibility(in_data, params, EXTRACTION_TARGET_COLORS[i], visible)?;
        set_param_visibility(in_data, params, EXTRACTION_COLOR_RANGES[i], visible)?;
    }
    // Merge: SourceTempColor[i] and TargetTempColor[i] as a pair per index i.
    const MERGE_COUNTS: [usize; 4] = [4, 8, 16, 32];
    let merge_count: usize = params
        .get(Params::MergeIslandCount)
        .ok()
        .and_then(|p| {
            p.as_popup().ok().map(|pd| {
                let v = (pd.value() as usize).saturating_sub(1);
                MERGE_COUNTS.get(v).copied().unwrap_or(4)
            })
        })
        .unwrap_or(4);
    for i in 0..MERGE_ISLAND_SETS {
        let visible = i < merge_count;
        set_param_visibility(in_data, params, MERGE_SOURCE_TEMP[i], visible)?;
        set_param_visibility(in_data, params, MERGE_TARGET_TEMP[i], visible)?;
    }
    // Gradient: all 7 params per index i (GradType, StartColor, EndColor, InvertGradient, Bias, Offset, NoiseAmount).
    let gradient_count: usize = params
        .get(Params::GradientSettingsCount)
        .ok()
        .and_then(|p| {
            p.as_popup().ok().map(|pd| {
                let v = (pd.value() as usize).saturating_sub(1);
                MERGE_COUNTS.get(v).copied().unwrap_or(4)
            })
        })
        .unwrap_or(4);
    for i in 0..GRADIENT_SETS {
        let visible = i < gradient_count;
        set_param_visibility(in_data, params, GRADIENT_GRAD_TYPE[i], visible)?;
        set_param_visibility(in_data, params, GRADIENT_START_COLOR[i], visible)?;
        set_param_visibility(in_data, params, GRADIENT_END_COLOR[i], visible)?;
        set_param_visibility(in_data, params, GRADIENT_INVERT[i], visible)?;
        set_param_visibility(in_data, params, GRADIENT_BIAS[i], visible)?;
        set_param_visibility(in_data, params, GRADIENT_OFFSET[i], visible)?;
        set_param_visibility(in_data, params, GRADIENT_NOISE_AMOUNT[i], visible)?;
    }
    Ok(())
}

impl AdobePluginGlobal for Plugin {
    fn params_setup(
        &self,
        params: &mut ae::Parameters<Params>,
        _in_data: InData,
        _: OutData,
    ) -> Result<(), Error> {
        // ----- 0. Output Settings (always visible) -----
        params.add(
            Params::OutputMode,
            "Output Mode",
            PopupDef::setup(|d| {
                d.set_options(&[
                    "Original",
                    "Extraction (Alpha)",
                    "Temp Color (Island ID)",
                    "Final Gradient",
                ]);
                d.set_default(1);
            }),
        )?;

        // ----- 1. Color Extraction group (collapsed by default) -----
        params.add_group(
            Params::ColorExtGroupStart,
            Params::ColorExtGroupEnd,
            "Color Extraction",
            true,
            |params| {
                params.add(
                    Params::InvertExtraction,
                    "Invert Extraction",
                    CheckBoxDef::setup(|d| {
                        d.set_default(false);
                    }),
                )?;
                params.add(
                    Params::ExtractionCount,
                    "Extraction Count",
                    PopupDef::setup(|d| {
                        d.set_options(&["1", "2", "3", "4", "5", "6", "7", "8"]);
                        d.set_default(1);
                    }),
                )?;

                let target_color_range = [
                    (Params::TargetColor0, Params::ColorRange0),
                    (Params::TargetColor1, Params::ColorRange1),
                    (Params::TargetColor2, Params::ColorRange2),
                    (Params::TargetColor3, Params::ColorRange3),
                    (Params::TargetColor4, Params::ColorRange4),
                    (Params::TargetColor5, Params::ColorRange5),
                    (Params::TargetColor6, Params::ColorRange6),
                    (Params::TargetColor7, Params::ColorRange7),
                ];
                const EXTRACTION_INITIAL_COUNT: usize = 1; // ExtractionCount default = 1 (1-based)
                for (i, (tc, cr)) in target_color_range.iter().enumerate() {
                    let initially_hidden = i >= EXTRACTION_INITIAL_COUNT;
                    let ui_flags = if initially_hidden {
                        ParamUIFlags::INVISIBLE
                    } else {
                        ParamUIFlags::NONE
                    };
                    params.add_with_flags(
                        *tc,
                        &format!("Target Color {}", i + 1),
                        ColorDef::setup(|_d| {}),
                        ParamFlag::empty(),
                        ui_flags,
                    )?;
                    params.add_with_flags(
                        *cr,
                        &format!("Color Range {}", i + 1),
                        FloatSliderDef::setup(|d| {
                            d.set_valid_min(0.0);
                            d.set_valid_max(100.0);
                            d.set_slider_min(0.0);
                            d.set_slider_max(50.0);
                            d.set_default(0.0);
                            d.set_precision(1);
                        }),
                        ParamFlag::START_COLLAPSED,
                        if initially_hidden {
                            ParamUIFlags::INVISIBLE
                        } else {
                            ParamUIFlags::NONE
                        },
                    )?;
                }

                params.add(
                    Params::ChokeSpread,
                    "Choke / Spread",
                    FloatSliderDef::setup(|d| {
                        d.set_valid_min(-100.0);
                        d.set_valid_max(100.0);
                        d.set_slider_min(-20.0);
                        d.set_slider_max(20.0);
                        d.set_default(0.0);
                        d.set_precision(1);
                    }),
                )?;
                Ok(())
            },
        )?;

        // ----- 2. Island Tracking & Temp Colors group (collapsed) -----
        params.add_group(
            Params::IslandTrackGroupStart,
            Params::IslandTrackGroupEnd,
            "Island Tracking & Temp Colors",
            true,
            |params| {
                params.add(
                    Params::TrackingPath,
                    "Tracking Path",
                    PathDef::setup(|_| {}),
                )?;
                params.add(
                    Params::ShowTempColors,
                    "Show Temp Colors",
                    CheckBoxDef::setup(|d| {
                        d.set_default(true);
                    }),
                )?;
                params.add(
                    Params::MergeIslandCount,
                    "Merge Island Count",
                    PopupDef::setup(|d| {
                        d.set_options(&["4", "8", "16", "32"]);
                        d.set_default(2); // 8 (1-based index 2)
                    }),
                )?;

                const MERGE_INITIAL_COUNT: usize = 8; // default option "8"
                for i in 0..MERGE_ISLAND_SETS {
                    let merge_ui = if i >= MERGE_INITIAL_COUNT {
                        ParamUIFlags::INVISIBLE
                    } else {
                        ParamUIFlags::NONE
                    };
                    let src = match i {
                        0 => Params::SourceTempColor0,
                        1 => Params::SourceTempColor1,
                        2 => Params::SourceTempColor2,
                        3 => Params::SourceTempColor3,
                        4 => Params::SourceTempColor4,
                        5 => Params::SourceTempColor5,
                        6 => Params::SourceTempColor6,
                        7 => Params::SourceTempColor7,
                        8 => Params::SourceTempColor8,
                        9 => Params::SourceTempColor9,
                        10 => Params::SourceTempColor10,
                        11 => Params::SourceTempColor11,
                        12 => Params::SourceTempColor12,
                        13 => Params::SourceTempColor13,
                        14 => Params::SourceTempColor14,
                        15 => Params::SourceTempColor15,
                        16 => Params::SourceTempColor16,
                        17 => Params::SourceTempColor17,
                        18 => Params::SourceTempColor18,
                        19 => Params::SourceTempColor19,
                        20 => Params::SourceTempColor20,
                        21 => Params::SourceTempColor21,
                        22 => Params::SourceTempColor22,
                        23 => Params::SourceTempColor23,
                        24 => Params::SourceTempColor24,
                        25 => Params::SourceTempColor25,
                        26 => Params::SourceTempColor26,
                        27 => Params::SourceTempColor27,
                        28 => Params::SourceTempColor28,
                        29 => Params::SourceTempColor29,
                        30 => Params::SourceTempColor30,
                        _ => Params::SourceTempColor31,
                    };
                    let tgt = match i {
                        0 => Params::TargetTempColor0,
                        1 => Params::TargetTempColor1,
                        2 => Params::TargetTempColor2,
                        3 => Params::TargetTempColor3,
                        4 => Params::TargetTempColor4,
                        5 => Params::TargetTempColor5,
                        6 => Params::TargetTempColor6,
                        7 => Params::TargetTempColor7,
                        8 => Params::TargetTempColor8,
                        9 => Params::TargetTempColor9,
                        10 => Params::TargetTempColor10,
                        11 => Params::TargetTempColor11,
                        12 => Params::TargetTempColor12,
                        13 => Params::TargetTempColor13,
                        14 => Params::TargetTempColor14,
                        15 => Params::TargetTempColor15,
                        16 => Params::TargetTempColor16,
                        17 => Params::TargetTempColor17,
                        18 => Params::TargetTempColor18,
                        19 => Params::TargetTempColor19,
                        20 => Params::TargetTempColor20,
                        21 => Params::TargetTempColor21,
                        22 => Params::TargetTempColor22,
                        23 => Params::TargetTempColor23,
                        24 => Params::TargetTempColor24,
                        25 => Params::TargetTempColor25,
                        26 => Params::TargetTempColor26,
                        27 => Params::TargetTempColor27,
                        28 => Params::TargetTempColor28,
                        29 => Params::TargetTempColor29,
                        30 => Params::TargetTempColor30,
                        _ => Params::TargetTempColor31,
                    };
                    params.add_with_flags(
                        src,
                        &format!("Source Temp Color {}", i + 1),
                        ColorDef::setup(|_d| {}),
                        ParamFlag::empty(),
                        if i >= MERGE_INITIAL_COUNT {
                            ParamUIFlags::INVISIBLE
                        } else {
                            ParamUIFlags::NONE
                        },
                    )?;
                    params.add_with_flags(
                        tgt,
                        &format!("Target Temp Color {}", i + 1),
                        ColorDef::setup(|_d| {}),
                        ParamFlag::empty(),
                        merge_ui,
                    )?;
                }
                Ok(())
            },
        )?;

        // ----- 3. Gradient Render group (collapsed) -----
        params.add_group(
            Params::GradientGroupStart,
            Params::GradientGroupEnd,
            "Gradient Render",
            true,
            |params| {
                params.add(
                    Params::GradientSettingsCount,
                    "Gradient Settings Count",
                    PopupDef::setup(|d| {
                        d.set_options(&["4", "8", "16", "32"]);
                        d.set_default(2); // 8
                    }),
                )?;
                params.add(
                    Params::MasterAngle,
                    "Master Angle",
                    AngleDef::setup(|_d| {}),
                )?;

                let grad_params = [
                    (
                        Params::GradType0,
                        Params::StartColor0,
                        Params::EndColor0,
                        Params::InvertGradient0,
                        Params::Bias0,
                        Params::Offset0,
                        Params::NoiseAmount0,
                    ),
                    (
                        Params::GradType1,
                        Params::StartColor1,
                        Params::EndColor1,
                        Params::InvertGradient1,
                        Params::Bias1,
                        Params::Offset1,
                        Params::NoiseAmount1,
                    ),
                    (
                        Params::GradType2,
                        Params::StartColor2,
                        Params::EndColor2,
                        Params::InvertGradient2,
                        Params::Bias2,
                        Params::Offset2,
                        Params::NoiseAmount2,
                    ),
                    (
                        Params::GradType3,
                        Params::StartColor3,
                        Params::EndColor3,
                        Params::InvertGradient3,
                        Params::Bias3,
                        Params::Offset3,
                        Params::NoiseAmount3,
                    ),
                    (
                        Params::GradType4,
                        Params::StartColor4,
                        Params::EndColor4,
                        Params::InvertGradient4,
                        Params::Bias4,
                        Params::Offset4,
                        Params::NoiseAmount4,
                    ),
                    (
                        Params::GradType5,
                        Params::StartColor5,
                        Params::EndColor5,
                        Params::InvertGradient5,
                        Params::Bias5,
                        Params::Offset5,
                        Params::NoiseAmount5,
                    ),
                    (
                        Params::GradType6,
                        Params::StartColor6,
                        Params::EndColor6,
                        Params::InvertGradient6,
                        Params::Bias6,
                        Params::Offset6,
                        Params::NoiseAmount6,
                    ),
                    (
                        Params::GradType7,
                        Params::StartColor7,
                        Params::EndColor7,
                        Params::InvertGradient7,
                        Params::Bias7,
                        Params::Offset7,
                        Params::NoiseAmount7,
                    ),
                    (
                        Params::GradType8,
                        Params::StartColor8,
                        Params::EndColor8,
                        Params::InvertGradient8,
                        Params::Bias8,
                        Params::Offset8,
                        Params::NoiseAmount8,
                    ),
                    (
                        Params::GradType9,
                        Params::StartColor9,
                        Params::EndColor9,
                        Params::InvertGradient9,
                        Params::Bias9,
                        Params::Offset9,
                        Params::NoiseAmount9,
                    ),
                    (
                        Params::GradType10,
                        Params::StartColor10,
                        Params::EndColor10,
                        Params::InvertGradient10,
                        Params::Bias10,
                        Params::Offset10,
                        Params::NoiseAmount10,
                    ),
                    (
                        Params::GradType11,
                        Params::StartColor11,
                        Params::EndColor11,
                        Params::InvertGradient11,
                        Params::Bias11,
                        Params::Offset11,
                        Params::NoiseAmount11,
                    ),
                    (
                        Params::GradType12,
                        Params::StartColor12,
                        Params::EndColor12,
                        Params::InvertGradient12,
                        Params::Bias12,
                        Params::Offset12,
                        Params::NoiseAmount12,
                    ),
                    (
                        Params::GradType13,
                        Params::StartColor13,
                        Params::EndColor13,
                        Params::InvertGradient13,
                        Params::Bias13,
                        Params::Offset13,
                        Params::NoiseAmount13,
                    ),
                    (
                        Params::GradType14,
                        Params::StartColor14,
                        Params::EndColor14,
                        Params::InvertGradient14,
                        Params::Bias14,
                        Params::Offset14,
                        Params::NoiseAmount14,
                    ),
                    (
                        Params::GradType15,
                        Params::StartColor15,
                        Params::EndColor15,
                        Params::InvertGradient15,
                        Params::Bias15,
                        Params::Offset15,
                        Params::NoiseAmount15,
                    ),
                    (
                        Params::GradType16,
                        Params::StartColor16,
                        Params::EndColor16,
                        Params::InvertGradient16,
                        Params::Bias16,
                        Params::Offset16,
                        Params::NoiseAmount16,
                    ),
                    (
                        Params::GradType17,
                        Params::StartColor17,
                        Params::EndColor17,
                        Params::InvertGradient17,
                        Params::Bias17,
                        Params::Offset17,
                        Params::NoiseAmount17,
                    ),
                    (
                        Params::GradType18,
                        Params::StartColor18,
                        Params::EndColor18,
                        Params::InvertGradient18,
                        Params::Bias18,
                        Params::Offset18,
                        Params::NoiseAmount18,
                    ),
                    (
                        Params::GradType19,
                        Params::StartColor19,
                        Params::EndColor19,
                        Params::InvertGradient19,
                        Params::Bias19,
                        Params::Offset19,
                        Params::NoiseAmount19,
                    ),
                    (
                        Params::GradType20,
                        Params::StartColor20,
                        Params::EndColor20,
                        Params::InvertGradient20,
                        Params::Bias20,
                        Params::Offset20,
                        Params::NoiseAmount20,
                    ),
                    (
                        Params::GradType21,
                        Params::StartColor21,
                        Params::EndColor21,
                        Params::InvertGradient21,
                        Params::Bias21,
                        Params::Offset21,
                        Params::NoiseAmount21,
                    ),
                    (
                        Params::GradType22,
                        Params::StartColor22,
                        Params::EndColor22,
                        Params::InvertGradient22,
                        Params::Bias22,
                        Params::Offset22,
                        Params::NoiseAmount22,
                    ),
                    (
                        Params::GradType23,
                        Params::StartColor23,
                        Params::EndColor23,
                        Params::InvertGradient23,
                        Params::Bias23,
                        Params::Offset23,
                        Params::NoiseAmount23,
                    ),
                    (
                        Params::GradType24,
                        Params::StartColor24,
                        Params::EndColor24,
                        Params::InvertGradient24,
                        Params::Bias24,
                        Params::Offset24,
                        Params::NoiseAmount24,
                    ),
                    (
                        Params::GradType25,
                        Params::StartColor25,
                        Params::EndColor25,
                        Params::InvertGradient25,
                        Params::Bias25,
                        Params::Offset25,
                        Params::NoiseAmount25,
                    ),
                    (
                        Params::GradType26,
                        Params::StartColor26,
                        Params::EndColor26,
                        Params::InvertGradient26,
                        Params::Bias26,
                        Params::Offset26,
                        Params::NoiseAmount26,
                    ),
                    (
                        Params::GradType27,
                        Params::StartColor27,
                        Params::EndColor27,
                        Params::InvertGradient27,
                        Params::Bias27,
                        Params::Offset27,
                        Params::NoiseAmount27,
                    ),
                    (
                        Params::GradType28,
                        Params::StartColor28,
                        Params::EndColor28,
                        Params::InvertGradient28,
                        Params::Bias28,
                        Params::Offset28,
                        Params::NoiseAmount28,
                    ),
                    (
                        Params::GradType29,
                        Params::StartColor29,
                        Params::EndColor29,
                        Params::InvertGradient29,
                        Params::Bias29,
                        Params::Offset29,
                        Params::NoiseAmount29,
                    ),
                    (
                        Params::GradType30,
                        Params::StartColor30,
                        Params::EndColor30,
                        Params::InvertGradient30,
                        Params::Bias30,
                        Params::Offset30,
                        Params::NoiseAmount30,
                    ),
                    (
                        Params::GradType31,
                        Params::StartColor31,
                        Params::EndColor31,
                        Params::InvertGradient31,
                        Params::Bias31,
                        Params::Offset31,
                        Params::NoiseAmount31,
                    ),
                ];
                const GRADIENT_INITIAL_COUNT: usize = 8; // GradientSettingsCount default = 8
                for (idx, (grad_type, start_c, end_c, invert_grad, bias, offset, noise)) in
                    grad_params.iter().enumerate()
                {
                    let n = idx + 1;
                    let grad_hidden = idx >= GRADIENT_INITIAL_COUNT;
                    let g_ui = || {
                        if grad_hidden {
                            ParamUIFlags::INVISIBLE
                        } else {
                            ParamUIFlags::NONE
                        }
                    };
                    params.add_with_flags(
                        *grad_type,
                        &format!("Grad Type {}", n),
                        PopupDef::setup(|d| {
                            d.set_options(&["Linear", "Radial"]);
                            d.set_default(1);
                        }),
                        ParamFlag::empty(),
                        g_ui(),
                    )?;
                    params.add_with_flags(
                        *start_c,
                        &format!("Start Color {}", n),
                        ColorDef::setup(|_d| {}),
                        ParamFlag::empty(),
                        g_ui(),
                    )?;
                    params.add_with_flags(
                        *end_c,
                        &format!("End Color {}", n),
                        ColorDef::setup(|_d| {}),
                        ParamFlag::empty(),
                        g_ui(),
                    )?;
                    params.add_with_flags(
                        *invert_grad,
                        &format!("Invert Gradient {}", n),
                        CheckBoxDef::setup(|d| {
                            d.set_default(false);
                        }),
                        ParamFlag::empty(),
                        g_ui(),
                    )?;
                    params.add_with_flags(
                        *bias,
                        &format!("Bias {}", n),
                        FloatSliderDef::setup(|d| {
                            d.set_valid_min(0.0);
                            d.set_valid_max(100.0);
                            d.set_slider_min(0.0);
                            d.set_slider_max(100.0);
                            d.set_default(50.0);
                            d.set_precision(1);
                        }),
                        ParamFlag::empty(),
                        g_ui(),
                    )?;
                    params.add_with_flags(
                        *offset,
                        &format!("Offset {}", n),
                        FloatSliderDef::setup(|d| {
                            d.set_valid_min(-100.0);
                            d.set_valid_max(100.0);
                            d.set_slider_min(-100.0);
                            d.set_slider_max(100.0);
                            d.set_default(0.0);
                            d.set_precision(1);
                        }),
                        ParamFlag::empty(),
                        g_ui(),
                    )?;
                    params.add_with_flags(
                        *noise,
                        &format!("Noise Amount {}", n),
                        FloatSliderDef::setup(|d| {
                            d.set_valid_min(0.0);
                            d.set_valid_max(100.0);
                            d.set_slider_min(0.0);
                            d.set_slider_max(50.0);
                            d.set_default(0.0);
                            d.set_precision(1);
                        }),
                        ParamFlag::empty(),
                        g_ui(),
                    )?;
                }
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
                        "AOD_IslandIdColor - {version}\r\r{PLUGIN_DESCRIPTION}\rCopyright (c) 2026-{build_year} Aodaruma",
                        version = env!("CARGO_PKG_VERSION"),
                        build_year = env!("BUILD_YEAR")
                    )
                    .as_str(),
                );
            }
            ae::Command::GlobalSetup => {
                out_data.set_out_flag(ae::OutFlags::SendUpdateParamsUi, true);
                out_data.set_out_flag2(OutFlags2::SupportsSmartRender, true);
                out_data.set_out_flag2(OutFlags2::SupportsThreadedRendering, true);
                out_data.set_out_flag2(OutFlags2::SupportsGetFlattenedSequenceData, true);
                // When using GROUP_START params: set ParamGroupStartCollapsedFlag so twirly starts collapsed (AE_Rust_Knowledge).
                out_data.set_out_flag2(OutFlags2::ParamGroupStartCollapsedFlag, true);
            }
            ae::Command::UpdateParamsUi => {
                update_params_ui_visibility(in_data, params)?;
            }
            ae::Command::Render {
                in_layer,
                out_layer,
            } => {
                self.do_render(in_data, in_layer, out_data, out_layer, params)?;
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

/// Euclidean distance in RGB space (0..1). Alpha is ignored for extraction.
fn color_distance_f32(a: &PixelF32, b: &PixelF32) -> f32 {
    let dr = a.red - b.red;
    let dg = a.green - b.green;
    let db = a.blue - b.blue;
    (dr * dr + dg * dg + db * db).sqrt()
}

/// Convert AE color param (Pixel8, 0..255) to normalized PixelF32 (0..1).
fn target_color_to_f32(c: &ae::Pixel8) -> PixelF32 {
    let scale = 1.0 / ae::MAX_CHANNEL8 as f32;
    PixelF32 {
        red: c.red as f32 * scale,
        green: c.green as f32 * scale,
        blue: c.blue as f32 * scale,
        alpha: c.alpha as f32 * scale,
    }
}

impl Plugin {
    fn do_render(
        &self,
        _in_data: InData,
        in_layer: Layer,
        _out_data: OutData,
        mut out_layer: Layer,
        params: &mut Parameters<Params>,
    ) -> Result<(), Error> {
        let progress_final = out_layer.height() as i32;
        let in_world_type = in_layer.world_type();
        let out_world_type = out_layer.world_type();

        // 1. Parameter retrieval (AE Popup value() is 1-based)
        let output_mode: i32 = params
            .get(Params::OutputMode)
            .ok()
            .and_then(|p| p.as_popup().ok().map(|pd| pd.value()))
            .unwrap_or(1);
        let invert_extraction: bool = params
            .get(Params::InvertExtraction)
            .ok()
            .and_then(|p| p.as_checkbox().ok().map(|cb| cb.value()))
            .unwrap_or(false);
        // ExtractionCount: 1-based value 1..8 = effective count 1..8
        let extraction_count: usize = params
            .get(Params::ExtractionCount)
            .ok()
            .and_then(|p| p.as_popup().ok().map(|pd| pd.value() as usize))
            .unwrap_or(1)
            .clamp(1, EXTRACTION_SETS);

        let mut extraction_targets: Vec<(PixelF32, f32)> = Vec::with_capacity(extraction_count);
        for i in 0..extraction_count {
            let target_color = params
                .get(EXTRACTION_TARGET_COLORS[i])
                .ok()
                .and_then(|p| p.as_color().ok().map(|cd| cd.value()))
                .ok_or(Error::InvalidParms)?;
            let color_range_val = params
                .get(EXTRACTION_COLOR_RANGES[i])
                .ok()
                .and_then(|p| p.as_float_slider().ok().map(|fs| fs.value()))
                .ok_or(Error::InvalidParms)? as f32;
            // Color Range slider 0..100 → threshold in 0..1 normalized space
            let range_f32 = (color_range_val / 100.0).clamp(0.0, 1.0);
            extraction_targets.push((target_color_to_f32(&target_color), range_f32));
        }

        // 2–3. Per-pixel processing and output branching
        let out_world_type_copy = out_world_type;
        out_layer.iterate(0, progress_final, None, |x, y, mut dst| {
            let x = x as usize;
            let y = y as usize;
            let px = read_pixel_f32(&in_layer, in_world_type, x, y);

            // OutputMode: 1=Original, 2=Extraction, 3=TempColor, 4=FinalGradient (1-based)
            let out_px = match output_mode {
                1 => {
                    // Original: pass through
                    px
                }
                2 => {
                    // Extraction: match any target within range, then invert if requested
                    let extracted = extraction_targets
                        .iter()
                        .any(|(target, range)| color_distance_f32(&px, target) <= *range);
                    let show = extracted != invert_extraction;
                    if show {
                        PixelF32 {
                            red: 1.0,
                            green: 1.0,
                            blue: 1.0,
                            alpha: px.alpha,
                        }
                    } else {
                        PixelF32 {
                            red: 0.0,
                            green: 0.0,
                            blue: 0.0,
                            alpha: px.alpha,
                        }
                    }
                }
                3 | 4 => {
                    // Temp Color / Final Gradient: placeholder — pass through
                    px
                }
                _ => px,
            };

            match out_world_type_copy {
                ae::aegp::WorldType::U8 => dst.set_from_u8(out_px.to_pixel8()),
                ae::aegp::WorldType::U15 => dst.set_from_u16(out_px.to_pixel16()),
                ae::aegp::WorldType::F32 | ae::aegp::WorldType::None => dst.set_from_f32(out_px),
            }
            Ok(())
        })?;

        Ok(())
    }
}

fn read_pixel_f32(layer: &Layer, world_type: ae::aegp::WorldType, x: usize, y: usize) -> PixelF32 {
    match world_type {
        ae::aegp::WorldType::U8 => layer.as_pixel8(x, y).to_pixel32(),
        ae::aegp::WorldType::U15 => layer.as_pixel16(x, y).to_pixel32(),
        ae::aegp::WorldType::F32 | ae::aegp::WorldType::None => *layer.as_pixel32(x, y),
    }
}
