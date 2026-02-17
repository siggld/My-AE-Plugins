#![allow(clippy::drop_non_drop, clippy::question_mark)]

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
const INVERT_GRAD_SETS: usize = 32;

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
enum Params {
    // 0. Output Settings
    OutputMode,

    // 1. Color Extraction (group: when after-effects provides GroupStartDef/GroupEndDef, add them)
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

    // 2. Island Tracking & Temp Colors
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

    // 3. Gradient Render
    GradientSettingsCount,
    // GradType, StartColor, EndColor, Angle, Bias, Offset, NoiseAmount x GRADIENT_SETS
    GradType0,
    StartColor0,
    EndColor0,
    Angle0,
    Bias0,
    Offset0,
    NoiseAmount0,
    GradType1,
    StartColor1,
    EndColor1,
    Angle1,
    Bias1,
    Offset1,
    NoiseAmount1,
    GradType2,
    StartColor2,
    EndColor2,
    Angle2,
    Bias2,
    Offset2,
    NoiseAmount2,
    GradType3,
    StartColor3,
    EndColor3,
    Angle3,
    Bias3,
    Offset3,
    NoiseAmount3,
    GradType4,
    StartColor4,
    EndColor4,
    Angle4,
    Bias4,
    Offset4,
    NoiseAmount4,
    GradType5,
    StartColor5,
    EndColor5,
    Angle5,
    Bias5,
    Offset5,
    NoiseAmount5,
    GradType6,
    StartColor6,
    EndColor6,
    Angle6,
    Bias6,
    Offset6,
    NoiseAmount6,
    GradType7,
    StartColor7,
    EndColor7,
    Angle7,
    Bias7,
    Offset7,
    NoiseAmount7,
    GradType8,
    StartColor8,
    EndColor8,
    Angle8,
    Bias8,
    Offset8,
    NoiseAmount8,
    GradType9,
    StartColor9,
    EndColor9,
    Angle9,
    Bias9,
    Offset9,
    NoiseAmount9,
    GradType10,
    StartColor10,
    EndColor10,
    Angle10,
    Bias10,
    Offset10,
    NoiseAmount10,
    GradType11,
    StartColor11,
    EndColor11,
    Angle11,
    Bias11,
    Offset11,
    NoiseAmount11,
    GradType12,
    StartColor12,
    EndColor12,
    Angle12,
    Bias12,
    Offset12,
    NoiseAmount12,
    GradType13,
    StartColor13,
    EndColor13,
    Angle13,
    Bias13,
    Offset13,
    NoiseAmount13,
    GradType14,
    StartColor14,
    EndColor14,
    Angle14,
    Bias14,
    Offset14,
    NoiseAmount14,
    GradType15,
    StartColor15,
    EndColor15,
    Angle15,
    Bias15,
    Offset15,
    NoiseAmount15,
    GradType16,
    StartColor16,
    EndColor16,
    Angle16,
    Bias16,
    Offset16,
    NoiseAmount16,
    GradType17,
    StartColor17,
    EndColor17,
    Angle17,
    Bias17,
    Offset17,
    NoiseAmount17,
    GradType18,
    StartColor18,
    EndColor18,
    Angle18,
    Bias18,
    Offset18,
    NoiseAmount18,
    GradType19,
    StartColor19,
    EndColor19,
    Angle19,
    Bias19,
    Offset19,
    NoiseAmount19,
    GradType20,
    StartColor20,
    EndColor20,
    Angle20,
    Bias20,
    Offset20,
    NoiseAmount20,
    GradType21,
    StartColor21,
    EndColor21,
    Angle21,
    Bias21,
    Offset21,
    NoiseAmount21,
    GradType22,
    StartColor22,
    EndColor22,
    Angle22,
    Bias22,
    Offset22,
    NoiseAmount22,
    GradType23,
    StartColor23,
    EndColor23,
    Angle23,
    Bias23,
    Offset23,
    NoiseAmount23,
    GradType24,
    StartColor24,
    EndColor24,
    Angle24,
    Bias24,
    Offset24,
    NoiseAmount24,
    GradType25,
    StartColor25,
    EndColor25,
    Angle25,
    Bias25,
    Offset25,
    NoiseAmount25,
    GradType26,
    StartColor26,
    EndColor26,
    Angle26,
    Bias26,
    Offset26,
    NoiseAmount26,
    GradType27,
    StartColor27,
    EndColor27,
    Angle27,
    Bias27,
    Offset27,
    NoiseAmount27,
    GradType28,
    StartColor28,
    EndColor28,
    Angle28,
    Bias28,
    Offset28,
    NoiseAmount28,
    GradType29,
    StartColor29,
    EndColor29,
    Angle29,
    Bias29,
    Offset29,
    NoiseAmount29,
    GradType30,
    StartColor30,
    EndColor30,
    Angle30,
    Bias30,
    Offset30,
    NoiseAmount30,
    GradType31,
    StartColor31,
    EndColor31,
    Angle31,
    Bias31,
    Offset31,
    NoiseAmount31,
    InvertGradCount,
    InvertTempColor0,
    InvertTempColor1,
    InvertTempColor2,
    InvertTempColor3,
    InvertTempColor4,
    InvertTempColor5,
    InvertTempColor6,
    InvertTempColor7,
    InvertTempColor8,
    InvertTempColor9,
    InvertTempColor10,
    InvertTempColor11,
    InvertTempColor12,
    InvertTempColor13,
    InvertTempColor14,
    InvertTempColor15,
    InvertTempColor16,
    InvertTempColor17,
    InvertTempColor18,
    InvertTempColor19,
    InvertTempColor20,
    InvertTempColor21,
    InvertTempColor22,
    InvertTempColor23,
    InvertTempColor24,
    InvertTempColor25,
    InvertTempColor26,
    InvertTempColor27,
    InvertTempColor28,
    InvertTempColor29,
    InvertTempColor30,
    InvertTempColor31,
}

