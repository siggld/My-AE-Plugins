#![allow(clippy::drop_non_drop, clippy::question_mark, dead_code)]

use after_effects as ae;
use std::env;

use ae::pf::*;
use utils::ToPixel;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------
const EXTRACTION_SETS: usize = 32;
const MERGE_ISLAND_SETS: usize = 32;
const GRADIENT_SETS: usize = 32;
const COUNTS: [usize; 4] = [4, 8, 16, 32];
const INITIAL_EXTRACTION: usize = 4;
const INITIAL_MERGE: usize = 8;
const INITIAL_GRADIENT: usize = 8;

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
enum Params {
    // 0. Output Settings
    OutputMode,

    // 1. Color Extraction group
    ColorExtGroupStart,
    ColorExtGroupEnd,
    InvertExtraction,
    ExtractionCount,
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
    TargetColor8,
    ColorRange8,
    TargetColor9,
    ColorRange9,
    TargetColor10,
    ColorRange10,
    TargetColor11,
    ColorRange11,
    TargetColor12,
    ColorRange12,
    TargetColor13,
    ColorRange13,
    TargetColor14,
    ColorRange14,
    TargetColor15,
    ColorRange15,
    TargetColor16,
    ColorRange16,
    TargetColor17,
    ColorRange17,
    TargetColor18,
    ColorRange18,
    TargetColor19,
    ColorRange19,
    TargetColor20,
    ColorRange20,
    TargetColor21,
    ColorRange21,
    TargetColor22,
    ColorRange22,
    TargetColor23,
    ColorRange23,
    TargetColor24,
    ColorRange24,
    TargetColor25,
    ColorRange25,
    TargetColor26,
    ColorRange26,
    TargetColor27,
    ColorRange27,
    TargetColor28,
    ColorRange28,
    TargetColor29,
    ColorRange29,
    TargetColor30,
    ColorRange30,
    TargetColor31,
    ColorRange31,
    ChokeSpread,

    // 2. Island Tracking & Temp Colors group
    IslandTrackGroupStart,
    IslandTrackGroupEnd,
    TrackingPath,
    ShowTempColors,
    MergeIslandCount,
    // per‑index: MergeGroupStart, EnableMerge, SourceTempColor, TargetTempColor, MergeGroupEnd
    MergeGroupStart0,
    EnableMerge0,
    SourceTempColor0,
    TargetTempColor0,
    MergeGroupEnd0,
    MergeGroupStart1,
    EnableMerge1,
    SourceTempColor1,
    TargetTempColor1,
    MergeGroupEnd1,
    MergeGroupStart2,
    EnableMerge2,
    SourceTempColor2,
    TargetTempColor2,
    MergeGroupEnd2,
    MergeGroupStart3,
    EnableMerge3,
    SourceTempColor3,
    TargetTempColor3,
    MergeGroupEnd3,
    MergeGroupStart4,
    EnableMerge4,
    SourceTempColor4,
    TargetTempColor4,
    MergeGroupEnd4,
    MergeGroupStart5,
    EnableMerge5,
    SourceTempColor5,
    TargetTempColor5,
    MergeGroupEnd5,
    MergeGroupStart6,
    EnableMerge6,
    SourceTempColor6,
    TargetTempColor6,
    MergeGroupEnd6,
    MergeGroupStart7,
    EnableMerge7,
    SourceTempColor7,
    TargetTempColor7,
    MergeGroupEnd7,
    MergeGroupStart8,
    EnableMerge8,
    SourceTempColor8,
    TargetTempColor8,
    MergeGroupEnd8,
    MergeGroupStart9,
    EnableMerge9,
    SourceTempColor9,
    TargetTempColor9,
    MergeGroupEnd9,
    MergeGroupStart10,
    EnableMerge10,
    SourceTempColor10,
    TargetTempColor10,
    MergeGroupEnd10,
    MergeGroupStart11,
    EnableMerge11,
    SourceTempColor11,
    TargetTempColor11,
    MergeGroupEnd11,
    MergeGroupStart12,
    EnableMerge12,
    SourceTempColor12,
    TargetTempColor12,
    MergeGroupEnd12,
    MergeGroupStart13,
    EnableMerge13,
    SourceTempColor13,
    TargetTempColor13,
    MergeGroupEnd13,
    MergeGroupStart14,
    EnableMerge14,
    SourceTempColor14,
    TargetTempColor14,
    MergeGroupEnd14,
    MergeGroupStart15,
    EnableMerge15,
    SourceTempColor15,
    TargetTempColor15,
    MergeGroupEnd15,
    MergeGroupStart16,
    EnableMerge16,
    SourceTempColor16,
    TargetTempColor16,
    MergeGroupEnd16,
    MergeGroupStart17,
    EnableMerge17,
    SourceTempColor17,
    TargetTempColor17,
    MergeGroupEnd17,
    MergeGroupStart18,
    EnableMerge18,
    SourceTempColor18,
    TargetTempColor18,
    MergeGroupEnd18,
    MergeGroupStart19,
    EnableMerge19,
    SourceTempColor19,
    TargetTempColor19,
    MergeGroupEnd19,
    MergeGroupStart20,
    EnableMerge20,
    SourceTempColor20,
    TargetTempColor20,
    MergeGroupEnd20,
    MergeGroupStart21,
    EnableMerge21,
    SourceTempColor21,
    TargetTempColor21,
    MergeGroupEnd21,
    MergeGroupStart22,
    EnableMerge22,
    SourceTempColor22,
    TargetTempColor22,
    MergeGroupEnd22,
    MergeGroupStart23,
    EnableMerge23,
    SourceTempColor23,
    TargetTempColor23,
    MergeGroupEnd23,
    MergeGroupStart24,
    EnableMerge24,
    SourceTempColor24,
    TargetTempColor24,
    MergeGroupEnd24,
    MergeGroupStart25,
    EnableMerge25,
    SourceTempColor25,
    TargetTempColor25,
    MergeGroupEnd25,
    MergeGroupStart26,
    EnableMerge26,
    SourceTempColor26,
    TargetTempColor26,
    MergeGroupEnd26,
    MergeGroupStart27,
    EnableMerge27,
    SourceTempColor27,
    TargetTempColor27,
    MergeGroupEnd27,
    MergeGroupStart28,
    EnableMerge28,
    SourceTempColor28,
    TargetTempColor28,
    MergeGroupEnd28,
    MergeGroupStart29,
    EnableMerge29,
    SourceTempColor29,
    TargetTempColor29,
    MergeGroupEnd29,
    MergeGroupStart30,
    EnableMerge30,
    SourceTempColor30,
    TargetTempColor30,
    MergeGroupEnd30,
    MergeGroupStart31,
    EnableMerge31,
    SourceTempColor31,
    TargetTempColor31,
    MergeGroupEnd31,

