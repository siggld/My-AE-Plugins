# island-id-color ( AOD_IslandIdColor )

Tracks colored regions as islands and applies per-island gradients or temp colors.

This is the After Effects plugin **AOD_IslandIdColor**, which provides the **IslandIdColor.aex** plugin file for Adobe After Effects.

## Building the Plugin

See the [main README](../../README.md) for instructions on how to build the plugin.

**Windows:** Use **Visual Studio 2022** (Community or Build Tools with C++ workload) for building. This matches GitHub CI (windows-latest). Other MSVC versions may produce binaries that crash AE on startup; in that case use the pre-built `.aex` from [GitHub Releases](https://github.com/Aodaruma/aodaruma-ae-plugin/releases).

## UI layout (skeleton)

- **Output Mode**: Original / Extraction (Alpha) / Temp Color (Island ID) / Final Gradient
- **Color Extraction**: Invert, Extraction Count (1–8), Target Color + Color Range × 8, Choke/Spread
- **Island Tracking & Temp Colors**: Tracking Path (mask path), Show Temp Colors, Merge Island Count (4/8/16/32), Source/Target Temp Color × 32
- **Gradient Render**: Gradient Settings Count (4/8/16/32), per-slot Grad Type, Start/End Color, Angle, Bias, Offset, Noise Amount × 32; Invert Gradient Count + Invert Temp Color × 32

Dynamic visibility (UpdateParamsUi) for ExtractionCount, MergeIslandCount, GradientSettingsCount, and InvertGradCount is to be implemented in `Command::UpdateParamsUi`. Grouping (GROUP_START / GROUP_END) can be added when the after-effects crate exposes a Def for them.