#[derive(Default)]
struct Plugin {}

ae::define_effect!(Plugin, (), Params);

const PLUGIN_DESCRIPTION: &str =
    "Tracks colored regions as islands and applies per-island gradients or temp colors.";

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

        // ----- 1. Color Extraction -----
        params.add(
            Params::InvertExtraction,
            "Invert Extraction",
            CheckBoxDef::setup(|d| { d.set_default(false); }),
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
        for (i, (tc, cr)) in target_color_range.iter().enumerate() {
            params.add(
                *tc,
                &format!("Target Color {}", i + 1),
                ColorDef::setup(|_d| {}),
            )?;
            params.add(
                *cr,
                &format!("Color Range {}", i + 1),
                FloatSliderDef::setup(|d| {
                    d.set_valid_min(0.0);
                    d.set_valid_max(100.0);
                    d.set_slider_min(0.0);
                    d.set_slider_max(50.0);
                    d.set_default(10.0);
                    d.set_precision(1);
                }),
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

        // ----- 2. Island Tracking & Temp Colors -----
        params.add(Params::TrackingPath, "Tracking Path", PathDef::setup(|_| {}))?;
        params.add(
            Params::ShowTempColors,
            "Show Temp Colors",
            CheckBoxDef::setup(|d| { d.set_default(true); }),
        )?;
        params.add(
            Params::MergeIslandCount,
            "Merge Island Count",
            PopupDef::setup(|d| {
                d.set_options(&["4", "8", "16", "32"]);
                d.set_default(2); // 8
            }),
        )?;

        for i in 0..MERGE_ISLAND_SETS {
            params.add(
                match i {
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
                },
                &format!("Source Temp Color {}", i + 1),
                ColorDef::setup(|_d| {}),
            )?;
            params.add(
                match i {
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
                },
                &format!("Target Temp Color {}", i + 1),
                ColorDef::setup(|_d| {}),
            )?;
        }

        // ----- 3. Gradient Render -----
        params.add(
            Params::GradientSettingsCount,
            "Gradient Settings Count",
            PopupDef::setup(|d| {
                d.set_options(&["4", "8", "16", "32"]);
                d.set_default(2); // 8
            }),
        )?;

        let grad_params = [
            (Params::GradType0, Params::StartColor0, Params::EndColor0, Params::Angle0, Params::Bias0, Params::Offset0, Params::NoiseAmount0),
            (Params::GradType1, Params::StartColor1, Params::EndColor1, Params::Angle1, Params::Bias1, Params::Offset1, Params::NoiseAmount1),
            (Params::GradType2, Params::StartColor2, Params::EndColor2, Params::Angle2, Params::Bias2, Params::Offset2, Params::NoiseAmount2),
            (Params::GradType3, Params::StartColor3, Params::EndColor3, Params::Angle3, Params::Bias3, Params::Offset3, Params::NoiseAmount3),
            (Params::GradType4, Params::StartColor4, Params::EndColor4, Params::Angle4, Params::Bias4, Params::Offset4, Params::NoiseAmount4),
            (Params::GradType5, Params::StartColor5, Params::EndColor5, Params::Angle5, Params::Bias5, Params::Offset5, Params::NoiseAmount5),
            (Params::GradType6, Params::StartColor6, Params::EndColor6, Params::Angle6, Params::Bias6, Params::Offset6, Params::NoiseAmount6),
            (Params::GradType7, Params::StartColor7, Params::EndColor7, Params::Angle7, Params::Bias7, Params::Offset7, Params::NoiseAmount7),
            (Params::GradType8, Params::StartColor8, Params::EndColor8, Params::Angle8, Params::Bias8, Params::Offset8, Params::NoiseAmount8),
            (Params::GradType9, Params::StartColor9, Params::EndColor9, Params::Angle9, Params::Bias9, Params::Offset9, Params::NoiseAmount9),
            (Params::GradType10, Params::StartColor10, Params::EndColor10, Params::Angle10, Params::Bias10, Params::Offset10, Params::NoiseAmount10),
            (Params::GradType11, Params::StartColor11, Params::EndColor11, Params::Angle11, Params::Bias11, Params::Offset11, Params::NoiseAmount11),
            (Params::GradType12, Params::StartColor12, Params::EndColor12, Params::Angle12, Params::Bias12, Params::Offset12, Params::NoiseAmount12),
            (Params::GradType13, Params::StartColor13, Params::EndColor13, Params::Angle13, Params::Bias13, Params::Offset13, Params::NoiseAmount13),
            (Params::GradType14, Params::StartColor14, Params::EndColor14, Params::Angle14, Params::Bias14, Params::Offset14, Params::NoiseAmount14),
            (Params::GradType15, Params::StartColor15, Params::EndColor15, Params::Angle15, Params::Bias15, Params::Offset15, Params::NoiseAmount15),
            (Params::GradType16, Params::StartColor16, Params::EndColor16, Params::Angle16, Params::Bias16, Params::Offset16, Params::NoiseAmount16),
            (Params::GradType17, Params::StartColor17, Params::EndColor17, Params::Angle17, Params::Bias17, Params::Offset17, Params::NoiseAmount17),
            (Params::GradType18, Params::StartColor18, Params::EndColor18, Params::Angle18, Params::Bias18, Params::Offset18, Params::NoiseAmount18),
            (Params::GradType19, Params::StartColor19, Params::EndColor19, Params::Angle19, Params::Bias19, Params::Offset19, Params::NoiseAmount19),
            (Params::GradType20, Params::StartColor20, Params::EndColor20, Params::Angle20, Params::Bias20, Params::Offset20, Params::NoiseAmount20),
            (Params::GradType21, Params::StartColor21, Params::EndColor21, Params::Angle21, Params::Bias21, Params::Offset21, Params::NoiseAmount21),
            (Params::GradType22, Params::StartColor22, Params::EndColor22, Params::Angle22, Params::Bias22, Params::Offset22, Params::NoiseAmount22),
            (Params::GradType23, Params::StartColor23, Params::EndColor23, Params::Angle23, Params::Bias23, Params::Offset23, Params::NoiseAmount23),
            (Params::GradType24, Params::StartColor24, Params::EndColor24, Params::Angle24, Params::Bias24, Params::Offset24, Params::NoiseAmount24),
            (Params::GradType25, Params::StartColor25, Params::EndColor25, Params::Angle25, Params::Bias25, Params::Offset25, Params::NoiseAmount25),
            (Params::GradType26, Params::StartColor26, Params::EndColor26, Params::Angle26, Params::Bias26, Params::Offset26, Params::NoiseAmount26),
            (Params::GradType27, Params::StartColor27, Params::EndColor27, Params::Angle27, Params::Bias27, Params::Offset27, Params::NoiseAmount27),
            (Params::GradType28, Params::StartColor28, Params::EndColor28, Params::Angle28, Params::Bias28, Params::Offset28, Params::NoiseAmount28),
            (Params::GradType29, Params::StartColor29, Params::EndColor29, Params::Angle29, Params::Bias29, Params::Offset29, Params::NoiseAmount29),
            (Params::GradType30, Params::StartColor30, Params::EndColor30, Params::Angle30, Params::Bias30, Params::Offset30, Params::NoiseAmount30),
            (Params::GradType31, Params::StartColor31, Params::EndColor31, Params::Angle31, Params::Bias31, Params::Offset31, Params::NoiseAmount31),
        ];
        for (idx, (grad_type, start_c, end_c, angle, bias, offset, noise)) in grad_params.iter().enumerate() {
            let n = idx + 1;
            params.add(
                *grad_type,
                &format!("Grad Type {}", n),
                PopupDef::setup(|d| {
                    d.set_options(&["Linear", "Radial"]);
                    d.set_default(1);
                }),
            )?;
            params.add(*start_c, &format!("Start Color {}", n), ColorDef::setup(|_d| {}))?;
            params.add(*end_c, &format!("End Color {}", n), ColorDef::setup(|_d| {}))?;
            params.add(
                *angle,
                &format!("Angle {}", n),
                AngleDef::setup(|_d| {}),
            )?;
            params.add(
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
            )?;
            params.add(
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
            )?;
            params.add(
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
            )?;
        }

        params.add(
            Params::InvertGradCount,
            "Invert Gradient Count",
            PopupDef::setup(|d| {
                d.set_options(&["4", "8", "16", "32"]);
                d.set_default(2);
            }),
        )?;

        let invert_colors = [
            Params::InvertTempColor0, Params::InvertTempColor1, Params::InvertTempColor2, Params::InvertTempColor3,
            Params::InvertTempColor4, Params::InvertTempColor5, Params::InvertTempColor6, Params::InvertTempColor7,
            Params::InvertTempColor8, Params::InvertTempColor9, Params::InvertTempColor10, Params::InvertTempColor11,
            Params::InvertTempColor12, Params::InvertTempColor13, Params::InvertTempColor14, Params::InvertTempColor15,
            Params::InvertTempColor16, Params::InvertTempColor17, Params::InvertTempColor18, Params::InvertTempColor19,
            Params::InvertTempColor20, Params::InvertTempColor21, Params::InvertTempColor22, Params::InvertTempColor23,
            Params::InvertTempColor24, Params::InvertTempColor25, Params::InvertTempColor26, Params::InvertTempColor27,
            Params::InvertTempColor28, Params::InvertTempColor29, Params::InvertTempColor30, Params::InvertTempColor31,
        ];
        for (i, &p) in invert_colors.iter().enumerate() {
            params.add(p, &format!("Invert Temp Color {}", i + 1), ColorDef::setup(|_d| {}))?;
        }

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
                out_data.set_out_flag2(OutFlags2::SupportsSmartRender, true);
                out_data.set_out_flag2(OutFlags2::SupportsThreadedRendering, true);
                out_data.set_out_flag2(OutFlags2::SupportsGetFlattenedSequenceData, true);
                // When using GROUP_START params: set ParamGroupStartCollapsedFlag so twirly starts collapsed (AE_Rust_Knowledge).
                out_data.set_out_flag2(OutFlags2::ParamGroupStartCollapsedFlag, true);
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
                    0, 0, &req,
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

impl Plugin {
    fn do_render(
        &self,
        _in_data: InData,
        _in_layer: Layer,
        _out_data: OutData,
        mut out_layer: Layer,
        _params: &mut Parameters<Params>,
    ) -> Result<(), Error> {
        let _progress_final = out_layer.height() as i32;
        // TODO: Core algorithm (extraction, CCL, path mapping, gradient render)
        Ok(())
    }
}