    // 3. Gradient Render group
    GradientGroupStart,
    GradientGroupEnd,
    GradientSettingsCount,
    MasterGradType,
    MasterAngle,
    MasterBias,
    MasterOffset,
    MasterNoiseAmount,
    // per-index: GradColorGroupStart, StartColor, EndColor, InvertGradient, GradColorGroupEnd
    GradColorGroupStart0,
    StartColor0,
    EndColor0,
    InvertGradient0,
    GradColorGroupEnd0,
    GradColorGroupStart1,
    StartColor1,
    EndColor1,
    InvertGradient1,
    GradColorGroupEnd1,
    GradColorGroupStart2,
    StartColor2,
    EndColor2,
    InvertGradient2,
    GradColorGroupEnd2,
    GradColorGroupStart3,
    StartColor3,
    EndColor3,
    InvertGradient3,
    GradColorGroupEnd3,
    GradColorGroupStart4,
    StartColor4,
    EndColor4,
    InvertGradient4,
    GradColorGroupEnd4,
    GradColorGroupStart5,
    StartColor5,
    EndColor5,
    InvertGradient5,
    GradColorGroupEnd5,
    GradColorGroupStart6,
    StartColor6,
    EndColor6,
    InvertGradient6,
    GradColorGroupEnd6,
    GradColorGroupStart7,
    StartColor7,
    EndColor7,
    InvertGradient7,
    GradColorGroupEnd7,
    GradColorGroupStart8,
    StartColor8,
    EndColor8,
    InvertGradient8,
    GradColorGroupEnd8,
    GradColorGroupStart9,
    StartColor9,
    EndColor9,
    InvertGradient9,
    GradColorGroupEnd9,
    GradColorGroupStart10,
    StartColor10,
    EndColor10,
    InvertGradient10,
    GradColorGroupEnd10,
    GradColorGroupStart11,
    StartColor11,
    EndColor11,
    InvertGradient11,
    GradColorGroupEnd11,
    GradColorGroupStart12,
    StartColor12,
    EndColor12,
    InvertGradient12,
    GradColorGroupEnd12,
    GradColorGroupStart13,
    StartColor13,
    EndColor13,
    InvertGradient13,
    GradColorGroupEnd13,
    GradColorGroupStart14,
    StartColor14,
    EndColor14,
    InvertGradient14,
    GradColorGroupEnd14,
    GradColorGroupStart15,
    StartColor15,
    EndColor15,
    InvertGradient15,
    GradColorGroupEnd15,
    GradColorGroupStart16,
    StartColor16,
    EndColor16,
    InvertGradient16,
    GradColorGroupEnd16,
    GradColorGroupStart17,
    StartColor17,
    EndColor17,
    InvertGradient17,
    GradColorGroupEnd17,
    GradColorGroupStart18,
    StartColor18,
    EndColor18,
    InvertGradient18,
    GradColorGroupEnd18,
    GradColorGroupStart19,
    StartColor19,
    EndColor19,
    InvertGradient19,
    GradColorGroupEnd19,
    GradColorGroupStart20,
    StartColor20,
    EndColor20,
    InvertGradient20,
    GradColorGroupEnd20,
    GradColorGroupStart21,
    StartColor21,
    EndColor21,
    InvertGradient21,
    GradColorGroupEnd21,
    GradColorGroupStart22,
    StartColor22,
    EndColor22,
    InvertGradient22,
    GradColorGroupEnd22,
    GradColorGroupStart23,
    StartColor23,
    EndColor23,
    InvertGradient23,
    GradColorGroupEnd23,
    GradColorGroupStart24,
    StartColor24,
    EndColor24,
    InvertGradient24,
    GradColorGroupEnd24,
    GradColorGroupStart25,
    StartColor25,
    EndColor25,
    InvertGradient25,
    GradColorGroupEnd25,
    GradColorGroupStart26,
    StartColor26,
    EndColor26,
    InvertGradient26,
    GradColorGroupEnd26,
    GradColorGroupStart27,
    StartColor27,
    EndColor27,
    InvertGradient27,
    GradColorGroupEnd27,
    GradColorGroupStart28,
    StartColor28,
    EndColor28,
    InvertGradient28,
    GradColorGroupEnd28,
    GradColorGroupStart29,
    StartColor29,
    EndColor29,
    InvertGradient29,
    GradColorGroupEnd29,
    GradColorGroupStart30,
    StartColor30,
    EndColor30,
    InvertGradient30,
    GradColorGroupEnd30,
    GradColorGroupStart31,
    StartColor31,
    EndColor31,
    InvertGradient31,
    GradColorGroupEnd31,
}

