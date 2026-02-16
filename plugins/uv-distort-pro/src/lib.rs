#![allow(clippy::drop_non_drop, clippy::question_mark)]

use after_effects as ae;
use std::env;

use ae::pf::*;
use utils::ToPixel;

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
enum Params {
    TextureLayer,       // ID: 1
    UvMapLayer,         // ID: 2
    DistortMapLayer,    // ID: 3  (UI: Displacement Map)
    DistortIntensityX,  // ID: 4  (UI: Displacement X)
    DistortIntensityY,  // ID: 5  (UI: Displacement Y)
    UOffset,            // ID: 6
    VOffset,            // ID: 7
    WrapMode,           // ID: 8
    TextureLayerFit,    // ID: 9
    UvMapLayerFit,      // ID: 10
    DistortMapLayerFit, // ID: 11 (UI: Displacement Map Fit)
    EdgesThreshold,     // ID: 12
    UseGpu,             // ID: 13
    TextureScaleU,      // ID: 14
    TextureScaleV,      // ID: 15
    TextureOffsetU,     // ID: 16
    TextureOffsetV,     // ID: 17
}

#[derive(Default)]
struct Plugin {}

ae::define_effect!(Plugin, (), Params);

const PLUGIN_DESCRIPTION: &str = "High-quality UV-based distortion mapping.";

