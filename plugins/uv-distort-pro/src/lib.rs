#![allow(clippy::drop_non_drop, clippy::question_mark)]

use after_effects as ae;
use std::env;

use ae::pf::*;
use utils::ToPixel;

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
enum Params {
    TextureLayer,        // 1
    TextureLayerFit,     // 2
    TextureCenterX,      // 3
    TextureCenterY,      // 4
    TextureScaleU,       // 5   (%, 100 = 1.0)
    TextureScaleV,       // 6
    TextureOffsetU,      // 7
    TextureOffsetV,      // 8
    UvMapLayer,          // 9
    UvMapLayerFit,       // 10
    UvMapScale,          // 11  (%, 100 = 1.0)
    UvMapCenterU,        // 12
    UvMapCenterV,        // 13
    WrapMode,            // 14
    UOffset,             // 15
    VOffset,             // 16
    DistortMapLayer,     // 17  (Displacement Map)
    DistortMapLayerFit,  // 18
    DistortIntensityX,   // 19
    DistortIntensityY,   // 20
    AlphaEdgesThreshold, // 21
    TextureEdgeMode,     // 22
    TextureFlipU,        // 23
    TextureFlipV,        // 24
    UseGpu,              // 25
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

/// How to sample at the texture image boundary (like Fast Box Blur "Repeat Edge Pixels").
#[derive(Clone, Copy, Debug, PartialEq)]
enum TextureEdgeMode {
    Transparent,      // outside 0..1 = transparent
    RepeatEdgePixels, // clamp to 0..1, extend edge pixel
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
        // UI: All float params use precision 2 (0.00). Fit defaults = Center (index 1).
        // Topic groups (Texture / UV Map / Displacement): PF_Param_TOPIC_START/END need raw
        // PF_ParamDef array; after_effects params.add() does not expose them.
        // ---- Texture Settings ----
        params.add(
            Params::TextureLayer,
            "Texture Layer",
            LayerDef::setup(|_d| {}),
        )?;
        params.add(
            Params::TextureLayerFit,
            "Texture Layer Fit",
            PopupDef::setup(|d| {
                d.set_options(&["Center", "Stretch"]);
                d.set_default(1);
            }),
        )?;
        params.add(
            Params::TextureCenterX,
            "Texture Center X",
            FloatSliderDef::setup(|d| {
                d.set_valid_min(0.0);
                d.set_valid_max(1.0);
                d.set_slider_min(0.0);
                d.set_slider_max(1.0);
                d.set_default(0.5);
                d.set_precision(2);
            }),
        )?;
        params.add(
            Params::TextureCenterY,
            "Texture Center Y",
            FloatSliderDef::setup(|d| {
                d.set_valid_min(0.0);
                d.set_valid_max(1.0);
                d.set_slider_min(0.0);
                d.set_slider_max(1.0);
                d.set_default(0.5);
                d.set_precision(2);
            }),
        )?;
        params.add(
            Params::TextureScaleU,
            "Texture Scale U (%)",
            FloatSliderDef::setup(|d| {
                d.set_valid_min(1.0);
                d.set_valid_max(1000.0);
                d.set_slider_min(25.0);
                d.set_slider_max(200.0);
                d.set_default(100.0);
                d.set_precision(2);
            }),
        )?;
        params.add(
            Params::TextureScaleV,
            "Texture Scale V (%)",
            FloatSliderDef::setup(|d| {
                d.set_valid_min(1.0);
                d.set_valid_max(1000.0);
                d.set_slider_min(25.0);
                d.set_slider_max(200.0);
                d.set_default(100.0);
                d.set_precision(2);
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
                d.set_precision(2);
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
                d.set_precision(2);
            }),
        )?;

