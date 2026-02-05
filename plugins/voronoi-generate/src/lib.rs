#![allow(clippy::drop_non_drop, clippy::question_mark)]

use after_effects as ae;
use std::env;

#[cfg(feature = "gpu_wgpu")]
use std::sync::{Arc, OnceLock};

use ae::pf::*;
use utils::ToPixel;

#[cfg(feature = "gpu_wgpu")]
mod gpu;
#[cfg(feature = "gpu_wgpu")]
use crate::gpu::wgpu::{WgpuContext, WgpuRenderParams};
#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
enum Params {
    CellGroupStart,
    CellGroupEnd,
    DistanceGroupStart,
    DistanceGroupEnd,
    OutputGroupStart,
    OutputGroupEnd,
    CellSize,
    ScaleX,
    ScaleY,
    Randomness,
    Seed,
    DistanceMetric,
    LpExponent,
    Smoothness,
    OutputType,
    ScaleW,
    W,
    Offset,
    Clamp32,
    UseOriginalAlpha,
}

#[derive(Clone, Copy)]
enum DistanceMetric {
    Euclidean,
    Manhattan,
    Chebyshev,
    Lp,
}

#[derive(Clone, Copy)]
enum OutputType {
    Color,
    Position,
    F,
    Distance,
    Edge,
}

#[derive(Clone, Copy, Default)]
struct Site {
    x: f32,
    y: f32,
    w: f32,
    hash: u32,
}

#[derive(Default)]
struct Plugin {}

ae::define_effect!(Plugin, (), Params);

const PLUGIN_DESCRIPTION: &str = "Generates Voronoi texture maps";

#[cfg(feature = "gpu_wgpu")]
static WGPU_CONTEXT: OnceLock<Result<Arc<WgpuContext>, ()>> = OnceLock::new();

#[cfg(feature = "gpu_wgpu")]
fn wgpu_context() -> Option<Arc<WgpuContext>> {
    match WGPU_CONTEXT.get_or_init(|| WgpuContext::new().map(Arc::new).map_err(|_| ())) {
        Ok(ctx) => Some(ctx.clone()),
        Err(_) => None,
    }
}