#[derive(Clone, Copy, Debug)]
enum WrapMode {
    Clamp,
    Repeat,
    Alternate,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum LayerFit {
    Center,
    Stretch,
}

/// Maps output pixel (ox, oy) to layer coordinates. Returns (layer_x, layer_y) in f32 and whether inside layer bounds.
fn output_to_layer_coord(
    ox: usize,
    oy: usize,
    out_w: usize,
    out_h: usize,
    layer_w: usize,
    layer_h: usize,
    fit: LayerFit,
) -> (f32, f32, bool) {
    let ox_f = ox as f32;
    let oy_f = oy as f32;
    let out_w_f = out_w as f32;
    let out_h_f = out_h as f32;
    let lw_f = layer_w as f32;
    let lh_f = layer_h as f32;

    let (lx, ly) = match fit {
        LayerFit::Stretch => {
            let lx = if out_w > 0 {
                ox_f * lw_f / out_w_f
            } else {
                0.0
            };
            let ly = if out_h > 0 {
                oy_f * lh_f / out_h_f
            } else {
                0.0
            };
            (lx, ly)
        }
        LayerFit::Center => {
            let cx = out_w_f * 0.5 - lw_f * 0.5;
            let cy = out_h_f * 0.5 - lh_f * 0.5;
            (ox_f - cx, oy_f - cy)
        }
    };

    let inside = lx >= 0.0 && lx < lw_f && ly >= 0.0 && ly < lh_f;
    (lx, ly, inside)
}

impl AdobePluginGlobal for Plugin {
    fn params_setup(
        &self,
        params: &mut ae::Parameters<Params>,
        _in_data: InData,
        _: OutData,
    ) -> Result<(), Error> {
        // Layer parameters (param indices 1, 2, 3). Index 0 is the effect input layer.
        params.add(
            Params::TextureLayer,
            "Texture Layer",
            LayerDef::setup(|_d| {}),
        )?;
        params.add(Params::UvMapLayer, "UV Map Layer", LayerDef::setup(|_d| {}))?;
        params.add(
            Params::DistortMapLayer,
            "Displacement Map",
            LayerDef::setup(|_d| {}),
        )?;

        // Displacement X (slider ±10, direct input ±100)
        params.add(
            Params::DistortIntensityX,
            "Displacement X",
            FloatSliderDef::setup(|d| {
                d.set_valid_min(-100.0);
                d.set_valid_max(100.0);
                d.set_slider_min(-10.0);
                d.set_slider_max(10.0);
                d.set_default(0.0);
                d.set_precision(3);
            }),
        )?;

        // Displacement Y
        params.add(
            Params::DistortIntensityY,
            "Displacement Y",
            FloatSliderDef::setup(|d| {
                d.set_valid_min(-100.0);
                d.set_valid_max(100.0);
                d.set_slider_min(-10.0);
                d.set_slider_max(10.0);
                d.set_default(0.0);
                d.set_precision(3);
            }),
        )?;

        // U Offset
        params.add(
            Params::UOffset,
            "U Offset",
            FloatSliderDef::setup(|d| {
                d.set_valid_min(-100.0);
                d.set_valid_max(100.0);
                d.set_slider_min(-10.0);
                d.set_slider_max(10.0);
                d.set_default(0.0);
                d.set_precision(3);
            }),
        )?;

        // V Offset
        params.add(
            Params::VOffset,
            "V Offset",
            FloatSliderDef::setup(|d| {
                d.set_valid_min(-100.0);
                d.set_valid_max(100.0);
                d.set_slider_min(-10.0);
                d.set_slider_max(10.0);
                d.set_default(0.0);
                d.set_precision(3);
            }),
        )?;

        // Wrap Mode: 1 = Clamp, 2 = Repeat, 3 = Alternate
        params.add(
            Params::WrapMode,
            "Wrap Mode",
            PopupDef::setup(|d| {
                d.set_options(&["Clamp", "Repeat", "Alternate"]);
                d.set_default(1);
            }),
        )?;

        // Edges Threshold (%): alpha below this is treated as transparent (reduces edge lines).
        params.add(
            Params::EdgesThreshold,
            "Edges Threshold (%)",
            FloatSliderDef::setup(|d| {
                d.set_valid_min(0.0);
                d.set_valid_max(100.0);
                d.set_slider_min(0.0);
                d.set_slider_max(20.0);
                d.set_default(1.0);
                d.set_precision(2);
            }),
        )?;

        // Use GPU (checkbox; GPU path not implemented, stored for future use).
        params.add(
            Params::UseGpu,
            "Use GPU",
            CheckBoxDef::setup(|d| d.set_default(false)),
        )?;

        // Texture scale/offset after UV (applied in 0..1 space before sampling).
        params.add(
            Params::TextureScaleU,
            "Texture Scale U",
            FloatSliderDef::setup(|d| {
                d.set_valid_min(0.01);
                d.set_valid_max(10.0);
                d.set_slider_min(0.25);
                d.set_slider_max(2.0);
                d.set_default(1.0);
                d.set_precision(3);
            }),
        )?;
        params.add(
            Params::TextureScaleV,
            "Texture Scale V",
            FloatSliderDef::setup(|d| {
                d.set_valid_min(0.01);
                d.set_valid_max(10.0);
                d.set_slider_min(0.25);
                d.set_slider_max(2.0);
                d.set_default(1.0);
                d.set_precision(3);
            }),
        )?;
        params.add(
            Params::TextureOffsetU,
            "Texture Offset U",
            FloatSliderDef::setup(|d| {
                d.set_valid_min(-1.0);
                d.set_valid_max(1.0);
                d.set_slider_min(-0.5);
                d.set_slider_max(0.5);
                d.set_default(0.0);
                d.set_precision(3);
            }),
        )?;
        params.add(
            Params::TextureOffsetV,
            "Texture Offset V",
            FloatSliderDef::setup(|d| {
                d.set_valid_min(-1.0);
                d.set_valid_max(1.0);
                d.set_slider_min(-0.5);
                d.set_slider_max(0.5);
                d.set_default(0.0);
                d.set_precision(3);
            }),
        )?;

        // Layer fit when smaller than comp: 1 = Center, 2 = Stretch
        params.add(
            Params::TextureLayerFit,
            "Texture Layer Fit",
            PopupDef::setup(|d| {
                d.set_options(&["Center", "Stretch"]);
                d.set_default(2);
            }),
        )?;
        params.add(
            Params::UvMapLayerFit,
            "UV Map Layer Fit",
            PopupDef::setup(|d| {
                d.set_options(&["Center", "Stretch"]);
                d.set_default(2);
            }),
        )?;
        params.add(
            Params::DistortMapLayerFit,
            "Displacement Map Fit",
            PopupDef::setup(|d| {
                d.set_options(&["Center", "Stretch"]);
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
                // Fallback: use the same input layer for texture / UV / displacement.
                self.do_render(
                    in_data,
                    &in_layer,
                    &in_layer,
                    Some(&in_layer),
                    out_data,
                    out_layer,
                    params,
                )?;
            }

            ae::Command::SmartPreRender { mut extra } => {
                let req = extra.output_request();
                let cb = extra.callbacks();
                let t = in_data.current_time();
                let ts = in_data.time_step();
                let tscale = in_data.time_scale();

                // Checkout layer params (indices 1,2,3) and the effect input layer (index 0) as fallback.
                // checkout_id: 0=Texture, 1=UV Map, 2=Distort, 3=input layer (used when a param is unset).
                for (param_index, checkout_id) in [(1, 0), (2, 1), (3, 2), (0, 3)] {
                    if let Ok(result) =
                        cb.checkout_layer(param_index, checkout_id, &req, t, ts, tscale)
                    {
                        let _ = extra.union_result_rect(result.result_rect.into());
                        let _ = extra.union_max_result_rect(result.max_result_rect.into());
                    } else {
                        return Err(Error::InterruptCancel);
                    }
                }
            }

            ae::Command::SmartRender { extra } => {
                let cb = extra.callbacks();

                // checkout_id 0=Texture, 1=UV Map, 2=Distort, 3=effect input layer (fallback).
                let tex_layer_opt = cb.checkout_layer_pixels(0)?;
                let uv_layer_opt = cb.checkout_layer_pixels(1)?;
                let dist_layer_opt = cb.checkout_layer_pixels(2)?;
                let input_layer_opt = cb.checkout_layer_pixels(3)?;
                let out_layer_opt = cb.checkout_output()?;

                if let Some(out_layer) = out_layer_opt {
                    let input_ref = input_layer_opt.as_ref();
                    let tex = tex_layer_opt.as_ref().or(input_ref);
                    let uv = uv_layer_opt.as_ref().or(input_ref);
                    // Displacement Map is optional: None = constant 0.5 gray (no displacement).
                    let dist = dist_layer_opt.as_ref();
                    if let (Some(tex), Some(uv)) = (tex, uv) {
                        self.do_render(in_data, tex, uv, dist, out_data, out_layer, params)?;
                    }
                }

                cb.checkin_layer_pixels(0)?;
                cb.checkin_layer_pixels(1)?;
                cb.checkin_layer_pixels(2)?;
                cb.checkin_layer_pixels(3)?;
            }

            _ => {}
        }
        Ok(())
    }
}

impl Plugin {
    #[allow(clippy::too_many_arguments)]
    fn do_render(
        &self,
        _in_data: InData,
        texture_layer: &Layer,
        uv_layer: &Layer,
        distort_layer: Option<&Layer>,
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
            3 => WrapMode::Alternate,
            _ => WrapMode::Clamp,
        };

