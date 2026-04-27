# ae_ui

Shared helper functions for Effect Controls Window (ECW) UI updates.

This crate is intentionally small and focused on reusable AE/Premiere UI operations:

- enabling `SendUpdateParamsUi` in `GlobalSetup`
- toggling `INVISIBLE` for Premiere
- toggling `Hidden` via AEGP streams for After Effects
- toggling `DISABLED` for both hosts

Plugin-specific parameter definitions and business logic should stay in each plugin crate.