impl AdobePluginGlobal for Plugin {
    fn params_setup(
        &self,
        params: &mut ae::Parameters<Params>,
        _in_data: InData,
        _: OutData,
    ) -> Result<(), Error> {
        params.add_group(
            Params::CellGroupStart,
            Params::CellGroupEnd,
            "Cell",
            false,
            |params| {
                params.add(
                    Params::CellSize,
                    "Cell Size (px)",
                    FloatSliderDef::setup(|d| {
                        d.set_valid_min(1.0);
                        d.set_valid_max(8192.0);
                        d.set_slider_min(4.0);
                        d.set_slider_max(512.0);
                        d.set_default(128.0);
                        d.set_precision(1);
                    }),
                )?;

                params.add(
                    Params::ScaleX,
                    "Scale X",
                    FloatSliderDef::setup(|d| {
                        d.set_valid_min(0.001);
                        d.set_valid_max(1000.0);
                        d.set_slider_min(0.1);
                        d.set_slider_max(10.0);
                        d.set_default(1.0);
                        d.set_precision(3);
                    }),
                )?;

                params.add(
                    Params::ScaleY,
                    "Scale Y",
                    FloatSliderDef::setup(|d| {
                        d.set_valid_min(0.001);
                        d.set_valid_max(1000.0);
                        d.set_slider_min(0.1);
                        d.set_slider_max(10.0);
                        d.set_default(1.0);
                        d.set_precision(3);
                    }),
                )?;

                params.add(
                    Params::Randomness,
                    "Randomness",
                    FloatSliderDef::setup(|d| {
                        d.set_valid_min(0.0);
                        d.set_valid_max(1.0);
                        d.set_slider_min(0.0);
                        d.set_slider_max(1.0);
                        d.set_default(1.0);
                        d.set_precision(3);
                    }),
                )?;

                params.add(
                    Params::Seed,
                    "Seed",
                    SliderDef::setup(|d| {
                        d.set_valid_min(0);
                        d.set_valid_max(10000);
                        d.set_slider_min(0);
                        d.set_slider_max(1000);
                        d.set_default(0);
                    }),
                )?;

                Ok(())
            },
        )?;

        params.add_group(
            Params::DistanceGroupStart,
            Params::DistanceGroupEnd,
            "Distance",
            false,
            |params| {
                params.add(
                    Params::DistanceMetric,
                    "Distance Metric",
                    PopupDef::setup(|d| {
                        d.set_options(&["Euclidean", "Manhattan", "Chebyshev", "Lp"]);
                        d.set_default(1);
                    }),
                )?;

                params.add(
                    Params::LpExponent,
                    "Lp Exponent",
                    FloatSliderDef::setup(|d| {
                        d.set_valid_min(0.1);
                        d.set_valid_max(16.0);
                        d.set_slider_min(0.5);
                        d.set_slider_max(8.0);
                        d.set_default(2.0);
                        d.set_precision(2);
                    }),
                )?;

                params.add(
                    Params::Smoothness,
                    "Smoothness",
                    FloatSliderDef::setup(|d| {
                        d.set_valid_min(0.0);
                        d.set_valid_max(1.0);
                        d.set_slider_min(0.0);
                        d.set_slider_max(1.0);
                        d.set_default(0.0);
                        d.set_precision(3);
                    }),
                )?;

                params.add(
                    Params::W,
                    "W",
                    FloatSliderDef::setup(|d| {
                        d.set_valid_min(-50000.0);
                        d.set_valid_max(50000.0);
                        d.set_slider_min(-1.0);
                        d.set_slider_max(1.0);
                        d.set_default(0.0);
                        d.set_precision(3);
                    }),
                )?;

                params.add(
                    Params::ScaleW,
                    "Scale W",
                    FloatSliderDef::setup(|d| {
                        d.set_valid_min(0.001);
                        d.set_valid_max(1000.0);
                        d.set_slider_min(0.1);
                        d.set_slider_max(10.0);
                        d.set_default(100.0);
                        d.set_precision(3);
                    }),
                )?;

                Ok(())
            },
        )?;

        params.add_group(
            Params::OutputGroupStart,
            Params::OutputGroupEnd,
            "Output",
            false,
            |params| {
                params.add(
                    Params::OutputType,
                    "Output",
                    PopupDef::setup(|d| {
                        d.set_options(&[
                            "Color",
                            "Position",
                            "F (Smooth F1)",
                            "Distance (F1)",
                            "Edge (F2 - F1)",
                        ]);
                        d.set_default(1);
                    }),
                )?;

                params.add(
                    Params::Offset,
                    "Offset",
                    PointDef::setup(|p| {
                        p.set_default((0.0, 0.0));
                    }),
                )?;

                params.add(
                    Params::Clamp32,
                    "Clamp (32bpc)",
                    CheckBoxDef::setup(|d| {
                        d.set_default(false);
                    }),
                )?;

                params.add(
                    Params::UseOriginalAlpha,
                    "Use Original Alpha",
                    CheckBoxDef::setup(|d| {
                        d.set_default(false);
                    }),
                )?;

                Ok(())
            },
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
                        "AOD_VoronoiGenerate - {version}\r\r{PLUGIN_DESCRIPTION}\rCopyright (c) 2026-{build_year} Aodaruma",
                        version = env!("CARGO_PKG_VERSION"),
                        build_year = env!("BUILD_YEAR")
                    )
                    .as_str(),
                );
            }
            ae::Command::GlobalSetup => {
                // Declare that we do or do not support smart rendering
                out_data.set_out_flag2(OutFlags2::SupportsSmartRender, true);
            }
            ae::Command::Render {
                in_layer,
                mut out_layer,
            } => {
                #[cfg(feature = "gpu_wgpu")]
                {
                    if let Some(ctx) = wgpu_context()
                        && self
                            .do_render_wgpu(in_data, &in_layer, &mut out_layer, params, &ctx)
                            .is_ok()
                    {
                        return Ok(());
                    }
                }
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
            ae::Command::UserChangedParam { param_index } => {
                if params.type_at(param_index) == Params::DistanceMetric {
                    out_data.set_out_flag(OutFlags::RefreshUi, true);
                }
            }
            ae::Command::UpdateParamsUi => {
                let mut params_copy = params.cloned();
                Self::update_params_ui(&mut params_copy)?;
            }
            _ => {}
        }
        Ok(())
    }
}

