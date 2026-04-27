#![allow(clippy::drop_non_drop, clippy::question_mark)]

use ae::pf::*;
use after_effects as ae;
use std::env;

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
#[allow(non_camel_case_types)]
enum Params {
    ColorSelectGroupStart,
    ColorA,
    ColorB,
    ColorThreshold,
    ColorSelectGroupEnd,
    Mode,
    Center,
    Angle,
    Amount,
    Offset,
    BlurIterations,
    KeepDivision,
}

#[derive(Default)]
struct Plugin {}

ae::define_effect!(Plugin, (), Params);

const PLUGIN_DESCRIPTION: &str =
    "Creates boundary color extraction and blur control UI for compositing.";

impl AdobePluginGlobal for Plugin {
    fn params_setup(
        &self,
        params: &mut ae::Parameters<Params>,
        _in_data: InData,
        _: OutData,
    ) -> Result<(), Error> {
        params.add_group(
            Params::ColorSelectGroupStart,
            Params::ColorSelectGroupEnd,
            "Color Select",
            true,
            |params| {
                params.add(
                    Params::ColorA,
                    "Color A",
                    ColorDef::setup(|d| {
                        d.set_default(ae::Pixel8 {
                            alpha: 255,
                            red: 255,
                            green: 255,
                            blue: 255,
                        });
                    }),
                )?;
                params.add(
                    Params::ColorB,
                    "Color B",
                    ColorDef::setup(|d| {
                        d.set_default(ae::Pixel8 {
                            alpha: 255,
                            red: 0,
                            green: 0,
                            blue: 0,
                        });
                    }),
                )?;
                params.add(
                    Params::ColorThreshold,
                    "Color Threshold",
                    FloatSliderDef::setup(|d| {
                        d.set_valid_min(0.0);
                        d.set_valid_max(100.0);
                        d.set_slider_min(0.0);
                        d.set_slider_max(100.0);
                        d.set_default(10.0);
                        d.set_precision(Precision::Tenths);
                        d.set_display_flags(ValueDisplayFlag::PERCENT);
                    }),
                )?;
                Ok(())
            },
        )?;

        params.add_with_flags(
            Params::Mode,
            "Mode",
            PopupDef::setup(|d| {
                d.set_options(&["Radial", "Directional"]);
                d.set_default(1);
            }),
            ParamFlag::SUPERVISE,
            ParamUIFlags::NONE,
        )?;

        params.add(
            Params::Center,
            "Center",
            PointDef::setup(|d| {
                d.set_default_x(0.5);
                d.set_default_y(0.5);
            }),
        )?;
        params.add(
            Params::Angle,
            "Angle",
            AngleDef::setup(|d| {
                d.set_default(0.0);
            }),
        )?;
        params.add(
            Params::Amount,
            "Amount",
            FloatSliderDef::setup(|d| {
                d.set_valid_min(0.0);
                d.set_valid_max(1000.0);
                d.set_slider_min(0.0);
                d.set_slider_max(100.0);
                d.set_default(25.0);
                d.set_precision(Precision::Tenths);
            }),
        )?;
        params.add(
            Params::Offset,
            "Offset",
            FloatSliderDef::setup(|d| {
                d.set_valid_min(-100.0);
                d.set_valid_max(100.0);
                d.set_slider_min(-100.0);
                d.set_slider_max(100.0);
                d.set_default(0.0);
                d.set_precision(Precision::Tenths);
            }),
        )?;
        params.add(
            Params::BlurIterations,
            "Blur Iterations",
            FloatSliderDef::setup(|d| {
                d.set_valid_min(1.0);
                d.set_valid_max(512.0);
                d.set_slider_min(1.0);
                d.set_slider_max(128.0);
                d.set_default(16.0);
                d.set_precision(Precision::Integer);
            }),
        )?;
        params.add(
            Params::KeepDivision,
            "Keep Division",
            CheckBoxDef::setup(|d| {
                d.set_default(true);
            }),
        )?;

        Ok(())
    }

    fn handle_command(
        &mut self,
        cmd: ae::Command,
        _in_data: InData,
        mut out_data: OutData,
        params: &mut ae::Parameters<Params>,
    ) -> Result<(), ae::Error> {
        match cmd {
            ae::Command::About => {
                out_data.set_return_msg(
                    format!(
                        "TKG_BoundaryColorSaturation - {version}\r\r{PLUGIN_DESCRIPTION}\rCopyright (c) 2026-{build_year} Aodaruma",
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
            }
            ae::Command::UpdateParamsUi => {
                let mut p = params.cloned();
                let mode = p.get(Params::Mode)?.as_popup()?.value();
                let is_radial = mode == 1;
                let is_directional = mode == 2;

                let mut pd_center = p.get_mut(Params::Center)?;
                pd_center.set_ui_flag(ae::ParamUIFlags::DISABLED, is_directional);
                pd_center.update_param_ui()?;

                let mut pd_angle = p.get_mut(Params::Angle)?;
                pd_angle.set_ui_flag(ae::ParamUIFlags::DISABLED, is_radial);
                pd_angle.update_param_ui()?;
            }
            _ => {}
        }
        Ok(())
    }
}
