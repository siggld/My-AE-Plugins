#![allow(clippy::drop_non_drop, clippy::question_mark, dead_code)]

use after_effects as ae;
use std::env;

use ae::pf::*;
use utils::ToPixel;

const EXTRACTION_SETS: usize = 32;
const MERGE_ISLAND_SETS: usize = 32;
const GRADIENT_SETS: usize = 32;
const INITIAL_EXTRACTION: usize = 4;
const INITIAL_MERGE: usize = 4;
const INITIAL_GRADIENT: usize = 4;

/// アイランド解析結果のキャッシュ。スレッド間共有に対応するため Clone を実装。
#[derive(Clone)]
pub struct IslandCacheData {
    pub width: usize,
    pub height: usize,
    /// 各ピクセルのアイランドID（0 = 背景, 1〜 = 島）
    pub labels: Vec<u32>,
    /// アイランドIDごとのバウンディングボックス
    pub bounding_boxes: std::collections::HashMap<u32, ae::Rect>,
}

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
enum Params {
    OutputMode,

    ColorExtGroupStart,
    ColorExtGroupEnd,
    InvertExtraction,
    ExtractionCount,
    EnableExtraction0,
    TargetColor0,
    ColorRange0,
    EnableExtraction1,
    TargetColor1,
    ColorRange1,
    EnableExtraction2,
    TargetColor2,
    ColorRange2,
    EnableExtraction3,
    TargetColor3,
    ColorRange3,
    EnableExtraction4,
    TargetColor4,
    ColorRange4,
    EnableExtraction5,
    TargetColor5,
    ColorRange5,
    EnableExtraction6,
    TargetColor6,
    ColorRange6,
    EnableExtraction7,
    TargetColor7,
    ColorRange7,
    EnableExtraction8,
    TargetColor8,
    ColorRange8,
    EnableExtraction9,
    TargetColor9,
    ColorRange9,
    EnableExtraction10,
    TargetColor10,
    ColorRange10,
    EnableExtraction11,
    TargetColor11,
    ColorRange11,
    EnableExtraction12,
    TargetColor12,
    ColorRange12,
    EnableExtraction13,
    TargetColor13,
    ColorRange13,
    EnableExtraction14,
    TargetColor14,
    ColorRange14,
    EnableExtraction15,
    TargetColor15,
    ColorRange15,
    EnableExtraction16,
    TargetColor16,
    ColorRange16,
    EnableExtraction17,
    TargetColor17,
    ColorRange17,
    EnableExtraction18,
    TargetColor18,
    ColorRange18,
    EnableExtraction19,
    TargetColor19,
    ColorRange19,
    EnableExtraction20,
    TargetColor20,
    ColorRange20,
    EnableExtraction21,
    TargetColor21,
    ColorRange21,
    EnableExtraction22,
    TargetColor22,
    ColorRange22,
    EnableExtraction23,
    TargetColor23,
    ColorRange23,
    EnableExtraction24,
    TargetColor24,
    ColorRange24,
    EnableExtraction25,
    TargetColor25,
    ColorRange25,
    EnableExtraction26,
    TargetColor26,
    ColorRange26,
    EnableExtraction27,
    TargetColor27,
    ColorRange27,
    EnableExtraction28,
    TargetColor28,
    ColorRange28,
    EnableExtraction29,
    TargetColor29,
    ColorRange29,
    EnableExtraction30,
    TargetColor30,
    ColorRange30,
    EnableExtraction31,
    TargetColor31,
    ColorRange31,
    ChokeSpread,

    IslandTrackGroupStart,
    IslandTrackGroupEnd,
    TrackingPath,
    ShowTempColors,
    MergeIslandCount,
    EnableMerge0,
    SourceTempColor0,
    TargetTempColor0,
    TrackingColorRange0,
    EnableMerge1,
    SourceTempColor1,
    TargetTempColor1,
    TrackingColorRange1,
    EnableMerge2,
    SourceTempColor2,
    TargetTempColor2,
    TrackingColorRange2,
    EnableMerge3,
    SourceTempColor3,
    TargetTempColor3,
    TrackingColorRange3,
    EnableMerge4,
    SourceTempColor4,
    TargetTempColor4,
    TrackingColorRange4,
    EnableMerge5,
    SourceTempColor5,
    TargetTempColor5,
    TrackingColorRange5,
    EnableMerge6,
    SourceTempColor6,
    TargetTempColor6,
    TrackingColorRange6,
    EnableMerge7,
    SourceTempColor7,
    TargetTempColor7,
    TrackingColorRange7,
    EnableMerge8,
    SourceTempColor8,
    TargetTempColor8,
    TrackingColorRange8,
    EnableMerge9,
    SourceTempColor9,
    TargetTempColor9,
    TrackingColorRange9,
    EnableMerge10,
    SourceTempColor10,
    TargetTempColor10,
    TrackingColorRange10,
    EnableMerge11,
    SourceTempColor11,
    TargetTempColor11,
    TrackingColorRange11,
    EnableMerge12,
    SourceTempColor12,
    TargetTempColor12,
    TrackingColorRange12,
    EnableMerge13,
    SourceTempColor13,
    TargetTempColor13,
    TrackingColorRange13,
    EnableMerge14,
    SourceTempColor14,
    TargetTempColor14,
    TrackingColorRange14,
    EnableMerge15,
    SourceTempColor15,
    TargetTempColor15,
    TrackingColorRange15,
    EnableMerge16,
    SourceTempColor16,
    TargetTempColor16,
    TrackingColorRange16,
    EnableMerge17,
    SourceTempColor17,
    TargetTempColor17,
    TrackingColorRange17,
    EnableMerge18,
    SourceTempColor18,
    TargetTempColor18,
    TrackingColorRange18,
    EnableMerge19,
    SourceTempColor19,
    TargetTempColor19,
    TrackingColorRange19,
    EnableMerge20,
    SourceTempColor20,
    TargetTempColor20,
    TrackingColorRange20,
    EnableMerge21,
    SourceTempColor21,
    TargetTempColor21,
    TrackingColorRange21,
    EnableMerge22,
    SourceTempColor22,
    TargetTempColor22,
    TrackingColorRange22,
    EnableMerge23,
    SourceTempColor23,
    TargetTempColor23,
    TrackingColorRange23,
    EnableMerge24,
    SourceTempColor24,
    TargetTempColor24,
    TrackingColorRange24,
    EnableMerge25,
    SourceTempColor25,
    TargetTempColor25,
    TrackingColorRange25,
    EnableMerge26,
    SourceTempColor26,
    TargetTempColor26,
    TrackingColorRange26,
    EnableMerge27,
    SourceTempColor27,
    TargetTempColor27,
    TrackingColorRange27,
    EnableMerge28,
    SourceTempColor28,
    TargetTempColor28,
    TrackingColorRange28,
    EnableMerge29,
    SourceTempColor29,
    TargetTempColor29,
    TrackingColorRange29,
    EnableMerge30,
    SourceTempColor30,
    TargetTempColor30,
    TrackingColorRange30,
    EnableMerge31,
    SourceTempColor31,
    TargetTempColor31,
    TrackingColorRange31,