        // ---- UV Map Settings ----
        params.add(Params::UvMapLayer, "UV Map Layer", LayerDef::setup(|_d| {}))?;
        params.add(
            Params::UvMapLayerFit,
            "UV Map Layer Fit",
            PopupDef::setup(|d| {
                d.set_options(&["Center", "Stretch"]);
                d.set_default(1);
            }),
        )?;
        params.add(
            Params::UvMapScale,
            "UV Map Scale (%)",
            FloatSliderDef::setup(|d| {
                d.set_valid_min(1.0);
                d.set_valid_max(1000.0);
                d.set_slider_min(25.0);
                d.set_slider_max(200.0);
                d.set_default(100.0);
                d.set_precision(2);
            }),
        )?;
        params.add(
            Params::UvMapCenterU,
            "UV Map Center U",
            FloatSliderDef::setup(|d| {
                d.set_valid_min(0.0);
                d.set_valid_max(1.0);
                d.set_slider_min(0.0);
                d.set_slider_max(1.0);
                d.set_default(0.5);
                d.set_precision(2);
            }),
        )?;
        params.add(
            Params::UvMapCenterV,
            "UV Map Center V",
            FloatSliderDef::setup(|d| {
                d.set_valid_min(0.0);
                d.set_valid_max(1.0);
                d.set_slider_min(0.0);
                d.set_slider_max(1.0);
                d.set_default(0.5);
                d.set_precision(2);
            }),
        )?;
        params.add(
            Params::WrapMode,
            "Wrap Mode",
            PopupDef::setup(|d| {
                d.set_options(&["Clamp", "Repeat", "Alternate"]);
                d.set_default(1);
            }),
        )?;
        params.add(
            Params::UOffset,
            "U Offset",
            FloatSliderDef::setup(|d| {
                d.set_valid_min(-100.0);
                d.set_valid_max(100.0);
                d.set_slider_min(-10.0);
                d.set_slider_max(10.0);
                d.set_default(0.0);
                d.set_precision(2);
            }),
        )?;
        params.add(
            Params::VOffset,
            "V Offset",
            FloatSliderDef::setup(|d| {
                d.set_valid_min(-100.0);
                d.set_valid_max(100.0);
                d.set_slider_min(-10.0);
                d.set_slider_max(10.0);
                d.set_default(0.0);
                d.set_precision(2);
            }),
        )?;

        // ---- Displacement Settings (disabled when X and Y are both 0) ----
        params.add(
            Params::DistortMapLayer,
            "Displacement Map",
            LayerDef::setup(|_d| {}),
        )?;
        params.add(
            Params::DistortMapLayerFit,
            "Displacement Map Fit",
            PopupDef::setup(|d| {
                d.set_options(&["Center", "Stretch"]);
                d.set_default(1);
            }),
        )?;
        params.add(
            Params::DistortIntensityX,
            "Displacement X",
            FloatSliderDef::setup(|d| {
                d.set_valid_min(-100.0);
                d.set_valid_max(100.0);
                d.set_slider_min(-10.0);
                d.set_slider_max(10.0);
                d.set_default(0.0);
                d.set_precision(2);
            }),
        )?;
        params.add(
            Params::DistortIntensityY,
            "Displacement Y",
            FloatSliderDef::setup(|d| {
                d.set_valid_min(-100.0);
                d.set_valid_max(100.0);
                d.set_slider_min(-10.0);
                d.set_slider_max(10.0);
                d.set_default(0.0);
                d.set_precision(2);
            }),
        )?;

        // ---- Edge / boundary ----
        params.add(
            Params::AlphaEdgesThreshold,
            "Alpha Edges Threshold (%)",
            FloatSliderDef::setup(|d| {
                d.set_valid_min(0.0);
                d.set_valid_max(100.0);
                d.set_slider_min(0.0);
                d.set_slider_max(20.0);
                d.set_default(1.0);
                d.set_precision(2);
            }),
        )?;
        params.add(
            Params::TextureEdgeMode,
            "Texture Edge",
            PopupDef::setup(|d| {
                d.set_options(&["Transparent", "Repeat Edge Pixels"]);
                d.set_default(1);
            }),
        )?;
        params.add(
            Params::TextureFlipU,
            "Texture Flip U",
            CheckBoxDef::setup(|d| {
                d.set_default(false);
            }),
        )?;
        params.add(
            Params::TextureFlipV,
            "Texture Flip V",
            CheckBoxDef::setup(|d| {
                d.set_default(false);
            }),
        )?;

