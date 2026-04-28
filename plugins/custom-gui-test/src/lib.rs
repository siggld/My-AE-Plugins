#![allow(clippy::drop_non_drop, clippy::question_mark)]

use ae::pf::*;
use after_effects as ae;
use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
enum Params {}

#[derive(Default)]
struct Plugin {}

ae::define_effect!(Plugin, (), Params);

const PLUGIN_DESCRIPTION: &str =
    "Provides a reusable test plugin for validating custom GUI behaviors in AE.";

fn debug_log(hypothesis_id: &str, location: &str, message: &str, data_json: &str) {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let line = format!(
        "{{\"sessionId\":\"1e940a\",\"runId\":\"pre-fix-ae1\",\"hypothesisId\":\"{}\",\"location\":\"{}\",\"message\":\"{}\",\"data\":{},\"timestamp\":{}}}\n",
        hypothesis_id, location, message, data_json, ts
    );
    // #region agent log
    if let Ok(mut f) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("debug-1e940a.log")
    {
        let _ = f.write_all(line.as_bytes());
    }
    // #endregion
}

impl AdobePluginGlobal for Plugin {
    fn params_setup(
        &self,
        _params: &mut ae::Parameters<Params>,
        _in_data: InData,
        _: OutData,
    ) -> Result<(), Error> {
        debug_log(
            "H2",
            "plugins/custom-gui-test/src/lib.rs:params_setup",
            "params_setup called",
            "{\"paramsDeclared\":0}",
        );
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
                debug_log(
                    "H1",
                    "plugins/custom-gui-test/src/lib.rs:global_setup",
                    "global setup flags set",
                    "{\"supportsSmartRender\":true,\"supportsThreadedRendering\":true}",
                );
                out_data.set_out_flag2(ae::OutFlags2::SupportsThreadedRendering, true);
                out_data.set_out_flag2(ae::OutFlags2::SupportsSmartRender, true);
                out_data.set_out_flag2(ae::OutFlags2::ParamGroupStartCollapsedFlag, true);
                out_data.set_out_flag(ae::OutFlags::SendUpdateParamsUi, true);
            }
            ae::Command::SmartPreRender { .. } => {
                debug_log(
                    "H1",
                    "plugins/custom-gui-test/src/lib.rs:smart_pre_render",
                    "smart pre render reached",
                    "{\"handlerImplemented\":false}",
                );
            }
            ae::Command::SmartRender { .. } => {
                debug_log(
                    "H1",
                    "plugins/custom-gui-test/src/lib.rs:smart_render",
                    "smart render reached",
                    "{\"handlerImplemented\":false}",
                );
            }
            ae::Command::Render { .. } => {
                debug_log(
                    "H3",
                    "plugins/custom-gui-test/src/lib.rs:render",
                    "legacy render reached",
                    "{\"path\":\"legacy-render\"}",
                );
            }
            ae::Command::UpdateParamsUi => {
                debug_log(
                    "H4",
                    "plugins/custom-gui-test/src/lib.rs:update_params_ui",
                    "update params ui reached",
                    "{\"uiElements\":0}",
                );
            }
            _ => {}
        }
        Ok(())
    }
}