    GradientGroupStart,
    GradientGroupEnd,
    GradientSettingsCount,
    MasterGradType,
    MasterAngle,
    MasterBias,
    MasterOffset,
    MasterNoiseAmount,
    EnableGradientColor0,
    StartColor0,
    EndColor0,
    InvertGradient0,
    EnableGradientColor1,
    StartColor1,
    EndColor1,
    InvertGradient1,
    EnableGradientColor2,
    StartColor2,
    EndColor2,
    InvertGradient2,
    EnableGradientColor3,
    StartColor3,
    EndColor3,
    InvertGradient3,
    EnableGradientColor4,
    StartColor4,
    EndColor4,
    InvertGradient4,
    EnableGradientColor5,
    StartColor5,
    EndColor5,
    InvertGradient5,
    EnableGradientColor6,
    StartColor6,
    EndColor6,
    InvertGradient6,
    EnableGradientColor7,
    StartColor7,
    EndColor7,
    InvertGradient7,
    EnableGradientColor8,
    StartColor8,
    EndColor8,
    InvertGradient8,
    EnableGradientColor9,
    StartColor9,
    EndColor9,
    InvertGradient9,
    EnableGradientColor10,
    StartColor10,
    EndColor10,
    InvertGradient10,
    EnableGradientColor11,
    StartColor11,
    EndColor11,
    InvertGradient11,
    EnableGradientColor12,
    StartColor12,
    EndColor12,
    InvertGradient12,
    EnableGradientColor13,
    StartColor13,
    EndColor13,
    InvertGradient13,
    EnableGradientColor14,
    StartColor14,
    EndColor14,
    InvertGradient14,
    EnableGradientColor15,
    StartColor15,
    EndColor15,
    InvertGradient15,
    EnableGradientColor16,
    StartColor16,
    EndColor16,
    InvertGradient16,
    EnableGradientColor17,
    StartColor17,
    EndColor17,
    InvertGradient17,
    EnableGradientColor18,
    StartColor18,
    EndColor18,
    InvertGradient18,
    EnableGradientColor19,
    StartColor19,
    EndColor19,
    InvertGradient19,
    EnableGradientColor20,
    StartColor20,
    EndColor20,
    InvertGradient20,
    EnableGradientColor21,
    StartColor21,
    EndColor21,
    InvertGradient21,
    EnableGradientColor22,
    StartColor22,
    EndColor22,
    InvertGradient22,
    EnableGradientColor23,
    StartColor23,
    EndColor23,
    InvertGradient23,
    EnableGradientColor24,
    StartColor24,
    EndColor24,
    InvertGradient24,
    EnableGradientColor25,
    StartColor25,
    EndColor25,
    InvertGradient25,
    EnableGradientColor26,
    StartColor26,
    EndColor26,
    InvertGradient26,
    EnableGradientColor27,
    StartColor27,
    EndColor27,
    InvertGradient27,
    EnableGradientColor28,
    StartColor28,
    EndColor28,
    InvertGradient28,
    EnableGradientColor29,
    StartColor29,
    EndColor29,
    InvertGradient29,
    EnableGradientColor30,
    StartColor30,
    EndColor30,
    InvertGradient30,
    EnableGradientColor31,
    StartColor31,
    EndColor31,
    InvertGradient31,
}

#[derive(Default)]
struct Plugin {
    my_id: ae::aegp::PluginId,
}

ae::define_effect!(Plugin, (), Params);

const PLUGIN_DESCRIPTION: &str =
    "Tracks colored regions as islands and applies per-island gradients or temp colors.";

