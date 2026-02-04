use after_effects as ae;
use std::env;

use ae::pf::*;

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
enum Params {
    // Mobius coefficients (complex): a,b,c,d
    ARe,
    AIm,
    BRe,
    BIm,
    CRe,
    CIm,
    DRe,
    DIm,

    // Mapping controls
    UseLayerCenter,
    Center,
    ScalePx,

    // Outside-destination behavior
    Edge,
}

#[derive(Default)]
struct MobiusPlugin {}

ae::define_effect!(MobiusPlugin, (), Params);

const PLUGIN_DESCRIPTION: &str = "A plugin for applying Mobius transformation to layers.";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EdgeMode {
    Expand,
    Repeat,
    Mirror,
    Tile,
    None,
}

impl EdgeMode {
    fn from_popup_value(v: i32) -> Self {
        match v {
            1 => EdgeMode::Expand,
            2 => EdgeMode::Repeat,
            3 => EdgeMode::Mirror,
            4 => EdgeMode::Tile,
            _ => EdgeMode::None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct C64 {
    re: f64,
    im: f64,
}
impl C64 {
    fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }
    fn add(self, o: Self) -> Self {
        Self::new(self.re + o.re, self.im + o.im)
    }
    fn sub(self, o: Self) -> Self {
        Self::new(self.re - o.re, self.im - o.im)
    }
    fn mul(self, o: Self) -> Self {
        Self::new(
            self.re * o.re - self.im * o.im,
            self.re * o.im + self.im * o.re,
        )
    }
    fn norm2(self) -> f64 {
        self.re * self.re + self.im * self.im
    }
    fn div(self, o: Self) -> Option<Self> {
        let d = o.norm2();
        if d < 1e-18 {
            return None;
        }
        Some(Self::new(
            (self.re * o.re + self.im * o.im) / d,
            (self.im * o.re - self.re * o.im) / d,
        ))
    }
}

impl AdobePluginGlobal for MobiusPlugin {
    fn params_setup(
        &self,
        params: &mut ae::Parameters<Params>,
        _in_data: InData,
        _: OutData,
    ) -> Result<(), Error> {
        // a
        params.add(
            Params::ARe,
            "a.re",
            FloatSliderDef::setup(|p| {
                p.set_valid_min(-10.0);
                p.set_valid_max(10.0);
                p.set_slider_min(-10.0);
                p.set_slider_max(10.0);
                p.set_default(1.0);
                p.set_precision(4);
            }),
        )?;
        params.add(
            Params::AIm,
            "a.im",
            FloatSliderDef::setup(|p| {
                p.set_valid_min(-10.0);
                p.set_valid_max(10.0);
                p.set_slider_min(-10.0);
                p.set_slider_max(10.0);
                p.set_default(0.0);
                p.set_precision(4);
            }),
        )?;

        // b
        params.add(
            Params::BRe,
            "b.re",
            FloatSliderDef::setup(|p| {
                p.set_valid_min(-10.0);
                p.set_valid_max(10.0);
                p.set_slider_min(-10.0);
                p.set_slider_max(10.0);
                p.set_default(0.0);
                p.set_precision(4);
            }),
        )?;
        params.add(
            Params::BIm,
            "b.im",
            FloatSliderDef::setup(|p| {
                p.set_valid_min(-10.0);
                p.set_valid_max(10.0);
                p.set_slider_min(-10.0);
                p.set_slider_max(10.0);
                p.set_default(0.0);
                p.set_precision(4);
            }),
        )?;

        // c
        params.add(
            Params::CRe,
            "c.re",
            FloatSliderDef::setup(|p| {
                p.set_valid_min(-10.0);
                p.set_valid_max(10.0);
                p.set_slider_min(-10.0);
                p.set_slider_max(10.0);
                p.set_default(0.0);
                p.set_precision(4);
            }),
        )?;
        params.add(
            Params::CIm,
            "c.im",
            FloatSliderDef::setup(|p| {
                p.set_valid_min(-10.0);
                p.set_valid_max(10.0);
                p.set_slider_min(-10.0);
                p.set_slider_max(10.0);
                p.set_default(0.0);
                p.set_precision(4);
            }),
        )?;

        // d
        params.add(
            Params::DRe,
            "d.re",
            FloatSliderDef::setup(|p| {
                p.set_valid_min(-10.0);
                p.set_valid_max(10.0);
                p.set_slider_min(-10.0);
                p.set_slider_max(10.0);
                p.set_default(1.0);
                p.set_precision(4);
            }),
        )?;
        params.add(
            Params::DIm,
            "d.im",
            FloatSliderDef::setup(|p| {
                p.set_valid_min(-10.0);
                p.set_valid_max(10.0);
                p.set_slider_min(-10.0);
                p.set_slider_max(10.0);
                p.set_default(0.0);
                p.set_precision(4);
            }),
        )?;

        params.add(
            Params::UseLayerCenter,
            "Use Layer Center",
            CheckBoxDef::setup(|c| {
                c.set_default(true);
            }),
        )?;

        params.add(
            Params::Center,
            "Center",
            PointDef::setup(|p| {
                p.set_default((50.0, 50.0));
                p.set_restrict_bounds(true);
            }),
        )?;

        params.add(
            Params::ScalePx,
            "Scale (px, 0=auto)",
            FloatSliderDef::setup(|p| {
                p.set_valid_min(0.0);
                p.set_valid_max(20000.0);
                p.set_slider_min(0.0);
                p.set_slider_max(20000.0);
                p.set_default(0.0);
                p.set_precision(2);
            }),
        )?;

        params.add(
            Params::Edge,
            "Edge",
            PopupDef::setup(|d| {
                d.set_options(&["Expand", "Repeat", "Mirror", "Tile", "None"]);
                d.set_default(5);
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
                out_data.set_return_msg(format!(
                    "AOD_MobiusTransform - {version}\r\r{PLUGIN_DESCRIPTION}\rCopyright (c) 2026-{build_year} Aodaruma",
                    version=env!("CARGO_PKG_VERSION"),
                    build_year=env!("BUILD_YEAR")
                ).as_str());
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

impl MobiusPlugin {
    fn do_render(
        &self,
        _in_data: InData,
        in_layer: Layer,
        _out_data: OutData,
        mut out_layer: Layer,
        params: &mut Parameters<Params>,
    ) -> Result<(), Error> {
        let width = in_layer.width();
        let height = in_layer.height();
        let progress_final = height as i32;

        let a = C64::new(
            params.get(Params::ARe)?.as_float_slider()?.value() as f64,
            params.get(Params::AIm)?.as_float_slider()?.value() as f64,
        );
        let b = C64::new(
            params.get(Params::BRe)?.as_float_slider()?.value() as f64,
            params.get(Params::BIm)?.as_float_slider()?.value() as f64,
        );
        let c = C64::new(
            params.get(Params::CRe)?.as_float_slider()?.value() as f64,
            params.get(Params::CIm)?.as_float_slider()?.value() as f64,
        );
        let d = C64::new(
            params.get(Params::DRe)?.as_float_slider()?.value() as f64,
            params.get(Params::DIm)?.as_float_slider()?.value() as f64,
        );

        let use_layer_center = params.get(Params::UseLayerCenter)?.as_checkbox()?.value();
        let (pcx, pcy) = params.get(Params::Center)?.as_point()?.value();
        let mut scale_px = params.get(Params::ScalePx)?.as_float_slider()?.value() as f64;
        let edge_mode =
            EdgeMode::from_popup_value(params.get(Params::Edge)?.as_popup()?.value() as i32);

        if scale_px <= 1e-9 {
            scale_px = (width.min(height) as f64 - 1.0) * 0.5;
        }

        let (cx, cy) = if use_layer_center {
            ((width as f64 - 1.0) * 0.5, (height as f64 - 1.0) * 0.5)
        } else {
            (pcx as f64, pcy as f64)
        };

        let out_depth = out_layer.bit_depth();

        // f(z) = (a z + b) / (c z + d)
        // inverse: z = (d w - b) / (-c w + a)
        in_layer.iterate_with(
            &mut out_layer,
            0,
            progress_final,
            None,
            |x, y, _in_px, mut out_px| {
                let wx = (x as f64 - cx) / scale_px;
                let wy = (y as f64 - cy) / scale_px;
                let w = C64::new(wx, wy);

                let num = d.mul(w).sub(b);
                let den = C64::new(-c.re, -c.im).mul(w).add(a);

                let Some(z) = num.div(den) else {
                    return Self::write_transparent(&mut out_px, out_depth);
                };

                let sx = z.re * scale_px + cx;
                let sy = z.im * scale_px + cy;

                if let Some(p) = Self::sample_bilinear_edge_f32(&in_layer, sx, sy, edge_mode) {
                    Self::write_f32(&mut out_px, out_depth, p)?;
                } else {
                    Self::write_transparent(&mut out_px, out_depth)?;
                }
                Ok(())
            },
        )?;

        Ok(())
    }

    fn write_transparent(out_px: &mut GenericPixelMut<'_>, depth: i16) -> Result<(), Error> {
        Self::write_f32(
            out_px,
            depth,
            PixelF32 {
                alpha: 0.0,
                red: 0.0,
                green: 0.0,
                blue: 0.0,
            },
        )
    }

    fn write_f32(out_px: &mut GenericPixelMut<'_>, depth: i16, p: PixelF32) -> Result<(), Error> {
        fn clamp01(v: f32) -> f32 {
            v.max(0.0).min(1.0)
        }
        match depth {
            8 => {
                let to_u8 = |v: f32| (clamp01(v) * 255.0 + 0.5) as u8;
                out_px.set_from_u8(Pixel8 {
                    alpha: to_u8(p.alpha),
                    red: to_u8(p.red),
                    green: to_u8(p.green),
                    blue: to_u8(p.blue),
                });
                Ok(())
            }
            16 => {
                let to_u16 = |v: f32| (clamp01(v) * 65535.0 + 0.5) as u16;
                out_px.set_from_u16(Pixel16 {
                    alpha: to_u16(p.alpha),
                    red: to_u16(p.red),
                    green: to_u16(p.green),
                    blue: to_u16(p.blue),
                });
                Ok(())
            }
            _ => {
                out_px.set_from_f32(p);
                Ok(())
            }
        }
    }

    fn read_f32(layer: &Layer, x: usize, y: usize) -> PixelF32 {
        match layer.bit_depth() {
            8 => {
                let p = layer.as_pixel8(x, y);
                PixelF32 {
                    alpha: p.alpha as f32 / 255.0,
                    red: p.red as f32 / 255.0,
                    green: p.green as f32 / 255.0,
                    blue: p.blue as f32 / 255.0,
                }
            }
            16 => {
                let p = layer.as_pixel16(x, y);
                PixelF32 {
                    alpha: p.alpha as f32 / 65535.0,
                    red: p.red as f32 / 65535.0,
                    green: p.green as f32 / 65535.0,
                    blue: p.blue as f32 / 65535.0,
                }
            }
            _ => *layer.as_pixel32(x, y),
        }
    }

    fn lerp(a: f32, b: f32, t: f32) -> f32 {
        a + (b - a) * t
    }

    fn lerp_px(a: PixelF32, b: PixelF32, t: f32) -> PixelF32 {
        PixelF32 {
            alpha: Self::lerp(a.alpha, b.alpha, t),
            red: Self::lerp(a.red, b.red, t),
            green: Self::lerp(a.green, b.green, t),
            blue: Self::lerp(a.blue, b.blue, t),
        }
    }

    fn sample_bilinear_f32(layer: &Layer, x: f64, y: f64) -> Option<PixelF32> {
        let w = layer.width() as i32;
        let h = layer.height() as i32;
        if w <= 0 || h <= 0 {
            return None;
        }

        if x < 0.0 || y < 0.0 || x > (w - 1) as f64 || y > (h - 1) as f64 {
            return None;
        }

        let x0 = x.floor() as i32;
        let y0 = y.floor() as i32;
        let x1 = (x0 + 1).min(w - 1);
        let y1 = (y0 + 1).min(h - 1);

        let tx = (x - x0 as f64) as f32;
        let ty = (y - y0 as f64) as f32;

        let p00 = Self::read_f32(layer, x0 as usize, y0 as usize);
        let p10 = Self::read_f32(layer, x1 as usize, y0 as usize);
        let p01 = Self::read_f32(layer, x0 as usize, y1 as usize);
        let p11 = Self::read_f32(layer, x1 as usize, y1 as usize);

        let a = Self::lerp_px(p00, p10, tx);
        let b = Self::lerp_px(p01, p11, tx);
        Some(Self::lerp_px(a, b, ty))
    }

    fn sample_bilinear_edge_f32(layer: &Layer, x: f64, y: f64, edge: EdgeMode) -> Option<PixelF32> {
        let w = layer.width() as i32;
        let h = layer.height() as i32;
        if w <= 0 || h <= 0 {
            return None;
        }

        let max_x = (w - 1) as f64;
        let max_y = (h - 1) as f64;
        let in_bounds = x >= 0.0 && y >= 0.0 && x <= max_x && y <= max_y;
        if in_bounds {
            return Self::sample_bilinear_f32(layer, x, y);
        }

        match edge {
            EdgeMode::None => None,
            EdgeMode::Expand => {
                let cx = x.clamp(0.0, max_x);
                let cy = y.clamp(0.0, max_y);
                Self::sample_bilinear_f32(layer, cx, cy)
            }
            EdgeMode::Repeat | EdgeMode::Tile => {
                let cx = Self::wrap_coord(x, w);
                let cy = Self::wrap_coord(y, h);
                Self::sample_bilinear_f32(layer, cx, cy)
            }
            EdgeMode::Mirror => {
                let cx = Self::mirror_coord(x, w);
                let cy = Self::mirror_coord(y, h);
                Self::sample_bilinear_f32(layer, cx, cy)
            }
        }
    }

    fn wrap_coord(v: f64, size: i32) -> f64 {
        if size <= 0 {
            return 0.0;
        }
        let size_f = size as f64;
        let mut t = v % size_f;
        if t < 0.0 {
            t += size_f;
        }
        t
    }

    fn mirror_coord(v: f64, size: i32) -> f64 {
        if size <= 1 {
            return 0.0;
        }
        let max = (size - 1) as f64;
        let period = 2.0 * max;
        let mut t = v % period;
        if t < 0.0 {
            t += period;
        }
        if t > max {
            t = period - t;
        }
        t
    }
}
