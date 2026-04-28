#![allow(clippy::drop_non_drop, clippy::question_mark)]

use ae::pf::*;
use after_effects as ae;
use std::env;

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
enum Params {}

#[derive(Default)]
struct Plugin {}

ae::define_effect!(Plugin, (), Params);

const PLUGIN_DESCRIPTION: &str =
    "Provides a reusable test plugin for validating custom GUI behaviors in AE.";

impl AdobePluginGlobal for Plugin {
    fn params_setup(
        &self,
        _params: &mut ae::Parameters<Params>,
        _in_data: InData,
        _: OutData,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn handle_command(
        &mut self,
        cmd: ae::Command,
        _in_data: InData,
        mut out_data: OutData,
        _params: &mut ae::Parameters<Params>,
    ) -> Result<(), ae::Error> {
        match cmd {
            ae::Command::About => {
                out_data.set_return_msg(
                    format!(
                        "TKG_CustomGUITest - {version}\r\r{PLUGIN_DESCRIPTION}\rCopyright (c) 2026-{build_year} Aodaruma",
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
            _ => {}
        }
        Ok(())
    }
}