#[derive(Default)]
struct Plugin {}

ae::define_effect!(Plugin, (), Params);

const PLUGIN_DESCRIPTION: &str =
    "Tracks colored regions as islands and applies per-island gradients or temp colors.";

// ---------------------------------------------------------------------------
// Lookup arrays  (index → Params variant)
// ---------------------------------------------------------------------------
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

const MERGE_GROUP_START: [Params; MERGE_ISLAND_SETS] = [
    Params::MergeGroupStart0,
    Params::MergeGroupStart1,
    Params::MergeGroupStart2,
    Params::MergeGroupStart3,
    Params::MergeGroupStart4,
    Params::MergeGroupStart5,
    Params::MergeGroupStart6,
    Params::MergeGroupStart7,
    Params::MergeGroupStart8,
    Params::MergeGroupStart9,
    Params::MergeGroupStart10,
    Params::MergeGroupStart11,
    Params::MergeGroupStart12,
    Params::MergeGroupStart13,
    Params::MergeGroupStart14,
    Params::MergeGroupStart15,
    Params::MergeGroupStart16,
    Params::MergeGroupStart17,
    Params::MergeGroupStart18,
    Params::MergeGroupStart19,
    Params::MergeGroupStart20,
    Params::MergeGroupStart21,
    Params::MergeGroupStart22,
    Params::MergeGroupStart23,
    Params::MergeGroupStart24,
    Params::MergeGroupStart25,
    Params::MergeGroupStart26,
    Params::MergeGroupStart27,
    Params::MergeGroupStart28,
    Params::MergeGroupStart29,
    Params::MergeGroupStart30,
    Params::MergeGroupStart31,
];
const MERGE_GROUP_END: [Params; MERGE_ISLAND_SETS] = [
    Params::MergeGroupEnd0,
    Params::MergeGroupEnd1,
    Params::MergeGroupEnd2,
    Params::MergeGroupEnd3,
    Params::MergeGroupEnd4,
    Params::MergeGroupEnd5,
    Params::MergeGroupEnd6,
    Params::MergeGroupEnd7,
    Params::MergeGroupEnd8,
    Params::MergeGroupEnd9,
    Params::MergeGroupEnd10,
    Params::MergeGroupEnd11,
    Params::MergeGroupEnd12,
    Params::MergeGroupEnd13,
    Params::MergeGroupEnd14,
    Params::MergeGroupEnd15,
    Params::MergeGroupEnd16,
    Params::MergeGroupEnd17,
    Params::MergeGroupEnd18,
    Params::MergeGroupEnd19,
    Params::MergeGroupEnd20,
    Params::MergeGroupEnd21,
    Params::MergeGroupEnd22,
    Params::MergeGroupEnd23,
    Params::MergeGroupEnd24,
    Params::MergeGroupEnd25,
    Params::MergeGroupEnd26,
    Params::MergeGroupEnd27,
    Params::MergeGroupEnd28,
    Params::MergeGroupEnd29,
    Params::MergeGroupEnd30,
    Params::MergeGroupEnd31,
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
const GRADIENT_GRAD_COLOR_GROUP_START: [Params; GRADIENT_SETS] = [
    Params::GradColorGroupStart0,
    Params::GradColorGroupStart1,
    Params::GradColorGroupStart2,
    Params::GradColorGroupStart3,
    Params::GradColorGroupStart4,
    Params::GradColorGroupStart5,
    Params::GradColorGroupStart6,
    Params::GradColorGroupStart7,
    Params::GradColorGroupStart8,
    Params::GradColorGroupStart9,
    Params::GradColorGroupStart10,
    Params::GradColorGroupStart11,
    Params::GradColorGroupStart12,
    Params::GradColorGroupStart13,
    Params::GradColorGroupStart14,
    Params::GradColorGroupStart15,
    Params::GradColorGroupStart16,
    Params::GradColorGroupStart17,
    Params::GradColorGroupStart18,
    Params::GradColorGroupStart19,
    Params::GradColorGroupStart20,
    Params::GradColorGroupStart21,
    Params::GradColorGroupStart22,
    Params::GradColorGroupStart23,
    Params::GradColorGroupStart24,
    Params::GradColorGroupStart25,
    Params::GradColorGroupStart26,
    Params::GradColorGroupStart27,
    Params::GradColorGroupStart28,
    Params::GradColorGroupStart29,
    Params::GradColorGroupStart30,
    Params::GradColorGroupStart31,
];
const GRADIENT_GRAD_COLOR_GROUP_END: [Params; GRADIENT_SETS] = [
    Params::GradColorGroupEnd0,
    Params::GradColorGroupEnd1,
    Params::GradColorGroupEnd2,
    Params::GradColorGroupEnd3,
    Params::GradColorGroupEnd4,
    Params::GradColorGroupEnd5,
    Params::GradColorGroupEnd6,
    Params::GradColorGroupEnd7,
    Params::GradColorGroupEnd8,
    Params::GradColorGroupEnd9,
    Params::GradColorGroupEnd10,
    Params::GradColorGroupEnd11,
    Params::GradColorGroupEnd12,
    Params::GradColorGroupEnd13,
    Params::GradColorGroupEnd14,
    Params::GradColorGroupEnd15,
    Params::GradColorGroupEnd16,
    Params::GradColorGroupEnd17,
    Params::GradColorGroupEnd18,
    Params::GradColorGroupEnd19,
    Params::GradColorGroupEnd20,
    Params::GradColorGroupEnd21,
    Params::GradColorGroupEnd22,
    Params::GradColorGroupEnd23,
    Params::GradColorGroupEnd24,
    Params::GradColorGroupEnd25,
    Params::GradColorGroupEnd26,
    Params::GradColorGroupEnd27,
    Params::GradColorGroupEnd28,
    Params::GradColorGroupEnd29,
    Params::GradColorGroupEnd30,
    Params::GradColorGroupEnd31,
];

// ---------------------------------------------------------------------------
// Helpers for dynamic UI
// ---------------------------------------------------------------------------
fn popup_to_count(params: &ae::Parameters<Params>, key: Params) -> usize {
    params
        .get(key)
        .ok()
        .and_then(|p| {
            p.as_popup().ok().map(|pd| {
                let idx = (pd.value() as usize).saturating_sub(1);
                COUNTS.get(idx).copied().unwrap_or(COUNTS[0])
            })
        })
        .unwrap_or(COUNTS[0])
}

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
    // --- Extraction ---
    let ext_count = popup_to_count(params, Params::ExtractionCount);
    for i in 0..EXTRACTION_SETS {
        let vis = i < ext_count;
        set_param_visibility(in_data, params, EXTRACTION_TARGET_COLORS[i], vis)?;
        set_param_visibility(in_data, params, EXTRACTION_COLOR_RANGES[i], vis)?;
    }
    // --- Merge ---
    let merge_count = popup_to_count(params, Params::MergeIslandCount);
    for i in 0..MERGE_ISLAND_SETS {
        let vis = i < merge_count;
        set_param_visibility(in_data, params, MERGE_GROUP_START[i], vis)?;
        set_param_visibility(in_data, params, MERGE_ENABLE[i], vis)?;
        set_param_visibility(in_data, params, MERGE_SOURCE_TEMP[i], vis)?;
        set_param_visibility(in_data, params, MERGE_TARGET_TEMP[i], vis)?;
        set_param_visibility(in_data, params, MERGE_GROUP_END[i], vis)?;
    }
    // --- Gradient ---
    let grad_count = popup_to_count(params, Params::GradientSettingsCount);
    for i in 0..GRADIENT_SETS {
        let vis = i < grad_count;
        set_param_visibility(in_data, params, GRADIENT_GRAD_COLOR_GROUP_START[i], vis)?;
        set_param_visibility(in_data, params, GRADIENT_START_COLOR[i], vis)?;
        set_param_visibility(in_data, params, GRADIENT_END_COLOR[i], vis)?;
        set_param_visibility(in_data, params, GRADIENT_INVERT[i], vis)?;
        set_param_visibility(in_data, params, GRADIENT_GRAD_COLOR_GROUP_END[i], vis)?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// params_setup
// ---------------------------------------------------------------------------
impl AdobePluginGlobal for Plugin {
    fn params_setup(
        &self,
        params: &mut ae::Parameters<Params>,
        _in_data: InData,
        _: OutData,
    ) -> Result<(), Error> {
        // ----- 0. Output Settings -----
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

        // ----- 1. Color Extraction -----
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
                    let ui = if hidden {
                        ParamUIFlags::INVISIBLE
                    } else {
                        ParamUIFlags::NONE
                    };
                    params.add_with_flags(
                        EXTRACTION_TARGET_COLORS[i],
                        &format!("Target Color {n}"),
                        ColorDef::setup(|_| {}),
                        ParamFlag::empty(),
                        ui,
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

        // ----- 2. Island Tracking & Temp Colors -----
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
                        d.set_default(2);
                    }),
                    ParamFlag::SUPERVISE,
                    ParamUIFlags::NONE,
                )?;

                for i in 0..MERGE_ISLAND_SETS {
                    let n = i + 1;
                    let hidden = i >= INITIAL_MERGE;
                    let name = format!("Merge Setting {n}");
                    params.add_group(
                        MERGE_GROUP_START[i],
                        MERGE_GROUP_END[i],
                        &name,
                        true,
                        |inner| {
                            inner.add_with_flags(
                                MERGE_ENABLE[i],
                                &format!("Enable Merge {n}"),
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
                            inner.add_with_flags(
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
                            inner.add_with_flags(
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
                            Ok(())
                        },
                    )?;
                }
                Ok(())
            },
        )?;

        // ----- 3. Gradient Render -----
        params.add_group(
            Params::GradientGroupStart,
            Params::GradientGroupEnd,
            "Gradient Render",
            true,
            |params| {
                params.add_with_flags(
                    Params::GradientSettingsCount,
                    "Gradient Settings Count",
                    PopupDef::setup(|d| {
                        d.set_options(&["4", "8", "16", "32"]);
                        d.set_default(2);
                    }),
                    ParamFlag::SUPERVISE,
                    ParamUIFlags::NONE,
                )?;
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

                let white = ae::Pixel8 {
                    red: 255,
                    green: 255,
                    blue: 255,
                    alpha: 255,
                };
                for i in 0..GRADIENT_SETS {
                    let n = i + 1;
                    let hidden = i >= INITIAL_GRADIENT;
                    let name = format!("Gradient Color {n}");
                    params.add_group(
                        GRADIENT_GRAD_COLOR_GROUP_START[i],
                        GRADIENT_GRAD_COLOR_GROUP_END[i],
                        &name,
                        true,
                        |inner| {
                            inner.add_with_flags(
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
                            inner.add_with_flags(
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
                            inner.add_with_flags(
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
                            Ok(())
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

// ---------------------------------------------------------------------------
// Rendering helpers
// ---------------------------------------------------------------------------
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
            let range_f32 = (color_range_val / 100.0).clamp(0.0, 1.0);
            extraction_targets.push((target_color_to_f32(&target_color), range_f32));
        }

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
                3 | 4 => px,
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
