## Unreleased
* [Change] Limited source-path collection to mask names `path` and `path_[n]`, and profile taper collection to mask name `Curve`.
* [Change] Replaced the old side selector with `Normal Side` and added `CenterLine (%)` for one-sided normal-band convergence control.
* [UI] Updated path/profile labels to show required mask names and changed both taper-curve defaults to `0.5`.
* [Fix] Changed `CenterLine (%)` default to `50` and made `Simple Taper` / `Profile Taper` contract around the same centerline axis.
* [Fix] Made fractal evaluation follow the post-taper centerline band and restored endpoint fade-out in `Falloff Mode = Blur Amount`.
* [UI] Changed defaults to `Fractal Scale = 15`, `Fractal Tangent Scale = 5`, and `Fractal Complexity = 15`.
* [Change] Moved `Normal Side` above `Swap Tangent`, redefined `Profile Taper` around `Profile Amount`, and split end fade from `Path Blur Offset` with `Subtraction Alpha`.
* [Feature] Added `Edge Preserve` modes (`Preserve Source Edge Color` / `Alpha Mask Ghost` / `Hybrid`) with mode-specific controls.
* [UI] Grouped `Normal Range` through `Negative Blur Amount`, changed `View Mode` to `Final / NormalMat / TangentMat / Fractal / Taper`, and simplified AA to `Antialiasing Quality (Non / Low / High)`.
* [Change] Moved `Fractal Amount` into `Edge Preserve`, leaving the `Fractal` group for texture-shape controls only.
* [Change] Rebuilt tangent processing around a shared fractal matte that drives displacement, tangent blur, and ghost compositing.
* [Fix] Reduced detail loss between similar colors by replacing the dominant-sample shortcut with displaced-center-weighted accumulation.
* [Fix] Removed the old `Fast Box Blur` softening path in favor of the new edge-preserve pipeline.

## v0.2.0 (2026-03-23)
* [Feature] Added Profile group (ID 20-28) with separate profile-mask extraction, per-side scaling, link mode, invert-X modes, and normal-side swap.
* [Enhancement] Applied profile curve multiplier to source-path normal influence while preserving tangent-direction blur and hairpin ambiguity damping.

## v0.1.1 (2026-03-23)
* [Refactor] Switched blur sampling to local tangent-direction blur for smoother directional streaks with similar runtime cost.
* [Bugfix] Added branch ambiguity damping near hairpins/right-angle turns to reduce cross-bleed between overlapping path influence regions.

## v0.1.0 (2026-03-23)
* [Feature] Added initial Vector Curve Blur plugin with SmartRender/MFR flags, path-based tangent-normal sampling, taper controls, and slit-fractal modulation.
* [Docs] Added development instruction document and initial plugin metadata/build files.