const EXTRACTION_ENABLE: [Params; EXTRACTION_SETS] = [
    Params::EnableExtraction0,
    Params::EnableExtraction1,
    Params::EnableExtraction2,
    Params::EnableExtraction3,
    Params::EnableExtraction4,
    Params::EnableExtraction5,
    Params::EnableExtraction6,
    Params::EnableExtraction7,
    Params::EnableExtraction8,
    Params::EnableExtraction9,
    Params::EnableExtraction10,
    Params::EnableExtraction11,
    Params::EnableExtraction12,
    Params::EnableExtraction13,
    Params::EnableExtraction14,
    Params::EnableExtraction15,
    Params::EnableExtraction16,
    Params::EnableExtraction17,
    Params::EnableExtraction18,
    Params::EnableExtraction19,
    Params::EnableExtraction20,
    Params::EnableExtraction21,
    Params::EnableExtraction22,
    Params::EnableExtraction23,
    Params::EnableExtraction24,
    Params::EnableExtraction25,
    Params::EnableExtraction26,
    Params::EnableExtraction27,
    Params::EnableExtraction28,
    Params::EnableExtraction29,
    Params::EnableExtraction30,
    Params::EnableExtraction31,
];
const EXTRACTION_TARGET_COLORS: [Params; EXTRACTION_SETS] = [
    Params::TargetColor0,
    Params::TargetColor1,
    Params::TargetColor2,
    Params::TargetColor3,
    Params::TargetColor4,
    Params::TargetColor5,
    Params::TargetColor6,
    Params::TargetColor7,
    Params::TargetColor8,
    Params::TargetColor9,
    Params::TargetColor10,
    Params::TargetColor11,
    Params::TargetColor12,
    Params::TargetColor13,
    Params::TargetColor14,
    Params::TargetColor15,
    Params::TargetColor16,
    Params::TargetColor17,
    Params::TargetColor18,
    Params::TargetColor19,
    Params::TargetColor20,
    Params::TargetColor21,
    Params::TargetColor22,
    Params::TargetColor23,
    Params::TargetColor24,
    Params::TargetColor25,
    Params::TargetColor26,
    Params::TargetColor27,
    Params::TargetColor28,
    Params::TargetColor29,
    Params::TargetColor30,
    Params::TargetColor31,
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
    Params::ColorRange8,
    Params::ColorRange9,
    Params::ColorRange10,
    Params::ColorRange11,
    Params::ColorRange12,
    Params::ColorRange13,
    Params::ColorRange14,
    Params::ColorRange15,
    Params::ColorRange16,
    Params::ColorRange17,
    Params::ColorRange18,
    Params::ColorRange19,
    Params::ColorRange20,
    Params::ColorRange21,
    Params::ColorRange22,
    Params::ColorRange23,
    Params::ColorRange24,
    Params::ColorRange25,
    Params::ColorRange26,
    Params::ColorRange27,
    Params::ColorRange28,
    Params::ColorRange29,
    Params::ColorRange30,
    Params::ColorRange31,
];

