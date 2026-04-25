# island-id-color ( AOD_IslandIdColor )

Tracks colored regions as islands and applies per-island gradients or temp colors.

This is the After Effects plugin **AOD_IslandIdColor**, which provides the **IslandIdColor.aex** plugin file for Adobe After Effects.

## Building the Plugin

See the [main README](../../README.md) for instructions on how to build the plugin.

**Windows:** Use **Visual Studio 2022** (Community or Build Tools with C++ workload) for building. This matches GitHub CI (windows-latest). Other MSVC versions may produce binaries that crash AE on startup; in that case use the pre-built `.aex` from [GitHub Releases](https://github.com/Aodaruma/aodaruma-ae-plugin/releases).

## UI layout

- **Output Mode**: Original / Extraction (Alpha) / Temp Color (Island ID) / Final Gradient
- **Color Extraction**: Invert, Alpha threshold, Extraction Count (4 / 8 / 16 / 32), Enable + Target Color + Color Range for each active slot, Choke/Spread
- **Island Tracking & Temp Colors**: Tracking path, sort modes (including path-based sort), temp-color display options, merge count (4 / 8 / 16 / 32), tracking algorithm (color match / area-weighted / IoU), source/target temp colors and ranges per slot
- **Gradient Render**: gradient settings count (4 / 8 / 16 / 32), per-slot gradient type, colors, angle, center/bias/offset/noise, invert options per slot

Parameter visibility and Premiere vs After Effects flags are updated in `Command::UpdateParamsUi` via `update_params_ui_visibility`. Grouped sections use `add_group` for Color Extraction, Island Tracking, and Gradient Render.
