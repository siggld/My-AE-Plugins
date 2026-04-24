## v1.1.0 (2026-04-23)
* [Feature] Added `Dark Expand Threshold`, `Dark Expand Radius`, and `Preserve Matte Edge` so dark pixels inside the effect matte can be prefilled from the brightest nearby source pixel before Displace / Blur / Ghost.
* [Change] Routed the Displace / Blur / Ghost source sampling through a preprocessed buffer, keeping the new dark-pixel expansion as a non-blended color fill that runs ahead of the existing tangent pipeline.

## v1.0.0 (2026-04-23)
* [Change] Limited source-path collection to mask names `path` and `path_[n]`, and profile taper collection to mask name `Curve`.
* [Change] Replaced the old side selector with `Normal Side` and added `CenterLine (%)` for one-sided normal-band convergence control.
* [UI] Updated path/profile labels to show required mask names and changed both taper-curve defaults to `0.5`.
* [Fix] Changed `CenterLine (%)` default to `50` and made `Simple Taper` / `Profile Taper` contract around the same centerline axis.
* [Fix] Made fractal evaluation follow the post-taper centerline band and restored endpoint fade-out for the unified blur-amount-style falloff.
* [UI] Changed defaults to `Fractal Scale = 15`, `Fractal Tangent Scale = 5`, and `Fractal Complexity = 15`.
* [Change] Moved `Normal Side` above `Swap Tangent`, redefined `Profile Taper` around `Profile Amount`, and split end fade from `TangentOffset` with `Subtraction Alpha`.
* [UI] Grouped `Normal Range` through `Negative Blur Amount`, changed `View Mode` to `Final / NormalMat / TangentMat / Fractal / Taper`, and simplified AA to `Antialiasing Quality (Non / Low / High)`.
* [Change] Moved `Fractal Amount` into `Edge Preserve`, leaving the `Fractal` group for texture-shape controls only.
* [Change] Rebuilt tangent processing around a shared fractal matte that drives displacement, tangent blur, and ghost compositing.
* [Fix] Reduced detail loss between similar colors by replacing the dominant-sample shortcut with displaced-center-weighted accumulation.
* [Fix] Removed the old `Fast Box Blur` softening path in favor of the new edge-preserve pipeline.
* [UI] Renamed `Path Blur Amount` to `TangentAmount(+)`, `Negative Blur Amount` to `TangentAmount(-)`, moved `TangentOffset` under the tangent amount controls, and set `TangentAmount(-)` default to `0`.
* [Fix] Changed `NormalMat` / `TangentMat` to grayscale weight previews instead of UV-like color previews.
* [Fix] Made the final normal/tangent matte attenuate displacement, blur, and ghost together, and changed `TangentOffset` into a post-process tangent offset.
* [Change] Reworked displacement to use positive/negative tangent amounts symmetrically without `SplitTangentDirection`, so plus/minus controls stay independently available.
* [Change] Unified rendering around the `Blur Amount` style falloff path and a stage-based edge preserve pipeline.
* [Feature] Upgraded `Antialiasing Quality = High` from color-only sampling to effect-wide supersampling for matte and geometry evaluation.
* [Change] Simplified `Edge Preserve` to `Fractal Amount` / `Displace Multiplier` / `Blur Multiplier` / `Ghost Multiplier` / `Ghost Alpha`.
* [Change] Removed `Source Edge Hold`, `Ghost Alpha Hold`, `Hybrid Balance`, and `Edge Preserve Mode` in favor of a separate ghost pass plus final `Ghost Alpha` overlay.
* [Change] Removed `SplitTangentDirection` and `Falloff Mode` from the UI now that negative tangent amount is always explicit and falloff behavior is fixed to the blur-amount-style path.
* [Fix] Applied `NormalFalloff` to effect strength itself instead of a final original/effect lerp so boundary double-image artifacts are reduced.
* [Fix] Smoothed the Fractal-driven displacement mask per AA quality so strong displace settings produce fewer dot-like artifacts.
* [UI] Moved `Antialiasing Quality` above `Normal Side`, renamed `Edge Preserve` to `Master Intensity`, set `Blur Multiplier` default to `10`, and set `Ghost Alpha` default to `0`.
* [Change] Reordered the parameter list to match the new UI order, with known compatibility risk for existing AEP/project parameter indexing.
* [UI] Changed `Antialiasing Quality` default to `High`, renamed `Normal Band` to `NormalControls`, set `Add Color` defaults to `Opacity=100`, `Color=#808080`, and changed both color modes to `Overlay`.
* [Change] Replaced the old `Enable Taper` / `Enable Profile Curve (Curve)` toggles with a single `Taper Mode` popup: `Non` / `SimpleTaper` / `ProfileTaper(Curve)`.
* [Change] Removed `Profile Amount`, `Invert Profile`, and `Profile Min Width`; `ProfileTaper(Curve)` now uses the auto-detected `Curve` path shape directly with `NormalRange`.

## v0.2.0 (2026-03-23)
* [Feature] Added Profile group (ID 20-28) with separate profile-mask extraction, per-side scaling, link mode, invert-X modes, and normal-side swap.
* [Enhancement] Applied profile curve multiplier to source-path normal influence while preserving tangent-direction blur and hairpin ambiguity damping.

## v0.1.1 (2026-03-23)
* [Refactor] Switched blur sampling to local tangent-direction blur for smoother directional streaks with similar runtime cost.
* [Bugfix] Added branch ambiguity damping near hairpins/right-angle turns to reduce cross-bleed between overlapping path influence regions.

## v0.1.0 (2026-03-23)
* [Feature] Added initial Vector Curve Blur plugin with SmartRender/MFR flags, path-based tangent-normal sampling, taper controls, and slit-fractal modulation.
* [Docs] Added development instruction document and initial plugin metadata/build files.

