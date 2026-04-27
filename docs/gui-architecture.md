# Shared ECW GUI Architecture

This document defines how to share Effect Controls Window (ECW) UI behavior across plugins in this workspace.

## Goal

Create reusable UI behavior once, and consume it from multiple plugins without duplicating host-specific logic.

## Scope split

### Shared in `crates/ae_ui`

- `GlobalSetup` helper for `SendUpdateParamsUi`
- Premiere visibility updates via `ParamUIFlags::INVISIBLE`
- After Effects visibility updates via `DynamicStreamFlags::Hidden`
- Common enable/disable updates via `ParamUIFlags::DISABLED`

### Keep in each plugin

- Parameter enums and parameter order
- Plugin-specific visibility rules and business conditions
- Render and image processing logic
- Plugin naming, PiPL identity, and descriptions

## Why this structure

- Matches current workspace convention (`crates/*` for shared logic).
- Keeps plugin behavior explicit while removing repetitive host-branching code.
- Avoids making a separate non-workspace runtime folder as a dependency root.

## Current reference implementations

- Rich dynamic UI pattern:
  - `plugins/island-id-color/src/lib.rs`
- Lightweight dynamic UI pattern:
  - `plugins/vector-curve-blur/src/lib.rs`
  - `plugins/tkg-path-line-hatching/src/lib.rs`

## Rollout process

1. Start with one plugin and move only repeated UI update mechanics into `crates/ae_ui`.
2. Keep plugin-specific rule calculation local.
3. Expand to other plugins only after the first plugin API is stable.
4. Update template dependencies (`templates/plugin/Cargo.toml.liquid`) once usage is validated.

## Notes

- A standalone `gui-dev` folder is acceptable only as a sandbox plugin or docs location.
- The reusable runtime code should stay in `crates/ae_ui` so all plugins can reference it consistently.
- For day-to-day role split and execution steps, see `docs/gui-development-flow.md`.