        let edges_threshold_pct = params
            .get(Params::EdgesThreshold)?
            .as_float_slider()?
            .value() as f32;
        let edges_threshold = (edges_threshold_pct / 100.0).clamp(0.0, 1.0);

        let _use_gpu = params.get(Params::UseGpu)?.as_checkbox()?.value();
        // GPU path not implemented; _use_gpu reserved for future use.

        let texture_scale_u = params
            .get(Params::TextureScaleU)?
            .as_float_slider()?
            .value() as f32;
        let texture_scale_v = params
            .get(Params::TextureScaleV)?
            .as_float_slider()?
            .value() as f32;
        let texture_offset_u = params
            .get(Params::TextureOffsetU)?
            .as_float_slider()?
            .value() as f32;
        let texture_offset_v = params
            .get(Params::TextureOffsetV)?
            .as_float_slider()?
            .value() as f32;

        let texture_fit = match params.get(Params::TextureLayerFit)?.as_popup()?.value() {
            1 => LayerFit::Center,
            2 => LayerFit::Stretch,
            _ => LayerFit::Stretch,
        };
        let uv_fit = match params.get(Params::UvMapLayerFit)?.as_popup()?.value() {
            1 => LayerFit::Center,
            2 => LayerFit::Stretch,
            _ => LayerFit::Stretch,
        };
        let distort_fit = match params.get(Params::DistortMapLayerFit)?.as_popup()?.value() {
            1 => LayerFit::Center,
            2 => LayerFit::Stretch,
            _ => LayerFit::Stretch,
        };

        let tex_world_type = texture_layer.world_type();
        let uv_world_type = uv_layer.world_type();
        let out_world_type = out_layer.world_type();

        let tex_w = texture_layer.width();
        let tex_h = texture_layer.height();
        let uv_w = uv_layer.width();
        let uv_h = uv_layer.height();
        let (dist_w, dist_h, dist_world_type) = match distort_layer {
            Some(d) => (d.width(), d.height(), d.world_type()),
            None => (0, 0, ae::aegp::WorldType::U8),
        };

        let out_w = out_layer.width();
        let out_h = out_layer.height();

