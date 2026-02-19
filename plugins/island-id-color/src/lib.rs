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
    AlphaThreshold,

    IslandTrackGroupStart,
    IslandTrackGroupEnd,
    TrackingPath,
    ShowTempColors,
    GrayscaleTempColor,
    IslandSort,
    SortAngle,
    SortMaskIndex,
    TrackingAlgorithm,
    AlgoColorScale,
    AlgoAreaWeight,
    AlgoIouThreshold,
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
    GradCenterPoint,
    GradMaskIndex,
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
    let tracking_algo: i32 = params
        .get(Params::TrackingAlgorithm)
        .ok()
        .and_then(|p| p.as_popup().ok().map(|pd| pd.value()))
        .unwrap_or(1);
    let island_sort: i32 = params
        .get(Params::IslandSort)
        .ok()
        .and_then(|p| p.as_popup().ok().map(|pd| pd.value()))
        .unwrap_or(1);
    let master_grad_type: i32 = params
        .get(Params::MasterGradType)
        .ok()
        .and_then(|p| p.as_popup().ok().map(|pd| pd.value()))
        .unwrap_or(1);

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
        // アルゴリズム専用パラメータの表示切り替え
        {
            let mut pd = p.get_mut(Params::AlgoColorScale)?;
            pd.set_ui_flag(ae::ParamUIFlags::INVISIBLE, tracking_algo != 1);
            pd.set_flag(ae::ParamFlag::START_COLLAPSED, true);
            pd.update_param_ui()?;
        }
        {
            let mut pd = p.get_mut(Params::AlgoAreaWeight)?;
            pd.set_ui_flag(ae::ParamUIFlags::INVISIBLE, tracking_algo != 2);
            pd.set_flag(ae::ParamFlag::START_COLLAPSED, true);
            pd.update_param_ui()?;
        }
        {
            let mut pd = p.get_mut(Params::AlgoIouThreshold)?;
            pd.set_ui_flag(ae::ParamUIFlags::INVISIBLE, tracking_algo != 3);
            pd.set_flag(ae::ParamFlag::START_COLLAPSED, true);
            pd.update_param_ui()?;
        }
        {
            let mut pd = p.get_mut(Params::SortAngle)?;
            pd.set_ui_flag(ae::ParamUIFlags::INVISIBLE, island_sort != 8);
            pd.set_flag(ae::ParamFlag::START_COLLAPSED, true);
            pd.update_param_ui()?;
        }
        {
            let mut pd = p.get_mut(Params::SortMaskIndex)?;
            pd.set_ui_flag(ae::ParamUIFlags::INVISIBLE, island_sort != 9);
            pd.update_param_ui()?;
        }
        {
            let mut pd = p.get_mut(Params::GradCenterPoint)?;
            pd.set_ui_flag(ae::ParamUIFlags::INVISIBLE, master_grad_type != 2);
            pd.update_param_ui()?;
        }
        {
            let mut pd = p.get_mut(Params::GradMaskIndex)?;
            pd.set_ui_flag(ae::ParamUIFlags::INVISIBLE, master_grad_type != 3);
            pd.update_param_ui()?;
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
        // アルゴリズム専用パラメータの Hidden 切り替え
        {
            let idx = params
                .index(Params::AlgoColorScale)
                .ok_or(ae::Error::InvalidIndex)? as i32;
            aegp_eff
                .new_stream_by_index(plugin_id, idx)?
                .set_dynamic_stream_flag(
                    ae::aegp::DynamicStreamFlags::Hidden,
                    false,
                    tracking_algo != 1,
                )?;
        }
        {
            let idx = params
                .index(Params::AlgoAreaWeight)
                .ok_or(ae::Error::InvalidIndex)? as i32;
            aegp_eff
                .new_stream_by_index(plugin_id, idx)?
                .set_dynamic_stream_flag(
                    ae::aegp::DynamicStreamFlags::Hidden,
                    false,
                    tracking_algo != 2,
                )?;
        }
        {
            let idx = params
                .index(Params::AlgoIouThreshold)
                .ok_or(ae::Error::InvalidIndex)? as i32;
            aegp_eff
                .new_stream_by_index(plugin_id, idx)?
                .set_dynamic_stream_flag(
                    ae::aegp::DynamicStreamFlags::Hidden,
                    false,
                    tracking_algo != 3,
                )?;
        }
        {
            let idx = params
                .index(Params::SortAngle)
                .ok_or(ae::Error::InvalidIndex)? as i32;
            aegp_eff
                .new_stream_by_index(plugin_id, idx)?
                .set_dynamic_stream_flag(
                    ae::aegp::DynamicStreamFlags::Hidden,
                    false,
                    island_sort != 8,
                )?;
        }
        {
            let idx = params
                .index(Params::SortMaskIndex)
                .ok_or(ae::Error::InvalidIndex)? as i32;
            aegp_eff
                .new_stream_by_index(plugin_id, idx)?
                .set_dynamic_stream_flag(
                    ae::aegp::DynamicStreamFlags::Hidden,
                    false,
                    island_sort != 9,
                )?;
        }
        {
            let idx = params
                .index(Params::GradCenterPoint)
                .ok_or(ae::Error::InvalidIndex)? as i32;
            aegp_eff
                .new_stream_by_index(plugin_id, idx)?
                .set_dynamic_stream_flag(
                    ae::aegp::DynamicStreamFlags::Hidden,
                    false,
                    master_grad_type != 2,
                )?;
        }
        {
            let idx = params
                .index(Params::GradMaskIndex)
                .ok_or(ae::Error::InvalidIndex)? as i32;
            aegp_eff
                .new_stream_by_index(plugin_id, idx)?
                .set_dynamic_stream_flag(
                    ae::aegp::DynamicStreamFlags::Hidden,
                    false,
                    master_grad_type != 3,
                )?;
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

    // ─── SortMaskIndex / GradMaskIndex ポップアップをレイヤーの実際のマスク名で更新 ──
    // PF_PathQuerySuite は UpdateParamsUi では使用不可のため、
    // AEGP (PFInterface + Mask + Stream) を使用してマスク名を取得する。
    // AEGP スイートは UpdateParamsUi でも動作する。
    //
    // 注意: AE では PF_UpdateParamUI でポップアップの選択肢「数」を変更できない。
    // 常に初期定義と同じ件数（MASK_POPUP_SLOTS = 4）を保持する。
    {
        const MASK_POPUP_SLOTS: usize = 4;
        let mask_names: Vec<String> = if !in_data.is_premiere() {
            let aegp_names: Option<Vec<String>> = (|| -> Option<Vec<String>> {
                let pf_iface = ae::aegp::suites::PFInterface::new().ok()?;
                let layer = pf_iface.effect_layer(in_data.effect_ref()).ok()?;
                let mask_suite = ae::aegp::suites::Mask::new().ok()?;
                let stream_suite = ae::aegp::suites::Stream::new().ok()?;
                let dyn_suite = ae::aegp::suites::DynamicStream::new().ok()?;
                let num = mask_suite.layer_num_masks(&layer).ok()? as usize;
                let names: Vec<String> = (0..MASK_POPUP_SLOTS)
                    .map(|i| -> String {
                        if i < num {
                            // マスクの Outline ストリームを取得し、
                            // その親ストリーム（マスクグループ）の名前を取得する。
                            // これがマスクの表示名（例: "マスク 2"）になる。
                            let maybe_name: Option<String> = (|| -> Option<String> {
                                let mask_ref =
                                    mask_suite.layer_mask_by_index(&layer, i as i32).ok()?;
                                let mask_stream = stream_suite
                                    .new_mask_stream(
                                        &mask_ref,
                                        plugin_id,
                                        ae::aegp::MaskStream::Outline,
                                    )
                                    .ok()?;
                                let parent_stream = dyn_suite
                                    .new_parent_stream_ref(&mask_stream, plugin_id)
                                    .ok()?;
                                stream_suite
                                    .stream_name(&parent_stream, plugin_id, false)
                                    .ok()
                            })();
                            maybe_name.unwrap_or_else(|| format!("Mask {}", i + 1))
                        } else {
                            format!("Mask {}", i + 1)
                        }
                    })
                    .collect();
                Some(names)
            })();
            aegp_names.unwrap_or_else(|| {
                (1..=MASK_POPUP_SLOTS)
                    .map(|i| format!("Mask {}", i))
                    .collect()
            })
        } else {
            (1..=MASK_POPUP_SLOTS)
                .map(|i| format!("Mask {}", i))
                .collect()
        };
        let options_ref: Vec<&str> = mask_names.iter().map(|s| s.as_str()).collect();
        {
            let mut p = params.cloned();
            let mut popup = p.get_mut(Params::SortMaskIndex)?;
            popup.as_popup_mut()?.set_options(&options_ref);
            popup.update_param_ui()?;
        }
        {
            let mut p = params.cloned();
            let mut popup = p.get_mut(Params::GradMaskIndex)?;
            popup.as_popup_mut()?.set_options(&options_ref);
            popup.update_param_ui()?;
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
                // トラックマットや合成後のピクセルに残る半透明成分を除外するための
                // alpha 閾値。0% = 完全透明のみ除外、5% = alpha 5% 未満を透明扱い。
                // トラックマットのソフトエッジが誤抽出される場合は値を上げる。
                params.add_with_flags(
                    Params::AlphaThreshold,
                    "Alpha Threshold (%)",
                    FloatSliderDef::setup(|d| {
                        d.set_valid_min(0.0);
                        d.set_valid_max(100.0);
                        d.set_slider_min(0.0);
                        d.set_slider_max(50.0);
                        d.set_default(2.0);
                        d.set_precision(1);
                    }),
                    ParamFlag::START_COLLAPSED,
                    ParamUIFlags::NONE,
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
                // TempColor を白黒グレースケールで表示するモード。
                // ソート順を視覚的に確認するために使う（ID小=暗、ID大=明）。
                params.add(
                    Params::GrayscaleTempColor,
                    "Grayscale Temp Color",
                    CheckBoxDef::setup(|d| {
                        d.set_default(false);
                    }),
                )?;
                // ─── 空間ソートモード ──────────────────────────────────
                // Island Sort が Off 以外のとき、CCL 後の島を重心/面積でソートして
                // スキャン順によるチラつきを解消する。
                // Off のときは下記 Tracking Algorithm による色ベーストラッキングを使用。
                params.add_with_flags(
                    Params::IslandSort,
                    "Island Sort",
                    PopupDef::setup(|d| {
                        d.set_options(&[
                            "Off",
                            "Left to Right",
                            "Right to Left",
                            "Top to Bottom",
                            "Bottom to Top",
                            "Largest First",
                            "Smallest First",
                            "By Angle",
                            "Mask Path",
                        ]);
                        d.set_default(1);
                    }),
                    ParamFlag::SUPERVISE,
                    ParamUIFlags::NONE,
                )?;
                // Island Sort = "By Angle" のときのみ表示される方向角度。
                // 0° = 左→右、90° = 上→下（スクリーン座標）、180° = 右→左、270° = 下→上。
                // 弧状に並ぶアイランドは弧の接線方向に合わせることで安定ソートが得られる。
                params.add_with_flags(
                    Params::SortAngle,
                    "Sort Angle",
                    AngleDef::setup(|d| {
                        d.set_default(0.0);
                    }),
                    ParamFlag::empty(),
                    ParamUIFlags::NONE,
                )?;
                // Island Sort = "Mask Path" のときのみ表示。
                // レイヤーに複数マスクがある場合、どのマスクをソートパスとして使うかを選ぶ。
                // マスクのモードは "None" にすることで実際の切り抜きには影響しない。
                params.add_with_flags(
                    Params::SortMaskIndex,
                    "Sort Mask",
                    PopupDef::setup(|d| {
                        d.set_options(&["Mask 1", "Mask 2", "Mask 3", "Mask 4"]);
                        d.set_default(1);
                    }),
                    ParamFlag::empty(),
                    ParamUIFlags::NONE,
                )?;
                // ─── アルゴリズム選択 ─────────────────────────────────
                // Three matching methods: selector switches the dedicated slider shown below.
                params.add_with_flags(
                    Params::TrackingAlgorithm,
                    "Tracking Algorithm",
                    PopupDef::setup(|d| {
                        d.set_options(&["Color Match", "Area Weighted", "IoU Overlap"]);
                        d.set_default(1);
                    }),
                    ParamFlag::SUPERVISE,
                    ParamUIFlags::NONE,
                )?;
                // algo=1: color-distance scale multiplier (100% = use TrackingColorRange as-is)
                params.add_with_flags(
                    Params::AlgoColorScale,
                    "Color Scale (%)",
                    FloatSliderDef::setup(|d| {
                        d.set_valid_min(0.0);
                        d.set_valid_max(500.0);
                        d.set_slider_min(0.0);
                        d.set_slider_max(200.0);
                        d.set_default(100.0);
                        d.set_precision(0);
                    }),
                    ParamFlag::empty(),
                    ParamUIFlags::NONE,
                )?;
                // algo=2: weight for area-difference score (0=color only, 100=area only)
                params.add_with_flags(
                    Params::AlgoAreaWeight,
                    "Area Weight (%)",
                    FloatSliderDef::setup(|d| {
                        d.set_valid_min(0.0);
                        d.set_valid_max(100.0);
                        d.set_slider_min(0.0);
                        d.set_slider_max(100.0);
                        d.set_default(50.0);
                        d.set_precision(0);
                    }),
                    ParamFlag::empty(),
                    ParamUIFlags::INVISIBLE,
                )?;
                // algo=3: minimum IoU required to accept a match (0-100%)
                params.add_with_flags(
                    Params::AlgoIouThreshold,
                    "IoU Threshold (%)",
                    FloatSliderDef::setup(|d| {
                        d.set_valid_min(0.0);
                        d.set_valid_max(100.0);
                        d.set_slider_min(0.0);
                        d.set_slider_max(100.0);
                        d.set_default(30.0);
                        d.set_precision(0);
                    }),
                    ParamFlag::empty(),
                    ParamUIFlags::INVISIBLE,
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
                params.add_with_flags(
                    Params::MasterGradType,
                    "Master Grad Type",
                    PopupDef::setup(|d| {
                        d.set_options(&["Linear", "Radial", "Mask Path"]);
                        d.set_default(1);
                    }),
                    ParamFlag::SUPERVISE,
                    ParamUIFlags::NONE,
                )?;
                params.add(Params::MasterAngle, "Master Angle", AngleDef::setup(|_| {}))?;
                // Radial モード時のみ表示。グラデーションの中心座標（0.0=左/上、1.0=右/下）。
                params.add_with_flags(
                    Params::GradCenterPoint,
                    "Grad Center Point",
                    PointDef::setup(|d| {
                        d.set_default_x(0.5);
                        d.set_default_y(0.5);
                    }),
                    ParamFlag::empty(),
                    ParamUIFlags::NONE,
                )?;
                // Mask Path モード時のみ表示。グラデーション方向に使うマスクを選択。
                params.add_with_flags(
                    Params::GradMaskIndex,
                    "Grad Mask",
                    PopupDef::setup(|d| {
                        d.set_options(&["Mask 1", "Mask 2", "Mask 3", "Mask 4"]);
                        d.set_default(1);
                    }),
                    ParamFlag::empty(),
                    ParamUIFlags::NONE,
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

/// アイランドIDをグレースケール値に変換する。
/// id=1 が最も暗く、id=total が白（1.0）になるよう均等分割する。
/// ソート順を視覚的に確認するために使う。
fn island_id_to_grayscale(id: u32, total: u32) -> PixelF32 {
    if id == 0 {
        return PixelF32 {
            red: 0.0,
            green: 0.0,
            blue: 0.0,
            alpha: 0.0,
        };
    }
    let val = id as f32 / total.max(1) as f32;
    PixelF32 {
        red: val,
        green: val,
        blue: val,
        alpha: 1.0,
    }
}

// ---------------------------------------------------------------------------
// グラデーションユーティリティ
// ---------------------------------------------------------------------------

/// t に bias（0.0-1.0, 0.5 = linear）を適用する。
/// t^(ln(0.5)/ln(bias)) によって中間値を前後にシフトする。
fn apply_bias(t: f32, bias: f32) -> f32 {
    if bias <= 0.01 {
        return 0.0;
    }
    if bias >= 0.99 {
        return 1.0;
    }
    if (bias - 0.5).abs() < 0.01 {
        return t;
    }
    t.powf((0.5_f32).ln() / bias.ln())
}

/// 決定論的な擬似乱数ノイズ（-1.0 〜 1.0）をピクセル座標から生成する。
fn pixel_noise(x: usize, y: usize) -> f32 {
    let h = (x as u32)
        .wrapping_mul(2654435761)
        .wrapping_add((y as u32).wrapping_mul(2246822519))
        .wrapping_mul(1664525)
        .wrapping_add(1013904223);
    (h as f32 / u32::MAX as f32) * 2.0 - 1.0
}

// ---------------------------------------------------------------------------
// トラッキングアルゴリズム
// ---------------------------------------------------------------------------
// すべての関数は CCL 仮ラベル → user_id (1〜32) のマッピングを返す。
// 競合解決: 1島に複数スロットがマッチした場合はスコア最小（最近接）を優先する。
//
// 【設計方針】
// 3アルゴリズムはいずれも「Source Temp Color スロット ↔ CCL 島」の
// 類似度スコアを計算するもので、1フレーム内で完結する（時系列不要）。
//   1. color_match_tracking : 色差最小（現行ロジック）
//   2. area_weighted_tracking: 色差 + 面積差の重み付きスコア（TODO）
//   3. iou_tracking          : Source Temp Color ピクセルの BB と島 BB の IoU（TODO）

/// アルゴリズム1: 色差マッチング（最小色差スロット優先）
///
/// `color_scale` は TrackingColorRange に対する倍率（1.0 = 100%）。
/// 1.0 より大きくすると許容範囲が広がり、小さくすると厳しくなる。
fn color_match_tracking(
    raw_labels: &[u32],
    in_layer: &Layer,
    in_world_type: ae::aegp::WorldType,
    width: usize,
    height: usize,
    tracking_targets: &[(usize, PixelF32, PixelF32, f32)],
    color_scale: f32,
) -> std::collections::HashMap<u32, u32> {
    // 中間テーブル: label → (best_user_id, best_dist_so_far)
    let mut label_best: std::collections::HashMap<u32, (u32, f32)> =
        std::collections::HashMap::new();
    if !tracking_targets.is_empty() {
        for y in 0..height {
            for x in 0..width {
                let lbl = raw_labels[y * width + x];
                if lbl == 0 {
                    continue;
                }
                let px = read_pixel_f32(in_layer, in_world_type, x, y);
                for (slot_idx, src_color, _, range) in tracking_targets {
                    let effective_range = range * color_scale.max(0.0);
                    let dist = color_distance_f32(&px, src_color);
                    if dist <= effective_range {
                        let uid = (*slot_idx as u32) + 1;
                        let entry = label_best.entry(lbl).or_insert((uid, f32::MAX));
                        if dist < entry.1 {
                            *entry = (uid, dist);
                        }
                    }
                }
            }
        }
    }
    label_best
        .into_iter()
        .map(|(lbl, (uid, _))| (lbl, uid))
        .collect()
}

/// Algorithm 2: Area-Weighted Matching
///
/// For each island, compute a combined score using color match count and area similarity:
///
///   color_match_score = source_match_count   (pixels in island that match source color)
///   area_score        = 1.0 - |island_pixel_count - source_match_count| / island_pixel_count
///                     = source_match_count / island_pixel_count   (simplified, since count <= total)
///   final_score       = (1 - area_weight) * color_match_score
///                     + area_weight * (color_match_score * area_score)
///
/// The island with the highest final_score is assigned to that source slot (user_id).
/// When area_weight = 0.0, behavior equals color_match_tracking.
fn area_weighted_tracking(
    raw_labels: &[u32],
    in_layer: &Layer,
    in_world_type: ae::aegp::WorldType,
    width: usize,
    height: usize,
    tracking_targets: &[(usize, PixelF32, PixelF32, f32)],
    area_weight: f32,
) -> std::collections::HashMap<u32, u32> {
    if tracking_targets.is_empty() {
        return std::collections::HashMap::new();
    }
    let n_slots = tracking_targets.len();

    // Pass 1: collect per-island statistics in a single scan.
    //   island_pixel_count[lbl]          = total pixels in island
    //   island_slot_matches[lbl][slot_pos] = pixels in island matching slot's source color
    let mut island_pixel_count: std::collections::HashMap<u32, u32> =
        std::collections::HashMap::new();
    let mut island_slot_matches: std::collections::HashMap<u32, Vec<u32>> =
        std::collections::HashMap::new();

    for y in 0..height {
        for x in 0..width {
            let lbl = raw_labels[y * width + x];
            if lbl == 0 {
                continue;
            }
            *island_pixel_count.entry(lbl).or_insert(0) += 1;

            let px = read_pixel_f32(in_layer, in_world_type, x, y);
            let matches = island_slot_matches
                .entry(lbl)
                .or_insert_with(|| vec![0u32; n_slots]);
            for (slot_pos, (_, src_color, _, range)) in tracking_targets.iter().enumerate() {
                if color_distance_f32(&px, src_color) <= *range {
                    matches[slot_pos] += 1;
                }
            }
        }
    }

    // Pass 2: compute final scores and assign best slot to each island.
    //
    // ──────────────────────────────────────────────────────────────────
    // 設計メモ: なぜスコア式の変更だけでは area_weight に視覚的変化が出ないか
    //
    // スコア式の変化（`base*(1-w)+base*purity*w` 等）は、同一スロットに
    // 複数の島が競合するときにのみ「勝者の逆転」を生む。
    // しかしユーザーのシーンでは Source Color が1島だけにマッチするのが
    // 典型的であるため、どんな式でも area_weight を動かしても順位変動なし。
    //
    // ── 解決策: Purity Gate（純度閾値）の導入 ──────────────────────
    //   purity = match_count / island_area  （Source Color が島を覆う割合）
    //
    //   `purity < area_weight` の島は「マッチ失格」として弾く。
    //
    //   効果（1島シナリオ）:
    //     area_weight < 島のpurity  → 通常トラッキング（安定色）
    //     area_weight > 島のpurity  → 失格 → auto-assigned(33+)色に変化  ← 視覚的変化!
    //
    //   効果（複数島シナリオ）:
    //     低purityの島が順次脱落し、残った島の中でスコア式による競合解決が行われる。
    //
    // スコア式:
    //   final_score = base_score * (1 - w) + base_score * purity * w
    //               = base_score * ((1 - w) + purity * w)
    //
    //   w=0: 絶対マッチ数優先（Color Match と同等）
    //   w=1: purity×count 優先（Source Color に純粋に覆われた島が有利）
    // ──────────────────────────────────────────────────────────────────
    let mut label_to_user_id: std::collections::HashMap<u32, u32> =
        std::collections::HashMap::new();

    for (lbl, matches) in &island_slot_matches {
        let total = *island_pixel_count.get(lbl).unwrap_or(&0);
        if total == 0 {
            continue;
        }
        let total_f = total as f32;

        let mut best_uid: Option<u32> = None;
        let mut best_score = -1.0_f32;

        for (slot_pos, &match_count) in matches.iter().enumerate() {
            if match_count == 0 {
                continue;
            }
            let (slot_idx, _, _, _) = tracking_targets[slot_pos];
            let uid = (slot_idx as u32) + 1;

            let base_score = match_count as f32;
            let purity = (base_score / total_f).min(1.0_f32);

            // Purity gate: area_weight を超えない purity の島はトラッキング対象外。
            // これにより 1島のみマッチするシナリオでもスライダーの変化が視覚に現れる。
            if purity < area_weight {
                continue;
            }

            let final_score = base_score * (1.0 - area_weight) + base_score * purity * area_weight;

            if final_score > best_score {
                best_score = final_score;
                best_uid = Some(uid);
            }
        }

        if let Some(uid) = best_uid {
            label_to_user_id.insert(*lbl, uid);
        }
    }

    label_to_user_id
}

/// アルゴリズム3: 矩形重複（IoU）マッチング
///
/// 各スロットの Source Temp Color にマッチするピクセル群の
/// バウンディングボックス（BB）と、各 CCL 島の BB の
/// IoU (Intersection over Union) を計算し、
/// `iou_threshold` 以上かつ最大 IoU のスロットを各島に割り当てる。
///
/// # 同色の孤立島に関する注意
/// 2つの島が完全に同一の色を持つ場合、Source BB が両島を包む大きな矩形に
/// なるため、両島の IoU が近い値になり区別できない。
/// その場合は Color Match (algo=1) または将来実装予定のフレームキャッシュ方式を
/// 併用することを推奨する。
fn iou_tracking(
    raw_labels: &[u32],
    in_layer: &Layer,
    in_world_type: ae::aegp::WorldType,
    width: usize,
    height: usize,
    tracking_targets: &[(usize, PixelF32, PixelF32, f32)],
    iou_threshold: f32,
) -> std::collections::HashMap<u32, u32> {
    if tracking_targets.is_empty() {
        return std::collections::HashMap::new();
    }
    let n_slots = tracking_targets.len();

    // バウンディングボックス: (min_x, min_y, max_x, max_y)
    // None = まだピクセルがない「空」状態
    type Bb = Option<(usize, usize, usize, usize)>;

    // BB にピクセル座標を取り込む
    fn bb_expand(bb: &mut Bb, x: usize, y: usize) {
        match bb {
            None => *bb = Some((x, y, x, y)),
            Some((x0, y0, x1, y1)) => {
                if x < *x0 {
                    *x0 = x;
                }
                if y < *y0 {
                    *y0 = y;
                }
                if x > *x1 {
                    *x1 = x;
                }
                if y > *y1 {
                    *y1 = y;
                }
            }
        }
    }

    // IoU = 重なり面積 / 合算面積
    fn bb_iou(a: &Bb, b: &Bb) -> f32 {
        let (a, b) = match (a, b) {
            (Some(a), Some(b)) => (a, b),
            _ => return 0.0,
        };
        let ix0 = a.0.max(b.0);
        let iy0 = a.1.max(b.1);
        let ix1 = a.2.min(b.2);
        let iy1 = a.3.min(b.3);
        if ix1 < ix0 || iy1 < iy0 {
            return 0.0; // 重なりなし
        }
        let inter = ((ix1 - ix0 + 1) * (iy1 - iy0 + 1)) as f32;
        let area_a = ((a.2 - a.0 + 1) * (a.3 - a.1 + 1)) as f32;
        let area_b = ((b.2 - b.0 + 1) * (b.3 - b.1 + 1)) as f32;
        let union = area_a + area_b - inter;
        if union <= 0.0 { 0.0 } else { inter / union }
    }

    // ── Pass 1: 1スキャンで island_bb と source_bb を同時に収集 ──────────
    // island_bb: CCL ラベルごとの島バウンディングボックス
    // source_bb: 各スロットの Source Color にマッチしたピクセルの BB
    //   ※島ピクセル（lbl != 0）のみを対象にすることで
    //     背景ノイズが BB を不必要に膨らませるのを防ぐ
    let mut island_bb: std::collections::HashMap<u32, Bb> = std::collections::HashMap::new();
    let mut source_bb: Vec<Bb> = vec![None; n_slots];

    for y in 0..height {
        for x in 0..width {
            let lbl = raw_labels[y * width + x];
            if lbl == 0 {
                continue;
            }
            bb_expand(island_bb.entry(lbl).or_insert(None), x, y);

            let px = read_pixel_f32(in_layer, in_world_type, x, y);
            for (slot_pos, (_, src_color, _, range)) in tracking_targets.iter().enumerate() {
                if color_distance_f32(&px, src_color) <= *range {
                    bb_expand(&mut source_bb[slot_pos], x, y);
                }
            }
        }
    }

    // ── Pass 2: 各島に IoU 最大のスロットを割り当て ────────────────────
    // iou_threshold を最低ラインとし、それを超えるスロットの中から
    // 最高 IoU のスロットを採用する（複数スロット競合時は高 IoU 優先）
    let mut label_to_user_id: std::collections::HashMap<u32, u32> =
        std::collections::HashMap::new();

    for (&lbl, ibb) in &island_bb {
        let mut best_uid: Option<u32> = None;
        let mut best_score = iou_threshold; // 閾値未満は採用しない

        for (slot_pos, (slot_idx, _, _, _)) in tracking_targets.iter().enumerate() {
            let score = bb_iou(ibb, &source_bb[slot_pos]);
            if score > best_score {
                best_score = score;
                best_uid = Some((*slot_idx as u32) + 1);
            }
        }

        if let Some(uid) = best_uid {
            label_to_user_id.insert(lbl, uid);
        }
    }

    label_to_user_id
}

/// レイヤーに設定されたマスクパスに沿ってアイランドを弧長順にソートし、
/// 安定した ID マッピング（ラベル → sort_id）を返す。
///
/// アルゴリズム:
/// 1. PF_PathQuerySuite でレイヤーのマスクパスを取得
/// 2. 各セグメントを SAMPLES_PER_SEG 点でサンプリングして累積弧長列を作成
/// 3. 各アイランドの重心に対し、パス上の最近傍サンプル点を探し
///    その累積弧長をソートキーとして使用
/// 4. 弧長が小さい順（A → E 方向）に ID 1, 2, ... を付与
///
/// マスクが存在しない、取得できないなどの場合は None を返す。
fn sort_by_mask_path(
    in_data: ae::InData,
    raw_labels: &[u32],
    width: usize,
    mask_index: i32,
) -> Option<std::collections::HashMap<u32, u32>> {
    // PF_PathQuerySuite は AE 専用（Premiere では使用不可）
    if in_data.is_premiere() {
        return None;
    }

    let path_query = ae::pf::suites::PathQuery::new().ok()?;
    let effect_ref = in_data.effect_ref();

    let num_paths = path_query.num_paths(effect_ref).ok()?;
    if mask_index >= num_paths {
        return None;
    }

    let path_id = path_query.path_info(effect_ref, mask_index).ok()?;
    let path_outline = path_query
        .checkout_path(
            effect_ref,
            path_id,
            in_data.current_time(),
            in_data.time_step(),
            in_data.time_scale(),
        )
        .ok()??;

    let num_segs = path_outline.num_segments().ok()?;
    if num_segs < 1 {
        return None;
    }

    // 頂点を座標タプルとして収集（開いたパス: num_segs+1 頂点）
    // (anchor_x, anchor_y, tan_out_x, tan_out_y, tan_in_x, tan_in_y) — すべて f64
    let num_verts = num_segs + 1;
    let mut verts: Vec<(f64, f64, f64, f64, f64, f64)> = Vec::with_capacity(num_verts as usize);
    for i in 0..num_verts {
        match path_outline.vertex(i) {
            Ok(v) => verts.push((v.x, v.y, v.tan_out_x, v.tan_out_y, v.tan_in_x, v.tan_in_y)),
            Err(_) => break,
        }
    }
    if verts.len() < 2 {
        return None;
    }

    // 各セグメントを cubic Bezier としてサンプリング、累積弧長を記録
    // P0 = vertex[i].{anchor_x, anchor_y}
    // P1 = vertex[i].{tan_out_x, tan_out_y}  (絶対座標)
    // P2 = vertex[i+1].{tan_in_x, tan_in_y}  (絶対座標)
    // P3 = vertex[i+1].{anchor_x, anchor_y}
    const SAMPLES_PER_SEG: usize = 64;
    let total_cap = verts.len().saturating_sub(1) * SAMPLES_PER_SEG + 1;
    let mut path_pts: Vec<(f32, f32)> = Vec::with_capacity(total_cap);
    let mut cum_lens: Vec<f32> = Vec::with_capacity(total_cap);
    let mut total_len = 0.0_f32;

    let n_segs = verts.len() - 1;
    for seg in 0..n_segs {
        let (ax0, ay0, oxt, oyt, _, _) = verts[seg];
        let (ax1, ay1, _, _, ixt, iyt) = verts[seg + 1];
        let p0x = ax0 as f32;
        let p0y = ay0 as f32;
        let p1x = oxt as f32;
        let p1y = oyt as f32;
        let p2x = ixt as f32;
        let p2y = iyt as f32;
        let p3x = ax1 as f32;
        let p3y = ay1 as f32;

        for k in 0..SAMPLES_PER_SEG {
            let t = k as f32 / SAMPLES_PER_SEG as f32;
            let u = 1.0 - t;
            let sx =
                u * u * u * p0x + 3.0 * u * u * t * p1x + 3.0 * u * t * t * p2x + t * t * t * p3x;
            let sy =
                u * u * u * p0y + 3.0 * u * u * t * p1y + 3.0 * u * t * t * p2y + t * t * t * p3y;

            if let Some(&(px, py)) = path_pts.last() {
                let dx = sx - px;
                let dy = sy - py;
                total_len += (dx * dx + dy * dy).sqrt();
            }
            path_pts.push((sx, sy));
            cum_lens.push(total_len);
        }
    }
    // 最終頂点を追加
    {
        let (ex64, ey64, _, _, _, _) = verts[verts.len() - 1];
        let ex = ex64 as f32;
        let ey = ey64 as f32;
        if let Some(&(px, py)) = path_pts.last() {
            let dx = ex - px;
            let dy = ey - py;
            total_len += (dx * dx + dy * dy).sqrt();
        }
        path_pts.push((ex, ey));
        cum_lens.push(total_len);
    }
    if path_pts.len() < 2 {
        return None;
    }

    // アイランドの重心を計算
    let mut sum_x: std::collections::HashMap<u32, f64> = std::collections::HashMap::new();
    let mut sum_y: std::collections::HashMap<u32, f64> = std::collections::HashMap::new();
    let mut cnt: std::collections::HashMap<u32, u32> = std::collections::HashMap::new();
    for (idx, &lbl) in raw_labels.iter().enumerate() {
        if lbl == 0 {
            continue;
        }
        *cnt.entry(lbl).or_insert(0) += 1;
        *sum_x.entry(lbl).or_insert(0.0) += (idx % width) as f64;
        *sum_y.entry(lbl).or_insert(0.0) += (idx / width) as f64;
    }

    // 各重心をパスに射影してソートキー（弧長）を取得
    let mut keys: Vec<(u32, f32)> = cnt
        .keys()
        .map(|&lbl| {
            let n = *cnt.get(&lbl).unwrap_or(&1) as f64;
            let cx = (*sum_x.get(&lbl).unwrap_or(&0.0) / n) as f32;
            let cy = (*sum_y.get(&lbl).unwrap_or(&0.0) / n) as f32;

            let (best_idx, _) = path_pts
                .iter()
                .enumerate()
                .map(|(i, &(px, py))| {
                    let dx = cx - px;
                    let dy = cy - py;
                    (i, dx * dx + dy * dy)
                })
                .min_by(|a, b| a.1.total_cmp(&b.1))
                .unwrap_or((0, 0.0));

            (lbl, cum_lens[best_idx])
        })
        .collect();

    // 同一弧長の場合は CCL ラベル値で tie-break → 毎レンダー同一結果
    keys.sort_by(|a, b| a.1.total_cmp(&b.1).then(a.0.cmp(&b.0)));

    Some(
        keys.iter()
            .enumerate()
            .map(|(i, &(lbl, _))| (lbl, (i + 1) as u32))
            .collect(),
    )
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
        // Alpha Threshold: この値未満の alpha を持つピクセルは透明扱いにして
        // CCL 抽出対象から除外する。トラックマットのソフトエッジ誤検出を防ぐ。
        let alpha_threshold: f32 = params
            .get(Params::AlphaThreshold)
            .ok()
            .and_then(|p| p.as_float_slider().ok().map(|fs| fs.value()))
            .unwrap_or(2.0) as f32
            / 100.0;
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

        // 空間ソートモードを読み取る（1=Off, 2=L→R, 3=R→L, 4=T→B, 5=B→T, 6=Largest, 7=Smallest, 8=By Angle）
        let island_sort: i32 = params
            .get(Params::IslandSort)
            .ok()
            .and_then(|p| p.as_popup().ok().map(|pd| pd.value()))
            .unwrap_or(1);
        // By Angle ソート用の方向角度（0° = 左→右、90° = 上→下）
        let sort_angle_deg: f32 = params
            .get(Params::SortAngle)
            .ok()
            .and_then(|p| p.as_angle().ok().map(|ad| ad.value()))
            .unwrap_or(0.0);
        // Mask Path ソート用マスクインデックス（popup 値は 1-based → 0-based に変換）
        let sort_mask_index: i32 = params
            .get(Params::SortMaskIndex)
            .ok()
            .and_then(|p| p.as_popup().ok().map(|pd| pd.value()))
            .unwrap_or(1)
            - 1;

        // TempColor をグレースケール表示するか（ソート順の視覚確認用）
        let grayscale_temp_color: bool = params
            .get(Params::GrayscaleTempColor)
            .ok()
            .and_then(|p| p.as_checkbox().ok().map(|cb| cb.value()))
            .unwrap_or(false);

        // ─── Gradient Render パラメータ ───────────────────────────────
        // 1=Linear, 2=Radial, 3=Mask Path
        let master_grad_type: i32 = params
            .get(Params::MasterGradType)
            .ok()
            .and_then(|p| p.as_popup().ok().map(|pd| pd.value()))
            .unwrap_or(1);
        let master_angle_deg: f32 = params
            .get(Params::MasterAngle)
            .ok()
            .and_then(|p| p.as_angle().ok().map(|ad| ad.value()))
            .unwrap_or(0.0);
        // GradCenterPoint: 0.0=左/上, 1.0=右/下（正規化座標）
        let grad_center_point: (f32, f32) = params
            .get(Params::GradCenterPoint)
            .ok()
            .and_then(|p| p.as_point().ok().map(|pt| pt.value()))
            .unwrap_or((0.5, 0.5));
        let grad_mask_index: i32 = params
            .get(Params::GradMaskIndex)
            .ok()
            .and_then(|p| p.as_popup().ok().map(|pd| pd.value()))
            .unwrap_or(1)
            - 1;
        let master_bias: f32 = params
            .get(Params::MasterBias)
            .ok()
            .and_then(|p| p.as_float_slider().ok().map(|fs| fs.value()))
            .unwrap_or(50.0) as f32
            / 100.0;
        let master_offset: f32 = params
            .get(Params::MasterOffset)
            .ok()
            .and_then(|p| p.as_float_slider().ok().map(|fs| fs.value()))
            .unwrap_or(0.0) as f32
            / 100.0;
        let master_noise: f32 = params
            .get(Params::MasterNoiseAmount)
            .ok()
            .and_then(|p| p.as_float_slider().ok().map(|fs| fs.value()))
            .unwrap_or(0.0) as f32
            / 100.0;

        // アルゴリズム選択とアルゴリズム専用パラメータを読み取る
        let tracking_algo: i32 = params
            .get(Params::TrackingAlgorithm)
            .ok()
            .and_then(|p| p.as_popup().ok().map(|pd| pd.value()))
            .unwrap_or(1);
        let algo_color_scale: f32 = params
            .get(Params::AlgoColorScale)
            .ok()
            .and_then(|p| p.as_float_slider().ok().map(|fs| fs.value()))
            .unwrap_or(100.0) as f32
            / 100.0;
        let algo_area_weight: f32 = params
            .get(Params::AlgoAreaWeight)
            .ok()
            .and_then(|p| p.as_float_slider().ok().map(|fs| fs.value()))
            .unwrap_or(50.0) as f32
            / 100.0;
        let algo_iou_threshold: f32 = params
            .get(Params::AlgoIouThreshold)
            .ok()
            .and_then(|p| p.as_float_slider().ok().map(|fs| fs.value()))
            .unwrap_or(30.0) as f32
            / 100.0;

        // Temp Color / Final Gradient モードの場合のみ CCL を実行
        let island_labels: Option<Vec<u32>> = if output_mode == 3 || output_mode == 4 {
            // ─── Step1: 2値マスクを構築 ─────────────────────────────
            // 注意: AE のマスク（ストレートアルファ）は alpha=0 でも RGB が残る。
            // alpha が実質 0 のピクセルは InvertExtraction の設定に関わらず
            // 常に mask=false（背景）として扱い、CCL に取り込まれないようにする。
            let mut mask = vec![false; width * height];
            for y in 0..height {
                for x in 0..width {
                    let px = read_pixel_f32(&in_layer, in_world_type, x, y);
                    if px.alpha < alpha_threshold {
                        continue;
                    }
                    let extracted = extraction_targets
                        .iter()
                        .any(|(target, range)| color_distance_f32(&px, target) <= *range);
                    mask[y * width + x] = extracted != invert_extraction;
                }
            }

            // ─── Step2: CCL（仮ラベル） ─────────────────────────────
            let raw_labels = compute_ccl(&mask, width, height);

            // ─── Step2.5: 空間ソート（Island Sort != Off のとき）────────
            // 各島の重心・面積を1パスで収集し、選択基準でソートして
            // スキャン順に依存しない安定 ID（1..N）を付与する。
            // Off のときは None → 下記 Step3 の色ベーストラッキングを使用。
            let sort_id_map: Option<std::collections::HashMap<u32, u32>> = match island_sort {
                1 => None,
                // Mask Path: PF_PathQuerySuite でマスクパスを取得して弧長順にソート
                9 => sort_by_mask_path(_in_data, &raw_labels, width, sort_mask_index),
                // 重心・面積ベースの空間ソート（2〜8）
                _ => {
                    let mut sum_x: std::collections::HashMap<u32, u64> =
                        std::collections::HashMap::new();
                    let mut sum_y: std::collections::HashMap<u32, u64> =
                        std::collections::HashMap::new();
                    let mut cnt: std::collections::HashMap<u32, u32> =
                        std::collections::HashMap::new();
                    for (idx, &lbl) in raw_labels.iter().enumerate() {
                        if lbl == 0 {
                            continue;
                        }
                        *cnt.entry(lbl).or_insert(0) += 1;
                        *sum_x.entry(lbl).or_insert(0) += (idx % width) as u64;
                        *sum_y.entry(lbl).or_insert(0) += (idx / width) as u64;
                    }
                    // By Angle 用: 角度をラジアンに変換して方向ベクトルを計算
                    // AE AngleDef: 0°=上（北）、時計回り正。
                    // (-sin, cos): 0°=(0,1)=下方向→上→下グラデ、時計回りで方向も時計回り
                    let angle_rad = sort_angle_deg.to_radians();
                    let dir_x = -angle_rad.sin();
                    let dir_y = angle_rad.cos();

                    // ソートキーを計算（昇順で並べたときに小さいほど ID=1 に近い）
                    let mut islands: Vec<(u32, f32)> = cnt
                        .keys()
                        .map(|&lbl| {
                            let n = *cnt.get(&lbl).unwrap_or(&1) as f32;
                            let cx = *sum_x.get(&lbl).unwrap_or(&0) as f32 / n;
                            let cy = *sum_y.get(&lbl).unwrap_or(&0) as f32 / n;
                            let key = match island_sort {
                                2 => cx,  // Left→Right
                                3 => -cx, // Right→Left
                                4 => cy,  // Top→Bottom
                                5 => -cy, // Bottom→Top
                                6 => -n,  // Largest First
                                7 => n,   // Smallest First
                                // By Angle: 重心を方向ベクトルに射影
                                8 => cx * dir_x + cy * dir_y,
                                _ => cx,
                            };
                            (lbl, key)
                        })
                        .collect();
                    // 同一キーの場合は CCL ラベル値で tie-break → 毎レンダー同一結果
                    islands.sort_by(|a, b| a.1.total_cmp(&b.1).then(a.0.cmp(&b.0)));
                    Some(
                        islands
                            .iter()
                            .enumerate()
                            .map(|(i, &(lbl, _))| (lbl, (i + 1) as u32))
                            .collect(),
                    )
                }
            };

            // ─── Step3: アルゴリズムに基づく ID マッピング ─────────────
            // sort_id_map がある場合はソート結果を直接使用（Step3 をスキップ）。
            // ない場合はアルゴリズムポップアップに応じて3種の関数をディスパッチ。
            let remapped: Vec<u32> = if let Some(ref smap) = sort_id_map {
                // 空間ソートモード: 全島を位置順 ID で確定
                raw_labels
                    .iter()
                    .map(|&lbl| {
                        if lbl == 0 {
                            0
                        } else {
                            *smap.get(&lbl).unwrap_or(&0)
                        }
                    })
                    .collect()
            } else {
                // 色ベーストラッキングモード
                let label_to_user_id: std::collections::HashMap<u32, u32> = match tracking_algo {
                    2 => area_weighted_tracking(
                        &raw_labels,
                        &in_layer,
                        in_world_type,
                        width,
                        height,
                        &tracking_targets,
                        algo_area_weight,
                    ),
                    3 => iou_tracking(
                        &raw_labels,
                        &in_layer,
                        in_world_type,
                        width,
                        height,
                        &tracking_targets,
                        algo_iou_threshold,
                    ),
                    _ => color_match_tracking(
                        &raw_labels,
                        &in_layer,
                        in_world_type,
                        width,
                        height,
                        &tracking_targets,
                        algo_color_scale,
                    ),
                };
                // ─── Step4: user_id 1〜32 + auto 33+ ───────────────────
                let mut next_untracked = (MERGE_ISLAND_SETS as u32) + 1;
                let mut untracked_remap: std::collections::HashMap<u32, u32> =
                    std::collections::HashMap::new();
                raw_labels
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
                    .collect()
            };

            // ─── 将来の Target 置換に向けたマッピング構造を準備 ────
            // user_id → Target Temp Color（OutputMode "Final Gradient" 等で活用予定）
            let _island_to_target: std::collections::HashMap<u32, PixelF32> = tracking_targets
                .iter()
                .map(|(slot_idx, _, tgt_color, _)| ((*slot_idx as u32) + 1, *tgt_color))
                .collect();

            Some(remapped)
        } else {
            None
        };

        // グレースケール表示用: 確定した仮ラベル配列から distinct な非ゼロ ID 数を数える。
        let total_islands: u32 = island_labels
            .as_ref()
            .map(|l| {
                let mut seen = std::collections::HashSet::new();
                for &id in l {
                    if id != 0 {
                        seen.insert(id);
                    }
                }
                seen.len() as u32
            })
            .unwrap_or(1)
            .max(1);

        // ─── Final Gradient 用の事前計算 ──────────────────────────────────
        // per-island グラデーション設定: (enabled, start_color, end_color, invert)
        let grad_count_final =
            popup_to_count(params, Params::GradientSettingsCount).min(GRADIENT_SETS);
        let white_f32 = PixelF32 {
            red: 1.0,
            green: 1.0,
            blue: 1.0,
            alpha: 1.0,
        };
        let black_f32 = PixelF32 {
            red: 0.0,
            green: 0.0,
            blue: 0.0,
            alpha: 1.0,
        };
        let grad_slots: Vec<(bool, PixelF32, PixelF32, bool)> = if output_mode == 4 {
            (0..GRADIENT_SETS)
                .map(|i| {
                    if i >= grad_count_final {
                        return (false, white_f32, black_f32, false);
                    }
                    let enabled = params
                        .get(GRADIENT_ENABLE[i])
                        .ok()
                        .and_then(|p| p.as_checkbox().ok().map(|cb| cb.value()))
                        .unwrap_or(false);
                    let start = params
                        .get(GRADIENT_START_COLOR[i])
                        .ok()
                        .and_then(|p| p.as_color().ok().map(|cd| target_color_to_f32(&cd.value())))
                        .unwrap_or(white_f32);
                    let end = params
                        .get(GRADIENT_END_COLOR[i])
                        .ok()
                        .and_then(|p| p.as_color().ok().map(|cd| target_color_to_f32(&cd.value())))
                        .unwrap_or(black_f32);
                    let invert = params
                        .get(GRADIENT_INVERT[i])
                        .ok()
                        .and_then(|p| p.as_checkbox().ok().map(|cb| cb.value()))
                        .unwrap_or(false);
                    (enabled, start, end, invert)
                })
                .collect()
        } else {
            vec![]
        };

        // アイランドごとのバウンディングボックス: (x_min, x_max, y_min, y_max)
        let island_bb: std::collections::HashMap<u32, (usize, usize, usize, usize)> =
            if output_mode == 4 {
                let mut bb: std::collections::HashMap<u32, (usize, usize, usize, usize)> =
                    std::collections::HashMap::new();
                if let Some(ref labels) = island_labels {
                    for yr in 0..height {
                        for xr in 0..width {
                            let id = labels[yr * width + xr];
                            if id == 0 {
                                continue;
                            }
                            let e = bb.entry(id).or_insert((xr, xr, yr, yr));
                            if xr < e.0 {
                                e.0 = xr;
                            }
                            if xr > e.1 {
                                e.1 = xr;
                            }
                            if yr < e.2 {
                                e.2 = yr;
                            }
                            if yr > e.3 {
                                e.3 = yr;
                            }
                        }
                    }
                }
                bb
            } else {
                std::collections::HashMap::new()
            };

        // Mask Path モード: アイランド重心の最近傍パス点における接線方向 (tx, ty)
        let island_grad_tangent: std::collections::HashMap<u32, (f32, f32)> = if output_mode == 4
            && master_grad_type == 3
        {
            (|| -> Option<std::collections::HashMap<u32, (f32, f32)>> {
                if _in_data.is_premiere() {
                    return None;
                }
                let pq = ae::pf::suites::PathQuery::new().ok()?;
                let effect_ref = _in_data.effect_ref();
                let num_paths = pq.num_paths(effect_ref).ok()?;
                if grad_mask_index >= num_paths {
                    return None;
                }
                let pid = pq.path_info(effect_ref, grad_mask_index).ok()?;
                let po = pq
                    .checkout_path(
                        effect_ref,
                        pid,
                        _in_data.current_time(),
                        _in_data.time_step(),
                        _in_data.time_scale(),
                    )
                    .ok()??;
                let n_segs = po.num_segments().ok()?;
                if n_segs < 1 {
                    return None;
                }
                let nv = n_segs + 1;
                let mut verts: Vec<(f64, f64, f64, f64, f64, f64)> =
                    Vec::with_capacity(nv as usize);
                for i in 0..nv {
                    match po.vertex(i) {
                        Ok(v) => {
                            verts.push((v.x, v.y, v.tan_out_x, v.tan_out_y, v.tan_in_x, v.tan_in_y))
                        }
                        Err(_) => break,
                    }
                }
                if verts.len() < 2 {
                    return None;
                }
                // Bezier パスをポリラインにサンプリング
                const TSPG: usize = 64;
                let cap = verts.len().saturating_sub(1) * TSPG + 1;
                let mut pts: Vec<(f32, f32)> = Vec::with_capacity(cap);
                for seg in 0..(verts.len() - 1) {
                    let (ax0, ay0, oxt, oyt, _, _) = verts[seg];
                    let (ax1, ay1, _, _, ixt, iyt) = verts[seg + 1];
                    for k in 0..TSPG {
                        let t = k as f32 / TSPG as f32;
                        let u = 1.0 - t;
                        let sx = u * u * u * ax0 as f32
                            + 3.0 * u * u * t * oxt as f32
                            + 3.0 * u * t * t * ixt as f32
                            + t * t * t * ax1 as f32;
                        let sy = u * u * u * ay0 as f32
                            + 3.0 * u * u * t * oyt as f32
                            + 3.0 * u * t * t * iyt as f32
                            + t * t * t * ay1 as f32;
                        pts.push((sx, sy));
                    }
                }
                let (ex, ey, _, _, _, _) = verts[verts.len() - 1];
                pts.push((ex as f32, ey as f32));
                if pts.len() < 2 {
                    return None;
                }
                // island_labels から各アイランドの重心を計算
                let mut sxm: std::collections::HashMap<u32, f64> = std::collections::HashMap::new();
                let mut sym: std::collections::HashMap<u32, f64> = std::collections::HashMap::new();
                let mut cnm: std::collections::HashMap<u32, u32> = std::collections::HashMap::new();
                if let Some(ref labels) = island_labels {
                    for (ii, &id) in labels.iter().enumerate() {
                        if id == 0 {
                            continue;
                        }
                        *cnm.entry(id).or_insert(0) += 1;
                        *sxm.entry(id).or_insert(0.0) += (ii % width) as f64;
                        *sym.entry(id).or_insert(0.0) += (ii / width) as f64;
                    }
                }
                // 重心の最近傍パス点で接線方向を計算
                let mut tmap: std::collections::HashMap<u32, (f32, f32)> =
                    std::collections::HashMap::new();
                for (&id, &c) in &cnm {
                    let cx = (sxm[&id] / c as f64) as f32;
                    let cy = (sym[&id] / c as f64) as f32;
                    let best = pts
                        .iter()
                        .enumerate()
                        .min_by(|a, b| {
                            let (ax, ay) = a.1;
                            let (bx, by) = b.1;
                            let da = (ax - cx) * (ax - cx) + (ay - cy) * (ay - cy);
                            let db = (bx - cx) * (bx - cx) + (by - cy) * (by - cy);
                            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
                        })
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                    let prev = best.saturating_sub(1);
                    let next = (best + 1).min(pts.len() - 1);
                    let (tx, ty) = if prev == next {
                        (1.0_f32, 0.0_f32)
                    } else {
                        let (px, py) = pts[prev];
                        let (nx, ny) = pts[next];
                        let dx = nx - px;
                        let dy = ny - py;
                        let len = (dx * dx + dy * dy).sqrt().max(1e-6);
                        (dx / len, dy / len)
                    };
                    tmap.insert(id, (tx, ty));
                }
                Some(tmap)
            })()
            .unwrap_or_default()
        } else {
            std::collections::HashMap::new()
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
                    // alpha_threshold 未満は完全透明として扱い、
                    // 誤抽出ピクセルが TempColor として表示されないようにする。
                    if px.alpha < alpha_threshold {
                        PixelF32 {
                            red: 0.0,
                            green: 0.0,
                            blue: 0.0,
                            alpha: 0.0,
                        }
                    } else {
                        let idx = y as usize * width + x as usize;
                        let id = island_labels.as_ref().map(|l| l[idx]).unwrap_or(0);
                        let mut color = if grayscale_temp_color {
                            island_id_to_grayscale(id, total_islands)
                        } else {
                            island_id_to_color(id)
                        };
                        color.alpha = px.alpha;
                        color
                    }
                }
                4 => {
                    // Final Gradient モード
                    // alpha_threshold 未満は透明として扱う
                    if px.alpha < alpha_threshold {
                        PixelF32 {
                            red: 0.0,
                            green: 0.0,
                            blue: 0.0,
                            alpha: 0.0,
                        }
                    } else {
                        let idx = y as usize * width + x as usize;
                        let id = island_labels.as_ref().map(|l| l[idx]).unwrap_or(0);
                        if id == 0 {
                            PixelF32 {
                                red: 0.0,
                                green: 0.0,
                                blue: 0.0,
                                alpha: 0.0,
                            }
                        } else {
                            let slot = (id as usize).saturating_sub(1);
                            let (enabled, start, end, invert) = grad_slots
                                .get(slot)
                                .copied()
                                .unwrap_or((false, white_f32, black_f32, false));
                            if !enabled {
                                PixelF32 {
                                    red: 0.0,
                                    green: 0.0,
                                    blue: 0.0,
                                    alpha: 0.0,
                                }
                            } else {
                                let (x_min, x_max, y_min, y_max) = island_bb
                                    .get(&id)
                                    .copied()
                                    .unwrap_or((x as usize, x as usize, y as usize, y as usize));
                                let corners = [
                                    (x_min as f32, y_min as f32),
                                    (x_max as f32, y_min as f32),
                                    (x_min as f32, y_max as f32),
                                    (x_max as f32, y_max as f32),
                                ];
                                // グラデーション t 値を計算（各モード）
                                let t_raw = match master_grad_type {
                                    2 => {
                                        // Radial: 指定中心点からの距離で正規化
                                        let cx = grad_center_point.0 * width as f32;
                                        let cy = grad_center_point.1 * height as f32;
                                        let r_max = corners
                                            .iter()
                                            .map(|&(bx, by)| {
                                                let dx = bx - cx;
                                                let dy = by - cy;
                                                (dx * dx + dy * dy).sqrt()
                                            })
                                            .fold(1.0_f32, f32::max);
                                        let dx = x as f32 - cx;
                                        let dy = y as f32 - cy;
                                        ((dx * dx + dy * dy).sqrt() / r_max).clamp(0.0, 1.0)
                                    }
                                    3 => {
                                        // Mask Path: 島重心の接線方向で BB 内線形グラデーション
                                        let (tx, ty) = island_grad_tangent
                                            .get(&id)
                                            .copied()
                                            .unwrap_or((1.0, 0.0));
                                        let projs: [f32; 4] = [
                                            corners[0].0 * tx + corners[0].1 * ty,
                                            corners[1].0 * tx + corners[1].1 * ty,
                                            corners[2].0 * tx + corners[2].1 * ty,
                                            corners[3].0 * tx + corners[3].1 * ty,
                                        ];
                                        let p_min = projs.iter().copied().fold(f32::MAX, f32::min);
                                        let p_max = projs.iter().copied().fold(f32::MIN, f32::max);
                                        let proj = x as f32 * tx + y as f32 * ty;
                                        if (p_max - p_min).abs() < 1.0 {
                                            0.0_f32
                                        } else {
                                            ((proj - p_min) / (p_max - p_min))
                                                .clamp(0.0_f32, 1.0_f32)
                                        }
                                    }
                                    _ => {
                                        // Linear: Master Angle 方向で BB 内線形グラデーション
                                        // AE AngleDef: 0°=上、時計回り正 → (-sin, cos)
                                        let angle_rad = master_angle_deg.to_radians();
                                        let (dx, dy) = (-angle_rad.sin(), angle_rad.cos());
                                        let projs: [f32; 4] = [
                                            corners[0].0 * dx + corners[0].1 * dy,
                                            corners[1].0 * dx + corners[1].1 * dy,
                                            corners[2].0 * dx + corners[2].1 * dy,
                                            corners[3].0 * dx + corners[3].1 * dy,
                                        ];
                                        let p_min = projs.iter().copied().fold(f32::MAX, f32::min);
                                        let p_max = projs.iter().copied().fold(f32::MIN, f32::max);
                                        let proj = x as f32 * dx + y as f32 * dy;
                                        if (p_max - p_min).abs() < 1.0 {
                                            0.0
                                        } else {
                                            ((proj - p_min) / (p_max - p_min)).clamp(0.0, 1.0)
                                        }
                                    }
                                };
                                // ノイズ → オフセット → バイアス → 反転
                                let t_noisy = if master_noise > 0.0 {
                                    (t_raw + pixel_noise(x as usize, y as usize) * master_noise)
                                        .clamp(0.0, 1.0)
                                } else {
                                    t_raw
                                };
                                let t_off = (t_noisy + master_offset).clamp(0.0, 1.0);
                                let t_biased = apply_bias(t_off, master_bias);
                                let t_final = if invert { 1.0 - t_biased } else { t_biased };
                                // 色補間（入力アルファを乗算）
                                PixelF32 {
                                    red: start.red + (end.red - start.red) * t_final,
                                    green: start.green + (end.green - start.green) * t_final,
                                    blue: start.blue + (end.blue - start.blue) * t_final,
                                    alpha: (start.alpha + (end.alpha - start.alpha) * t_final)
                                        * px.alpha,
                                }
                            }
                        }
                    }
                }
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
