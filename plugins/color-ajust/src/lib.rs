use after_effects as ae;
use palette::{FromColor, Hsl, LinSrgb, Oklch, Srgb};
use std::env;
use utils::ToPixel;

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
enum Params {
    ColorSpace,      // Popup: OKLCH / HSL
    HueShift,        // deg
    ChromaScale,     // multiplier
    LightnessDelta,  // delta
    ClampToSRgb,     // bool
    FallbackPreview, // bool (将来プレビュー用のフック。現状は簡易オーバーレイ)
}

#[derive(Default)]
struct Plugin {}

ae::define_effect!(Plugin, (), Params);

const PLUGIN_DESCRIPTION: &str =
    "A plugin to modify the chroma and lightness of an image using various color space";

impl AdobePluginGlobal for Plugin {
    fn params_setup(
        &self,
        params: &mut ae::Parameters<Params>,
        _in_data: InData,
        _: OutData,
    ) -> Result<(), Error> {
        // param definitions here
        // Color Space (将来: LCh などを足す想定)
        params.add(
            Params::ColorSpace,
            "Color Space",
            ae::pf::PopupDef::setup(|d| {
                d.set_options(&["OKLCH", "HSL"]);
                d.set_default(1); // 1-based
            }),
        )?;

        // Hue Shift (degrees)
        params.add(
            Params::HueShift,
            "Hue Shift (deg)",
            ae::pf::FloatSliderDef::setup(|d| {
                d.set_valid_min(-180.0);
                d.set_valid_max(180.0);
                d.set_slider_min(-180.0);
                d.set_slider_max(180.0);
                d.set_default(0.0);
                d.set_precision(1);
            }),
        )?;

        // Chroma Scale (multiplier)
        params.add(
            Params::ChromaScale,
            "Chroma Scale",
            ae::pf::FloatSliderDef::setup(|d| {
                d.set_valid_min(0.0);
                d.set_valid_max(4.0);
                d.set_slider_min(0.0);
                d.set_slider_max(2.0);
                d.set_default(1.0);
                d.set_precision(3);
            }),
        )?;

        // Lightness Delta
        params.add(
            Params::LightnessDelta,
            "Lightness Delta",
            ae::pf::FloatSliderDef::setup(|d| {
                d.set_valid_min(-1.0);
                d.set_valid_max(1.0);
                d.set_slider_min(-0.5);
                d.set_slider_max(0.5);
                d.set_default(0.0);
                d.set_precision(3);
            }),
        )?;

        // Clamp to sRGB (0..1)
        params.add(
            Params::ClampToSRgb,
            "Clamp to sRGB",
            ae::pf::CheckBoxDef::setup(|d| {
                d.set_default(true);
            }),
        )?;

        // Fallback preview (将来: “fallback 対象” を認知させる UI/表示へ拡張する想定)
        params.add(
            Params::FallbackPreview,
            "Fallback Preview",
            ae::pf::CheckBoxDef::setup(|d| {
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
                out_data.set_return_msg(format!(
                    "AOD_ColorAjust - {version}\r\r{PLUGIN_DESCRIPTION}\rCopyright (c) 2026-{build_year} Aodaruma",
                    version=env!("CARGO_PKG_VERSION"),
                    build_year=env!("BUILD_YEAR")
                ).as_str());
            }
            ae::Command::GlobalSetup => {
                // Declare that we do or do not support smart rendering
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

                if in_layer_opt.is_some() && out_layer_opt.is_some() {
                    self.do_render(
                        in_data,
                        in_layer_opt.unwrap(),
                        out_data,
                        out_layer_opt.unwrap(),
                        params,
                    )?;
                }

                // self.do_render(in_data, in_layer_opt, out_data, out_layer_opt, params)?;

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
        let width = in_layer.width() as usize;
        let height = in_layer.height() as usize;
        let frame_num = in_data.current_frame() as usize;
        let _ = (width, height, frame_num);

        // ---- param fetch ----
        let color_space = params.get(Params::ColorSpace)?.as_popup()?.value() as i32; // 1-based

        let hue_shift_deg = params.get(Params::HueShift)?.as_float_slider()?.value() as f32;

        let chroma_scale = params.get(Params::ChromaScale)?.as_float_slider()?.value() as f32;

        let lightness_delta = params
            .get(Params::LightnessDelta)?
            .as_float_slider()?
            .value() as f32;

        let clamp_to_srgb = params.get(Params::ClampToSRgb)?.as_checkbox()?.value();

        let fallback_preview = params.get(Params::FallbackPreview)?.as_checkbox()?.value();
        let in_world_type = in_layer.world_type();
        let out_world_type = out_layer.world_type();

        // ---- small helpers ----
        #[inline]
        fn clamp01(x: f32) -> f32 {
            x.max(0.0).min(1.0)
        }

        // 将来ここを OCIO/任意カラースペースに差し替える想定:
        // - decode_input: “入力” を処理用 RGB へ
        // - encode_output: 処理用 RGB を “出力” へ
        let decode_input = |r: f32, g: f32, b: f32| Srgb::new(r, g, b);
        let encode_output = |srgb: Srgb<f32>| srgb;

        // OKLCH adjust
        let adjust_oklch = |srgb_in: Srgb<f32>| -> (Srgb<f32>, bool) {
            let mut fallback_used = false;

            let lin: LinSrgb<f32> = srgb_in.into_linear();
            let mut c: Oklch<f32> = Oklch::from_color(lin);

            // hue shift
            c.hue = c.hue + palette::hues::OklabHue::from_degrees(hue_shift_deg);

            // chroma scale
            c.chroma = (c.chroma * chroma_scale).max(0.0);

            // lightness delta (0..1)
            let l0 = c.l;
            c.l = clamp01(c.l + lightness_delta);
            if (c.l - l0).abs() > 0.0 && (l0 + lightness_delta != c.l) {
                fallback_used = true; // L をクランプした
            }

            let lin_out: LinSrgb<f32> = LinSrgb::from_color(c);
            let mut out: Srgb<f32> = Srgb::from_linear(lin_out);

            // NaN/Inf guard
            if !out.red.is_finite() || !out.green.is_finite() || !out.blue.is_finite() {
                return (srgb_in, true);
            }

            // gamut/fallback: out-of-range を検知（将来: gamut mapping へ差し替え想定）
            let out_of_range = out.red < 0.0
                || out.red > 1.0
                || out.green < 0.0
                || out.green > 1.0
                || out.blue < 0.0
                || out.blue > 1.0;

            if out_of_range {
                fallback_used = true;
                if clamp_to_srgb {
                    out.red = clamp01(out.red);
                    out.green = clamp01(out.green);
                    out.blue = clamp01(out.blue);
                }
            }

            (out, fallback_used)
        };

        // HSL adjust (chroma ≒ saturation として扱う)
        let adjust_hsl = |srgb_in: Srgb<f32>| -> (Srgb<f32>, bool) {
            let mut fallback_used = false;

            let lin: LinSrgb<f32> = srgb_in.into_linear();
            let mut c = Hsl::from_color(lin);

            c.hue = c.hue + palette::hues::RgbHue::from_degrees(hue_shift_deg);
            c.saturation = (c.saturation * chroma_scale).max(0.0);

            let l0 = c.lightness;
            c.lightness = clamp01(c.lightness + lightness_delta);
            if (c.lightness - l0).abs() > 0.0 && (l0 + lightness_delta != c.lightness) {
                fallback_used = true;
            }

            let lin_out: LinSrgb<f32> = LinSrgb::from_color(c);
            let mut out: Srgb<f32> = Srgb::from_linear(lin_out);

            if !out.red.is_finite() || !out.green.is_finite() || !out.blue.is_finite() {
                return (srgb_in, true);
            }

            let out_of_range = out.red < 0.0
                || out.red > 1.0
                || out.green < 0.0
                || out.green > 1.0
                || out.blue < 0.0
                || out.blue > 1.0;

            if out_of_range {
                fallback_used = true;
                if clamp_to_srgb {
                    out.red = clamp01(out.red);
                    out.green = clamp01(out.green);
                    out.blue = clamp01(out.blue);
                }
            }

            (out, fallback_used)
        };

        // ---- render ----
        // let area = Some(in_data.extent_hint());
        in_layer.iterate_with(
            &mut out_layer,
            0,
            progress_final,
            None,
            |x, y, in_px, mut out_px| {
                let p = match in_world_type {
                    ae::aegp::WorldType::U8 => in_px.as_u8().to_pixel32(),
                    ae::aegp::WorldType::U15 => in_px.as_u16().to_pixel32(),
                    ae::aegp::WorldType::F32 | ae::aegp::WorldType::None => in_px.as_f32(),
                };

                // 入力 decode（将来ここに OCIO 等を挟む余地）
                let srgb_in = decode_input(p.red, p.green, p.blue);

                let (srgb_adj, fallback_used) = match color_space {
                    2 => adjust_hsl(srgb_in),
                    _ => adjust_oklch(srgb_in), // default: OKLCH
                };

                // 出力 encode（将来ここに OCIO 等を挟む余地）
                let mut srgb_out = encode_output(srgb_adj);

                // fallback preview（簡易。将来は “対象ピクセル表示” の本実装に置き換え）
                if fallback_preview && fallback_used {
                    // うっすらマゼンタを乗せる
                    srgb_out.red = clamp01(srgb_out.red * 0.5 + 0.5);
                    srgb_out.green = clamp01(srgb_out.green * 0.5);
                    srgb_out.blue = clamp01(srgb_out.blue * 0.5 + 0.5);
                }

                let out_f32 = PixelF32 {
                    alpha: p.alpha,
                    red: srgb_out.red,
                    green: srgb_out.green,
                    blue: srgb_out.blue,
                };

                match out_world_type {
                    ae::aegp::WorldType::U8 => out_px.set_from_u8(out_f32.to_pixel8()),
                    ae::aegp::WorldType::U15 => out_px.set_from_u16(out_f32.to_pixel16()),
                    ae::aegp::WorldType::F32 | ae::aegp::WorldType::None => {
                        out_px.set_from_f32(out_f32);
                    }
                }

                // x,y は将来の “fallback 対象マップ生成” 等で使えるので残す
                let _ = (x, y);

                Ok(())
            },
        )?;

        Ok(())
    }
}