        // ---- Global ----
        params.add(
            Params::UseGpu,
            "Use GPU",
            CheckBoxDef::setup(|d| {
                d.set_default(false);
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
                // Fallback: same layer for texture; UV = Some so UV map is used (same layer).
                self.do_render(
                    in_data,
                    &in_layer,
                    Some(&in_layer),
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

                // Checkout layer params (1=Texture, 9=UV Map, 17=Displacement) and input (0).
                for (param_index, checkout_id) in [(1, 0), (9, 1), (17, 2), (0, 3)] {
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
                    // UV Map None = no UV distortion (texture shown stretched to output).
                    let uv = uv_layer_opt.as_ref();
                    // Displacement Map None = constant 0.5 gray (no displacement).
                    let dist = dist_layer_opt.as_ref();
                    if let Some(tex) = tex {
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
        uv_layer: Option<&Layer>,
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

        let uv_map_scale =
            params.get(Params::UvMapScale)?.as_float_slider()?.value() as f32 / 100.0;
        let uv_map_center_u = params.get(Params::UvMapCenterU)?.as_float_slider()?.value() as f32;
        let uv_map_center_v = params.get(Params::UvMapCenterV)?.as_float_slider()?.value() as f32;

        let wrap_mode = match params.get(Params::WrapMode)?.as_popup()?.value() {
            1 => WrapMode::Clamp,
            2 => WrapMode::Repeat,
            3 => WrapMode::Alternate,
            _ => WrapMode::Clamp,
        };

        let alpha_edges_threshold_pct = params
            .get(Params::AlphaEdgesThreshold)?
            .as_float_slider()?
            .value() as f32;
        let alpha_edges_threshold = (alpha_edges_threshold_pct / 100.0).clamp(0.0, 1.0);

        let texture_edge_mode = match params.get(Params::TextureEdgeMode)?.as_popup()?.value() {
            1 => TextureEdgeMode::Transparent,
            2 => TextureEdgeMode::RepeatEdgePixels,
            _ => TextureEdgeMode::Transparent,
        };

        // Displacement disabled when both X and Y are 0 (no Enable checkbox).
        let displacement_disabled = intensity_x == 0.0 && intensity_y == 0.0;

        let _use_gpu = params.get(Params::UseGpu)?.as_checkbox()?.value();
        // GPU path not implemented; _use_gpu reserved for future use.

        let texture_center_x = params
            .get(Params::TextureCenterX)?
            .as_float_slider()?
            .value() as f32;
        let texture_center_y = params
            .get(Params::TextureCenterY)?
            .as_float_slider()?
            .value() as f32;
        // Scale: 100% = 1.0 (UI percentage).
        let texture_scale_u = params
            .get(Params::TextureScaleU)?
            .as_float_slider()?
            .value() as f32
            / 100.0;
        let texture_scale_v = params
            .get(Params::TextureScaleV)?
            .as_float_slider()?
            .value() as f32
            / 100.0;
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

        let texture_flip_u = params.get(Params::TextureFlipU)?.as_checkbox()?.value();
        let texture_flip_v = params.get(Params::TextureFlipV)?.as_checkbox()?.value();

        let tex_world_type = texture_layer.world_type();
        let out_world_type = out_layer.world_type();

        let tex_w = texture_layer.width();
        let tex_h = texture_layer.height();
        let (uv_w, uv_h, uv_world_type) = match uv_layer {
            Some(uv) => (uv.width(), uv.height(), uv.world_type()),
            None => (0, 0, ae::aegp::WorldType::U8),
        };
        let (dist_w, dist_h, dist_world_type) = match distort_layer {
            Some(d) => (d.width(), d.height(), d.world_type()),
            None => (0, 0, ae::aegp::WorldType::U8),
        };

        let out_w = out_layer.width();
        let out_h = out_layer.height();

        out_layer.iterate(0, progress_final, None, |x, y, mut dst| {
            let x = x as usize;
            let y = y as usize;

            // 3D UV: V=0 bottom, V=1 top. AE has V=0 top so we use v_3d = 1.0 - v_ae when reading.
            // When no UV Map: use output position (3D convention); when UV Map and outside bounds: transparent.
            let (u_base, v_base, _) = match uv_layer {
                None => {
                    let u = if out_w > 0 {
                        x as f32 / out_w as f32
                    } else {
                        0.5
                    };
                    let v_3d = if out_h > 0 {
                        1.0 - y as f32 / out_h as f32
                    } else {
                        0.5
                    };
                    (u, v_3d, 1.0)
                }
                Some(uv_layer) => {
                    let (lx_uv, ly_uv, uv_inside) =
                        output_to_layer_coord(x, y, out_w, out_h, uv_w, uv_h, uv_fit);
                    if uv_inside {
                        let x_uv = (lx_uv as usize).min(uv_w.saturating_sub(1));
                        let y_uv = (ly_uv as usize).min(uv_h.saturating_sub(1));
                        let uv_px = read_pixel_f32(uv_layer, uv_world_type, x_uv, y_uv);
                        let u = uv_px.red;
                        let v_3d = 1.0 - uv_px.green;
                        (u, v_3d, uv_px.alpha)
                    } else {
                        // UV layer outside: do not sample texture; output transparent.
                        let transparent = PixelF32 {
                            red: 0.0,
                            green: 0.0,
                            blue: 0.0,
                            alpha: 0.0,
                        };
                        match out_world_type {
                            ae::aegp::WorldType::U8 => dst.set_from_u8(transparent.to_pixel8()),
                            ae::aegp::WorldType::U15 => dst.set_from_u16(transparent.to_pixel16()),
                            ae::aegp::WorldType::F32 | ae::aegp::WorldType::None => {
                                dst.set_from_f32(transparent);
                            }
                        }
                        return Ok(());
                    }
                }
            };

            // Displacement: when both X and Y are 0 or no layer, use constant 0.5 (no displacement).
            let l = if displacement_disabled {
                0.5
            } else {
                match distort_layer {
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
                }
            };

            // UV Map pivot (3D): origin → scale → offset → restore. Then displacement.
            let u_uv = (u_base - uv_map_center_u) / uv_map_scale - u_offset + uv_map_center_u;
            let v_uv = (v_base - uv_map_center_v) / uv_map_scale - v_offset + uv_map_center_v;
            let u_final = u_uv + (l - 0.5) * intensity_x;
            let v_final = v_uv + (l - 0.5) * intensity_y;

            let u_wrapped = wrap_coord(u_final, wrap_mode);
            let v_wrapped = wrap_coord(v_final, wrap_mode);

            // UV map alpha at transformed coords (0..1): trim by shape; outside 0..1 = 0.
            let uv_alpha = match uv_layer {
                None => 1.0,
                Some(uv_layer) => {
                    if (0.0..=1.0).contains(&u_wrapped) && (0.0..=1.0).contains(&v_wrapped) {
                        sample_layer_alpha_at_normalized(
                            uv_layer,
                            uv_world_type,
                            uv_w,
                            uv_h,
                            u_wrapped,
                            v_wrapped,
                        )
                    } else {
                        0.0
                    }
                }
            };

            // Texture pivot (3D): same formula. Then V flip for sampling (3D → AE: v_samp = 1 - v_3d).
            let u_scaled = wrap_coord(
                (u_wrapped - texture_center_x) / texture_scale_u - texture_offset_u
                    + texture_center_x,
                wrap_mode,
            );
            let v_scaled = wrap_coord(
                (v_wrapped - texture_center_y) / texture_scale_v - texture_offset_v
                    + texture_center_y,
                wrap_mode,
            );
            let v_for_sampling = 1.0 - v_scaled;

            // Texture sampling: use (u_scaled, v_for_sampling). Center mode = letterbox.
            let (u_tex, v_tex) = match texture_fit {
                LayerFit::Stretch => (u_scaled, v_for_sampling),
                LayerFit::Center => {
                    let out_w_f = out_w as f32;
                    let out_h_f = out_h as f32;
                    let tw_f = tex_w as f32;
                    let th_f = tex_h as f32;
                    let u_tex = u_scaled * out_w_f / tw_f - out_w_f / (2.0 * tw_f) + 0.5;
                    let v_tex = v_for_sampling * out_h_f / th_f - out_h_f / (2.0 * th_f) + 0.5;
                    (u_tex, v_tex)
                }
            };
            let u_tex = if texture_flip_u { 1.0 - u_tex } else { u_tex };
            let v_tex = if texture_flip_v { 1.0 - v_tex } else { v_tex };

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
                    alpha_edges_threshold,
                    texture_edge_mode,
                )
            };

            // Alpha Edges Threshold: alpha below this is fully transparent (reduces alpha edge lines).
            if tex_px.alpha < alpha_edges_threshold {
                tex_px.red = 0.0;
                tex_px.green = 0.0;
                tex_px.blue = 0.0;
                tex_px.alpha = 0.0;
            }

            // Final alpha = texture alpha × UV map alpha (UV alpha trims the result).
            let a = tex_px.alpha * uv_alpha;
            // Output premultiplied alpha so the host composites correctly.
            let out_px = PixelF32 {
                red: tex_px.red * a,
                green: tex_px.green * a,
                blue: tex_px.blue * a,
                alpha: a,
            };

            // Write to output with correct bit depth.
            match out_world_type {
                ae::aegp::WorldType::U8 => dst.set_from_u8(out_px.to_pixel8()),
                ae::aegp::WorldType::U15 => dst.set_from_u16(out_px.to_pixel16()),
                ae::aegp::WorldType::F32 | ae::aegp::WorldType::None => {
                    dst.set_from_f32(out_px);
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

/// Sample a layer's alpha at normalized (u, v) in 0..1. (u, v) are in 3D UV (V=0 bottom);
/// we convert to layer coords (V=0 top) with v_layer = 1 - v for sampling.
fn sample_layer_alpha_at_normalized(
    layer: &Layer,
    world_type: ae::aegp::WorldType,
    width: usize,
    height: usize,
    u: f32,
    v_3d: f32,
) -> f32 {
    if width == 0 || height == 0 {
        return 0.0;
    }
    let u = u.clamp(0.0, 1.0);
    let v_3d = v_3d.clamp(0.0, 1.0);
    let v_layer = 1.0 - v_3d;
    let fx = u * (width as f32 - 1.0);
    let fy = v_layer * (height as f32 - 1.0);
    let x0 = fx.floor() as isize;
    let y0 = fy.floor() as isize;
    let x1 = (x0 + 1).min(width as isize - 1);
    let y1 = (y0 + 1).min(height as isize - 1);
    let sx = fx - x0 as f32;
    let sy = fy - y0 as f32;
    let a00 = read_pixel_f32(layer, world_type, x0 as usize, y0 as usize).alpha;
    let a10 = read_pixel_f32(layer, world_type, x1 as usize, y0 as usize).alpha;
    let a01 = read_pixel_f32(layer, world_type, x0 as usize, y1 as usize).alpha;
    let a11 = read_pixel_f32(layer, world_type, x1 as usize, y1 as usize).alpha;
    let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t;
    lerp(lerp(a00, a10, sx), lerp(a01, a11, sx), sy)
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

#[allow(clippy::too_many_arguments)]
fn sample_layer_f32(
    layer: &Layer,
    world_type: ae::aegp::WorldType,
    width: usize,
    height: usize,
    u: f32,
    v: f32,
    alpha_edges_threshold: f32,
    texture_edge_mode: TextureEdgeMode,
) -> PixelF32 {
    if width == 0 || height == 0 {
        return PixelF32 {
            red: 0.0,
            green: 0.0,
            blue: 0.0,
            alpha: 0.0,
        };
    }

    // Transparent: outside 0..1 returns transparent. Repeat Edge Pixels: clamp to 0..1 so edge extends.
    let (u, v) = match texture_edge_mode {
        TextureEdgeMode::Transparent => {
            if !(0.0..=1.0).contains(&u) || !(0.0..=1.0).contains(&v) {
                return PixelF32 {
                    red: 0.0,
                    green: 0.0,
                    blue: 0.0,
                    alpha: 0.0,
                };
            }
            (u, v)
        }
        TextureEdgeMode::RepeatEdgePixels => (u.clamp(0.0, 1.0), v.clamp(0.0, 1.0)),
    };

    let fx = (u * (width as f32 - 1.0)).max(0.0);
    let fy = (v * (height as f32 - 1.0)).max(0.0);

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

    // If any of the 4 samples has alpha below threshold (alpha edge), force transparent to reduce diagonal lines.
    let min_alpha = c00.alpha.min(c10.alpha).min(c01.alpha).min(c11.alpha);
    if min_alpha < alpha_edges_threshold {
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
