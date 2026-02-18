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
    MasterGradType,
    MasterAngle,
    MasterBias,
    MasterOffset,
    MasterNoiseAmount,
    // StartColor[n], EndColor[n], InvertGradient[n] x GRADIENT_SETS
    StartColor0,
    EndColor0,
    InvertGradient0,
    StartColor1,
    EndColor1,
    InvertGradient1,
    StartColor2,
    EndColor2,
    InvertGradient2,
    StartColor3,
    EndColor3,
    InvertGradient3,
    StartColor4,
    EndColor4,
    InvertGradient4,
    StartColor5,
    EndColor5,
    InvertGradient5,
    StartColor6,
    EndColor6,
    InvertGradient6,
    StartColor7,
    EndColor7,
    InvertGradient7,
    StartColor8,
    EndColor8,
    InvertGradient8,
    StartColor9,
    EndColor9,
    InvertGradient9,
    StartColor10,
    EndColor10,
    InvertGradient10,
    StartColor11,
    EndColor11,
    InvertGradient11,
    StartColor12,
    EndColor12,
    InvertGradient12,
    StartColor13,
    EndColor13,
    InvertGradient13,
    StartColor14,
    EndColor14,
    InvertGradient14,
    StartColor15,
    EndColor15,
    InvertGradient15,
    StartColor16,
    EndColor16,
    InvertGradient16,
    StartColor17,
    EndColor17,
    InvertGradient17,
    StartColor18,
    EndColor18,
    InvertGradient18,
    StartColor19,
    EndColor19,
    InvertGradient19,
    StartColor20,
    EndColor20,
    InvertGradient20,
    StartColor21,
    EndColor21,
    InvertGradient21,
    StartColor22,
    EndColor22,
    InvertGradient22,
    StartColor23,
    EndColor23,
    InvertGradient23,
    StartColor24,
    EndColor24,
    InvertGradient24,
    StartColor25,
    EndColor25,
    InvertGradient25,
    StartColor26,
    EndColor26,
    InvertGradient26,
    StartColor27,
    EndColor27,
    InvertGradient27,
    StartColor28,
    EndColor28,
    InvertGradient28,
    StartColor29,
    EndColor29,
    InvertGradient29,
    StartColor30,
    EndColor30,
    InvertGradient30,
    StartColor31,
    EndColor31,
    InvertGradient31,
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
    let mut flags = ae::ParamUIFlags::from_bits_truncate(raw.ui_flags);
    if visible {
        flags.remove(ae::ParamUIFlags::INVISIBLE);
    } else {
        flags.insert(ae::ParamUIFlags::INVISIBLE);
    }
    raw.ui_flags = flags.bits() as _;
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
    // Gradient: only StartColor[i], EndColor[i], InvertGradient[i] are dynamically shown/hidden.
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
        set_param_visibility(in_data, params, GRADIENT_START_COLOR[i], visible)?;
        set_param_visibility(in_data, params, GRADIENT_END_COLOR[i], visible)?;
        set_param_visibility(in_data, params, GRADIENT_INVERT[i], visible)?;
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
                    Params::MasterGradType,
                    "Master Grad Type",
                    PopupDef::setup(|d| {
                        d.set_options(&["Linear", "Radial"]);
                        d.set_default(1);
                    }),
                )?;
                params.add(
                    Params::MasterAngle,
                    "Master Angle",
                    AngleDef::setup(|_d| {}),
                )?;
                params.add(
                    Params::MasterBias,
                    "Master Bias",
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
                    Params::MasterOffset,
                    "Master Offset",
                    FloatSliderDef::setup(|d| {
                        d.set_valid_min(-100.0);
                        d.set_valid_max(100.0);
                        d.set_slider_min(-100.0);
                        d.set_slider_max(100.0);
                        d.set_default(0.0);
                        d.set_precision(1);
                    }),
                )?;
                params.add(
                    Params::MasterNoiseAmount,
                    "Master Noise Amount",
                    FloatSliderDef::setup(|d| {
                        d.set_valid_min(0.0);
                        d.set_valid_max(100.0);
                        d.set_slider_min(0.0);
                        d.set_slider_max(50.0);
                        d.set_default(0.0);
                        d.set_precision(1);
                    }),
                )?;

                const GRADIENT_INITIAL_COUNT: usize = 8; // GradientSettingsCount default = 8
                for i in 0..GRADIENT_SETS {
                    let n = i + 1;
                    let hidden = i >= GRADIENT_INITIAL_COUNT;
                    let ui_flags = if hidden {
                        ParamUIFlags::INVISIBLE
                    } else {
                        ParamUIFlags::NONE
                    };
                    params.add_with_flags(
                        GRADIENT_START_COLOR[i],
                        &format!("Start Color {}", n),
                        ColorDef::setup(|_d| {}),
                        ParamFlag::empty(),
                        ui_flags,
                    )?;
                    params.add_with_flags(
                        GRADIENT_END_COLOR[i],
                        &format!("End Color {}", n),
                        ColorDef::setup(|_d| {}),
                        ParamFlag::empty(),
                        if hidden {
                            ParamUIFlags::INVISIBLE
                        } else {
                            ParamUIFlags::NONE
                        },
                    )?;
                    params.add_with_flags(
                        GRADIENT_INVERT[i],
                        &format!("Invert Gradient {}", n),
                        CheckBoxDef::setup(|d| {
                            d.set_default(false);
                        }),
                        ParamFlag::empty(),
                        if hidden {
                            ParamUIFlags::INVISIBLE
                        } else {
                            ParamUIFlags::NONE
                        },
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