impl Plugin {
    fn update_params_ui(params: &mut Parameters<Params>) -> Result<(), Error> {
        let metric = params.get(Params::DistanceMetric)?.as_popup()?.value();
        let is_lp = metric == 4;
        Self::set_param_enabled(params, Params::LpExponent, is_lp)?;

        Ok(())
    }

    fn set_param_enabled(
        params: &mut ae::Parameters<Params>,
        id: Params,
        enabled: bool,
    ) -> Result<(), Error> {
        Self::set_param_ui_flag(params, id, ae::pf::ParamUIFlags::DISABLED, !enabled)
    }

    fn set_param_ui_flag(
        params: &mut ae::Parameters<Params>,
        id: Params,
        flag: ae::pf::ParamUIFlags,
        status: bool,
    ) -> Result<(), Error> {
        let mut p = params.get_mut(id)?;
        p.set_ui_flag(flag, status);
        p.update_param_ui()?;
        Ok(())
    }

    #[cfg(feature = "gpu_wgpu")]
    fn do_render_wgpu(
        &self,
        _in_data: InData,
        in_layer: &Layer,
        out_layer: &mut Layer,
        params: &mut Parameters<Params>,
        ctx: &WgpuContext,
    ) -> Result<(), Error> {
        let out_w = out_layer.width();
        let out_h = out_layer.height();
        if out_w == 0 || out_h == 0 {
            return Ok(());
        }

        let out_world_type = out_layer.world_type();
        let out_is_f32 = matches!(
            out_world_type,
            ae::aegp::WorldType::F32 | ae::aegp::WorldType::None
        );
        let in_world_type = in_layer.world_type();

        // --- read params ---
        let cell_size = params.get(Params::CellSize)?.as_float_slider()?.value() as f32;
        let cell_size = cell_size.max(1.0e-3);
        let scale_x = params.get(Params::ScaleX)?.as_float_slider()?.value() as f32;
        let scale_y = params.get(Params::ScaleY)?.as_float_slider()?.value() as f32;
        let scale_w = params.get(Params::ScaleW)?.as_float_slider()?.value() as f32;
        let scale_x = scale_x.max(1.0e-3);
        let scale_y = scale_y.max(1.0e-3);
        let scale_w = scale_w.max(1.0e-3);
        let inv_cell_x = scale_x / cell_size;
        let inv_cell_y = scale_y / cell_size;
        let inv_cell_w = scale_w / cell_size;

        let randomness = params.get(Params::Randomness)?.as_float_slider()?.value() as f32;
        let randomness = randomness.clamp(0.0, 1.0);

        let seed = params.get(Params::Seed)?.as_slider()?.value() as u32;

        let distance_metric = match params.get(Params::DistanceMetric)?.as_popup()?.value() {
            2 => 1,
            3 => 2,
            4 => 3,
            _ => 0,
        };

        let lp_exp = params.get(Params::LpExponent)?.as_float_slider()?.value() as f32;
        let lp_exp = lp_exp.max(0.1);

        let smoothness = params.get(Params::Smoothness)?.as_float_slider()?.value() as f32;
        let smoothness = smoothness.clamp(0.0, 1.0);

        let output_type = match params.get(Params::OutputType)?.as_popup()?.value() {
            2 => 1,
            3 => 2,
            4 => 3,
            5 => 4,
            _ => 0,
        };

        let w_value = params.get(Params::W)?.as_float_slider()?.value() as f32;
        let offset_param = params.get(Params::Offset)?;
        let offset_point = offset_param.as_point()?;
        let (offset_x, offset_y) = point_value_f32(&offset_point);
        let clamp_32 = params.get(Params::Clamp32)?.as_checkbox()?.value();
        let use_original_alpha = params.get(Params::UseOriginalAlpha)?.as_checkbox()?.value();

        let render_params = WgpuRenderParams {
            out_w: out_w as u32,
            out_h: out_h as u32,
            inv_cell_x,
            inv_cell_y,
            inv_cell_w,
            randomness,
            seed,
            distance_metric,
            lp_exp,
            smoothness,
            output_type,
            w_value,
            offset_x,
            offset_y,
        };

        let output = ctx.render(&render_params)?;
        if output.data.is_empty() {
            return Ok(());
        }

        out_layer.iterate(0, out_h as i32, None, |x, y, mut dst| {
            let idx = (y as usize * out_w + x as usize) * 4;
            let mut r = sanitize_value(output.data[idx], out_is_f32, clamp_32);
            let mut g = sanitize_value(output.data[idx + 1], out_is_f32, clamp_32);
            let mut b = sanitize_value(output.data[idx + 2], out_is_f32, clamp_32);

            let a = if use_original_alpha {
                let mut out_alpha =
                    read_pixel_f32(in_layer, in_world_type, x as usize, y as usize).alpha;
                if !out_alpha.is_finite() {
                    out_alpha = 0.0;
                }
                out_alpha = out_alpha.clamp(0.0, 1.0);
                r *= out_alpha;
                g *= out_alpha;
                b *= out_alpha;
                out_alpha
            } else {
                1.0
            };

            let out_px = PixelF32 {
                alpha: a,
                red: r,
                green: g,
                blue: b,
            };

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

    fn do_render(
        &self,
        _in_data: InData,
        in_layer: Layer,
        _out_data: OutData,
        mut out_layer: Layer,
        params: &mut Parameters<Params>,
    ) -> Result<(), Error> {
        let w = out_layer.width();
        let h = out_layer.height();
        let progress_final = h as i32;

        let out_world_type = out_layer.world_type();
        let out_is_f32 = matches!(
            out_world_type,
            ae::aegp::WorldType::F32 | ae::aegp::WorldType::None
        );
        let in_world_type = in_layer.world_type();

        // --- read params ---
        let cell_size = params.get(Params::CellSize)?.as_float_slider()?.value() as f32;
        let cell_size = cell_size.max(1.0e-3);
        let scale_x = params.get(Params::ScaleX)?.as_float_slider()?.value() as f32;
        let scale_y = params.get(Params::ScaleY)?.as_float_slider()?.value() as f32;
        let scale_w = params.get(Params::ScaleW)?.as_float_slider()?.value() as f32;
        let scale_x = scale_x.max(1.0e-3);
        let scale_y = scale_y.max(1.0e-3);
        let scale_w = scale_w.max(1.0e-3);
        let inv_cell_x = scale_x / cell_size;
        let inv_cell_y = scale_y / cell_size;
        let inv_cell_w = scale_w / cell_size;

        let randomness = params.get(Params::Randomness)?.as_float_slider()?.value() as f32;
        let randomness = randomness.clamp(0.0, 1.0);

        let seed = params.get(Params::Seed)?.as_slider()?.value() as u32;

        let distance_metric = match params.get(Params::DistanceMetric)?.as_popup()?.value() {
            2 => DistanceMetric::Manhattan,
            3 => DistanceMetric::Chebyshev,
            4 => DistanceMetric::Lp,
            _ => DistanceMetric::Euclidean,
        };

        let lp_exp = params.get(Params::LpExponent)?.as_float_slider()?.value() as f32;
        let lp_exp = lp_exp.max(0.1);

        let smoothness = params.get(Params::Smoothness)?.as_float_slider()?.value() as f32;
        let smoothness = smoothness.clamp(0.0, 1.0);

        let output_type = match params.get(Params::OutputType)?.as_popup()?.value() {
            2 => OutputType::Position,
            3 => OutputType::F,
            4 => OutputType::Distance,
            5 => OutputType::Edge,
            _ => OutputType::Color,
        };

        let w_value = params.get(Params::W)?.as_float_slider()?.value() as f32;
        let offset_param = params.get(Params::Offset)?;
        let offset_point = offset_param.as_point()?;
        let (offset_x, offset_y) = point_value_f32(&offset_point);

        let clamp_32 = params.get(Params::Clamp32)?.as_checkbox()?.value();
        let use_original_alpha = params.get(Params::UseOriginalAlpha)?.as_checkbox()?.value();

        let grid_w = (w as f32) * inv_cell_x;
        let grid_h = (h as f32) * inv_cell_y;
        let grid_w = grid_w.max(1.0e-6);
        let grid_h = grid_h.max(1.0e-6);

        out_layer.iterate(0, progress_final, None, |x, y, mut dst| {
            let px = (x as f32 + 0.5 - offset_x) * inv_cell_x;
            let py = (y as f32 + 0.5 - offset_y) * inv_cell_y;
            let pw = w_value * inv_cell_w;
            let cell_x = px.floor() as i32;
            let cell_y = py.floor() as i32;
            let cell_w = pw.floor() as i32;

            let mut d1 = f32::INFINITY;
            let mut d2 = f32::INFINITY;
            let mut nearest = Site::default();
            let mut second = Site::default();

            for nw in (cell_w - 1)..=(cell_w + 1) {
                for ny in (cell_y - 1)..=(cell_y + 1) {
                    for nx in (cell_x - 1)..=(cell_x + 1) {
                        let site = cell_point(nx, ny, nw, randomness, seed);
                        let dx = px - site.x;
                        let dy = py - site.y;
                        let dw = pw - site.w;
                        let d = metric_distance(dx, dy, dw, distance_metric, lp_exp);

                        if d < d1 {
                            d2 = d1;
                            second = nearest;
                            d1 = d;
                            nearest = site;
                        } else if d < d2 {
                            d2 = d;
                            second = site;
                        }
                    }
                }
            }

            if !d1.is_finite() {
                d1 = 0.0;
            }
            if !d2.is_finite() {
                d2 = d1;
                second = nearest;
            }

            let blend = smooth_blend(d1, d2, smoothness);

            let mut out_px = match output_type {
                OutputType::Color => {
                    let (r1, g1, b1) = hash_color(nearest.hash);
                    let (r2, g2, b2) = hash_color(second.hash);
                    let r = lerp(r1, r2, blend);
                    let g = lerp(g1, g2, blend);
                    let b = lerp(b1, b2, blend);
                    PixelF32 {
                        alpha: 1.0,
                        red: r,
                        green: g,
                        blue: b,
                    }
                }
                OutputType::Position => {
                    let mut r = nearest.x / grid_w;
                    let mut g = nearest.y / grid_h;
                    let mut b = 0.0;

                    r = sanitize_value(r, out_is_f32, clamp_32);
                    g = sanitize_value(g, out_is_f32, clamp_32);
                    b = sanitize_value(b, out_is_f32, clamp_32);

                    PixelF32 {
                        alpha: 1.0,
                        red: r,
                        green: g,
                        blue: b,
                    }
                }
                OutputType::F => {
                    let mut v = lerp(d1, d2, blend);
                    v = sanitize_value(v, out_is_f32, clamp_32);
                    PixelF32 {
                        alpha: 1.0,
                        red: v,
                        green: v,
                        blue: v,
                    }
                }
                OutputType::Distance => {
                    let v = sanitize_value(d1, out_is_f32, clamp_32);
                    PixelF32 {
                        alpha: 1.0,
                        red: v,
                        green: v,
                        blue: v,
                    }
                }
                OutputType::Edge => {
                    let mut v = (d2 - d1).max(0.0);
                    v = sanitize_value(v, out_is_f32, clamp_32);
                    PixelF32 {
                        alpha: 1.0,
                        red: v,
                        green: v,
                        blue: v,
                    }
                }
            };

            if use_original_alpha {
                let mut out_alpha =
                    read_pixel_f32(&in_layer, in_world_type, x as usize, y as usize).alpha;
                if !out_alpha.is_finite() {
                    out_alpha = 0.0;
                }
                out_alpha = out_alpha.clamp(0.0, 1.0);
                out_px.red *= out_alpha;
                out_px.green *= out_alpha;
                out_px.blue *= out_alpha;
                out_px.alpha = out_alpha;
            }

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

fn point_value_f32(point: &PointDef<'_>) -> (f32, f32) {
    match point.float_value() {
        Ok(p) => (p.x as f32, p.y as f32),
        Err(_) => point.value(),
    }
}

// --- voronoi helpers ---
fn metric_distance(dx: f32, dy: f32, dw: f32, metric: DistanceMetric, lp_exp: f32) -> f32 {
    match metric {
        DistanceMetric::Euclidean => (dx * dx + dy * dy + dw * dw).sqrt(),
        DistanceMetric::Manhattan => dx.abs() + dy.abs() + dw.abs(),
        DistanceMetric::Chebyshev => dx.abs().max(dy.abs()).max(dw.abs()),
        DistanceMetric::Lp => {
            let p = lp_exp.max(0.1);
            let s = dx.abs().powf(p) + dy.abs().powf(p) + dw.abs().powf(p);
            s.powf(1.0 / p)
        }
    }
}

fn cell_point(cell_x: i32, cell_y: i32, cell_w: i32, randomness: f32, seed: u32) -> Site {
    let h = hash3(cell_x, cell_y, cell_w, seed);
    let rx = rand01(hash_u32(h ^ 0xA511_E9B3));
    let ry = rand01(hash_u32(h ^ 0x63D8_3595));
    let ox = 0.5 + (rx - 0.5) * randomness;
    let oy = 0.5 + (ry - 0.5) * randomness;
    let rw = rand01(hash_u32(h ^ 0x1F1D_8E33));
    let ow = 0.5 + (rw - 0.5) * randomness;
    Site {
        x: cell_x as f32 + ox,
        y: cell_y as f32 + oy,
        w: cell_w as f32 + ow,
        hash: h,
    }
}

fn hash_color(h: u32) -> (f32, f32, f32) {
    let r = rand01(hash_u32(h ^ 0xB529_7A4D));
    let g = rand01(hash_u32(h ^ 0x68E3_1DA4));
    let b = rand01(hash_u32(h ^ 0x1B56_C4E9));
    (r, g, b)
}

fn hash3(x: i32, y: i32, w: i32, seed: u32) -> u32 {
    let mut h = seed ^ 0x9E37_79B9;
    h = h.wrapping_add((x as u32).wrapping_mul(0x85EB_CA6B));
    h = h.wrapping_add((y as u32).wrapping_mul(0xC2B2_AE35));
    h = h.wrapping_add((w as u32).wrapping_mul(0x27D4_EB2D));
    hash_u32(h)
}

fn hash_u32(mut x: u32) -> u32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x7FEB_352D);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846C_A68B);
    x ^= x >> 16;
    x
}

fn rand01(h: u32) -> f32 {
    h as f32 / u32::MAX as f32
}

fn smooth_blend(d1: f32, d2: f32, smoothness: f32) -> f32 {
    if smoothness <= 0.0 || !d1.is_finite() || !d2.is_finite() {
        return 0.0;
    }
    let t = ((d2 - d1) / smoothness).clamp(0.0, 1.0);
    0.5 * (1.0 - smoothstep01(t))
}

fn smoothstep01(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn sanitize_value(mut v: f32, out_is_f32: bool, clamp_32: bool) -> f32 {
    if !v.is_finite() {
        v = 0.0;
    }
    if out_is_f32 {
        if clamp_32 {
            v = v.clamp(0.0, 1.0);
        }
    } else {
        v = v.clamp(0.0, 1.0);
    }
    v
}

// --- pixel helpers ---
fn read_pixel_f32(layer: &Layer, world_type: ae::aegp::WorldType, x: usize, y: usize) -> PixelF32 {
    match world_type {
        ae::aegp::WorldType::U8 => layer.as_pixel8(x, y).to_pixel32(),
        ae::aegp::WorldType::U15 => layer.as_pixel16(x, y).to_pixel32(),
        ae::aegp::WorldType::F32 | ae::aegp::WorldType::None => *layer.as_pixel32(x, y),
    }
}
