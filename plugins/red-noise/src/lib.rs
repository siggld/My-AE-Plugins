#![allow(clippy::drop_non_drop, clippy::question_mark)]

use after_effects as ae;
use std::env;

use ae::pf::*;
use utils::ToPixel;

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
enum Params {
    NoiseStrength,
}

#[derive(Default)]
struct Plugin {}

ae::define_effect!(Plugin, (), Params);

const PLUGIN_DESCRIPTION: &str = "Applies red noise over the entire image.";

impl AdobePluginGlobal for Plugin {
    fn params_setup(
        &self,
        params: &mut ae::Parameters<Params>,
        _in_data: InData,
        _: OutData,
    ) -> Result<(), Error> {
        params.add(
            Params::NoiseStrength,
            "Noise Strength",
            FloatSliderDef::setup(|d| {
                d.set_valid_min(0.0);
                d.set_valid_max(1.0);
                d.set_slider_min(0.0);
                d.set_slider_max(1.0);
                d.set_default(0.3);
                d.set_precision(3);
            }),
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
                        "AOD_RedNoise - {version}\r\r{PLUGIN_DESCRIPTION}\rCopyright (c) 2026-{build_year} Aodaruma",
                        version = env!("CARGO_PKG_VERSION"),
                        build_year = env!("BUILD_YEAR")
                    )
                    .as_str(),
                );
            }
            ae::Command::GlobalSetup => {
                out_data.set_out_flag2(OutFlags2::SupportsSmartRender, true);
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

impl Plugin {
    fn do_render(
        &self,
        in_data: InData,
        in_layer: Layer,
        _out_data: OutData,
        mut out_layer: Layer,
        params: &mut Parameters<Params>,
    ) -> Result<(), Error> {
        let progress_final = out_layer.height() as i32;
        let frame_num = in_data.current_frame() as usize;

        let strength = params
            .get(Params::NoiseStrength)?
            .as_float_slider()?
            .value() as f32;

        in_layer.iterate_with(&mut out_layer, 0, progress_final, None, |x, y, ip, op| {
            let x = x as usize;
            let y = y as usize;

            let mut px = ip.as_f32();

            let n = pseudo_random(x, y, frame_num);
            let noise = (n * 2.0 - 1.0) * strength;

            px.red = (px.red + noise).clamp(0.0, 1.0);

            match op {
                GenericPixelMut::Pixel8(p) => {
                    *p = px.to_pixel8();
                }
                GenericPixelMut::Pixel16(p) => {
                    *p = px.to_pixel16();
                }
                GenericPixelMut::PixelF32(p) => {
                    *p = px;
                }
                GenericPixelMut::PixelF64(p) => {
                    p.alphaF = px.alpha as f64;
                    p.redF = px.red as f64;
                    p.greenF = px.green as f64;
                    p.blueF = px.blue as f64;
                }
            }

            Ok(())
        })?;

        Ok(())
    }
}

fn pseudo_random(x: usize, y: usize, frame: usize) -> f32 {
    let mut v = (x as u32).wrapping_mul(73856093)
        ^ (y as u32).wrapping_mul(19349663)
        ^ (frame as u32).wrapping_mul(83492791);
    v = v.wrapping_mul(747796405u32).wrapping_add(2891336453u32);
    let max = u32::MAX as f32;
    (v as f32) / max
}
