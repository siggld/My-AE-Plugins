use ae::pf::*;
use after_effects as ae;
use std::env;

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
enum Params {
    ShowSourceLayerNames,
    TransferKeyframes,
    TransferExpressions,
    RewriteLayerContext,
    WarnOnAmbiguousLayers,
    MvpStatus,
}

#[derive(Default)]
struct Plugin {}

ae::define_effect!(Plugin, (), Params);

const PLUGIN_DESCRIPTION: &str =
    "Scaffolds a hub-style controller for collecting and relaying AE effect parameters.";

impl AdobePluginGlobal for Plugin {
    fn params_setup(
        &self,
        params: &mut ae::Parameters<Params>,
        _in_data: InData,
        _: OutData,
    ) -> Result<(), Error> {
        params.add(
            Params::ShowSourceLayerNames,
            "Show Source Layer Names",
            CheckBoxDef::setup(|d| {
                d.set_default(false);
            }),
        )?;
        params.add(
            Params::TransferKeyframes,
            "Transfer Keyframes",
            CheckBoxDef::setup(|d| {
                d.set_default(true);
            }),
        )?;
        params.add(
            Params::TransferExpressions,
            "Transfer Expressions",
            CheckBoxDef::setup(|d| {
                d.set_default(true);
            }),
        )?;
        params.add(
            Params::RewriteLayerContext,
            "Rewrite Layer Context",
            CheckBoxDef::setup(|d| {
                d.set_default(true);
            }),
        )?;
        params.add(
            Params::WarnOnAmbiguousLayers,
            "Warn On Ambiguous Layers",
            CheckBoxDef::setup(|d| {
                d.set_default(true);
            }),
        )?;
        params.add(
            Params::MvpStatus,
            "MVP Status",
            PopupDef::setup(|d| {
                d.set_options(&["Scaffold Only", "Needs AEGP or Script"]);
                d.set_default(2);
            }),
        )?;

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
                        "TKG_ParameterHub - {version}\r\r{PLUGIN_DESCRIPTION}\rCopyright (c) 2026-{build_year} Aodaruma",
                        version = env!("CARGO_PKG_VERSION"),
                        build_year = env!("BUILD_YEAR")
                    )
                    .as_str(),
                );
            }
            ae::Command::GlobalSetup => {
                out_data.set_out_flag2(OutFlags2::SupportsThreadedRendering, true);
                out_data.set_out_flag2(OutFlags2::SupportsGetFlattenedSequenceData, true);
                out_data.set_out_flag2(OutFlags2::SupportsSmartRender, true);
            }
            ae::Command::Render {
                in_layer,
                mut out_layer,
            } => {
                self.do_render(in_layer, &mut out_layer)?;
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

                if let (Some(in_layer), Some(mut out_layer)) = (in_layer_opt, out_layer_opt) {
                    self.do_render(in_layer, &mut out_layer)?;
                }

                cb.checkin_layer_pixels(0)?;
            }
            _ => {}
        }
        Ok(())
    }
}

impl Plugin {
    fn do_render(&self, in_layer: Layer, out_layer: &mut Layer) -> Result<(), Error> {
        // Initial scaffold: keep rendering behavior neutral while the Hub workflow is built.
        out_layer.copy_from(&in_layer, None, None)?;
        Ok(())
    }
}
