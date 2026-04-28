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
    let mut paths: Vec<std::path::PathBuf> = Vec::new();
    // 1) relative to current working directory
    paths.push(std::path::PathBuf::from("debug-1e940a.log"));
    // 2) absolute workspace path for local debug runs
    paths.push(std::path::PathBuf::from(
        r"W:\work\My-AE-Plugins\debug-1e940a.log",
    ));
    // 3) temp directory fallback
    paths.push(std::env::temp_dir().join("debug-1e940a.log"));

    for p in &paths {
        if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(p) {
            let _ = f.write_all(line.as_bytes());
        }
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
        in_data: InData,
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
            ae::Command::SmartPreRender { mut extra } => {
                debug_log(
                    "H1",
                    "plugins/custom-gui-test/src/lib.rs:smart_pre_render",
                    "smart pre render reached",
                    "{\"handlerImplemented\":true}",
                );
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
                debug_log(
                    "H1",
                    "plugins/custom-gui-test/src/lib.rs:smart_render",
                    "smart render reached",
                    "{\"handlerImplemented\":true}",
                );
                let cb = extra.callbacks();
                let in_layer_opt = cb.checkout_layer_pixels(0)?;
                let out_layer_opt = cb.checkout_output()?;
                if let (Some(in_layer), Some(mut out_layer)) = (in_layer_opt, out_layer_opt) {
                    out_layer.copy_from(&in_layer, None, None)?;
                }
                cb.checkin_layer_pixels(0)?;
            }
            ae::Command::Render { in_layer, mut out_layer } => {
                debug_log(
                    "H3",
                    "plugins/custom-gui-test/src/lib.rs:render",
                    "legacy render reached",
                    "{\"path\":\"legacy-render\",\"implemented\":true}",
                );
                out_layer.copy_from(&in_layer, None, None)?;
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
