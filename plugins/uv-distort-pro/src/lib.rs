#![allow(clippy::drop_non_drop, clippy::question_mark)]

use after_effects as ae;
use std::env;

use ae::pf::*;
use utils::ToPixel;

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
enum Params {
    TextureLayer,      // ID: 1
    UvMapLayer,        // ID: 2
    DistortMapLayer,   // ID: 3
    DistortIntensityX, // ID: 4
    DistortIntensityY, // ID: 5
    UOffset,           // ID: 6
    VOffset,           // ID: 7
    WrapMode,          // ID: 8
}

#[derive(Default)]
struct Plugin {}

ae::define_effect!(Plugin, (), Params);

const PLUGIN_DESCRIPTION: &str = "High-quality UV-based distortion mapping.";

#[derive(Clone, Copy, Debug)]
enum WrapMode {
    Clamp,
    Repeat,
}

impl AdobePluginGlobal for Plugin {
    fn params_setup(
        &self,
        params: &mut ae::Parameters<Params>,
        _in_data: InData,
        _: OutData,
    ) -> Result<(), Error> {
        // Texture / UV / Distort layers are supplied by the host as input layers (indices 0..=2).
        // Here we just expose controls for intensity / offset / wrap mode.

        // Distort Intensity X
        params.add(
            Params::DistortIntensityX,
            "Distort Intensity X",
            FloatSliderDef::setup(|d| {
                d.set_valid_min(-1.0);
                d.set_valid_max(1.0);
                d.set_slider_min(-1.0);
                d.set_slider_max(1.0);
                d.set_default(0.0);
                d.set_precision(3);
            }),
        )?;

        // Distort Intensity Y
        params.add(
            Params::DistortIntensityY,
            "Distort Intensity Y",
            FloatSliderDef::setup(|d| {
                d.set_valid_min(-1.0);
                d.set_valid_max(1.0);
                d.set_slider_min(-1.0);
                d.set_slider_max(1.0);
                d.set_default(0.0);
                d.set_precision(3);
            }),
        )?;

        // U Offset
        params.add(
            Params::UOffset,
            "U Offset",
            FloatSliderDef::setup(|d| {
                d.set_valid_min(-1.0);
                d.set_valid_max(1.0);
                d.set_slider_min(-1.0);
                d.set_slider_max(1.0);
                d.set_default(0.0);
                d.set_precision(3);
            }),
        )?;

        // V Offset
        params.add(
            Params::VOffset,
            "V Offset",
            FloatSliderDef::setup(|d| {
                d.set_valid_min(-1.0);
                d.set_valid_max(1.0);
                d.set_slider_min(-1.0);
                d.set_slider_max(1.0);
                d.set_default(0.0);
                d.set_precision(3);
            }),
        )?;

        // Wrap Mode: 1 = Clamp, 2 = Repeat
        params.add(
            Params::WrapMode,
            "Wrap Mode",
            PopupDef::setup(|d| {
                d.set_options(&["Clamp", "Repeat"]);
                d.set_default(1);
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
                        "AOD_UvDistortPro - {version}\r\r{PLUGIN_DESCRIPTION}\rCopyright (c) 2026-{build_year} Aodaruma",
                        version = env!("CARGO_PKG_VERSION"),
                        build_year = env!("BUILD_YEAR")
                    )
                    .as_str(),
                );
            }
            ae::Command::GlobalSetup => {
                // Smart Render is enabled via PiPL flags; we also mark here.
                out_data.set_out_flag2(OutFlags2::SupportsSmartRender, true);
            }
            ae::Command::Render {
                in_layer,
                out_layer,
            } => {
                // Fallback: use the same input layer for texture / UV / distort.
                self.do_render(
                    in_data,
                    in_layer,
                    in_layer,
                    in_layer,
                    out_data,
                    out_layer,
                    params,
                )?;
            }

            ae::Command::SmartPreRender { mut extra } => {
                let req = extra.output_request();

                // We at least union the main input (index 0).
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

                // We expect:
                //  - index 0: Texture Layer
                //  - index 1: UV Map Layer
                //  - index 2: Distort Map Layer
                let tex_layer_opt = cb.checkout_layer_pixels(0)?;
                let uv_layer_opt = cb.checkout_layer_pixels(1)?;
                let dist_layer_opt = cb.checkout_layer_pixels(2)?;
                let out_layer_opt = cb.checkout_output()?;

                if let (Some(tex), Some(uv), Some(dist), Some(out_layer)) =
                    (tex_layer_opt, uv_layer_opt, dist_layer_opt, out_layer_opt)
                {
                    self.do_render(in_data, tex, uv, dist, out_data, out_layer, params)?;
                }

                cb.checkin_layer_pixels(0)?;
                cb.checkin_layer_pixels(1)?;
                cb.checkin_layer_pixels(2)?;
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
        texture_layer: Layer,
        uv_layer: Layer,
        distort_layer: Layer,
        _out_data: OutData,
        mut out_layer: Layer,
        params: &mut Parameters<Params>,
    ) -> Result<(), Error> {
        let progress_final = out_layer.height() as i32;

        // Read parameters.
        let intensity_x = params
            .get(Params::DistortIntensityX)?
            .as_float_slider()?
            .value() as f32;
        let intensity_y = params
            .get(Params::DistortIntensityY)?
            .as_float_slider()?
            .value() as f32;
        let u_offset = params.get(Params::UOffset)?.as_float_slider()?.value() as f32;
        let v_offset = params.get(Params::VOffset)?.as_float_slider()?.value() as f32;

        let wrap_mode = match params.get(Params::WrapMode)?.as_popup()?.value() {
            1 => WrapMode::Clamp,
            2 => WrapMode::Repeat,
            _ => WrapMode::Clamp,
        };

        let tex_world_type = texture_layer.world_type();
        let uv_world_type = uv_layer.world_type();
        let dist_world_type = distort_layer.world_type();
        let out_world_type = out_layer.world_type();

        let tex_w = texture_layer.width() as usize;
        let tex_h = texture_layer.height() as usize;
        let uv_w = uv_layer.width() as usize;
        let uv_h = uv_layer.height() as usize;
        let dist_w = distort_layer.width() as usize;
        let dist_h = distort_layer.height() as usize;
        let out_w = out_layer.width() as usize;
        let out_h = out_layer.height() as usize;

        out_layer.iterate(0, progress_final, None, |x, y, mut dst| {
            let x = x as usize;
            let y = y as usize;

            // Clamp coordinates for UV / Distort maps to their sizes.
            let x_uv = x.min(uv_w.saturating_sub(1));
            let y_uv = y.min(uv_h.saturating_sub(1));
            let x_dist = x.min(dist_w.saturating_sub(1));
            let y_dist = y.min(dist_h.saturating_sub(1));

            // Base UV from UV map (R=U, G=V).
            let uv_px = read_pixel_f32(&uv_layer, uv_world_type, x_uv, y_uv);
            let u_base = uv_px.red;
            let v_base = uv_px.green;

            // Distort luminance from Distort map.
            let dist_px = read_pixel_f32(&distort_layer, dist_world_type, x_dist, y_dist);
            let l = luminance(dist_px); // 0..1

            // UV distortion formula.
            let u_final = u_base + (l - 0.5) * intensity_x + u_offset;
            let v_final = v_base + (l - 0.5) * intensity_y + v_offset;

            // Apply wrap mode in normalized 0..1 space.
            let u_wrapped = wrap_coord(u_final, wrap_mode);
            let v_wrapped = wrap_coord(v_final, wrap_mode);

            // Sample texture with bilinear interpolation (high-quality sampling).
            let tex_px = sample_layer_f32(
                &texture_layer,
                tex_world_type,
                tex_w,
                tex_h,
                u_wrapped,
                v_wrapped,
            );

            // Write to output with correct bit depth.
            match out_world_type {
                ae::aegp::WorldType::U8 => dst.set_from_u8(tex_px.to_pixel8()),
                ae::aegp::WorldType::U15 => dst.set_from_u16(tex_px.to_pixel16()),
                ae::aegp::WorldType::F32 | ae::aegp::WorldType::None => {
                    dst.set_from_f32(tex_px);
                }
            }

            Ok(())
        })?;

        Ok(())
    }
}

fn read_pixel_f32(layer: &Layer, world_type: ae::aegp::WorldType, x: usize, y: usize) -> PixelF32 {
    match world_type {
        ae::aegp::WorldType::U8 => layer.as_pixel8(x, y).to_pixel32(),
        ae::aegp::WorldType::U15 => layer.as_pixel16(x, y).to_pixel32(),
        ae::aegp::WorldType::F32 | ae::aegp::WorldType::None => *layer.as_pixel32(x, y),
    }
}

fn luminance(px: PixelF32) -> f32 {
    // Simple Rec. 709 luma.
    (0.2126 * px.red + 0.7152 * px.green + 0.0722 * px.blue).clamp(0.0, 1.0)
}

fn wrap_coord(v: f32, mode: WrapMode) -> f32 {
    match mode {
        WrapMode::Clamp => v.clamp(0.0, 1.0),
        WrapMode::Repeat => {
            let r = v.rem_euclid(1.0);
            if r < 0.0 { r + 1.0 } else { r }
        }
    }
}

fn sample_layer_f32(
    layer: &Layer,
    world_type: ae::aegp::WorldType,
    width: usize,
    height: usize,
    u: f32,
    v: f32,
) -> PixelF32 {
    if width == 0 || height == 0 {
        return PixelF32 {
            red: 0.0,
            green: 0.0,
            blue: 0.0,
            alpha: 0.0,
        };
    }

    let fx = (u.clamp(0.0, 1.0) * (width as f32 - 1.0)).max(0.0);
    let fy = (v.clamp(0.0, 1.0) * (height as f32 - 1.0)).max(0.0);

    let x0 = fx.floor() as isize;
    let y0 = fy.floor() as isize;
    let x1 = (x0 + 1).min(width as isize - 1);
    let y1 = (y0 + 1).min(height as isize - 1);

    let sx = fx - x0 as f32;
    let sy = fy - y0 as f32;

    let c00 = read_pixel_f32(layer, world_type, x0 as usize, y0 as usize);
    let c10 = read_pixel_f32(layer, world_type, x1 as usize, y0 as usize);
    let c01 = read_pixel_f32(layer, world_type, x0 as usize, y1 as usize);
    let c11 = read_pixel_f32(layer, world_type, x1 as usize, y1 as usize);

    // Bilinear interpolation.
    let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t;

    let mut out = PixelF32 {
        alpha: 0.0,
        red: 0.0,
        green: 0.0,
        blue: 0.0,
    };

    out.alpha = lerp(
        lerp(c00.alpha, c10.alpha, sx),
        lerp(c01.alpha, c11.alpha, sx),
        sy,
    );
    out.red = lerp(lerp(c00.red, c10.red, sx), lerp(c01.red, c11.red, sx), sy);
    out.green = lerp(
        lerp(c00.green, c10.green, sx),
        lerp(c01.green, c11.green, sx),
        sy,
    );
    out.blue = lerp(
        lerp(c00.blue, c10.blue, sx),
        lerp(c01.blue, c11.blue, sx),
        sy,
    );

    out
}