const MERGE_ENABLE: [Params; MERGE_ISLAND_SETS] = [
    Params::EnableMerge0,
    Params::EnableMerge1,
    Params::EnableMerge2,
    Params::EnableMerge3,
    Params::EnableMerge4,
    Params::EnableMerge5,
    Params::EnableMerge6,
    Params::EnableMerge7,
    Params::EnableMerge8,
    Params::EnableMerge9,
    Params::EnableMerge10,
    Params::EnableMerge11,
    Params::EnableMerge12,
    Params::EnableMerge13,
    Params::EnableMerge14,
    Params::EnableMerge15,
    Params::EnableMerge16,
    Params::EnableMerge17,
    Params::EnableMerge18,
    Params::EnableMerge19,
    Params::EnableMerge20,
    Params::EnableMerge21,
    Params::EnableMerge22,
    Params::EnableMerge23,
    Params::EnableMerge24,
    Params::EnableMerge25,
    Params::EnableMerge26,
    Params::EnableMerge27,
    Params::EnableMerge28,
    Params::EnableMerge29,
    Params::EnableMerge30,
    Params::EnableMerge31,
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
const TRACKING_COLOR_RANGE: [Params; MERGE_ISLAND_SETS] = [
    Params::TrackingColorRange0,
    Params::TrackingColorRange1,
    Params::TrackingColorRange2,
    Params::TrackingColorRange3,
    Params::TrackingColorRange4,
    Params::TrackingColorRange5,
    Params::TrackingColorRange6,
    Params::TrackingColorRange7,
    Params::TrackingColorRange8,
    Params::TrackingColorRange9,
    Params::TrackingColorRange10,
    Params::TrackingColorRange11,
    Params::TrackingColorRange12,
    Params::TrackingColorRange13,
    Params::TrackingColorRange14,
    Params::TrackingColorRange15,
    Params::TrackingColorRange16,
    Params::TrackingColorRange17,
    Params::TrackingColorRange18,
    Params::TrackingColorRange19,
    Params::TrackingColorRange20,
    Params::TrackingColorRange21,
    Params::TrackingColorRange22,
    Params::TrackingColorRange23,
    Params::TrackingColorRange24,
    Params::TrackingColorRange25,
    Params::TrackingColorRange26,
    Params::TrackingColorRange27,
    Params::TrackingColorRange28,
    Params::TrackingColorRange29,
    Params::TrackingColorRange30,
    Params::TrackingColorRange31,
];

const GRADIENT_ENABLE: [Params; GRADIENT_SETS] = [
    Params::EnableGradientColor0,
    Params::EnableGradientColor1,
    Params::EnableGradientColor2,
    Params::EnableGradientColor3,
    Params::EnableGradientColor4,
    Params::EnableGradientColor5,
    Params::EnableGradientColor6,
    Params::EnableGradientColor7,
    Params::EnableGradientColor8,
    Params::EnableGradientColor9,
    Params::EnableGradientColor10,
    Params::EnableGradientColor11,
    Params::EnableGradientColor12,
    Params::EnableGradientColor13,
    Params::EnableGradientColor14,
    Params::EnableGradientColor15,
    Params::EnableGradientColor16,
    Params::EnableGradientColor17,
    Params::EnableGradientColor18,
    Params::EnableGradientColor19,
    Params::EnableGradientColor20,
    Params::EnableGradientColor21,
    Params::EnableGradientColor22,
    Params::EnableGradientColor23,
    Params::EnableGradientColor24,
    Params::EnableGradientColor25,
    Params::EnableGradientColor26,
    Params::EnableGradientColor27,
    Params::EnableGradientColor28,
    Params::EnableGradientColor29,
    Params::EnableGradientColor30,
    Params::EnableGradientColor31,
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

fn popup_to_count(params: &ae::Parameters<Params>, key: Params) -> usize {
    let val = params
        .get(key)
        .ok()
        .and_then(|p| p.as_popup().ok().map(|pd| pd.value()))
        .unwrap_or(1);
    match val {
        1 => 4,
        2 => 8,
        3 => 16,
        _ => 32,
    }
}

fn update_params_ui_visibility(
    in_data: InData,
    plugin_id: ae::aegp::PluginId,
    params: &mut ae::Parameters<Params>,
) -> Result<(), ae::Error> {
    let ext_count = popup_to_count(params, Params::ExtractionCount);
    let merge_count = popup_to_count(params, Params::MergeIslandCount);
    let grad_count = popup_to_count(params, Params::GradientSettingsCount);

    // Read enable states from original params before any mutation
    let ext_enabled: [bool; EXTRACTION_SETS] = {
        let mut arr = [true; EXTRACTION_SETS];
        for i in 0..EXTRACTION_SETS {
            arr[i] = params
                .get(EXTRACTION_ENABLE[i])
                .ok()
                .and_then(|p| p.as_checkbox().ok().map(|cb| cb.value()))
                .unwrap_or(true);
        }
        arr
    };
    let merge_enabled: [bool; MERGE_ISLAND_SETS] = {
        let mut arr = [true; MERGE_ISLAND_SETS];
        for i in 0..MERGE_ISLAND_SETS {
            arr[i] = params
                .get(MERGE_ENABLE[i])
                .ok()
                .and_then(|p| p.as_checkbox().ok().map(|cb| cb.value()))
                .unwrap_or(true);
        }
        arr
    };
    let grad_enabled: [bool; GRADIENT_SETS] = {
        let mut arr = [true; GRADIENT_SETS];
        for i in 0..GRADIENT_SETS {
            arr[i] = params
                .get(GRADIENT_ENABLE[i])
                .ok()
                .and_then(|p| p.as_checkbox().ok().map(|cb| cb.value()))
                .unwrap_or(true);
        }
        arr
    };

    if in_data.is_premiere() {
        // Premiere Pro: INVISIBLE flag is honored via update_param_ui
        let mut p = params.cloned();

        for i in 0..EXTRACTION_SETS {
            let vis = i < ext_count;
            {
                let mut pd = p.get_mut(EXTRACTION_ENABLE[i])?;
                pd.set_ui_flag(ae::ParamUIFlags::INVISIBLE, !vis);
                pd.update_param_ui()?;
            }
            {
                let mut pd = p.get_mut(EXTRACTION_TARGET_COLORS[i])?;
                pd.set_ui_flag(ae::ParamUIFlags::INVISIBLE, !vis);
                pd.update_param_ui()?;
            }
            {
                let mut pd = p.get_mut(EXTRACTION_COLOR_RANGES[i])?;
                pd.set_ui_flag(ae::ParamUIFlags::INVISIBLE, !vis);
                pd.set_flag(ae::ParamFlag::START_COLLAPSED, true);
                pd.update_param_ui()?;
            }
        }
        for i in 0..MERGE_ISLAND_SETS {
            let vis = i < merge_count;
            {
                let mut pd = p.get_mut(MERGE_ENABLE[i])?;
                pd.set_ui_flag(ae::ParamUIFlags::INVISIBLE, !vis);
                pd.update_param_ui()?;
            }
            {
                let mut pd = p.get_mut(MERGE_SOURCE_TEMP[i])?;
                pd.set_ui_flag(ae::ParamUIFlags::INVISIBLE, !vis);
                pd.update_param_ui()?;
            }
            {
                let mut pd = p.get_mut(MERGE_TARGET_TEMP[i])?;
                pd.set_ui_flag(ae::ParamUIFlags::INVISIBLE, !vis);
                pd.update_param_ui()?;
            }
            {
                let mut pd = p.get_mut(TRACKING_COLOR_RANGE[i])?;
                pd.set_ui_flag(ae::ParamUIFlags::INVISIBLE, !vis);
                pd.set_flag(ae::ParamFlag::START_COLLAPSED, true);
                pd.update_param_ui()?;
            }
        }
        for i in 0..GRADIENT_SETS {
            let vis = i < grad_count;
            {
                let mut pd = p.get_mut(GRADIENT_ENABLE[i])?;
                pd.set_ui_flag(ae::ParamUIFlags::INVISIBLE, !vis);
                pd.update_param_ui()?;
            }
            {
                let mut pd = p.get_mut(GRADIENT_START_COLOR[i])?;
                pd.set_ui_flag(ae::ParamUIFlags::INVISIBLE, !vis);
                pd.update_param_ui()?;
            }
            {
                let mut pd = p.get_mut(GRADIENT_END_COLOR[i])?;
                pd.set_ui_flag(ae::ParamUIFlags::INVISIBLE, !vis);
                pd.update_param_ui()?;
            }
            {
                let mut pd = p.get_mut(GRADIENT_INVERT[i])?;
                pd.set_ui_flag(ae::ParamUIFlags::INVISIBLE, !vis);
                pd.update_param_ui()?;
            }
        }
    } else {
        // After Effects: INVISIBLE is not supported via update_param_ui.
        // Use AEGP DynamicStreamFlags::Hidden to show/hide parameters.
        let effect = in_data.effect();
        let aegp_eff = effect.aegp_effect(plugin_id)?;

        for i in 0..EXTRACTION_SETS {
            let hidden = i >= ext_count;
            let idx_en = params
                .index(EXTRACTION_ENABLE[i])
                .ok_or(ae::Error::InvalidIndex)? as i32;
            let idx_tc = params
                .index(EXTRACTION_TARGET_COLORS[i])
                .ok_or(ae::Error::InvalidIndex)? as i32;
            let idx_cr = params
                .index(EXTRACTION_COLOR_RANGES[i])
                .ok_or(ae::Error::InvalidIndex)? as i32;
            aegp_eff
                .new_stream_by_index(plugin_id, idx_en)?
                .set_dynamic_stream_flag(ae::aegp::DynamicStreamFlags::Hidden, false, hidden)?;
            aegp_eff
                .new_stream_by_index(plugin_id, idx_tc)?
                .set_dynamic_stream_flag(ae::aegp::DynamicStreamFlags::Hidden, false, hidden)?;
            aegp_eff
                .new_stream_by_index(plugin_id, idx_cr)?
                .set_dynamic_stream_flag(ae::aegp::DynamicStreamFlags::Hidden, false, hidden)?;
        }
        for i in 0..MERGE_ISLAND_SETS {
            let hidden = i >= merge_count;
            let idx_en = params
                .index(MERGE_ENABLE[i])
                .ok_or(ae::Error::InvalidIndex)? as i32;
            let idx_src = params
                .index(MERGE_SOURCE_TEMP[i])
                .ok_or(ae::Error::InvalidIndex)? as i32;
            let idx_tgt = params
                .index(MERGE_TARGET_TEMP[i])
                .ok_or(ae::Error::InvalidIndex)? as i32;
            let idx_tcr = params
                .index(TRACKING_COLOR_RANGE[i])
                .ok_or(ae::Error::InvalidIndex)? as i32;
            aegp_eff
                .new_stream_by_index(plugin_id, idx_en)?
                .set_dynamic_stream_flag(ae::aegp::DynamicStreamFlags::Hidden, false, hidden)?;
            aegp_eff
                .new_stream_by_index(plugin_id, idx_src)?
                .set_dynamic_stream_flag(ae::aegp::DynamicStreamFlags::Hidden, false, hidden)?;
            aegp_eff
                .new_stream_by_index(plugin_id, idx_tgt)?
                .set_dynamic_stream_flag(ae::aegp::DynamicStreamFlags::Hidden, false, hidden)?;
            aegp_eff
                .new_stream_by_index(plugin_id, idx_tcr)?
                .set_dynamic_stream_flag(ae::aegp::DynamicStreamFlags::Hidden, false, hidden)?;
        }
        for i in 0..GRADIENT_SETS {
            let hidden = i >= grad_count;
            let idx_en = params
                .index(GRADIENT_ENABLE[i])
                .ok_or(ae::Error::InvalidIndex)? as i32;
            let idx_sc = params
                .index(GRADIENT_START_COLOR[i])
                .ok_or(ae::Error::InvalidIndex)? as i32;
            let idx_ec = params
                .index(GRADIENT_END_COLOR[i])
                .ok_or(ae::Error::InvalidIndex)? as i32;
            let idx_iv = params
                .index(GRADIENT_INVERT[i])
                .ok_or(ae::Error::InvalidIndex)? as i32;
            aegp_eff
                .new_stream_by_index(plugin_id, idx_en)?
                .set_dynamic_stream_flag(ae::aegp::DynamicStreamFlags::Hidden, false, hidden)?;
            aegp_eff
                .new_stream_by_index(plugin_id, idx_sc)?
                .set_dynamic_stream_flag(ae::aegp::DynamicStreamFlags::Hidden, false, hidden)?;
            aegp_eff
                .new_stream_by_index(plugin_id, idx_ec)?
                .set_dynamic_stream_flag(ae::aegp::DynamicStreamFlags::Hidden, false, hidden)?;
            aegp_eff
                .new_stream_by_index(plugin_id, idx_iv)?
                .set_dynamic_stream_flag(ae::aegp::DynamicStreamFlags::Hidden, false, hidden)?;
        }
    }

    // DISABLED (grayed-out) handling: update_param_ui honors DISABLED in both AE and Premiere
    {
        let mut p = params.cloned();
        for i in 0..EXTRACTION_SETS {
            if i < ext_count {
                let sub_disabled = !ext_enabled[i];
                {
                    let mut pd = p.get_mut(EXTRACTION_TARGET_COLORS[i])?;
                    pd.set_ui_flag(ae::ParamUIFlags::DISABLED, sub_disabled);
                    pd.update_param_ui()?;
                }
                {
                    let mut pd = p.get_mut(EXTRACTION_COLOR_RANGES[i])?;
                    pd.set_ui_flag(ae::ParamUIFlags::DISABLED, sub_disabled);
                    pd.set_flag(ae::ParamFlag::START_COLLAPSED, true);
                    pd.update_param_ui()?;
                }
            }
        }
        for i in 0..MERGE_ISLAND_SETS {
            if i < merge_count {
                let sub_disabled = !merge_enabled[i];
                {
                    let mut pd = p.get_mut(MERGE_SOURCE_TEMP[i])?;
                    pd.set_ui_flag(ae::ParamUIFlags::DISABLED, sub_disabled);
                    pd.update_param_ui()?;
                }
                {
                    let mut pd = p.get_mut(MERGE_TARGET_TEMP[i])?;
                    pd.set_ui_flag(ae::ParamUIFlags::DISABLED, sub_disabled);
                    pd.update_param_ui()?;
                }
                {
                    let mut pd = p.get_mut(TRACKING_COLOR_RANGE[i])?;
                    pd.set_ui_flag(ae::ParamUIFlags::DISABLED, sub_disabled);
                    pd.set_flag(ae::ParamFlag::START_COLLAPSED, true);
                    pd.update_param_ui()?;
                }
            }
        }
        for i in 0..GRADIENT_SETS {
            if i < grad_count {
                let sub_disabled = !grad_enabled[i];
                {
                    let mut pd = p.get_mut(GRADIENT_START_COLOR[i])?;
                    pd.set_ui_flag(ae::ParamUIFlags::DISABLED, sub_disabled);
                    pd.update_param_ui()?;
                }
                {
                    let mut pd = p.get_mut(GRADIENT_END_COLOR[i])?;
                    pd.set_ui_flag(ae::ParamUIFlags::DISABLED, sub_disabled);
                    pd.update_param_ui()?;
                }
                {
                    let mut pd = p.get_mut(GRADIENT_INVERT[i])?;
                    pd.set_ui_flag(ae::ParamUIFlags::DISABLED, sub_disabled);
                    pd.update_param_ui()?;
                }
            }
        }
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

        params.add_group(
            Params::ColorExtGroupStart,
            Params::ColorExtGroupEnd,
            "Color Extraction",
            true,
            |params| {
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
                params.add(
                    Params::InvertExtraction,
                    "Invert Extraction",
                    CheckBoxDef::setup(|d| {
                        d.set_default(false);
                    }),
                )?;
                params.add_with_flags(
                    Params::ExtractionCount,
                    "Extraction Count",
                    PopupDef::setup(|d| {
                        d.set_options(&["4", "8", "16", "32"]);
                        d.set_default(1);
                    }),
                    ParamFlag::SUPERVISE,
                    ParamUIFlags::NONE,
                )?;

                for i in 0..EXTRACTION_SETS {
                    let n = i + 1;
                    let hidden = i >= INITIAL_EXTRACTION;
                    params.add_with_flags(
                        EXTRACTION_ENABLE[i],
                        &format!("Enable Extraction {n}"),
                        CheckBoxDef::setup(|d| {
                            d.set_default(true);
                        }),
                        ParamFlag::SUPERVISE,
                        if hidden {
                            ParamUIFlags::INVISIBLE
                        } else {
                            ParamUIFlags::NONE
                        },
                    )?;
                    params.add_with_flags(
                        EXTRACTION_TARGET_COLORS[i],
                        &format!("Target Color {n}"),
                        ColorDef::setup(|_| {}),
                        ParamFlag::empty(),
                        if hidden {
                            ParamUIFlags::INVISIBLE
                        } else {
                            ParamUIFlags::NONE
                        },
                    )?;
                    params.add_with_flags(
                        EXTRACTION_COLOR_RANGES[i],
                        &format!("Color Range {n}"),
                        FloatSliderDef::setup(|d| {
                            d.set_valid_min(0.0);
                            d.set_valid_max(100.0);
                            d.set_slider_min(0.0);
                            d.set_slider_max(50.0);
                            d.set_default(0.0);
                            d.set_precision(1);
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
                params.add_with_flags(
                    Params::MergeIslandCount,
                    "Merge Island Count",
                    PopupDef::setup(|d| {
                        d.set_options(&["4", "8", "16", "32"]);
                        d.set_default(1);
                    }),
                    ParamFlag::SUPERVISE,
                    ParamUIFlags::NONE,
                )?;

                for i in 0..MERGE_ISLAND_SETS {
                    let n = i + 1;
                    let hidden = i >= INITIAL_MERGE;
                    let enable_default = i == 0;
                    params.add_with_flags(
                        MERGE_ENABLE[i],
                        &format!("Enable Merge {n}"),
                        CheckBoxDef::setup(move |d| {
                            d.set_default(enable_default);
                        }),
                        ParamFlag::SUPERVISE,
                        if hidden {
                            ParamUIFlags::INVISIBLE
                        } else {
                            ParamUIFlags::NONE
                        },
                    )?;
                    params.add_with_flags(
                        MERGE_SOURCE_TEMP[i],
                        &format!("Source Temp Color {n}"),
                        ColorDef::setup(|_| {}),
                        ParamFlag::empty(),
                        if hidden {
                            ParamUIFlags::INVISIBLE
                        } else {
                            ParamUIFlags::NONE
                        },
                    )?;
                    params.add_with_flags(
                        MERGE_TARGET_TEMP[i],
                        &format!("Target Temp Color {n}"),
                        ColorDef::setup(|_| {}),
                        ParamFlag::empty(),
                        if hidden {
                            ParamUIFlags::INVISIBLE
                        } else {
                            ParamUIFlags::NONE
                        },
                    )?;
                    params.add_with_flags(
                        TRACKING_COLOR_RANGE[i],
                        &format!("Tracking Color Range {n}"),
                        FloatSliderDef::setup(|d| {
                            d.set_valid_min(0.0);
                            d.set_valid_max(100.0);
                            d.set_slider_min(0.0);
                            d.set_slider_max(50.0);
                            d.set_default(5.0);
                            d.set_precision(1);
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

        params.add_group(
            Params::GradientGroupStart,
            Params::GradientGroupEnd,
            "Gradient Render",
            true,
            |params| {
                params.add(
                    Params::MasterGradType,
                    "Master Grad Type",
                    PopupDef::setup(|d| {
                        d.set_options(&["Linear", "Radial"]);
                        d.set_default(1);
                    }),
                )?;
                params.add(Params::MasterAngle, "Master Angle", AngleDef::setup(|_| {}))?;
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
                params.add_with_flags(
                    Params::GradientSettingsCount,
                    "Gradient Settings Count",
                    PopupDef::setup(|d| {
                        d.set_options(&["4", "8", "16", "32"]);
                        d.set_default(1);
                    }),
                    ParamFlag::SUPERVISE,
                    ParamUIFlags::NONE,
                )?;

                let white = ae::Pixel8 {
                    red: 255,
                    green: 255,
                    blue: 255,
                    alpha: 255,
                };
                for i in 0..GRADIENT_SETS {
                    let n = i + 1;
                    let hidden = i >= INITIAL_GRADIENT;
                    let enable_default = i == 0;
                    params.add_with_flags(
                        GRADIENT_ENABLE[i],
                        &format!("Enable Gradient Color {n}"),
                        CheckBoxDef::setup(move |d| {
                            d.set_default(enable_default);
                        }),
                        ParamFlag::SUPERVISE,
                        if hidden {
                            ParamUIFlags::INVISIBLE
                        } else {
                            ParamUIFlags::NONE
                        },
                    )?;
                    params.add_with_flags(
                        GRADIENT_START_COLOR[i],
                        &format!("Start Color {n}"),
                        ColorDef::setup(|d| {
                            d.set_default(white);
                        }),
                        ParamFlag::empty(),
                        if hidden {
                            ParamUIFlags::INVISIBLE
                        } else {
                            ParamUIFlags::NONE
                        },
                    )?;
                    params.add_with_flags(
                        GRADIENT_END_COLOR[i],
                        &format!("End Color {n}"),
                        ColorDef::setup(|_| {}),
                        ParamFlag::empty(),
                        if hidden {
                            ParamUIFlags::INVISIBLE
                        } else {
                            ParamUIFlags::NONE
                        },
                    )?;
                    params.add_with_flags(
                        GRADIENT_INVERT[i],
                        &format!("Invert Gradient {n}"),
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
                out_data.set_out_flag2(OutFlags2::ParamGroupStartCollapsedFlag, true);
                if let Ok(suite) = ae::aegp::suites::Utility::new()
                    && let Ok(id) = suite.register_with_aegp("AOD_IslandIdColor")
                {
                    self.my_id = id;
                }
            }
            ae::Command::UpdateParamsUi => {
                update_params_ui_visibility(in_data, self.my_id, params)?;
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

// ---------------------------------------------------------------------------
// Union-Find (path-halving, union by min-root)
// ---------------------------------------------------------------------------
struct UnionFind {
    parent: Vec<u32>,
}

impl UnionFind {
    fn new(n: usize) -> Self {
        Self {
            parent: (0..n as u32).collect(),
        }
    }

    fn find(&mut self, mut x: u32) -> u32 {
        while self.parent[x as usize] != x {
            let pp = self.parent[self.parent[x as usize] as usize];
            self.parent[x as usize] = pp;
            x = pp;
        }
        x
    }

    fn union(&mut self, x: u32, y: u32) {
        let rx = self.find(x);
        let ry = self.find(y);
        if rx != ry {
            if rx < ry {
                self.parent[ry as usize] = rx;
            } else {
                self.parent[rx as usize] = ry;
            }
        }
    }

    fn ensure_size(&mut self, label: u32) {
        let need = label as usize + 1;
        if self.parent.len() < need {
            let start = self.parent.len() as u32;
            self.parent.extend(start..need as u32);
        }
    }
}

/// 2値マスクに対して4連結CCLを実行し、各ピクセルのアイランドID配列を返す。
/// 0 = 背景、1〜 = アイランド（連番に圧縮済み）。
fn compute_ccl(mask: &[bool], width: usize, height: usize) -> Vec<u32> {
    let n = width * height;
    let mut labels = vec![0u32; n];
    let mut uf = UnionFind::new(1); // label 0 reserved for background
    let mut next_label = 1u32;

    // First pass: assign provisional labels and record equivalences
    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            if !mask[idx] {
                continue;
            }
            let left = if x > 0 { labels[idx - 1] } else { 0 };
            let top = if y > 0 { labels[idx - width] } else { 0 };

            match (left, top) {
                (0, 0) => {
                    uf.ensure_size(next_label);
                    labels[idx] = next_label;
                    next_label += 1;
                }
                (a, 0) | (0, a) => {
                    labels[idx] = a;
                }
                (a, b) => {
                    uf.union(a, b);
                    labels[idx] = a.min(b);
                }
            }
        }
    }

    // Second pass: resolve labels through Union-Find
    for label in labels.iter_mut() {
        if *label != 0 {
            *label = uf.find(*label);
        }
    }

    // Compact provisional IDs to consecutive 1, 2, 3, ...
    let mut id_map: std::collections::HashMap<u32, u32> = std::collections::HashMap::new();
    let mut next_id = 1u32;
    for label in labels.iter_mut() {
        if *label != 0 {
            let entry = id_map.entry(*label).or_insert_with(|| {
                let id = next_id;
                next_id += 1;
                id
            });
            *label = *entry;
        }
    }

    labels
}

/// アイランドIDを疑似カラーに変換する（0 = 背景 → 透明黒）。
fn island_id_to_color(id: u32) -> PixelF32 {
    if id == 0 {
        return PixelF32 {
            red: 0.0,
            green: 0.0,
            blue: 0.0,
            alpha: 0.0,
        };
    }
    PixelF32 {
        red: (id.wrapping_mul(50) % 255) as f32 / 255.0,
        green: (id.wrapping_mul(80) % 255) as f32 / 255.0,
        blue: (id.wrapping_mul(110) % 255) as f32 / 255.0,
        alpha: 1.0,
    }
}

fn color_distance_f32(a: &PixelF32, b: &PixelF32) -> f32 {
    let dr = a.red - b.red;
    let dg = a.green - b.green;
    let db = a.blue - b.blue;
    (dr * dr + dg * dg + db * db).sqrt()
}

fn target_color_to_f32(c: &ae::Pixel8) -> PixelF32 {
    let scale = 1.0 / ae::MAX_CHANNEL8 as f32;
    PixelF32 {
        red: c.red as f32 * scale,
        green: c.green as f32 * scale,
        blue: c.blue as f32 * scale,
        alpha: c.alpha as f32 * scale,
    }
}

fn read_pixel_f32(layer: &Layer, world_type: ae::aegp::WorldType, x: usize, y: usize) -> PixelF32 {
    match world_type {
        ae::aegp::WorldType::U8 => layer.as_pixel8(x, y).to_pixel32(),
        ae::aegp::WorldType::U15 => layer.as_pixel16(x, y).to_pixel32(),
        ae::aegp::WorldType::F32 | ae::aegp::WorldType::None => *layer.as_pixel32(x, y),
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
        let extraction_count = popup_to_count(params, Params::ExtractionCount).min(EXTRACTION_SETS);

        // EnableExtraction フラグを読み取り、有効なターゲットだけを収集
        let mut extraction_targets: Vec<(PixelF32, f32)> = Vec::with_capacity(extraction_count);
        for i in 0..extraction_count {
            let enabled = params
                .get(EXTRACTION_ENABLE[i])
                .ok()
                .and_then(|p| p.as_checkbox().ok().map(|cb| cb.value()))
                .unwrap_or(true);
            if !enabled {
                continue;
            }
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
            // AEのスポイトはディスプレイカラーマネジメントを経由するため、
            // レイヤーバッファ内のf32値と完全一致しないことがある。
            // Range=0 でも確実に拾えるよう最小 epsilon (≒ 0.1%スライダー相当) を設ける。
            let range_f32 = (color_range_val / 100.0).clamp(0.0, 1.0).max(1e-3_f32);
            extraction_targets.push((target_color_to_f32(&target_color), range_f32));
        }

        let width = in_layer.width();
        let height = in_layer.height();

        // Tracking ターゲットの収集（スロットインデックス付き）
        // 要素: (slot_index, src_color, tgt_color, range)
        let merge_count = popup_to_count(params, Params::MergeIslandCount).min(MERGE_ISLAND_SETS);
        let mut tracking_targets: Vec<(usize, PixelF32, PixelF32, f32)> =
            Vec::with_capacity(merge_count);
        for i in 0..merge_count {
            let enabled = params
                .get(MERGE_ENABLE[i])
                .ok()
                .and_then(|p| p.as_checkbox().ok().map(|cb| cb.value()))
                .unwrap_or(true);
            if !enabled {
                continue;
            }
            let src_color = params
                .get(MERGE_SOURCE_TEMP[i])
                .ok()
                .and_then(|p| p.as_color().ok().map(|cd| cd.value()));
            let tgt_color = params
                .get(MERGE_TARGET_TEMP[i])
                .ok()
                .and_then(|p| p.as_color().ok().map(|cd| cd.value()));
            let range_val = params
                .get(TRACKING_COLOR_RANGE[i])
                .ok()
                .and_then(|p| p.as_float_slider().ok().map(|fs| fs.value()))
                .unwrap_or(5.0) as f32;
            // スポイト時の色空間変換誤差を吸収する最小 epsilon を設ける
            let range_f32 = (range_val / 100.0).clamp(0.0, 1.0).max(1e-3_f32);
            if let (Some(src), Some(tgt)) = (src_color, tgt_color) {
                tracking_targets.push((
                    i,
                    target_color_to_f32(&src),
                    target_color_to_f32(&tgt),
                    range_f32,
                ));
            }
        }

        // Temp Color モードの場合のみ CCL を実行
        let island_labels: Option<Vec<u32>> = if output_mode == 3 {
            // ─── Step1: 2値マスクを構築 ─────────────────────────────
            let mut mask = vec![false; width * height];
            for y in 0..height {
                for x in 0..width {
                    let px = read_pixel_f32(&in_layer, in_world_type, x, y);
                    let extracted = extraction_targets
                        .iter()
                        .any(|(target, range)| color_distance_f32(&px, target) <= *range);
                    mask[y * width + x] = extracted != invert_extraction;
                }
            }

            // ─── Step2: CCL（仮ラベル） ─────────────────────────────
            let raw_labels = compute_ccl(&mask, width, height);

            // ─── Step3: Source Temp Color によるアンカーベース ID マッピング ──
            // 競合解決ルール: 1つの島に複数の Source Color がヒットした場合、
            //   島の任意ピクセルとの色差が最小のスロットを優先する。
            // user_id = slot_index + 1 (1〜32。0 は背景予約)
            //
            // 中間テーブル: label → (best_user_id, best_dist_so_far)
            // 全ピクセルを走査し、各島ごとに最小距離スロットを確定する。
            let label_to_user_id: std::collections::HashMap<u32, u32> = {
                let mut label_best: std::collections::HashMap<u32, (u32, f32)> =
                    std::collections::HashMap::new();
                if !tracking_targets.is_empty() {
                    for y in 0..height {
                        for x in 0..width {
                            let lbl = raw_labels[y * width + x];
                            if lbl == 0 {
                                continue;
                            }
                            let px = read_pixel_f32(&in_layer, in_world_type, x, y);
                            for (slot_idx, src_color, _, range) in &tracking_targets {
                                let dist = color_distance_f32(&px, src_color);
                                if dist <= *range {
                                    let uid = (*slot_idx as u32) + 1;
                                    // この島にまだ候補がないか、より近い色差なら更新
                                    let entry = label_best.entry(lbl).or_insert((uid, f32::MAX));
                                    if dist < entry.1 {
                                        *entry = (uid, dist);
                                    }
                                }
                            }
                        }
                    }
                }
                // 最小距離のスロットだけを残す（距離情報は不要なので捨てる）
                label_best
                    .into_iter()
                    .map(|(lbl, (uid, _))| (lbl, uid))
                    .collect()
            };

            // ─── Step4: 最終ラベル配列を構築 ───────────────────────
            // user_id 1〜32   : ユーザー指定スロットに紐づいた島（安定色）
            // user_id 33+     : 自動割り当て（その他のノイズ・未追跡島）
            let mut next_untracked = (MERGE_ISLAND_SETS as u32) + 1; // 33
            let mut untracked_remap: std::collections::HashMap<u32, u32> =
                std::collections::HashMap::new();

            let remapped: Vec<u32> = raw_labels
                .iter()
                .map(|&lbl| {
                    if lbl == 0 {
                        0
                    } else if let Some(&uid) = label_to_user_id.get(&lbl) {
                        uid
                    } else {
                        *untracked_remap.entry(lbl).or_insert_with(|| {
                            let id = next_untracked;
                            next_untracked += 1;
                            id
                        })
                    }
                })
                .collect();

            // ─── 将来の Target 置換に向けたマッピング構造を準備 ────
            // user_id → Target Temp Color。
            // 現時点は未使用だが、OutputMode "Final Gradient" 等で活用予定。
            let _island_to_target: std::collections::HashMap<u32, PixelF32> = tracking_targets
                .iter()
                .map(|(slot_idx, _, tgt_color, _)| ((*slot_idx as u32) + 1, *tgt_color))
                .collect();

            Some(remapped)
        } else {
            None
        };

        let out_wt = out_world_type;
        out_layer.iterate(0, progress_final, None, |x, y, mut dst| {
            let px = read_pixel_f32(&in_layer, in_world_type, x as usize, y as usize);
            let out_px = match output_mode {
                1 => px,
                2 => {
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
                3 => {
                    let idx = y as usize * width + x as usize;
                    let id = island_labels.as_ref().map(|l| l[idx]).unwrap_or(0);
                    island_id_to_color(id)
                }
                4 => px,
                _ => px,
            };
            match out_wt {
                ae::aegp::WorldType::U8 => dst.set_from_u8(out_px.to_pixel8()),
                ae::aegp::WorldType::U15 => dst.set_from_u16(out_px.to_pixel16()),
                ae::aegp::WorldType::F32 | ae::aegp::WorldType::None => dst.set_from_f32(out_px),
            }
            Ok(())
        })?;
        Ok(())
    }
}