        out_layer.iterate(0, progress_final, None, |x, y, mut dst| {
            let x = x as usize;
            let y = y as usize;

            // Map output (x,y) to UV map layer coords (Center or Stretch).
            let (lx_uv, ly_uv, uv_inside) =
                output_to_layer_coord(x, y, out_w, out_h, uv_w, uv_h, uv_fit);
            let (u_base, v_base) = if uv_inside {
                let x_uv = (lx_uv as usize).min(uv_w.saturating_sub(1));
                let y_uv = (ly_uv as usize).min(uv_h.saturating_sub(1));
                let uv_px = read_pixel_f32(uv_layer, uv_world_type, x_uv, y_uv);
                // Standard UV: flip V so that top-left in image = top in texture (V=0).
                let u = uv_px.red;
                let v = 1.0 - uv_px.green;
                (u, v)
            } else {
                (0.5, 0.5)
            };

            // Displacement: when no layer, use constant 0.5 (no displacement).
            let l = match distort_layer {
                None => 0.5,
                Some(distort_layer) => {
                    let (lx_dist, ly_dist, dist_inside) =
                        output_to_layer_coord(x, y, out_w, out_h, dist_w, dist_h, distort_fit);
                    if dist_inside {
                        let x_dist = (lx_dist as usize).min(dist_w.saturating_sub(1));
                        let y_dist = (ly_dist as usize).min(dist_h.saturating_sub(1));
                        let dist_px =
                            read_pixel_f32(distort_layer, dist_world_type, x_dist, y_dist);
                        luminance(dist_px)
                    } else {
                        0.5
                    }
                }
            };

            // UV distortion formula.
            let u_final = u_base + (l - 0.5) * intensity_x + u_offset;
            let v_final = v_base + (l - 0.5) * intensity_y + v_offset;

            // Apply wrap mode in normalized 0..1 space.
            let u_wrapped = wrap_coord(u_final, wrap_mode);
            let v_wrapped = wrap_coord(v_final, wrap_mode);

            // Texture scale and offset after UV (in 0..1 space), then wrap again.
            let u_scaled = wrap_coord(u_wrapped * texture_scale_u + texture_offset_u, wrap_mode);
            let v_scaled = wrap_coord(v_wrapped * texture_scale_v + texture_offset_v, wrap_mode);

            // Texture sampling: in Center mode map 0..1 to texture with letterbox.
            let (u_tex, v_tex) = match texture_fit {
                LayerFit::Stretch => (u_scaled, v_scaled),
                LayerFit::Center => {
                    let out_w_f = out_w as f32;
                    let out_h_f = out_h as f32;
                    let tw_f = tex_w as f32;
                    let th_f = tex_h as f32;
                    let u_tex = u_scaled * out_w_f / tw_f - out_w_f / (2.0 * tw_f) + 0.5;
                    let v_tex = v_scaled * out_h_f / th_f - out_h_f / (2.0 * th_f) + 0.5;
                    (u_tex, v_tex)
                }
            };

            let mut tex_px = if texture_fit == LayerFit::Center
                && (!(0.0..=1.0).contains(&u_tex) || !(0.0..=1.0).contains(&v_tex))
            {
                PixelF32 {
                    red: 0.0,
                    green: 0.0,
                    blue: 0.0,
                    alpha: 0.0,
                }
            } else {
                sample_layer_f32(
                    texture_layer,
                    tex_world_type,
                    tex_w,
                    tex_h,
                    u_tex,
                    v_tex,
                    edges_threshold,
                )
            };

            // Edges Threshold: alpha below this is fully transparent (reduces edge lines).
            if tex_px.alpha < edges_threshold {
                tex_px.red = 0.0;
                tex_px.green = 0.0;
                tex_px.blue = 0.0;
                tex_px.alpha = 0.0;
            }

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
        WrapMode::Alternate => {
            let r = v.rem_euclid(1.0);
            let r = if r < 0.0 { r + 1.0 } else { r };
            let period = (v / 1.0).floor() as i32;
            if period.rem_euclid(2) == 1 {
                1.0 - r
            } else {
                r
            }
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
    edges_threshold: f32,
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

    // If any of the 4 samples has alpha below threshold (edge), force transparent to reduce diagonal lines.
    let min_alpha = c00.alpha.min(c10.alpha).min(c01.alpha).min(c11.alpha);
    if min_alpha < edges_threshold {
        return PixelF32 {
            red: 0.0,
            green: 0.0,
            blue: 0.0,
            alpha: 0.0,
        };
    }

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
