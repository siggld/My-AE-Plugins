# aod-AE-plugin

このリポジトリは、Rust で書かれた Adobe After Effects 向けプラグインの Fork 版ワークスペースです。  
`AOD_` で始まるプラグインはオリジナル作者の制作物として `plugins_aod/` に隔離し、この Fork では `TKG_` で始まるプラグインを中心に管理します。

This repository is a forked Cargo workspace for Adobe After Effects plugins written in Rust.  
Plugins prefixed with `AOD_` are isolated under `plugins_aod/` as original author works, while this fork focuses on `TKG_` plugins.

## 1. Plugins / プラグイン説明

> [!TIP]
> この Fork の公開・配布状況はリポジトリ運用方針に従ってください。  
> Follow this fork's own release policy for distribution status.

- TKG_BoundaryColorSaturation（制作中）
  - 境界色抽出と彩度制御のための基礎エフェクトです / Base effect for boundary color extraction and saturation control.
- TKG_IslandIdColor（制作中）
  - 領域ごとにIDや色情報を扱うためのエフェクトです / Effect for handling per-region IDs and color information.
- TKG_PathLineHatching（制作中）
  - パス方向を利用した線ハッチ表現を生成します / Generates line hatching based on path direction.
- TKG_VectorCurveBlur（制作中）
  - ベクトルや曲線情報を利用したブラー表現を検証中です / Work-in-progress blur effect using vector and curve information.
- TKG_CustomGUITest（制作中）
  - カスタムGUI実装の実機検証用サンドボックスです / Reusable sandbox for validating custom GUI implementations in AE.
- TKG_BoundaryColorSaturation
  - 境界色抽出とブラー制御のベースUIを構築します / Creates a base UI for boundary color extraction and blur controls.

## 2. Issue / バグ報告

もしバグを見つけた場合は、この Fork の Issue ページで報告してください。

If you find a bug, please report it on this fork's issue tracker.

## 3. Support / 支援

> [!NOTE]
> 支援・連絡先はこの Fork の運用者ポリシーに従ってください。  
> Follow this fork maintainer policy for support/contact.

## 4. License

ライセンスはMPL-2.0です。`LICENSE` ファイルを参照してください。

Licensed under the MPL-2.0. See `LICENSE`.

---

## 5. For Developers / 開発者向け情報

> [!NOTE]
> 以下は開発者向け情報です。利用のみの場合は上部のReleasesを参照してください（英語のみ）。
> 
> The following is for developers. If you only want to use the plugins, see the Releases section above.

### Build and install

Prerequisites:

- Rust toolchain and cargo
- cargo-generate
- just (recommended)
- **Windows:** Visual Studio 2022 (Community or Build Tools, C++ workload). See [Environment setup review](docs/ENV_SETUP_REVIEW.md) for details and how this differs from C/C++ AE plugin projects.

Build all plugins:

```sh
# for debug versions:
just build

# you can also build release versions:
just release
```

> [!WARNING]
> `just build` installs to the Adobe Common Plug-ins folder by default.  
> Skip installation with `NO_INSTALL=1 just build`.

By default the build installs to the Adobe Common Plug-ins folder. To skip installation:

```sh
NO_INSTALL=1 just build
```

Outputs:

- Windows: `target/debug/*.aex` or `target/release/*.aex`
- macOS: `target/debug/*.plugin` or `target/release/*.plugin`

You can also build a single plugin:

```sh
just -f plugins/boundary-color-saturation/Justfile build
```

### Create a new plugin

The repo includes a `cargo-generate` template:

```sh
cargo new-plugin

# or manually:
cargo generate --path templates/plugin --destination plugins
```

### Repository layout

- `plugins/`: `TKG_` 系の開発対象プラグイン / actively developed `TKG_` plugins
- `plugins_aod/`: `AOD_` 系の隔離済みプラグイン / isolated `AOD_` plugins
- `crates/utils/`: shared pixel conversion helpers
- `crates/ae_ui/`: shared Effect Controls Window UI helpers for AE/Premiere
- `templates/plugin/`: plugin template for `cargo-generate`
- `tester/`: sample After Effects project for manual testing

### Shared ECW UI policy

- Reusable Effect Controls Window UI logic should live in `crates/ae_ui/`.
- Plugin-specific parameters and render logic should remain in each plugin crate.
- If you need a GUI sandbox, keep it as a test plugin or docs, not as a separate runtime dependency root.
- See `docs/gui-architecture.md` for responsibilities and rollout steps.
- See `docs/gui-development-flow.md` for multi-agent development flow and handoff.

### Contribution

Issues and pull requests are welcome. Please keep `cargo fmt` and `cargo clippy` clean when possible.
