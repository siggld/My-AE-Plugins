# GitHub/CI で問題になりやすい項目チェックリスト

「GitHub でビルドしたらエラーが出た」対応として過去に修正した内容を、現在のコードで再確認するためのチェックリストです。Cursor 上ではエラーとして検出されにくく、CI や実機で表面化しやすい項目をまとめています。

---

## 1. PiPL（プラグイン情報リソース）

| 項目 | 確認内容 | 参照 |
|------|----------|------|
| **OutFlags / OutFlags2 を必ず含める** | build.rs の `Property::AE_Effect_Global_OutFlags` と `Property::AE_Effect_Global_OutFlags_2` が存在し、必要なフラグが含まれていること。変更時は **cargo clean 後に再ビルド**。 | PiPL (25, 31) エラー対策 |
| **Windows で BigEndian** | ルート Cargo.toml で `pipl = { path = "crates/pipl" }` を使用していること（crates/pipl は Windows でも BigEndian で PiPL を出力）。 | docs/pipl-windows-byteorder-fix.md |
| **AE_Reserved_Info / PIPL_AE_RESERVED** | red_noise / uv-distort-pro は `Property::AE_Reserved_Info(8)` で aeFL を出力。**island-id-color では AE_Reserved_Info を付けず** `println!("cargo:rustc-env=PIPL_AE_RESERVED=0")` のみ使用（aeFL を出すとエラー 20008 が出たため、従来どおり aeFL なしで運用）。 |
| **動的 UI を使う場合** | PiPL の OutFlags に `SendUpdateParamsUI` を設定し、**かつ** lib.rs の `GlobalSetup` で `out_data.set_out_flag(OutFlags::SendUpdateParamsUi, true)` を呼ぶこと。 | AGENTS.md 動的UIルール |

---

## 2. build.rs 共通

| 項目 | 確認内容 |
|------|----------|
| **CARGO_PKG_VERSION** | `major.minor.patch` の 3 要素であること。そうでないと `panic!` でビルド失敗。 |
| **BUILD_YEAR** | `chrono::Local::now().year()` で設定されていること。 |
| **PIPL_SUPPORT_URL** | island-id-color では `cargo:rustc-env=PIPL_SUPPORT_URL=...` を設定。他プラグインは `Property::AE_Effect_Support_URL` で設定。 |
| **AE_PiPL_Version** | `{ major: 2, minor: 0 }` で統一。 |
| **CodeWin64X86 / CodeMac*** | 各 OS 用の `EffectMain` エントリが定義されていること。 |

---

## 3. 名前・URL の一貫性

| 項目 | 確認内容 |
|------|----------|
| **AE_Effect_Match_Name** | 既存プラグインの Match Name を変更するとプロジェクト互換性が壊れる。変更は原則禁止。 |
| **AE_Effect_Support_URL** | リポジトリ URL を統一（例: `https://github.com/Aodaruma/aodaruma-ae-plugin`）。typo で `aod-AE-plugin` と `aodaruma-ae-plugin` が混在しないこと。 |

---

## 4. CI で実行されるチェック（ローカルで再確認）

| コマンド | 内容 |
|----------|------|
| `cargo fmt --all -- --check` | フォーマットが揃っていること。 |
| `cargo clippy --workspace` | 警告・エラーが出ないこと。 |
| `cargo test` | テストが通ること。 |

※ ローカルで実行する場合は、リポジトリルートで上記を実行。CI は main/dev への push および PR で実行。

---

## 5. Justfile / ビルドスクリプト

| 項目 | 確認内容 |
|------|----------|
| **CARGO_TARGET_DIR** | CI では `${{ github.workspace }}/target`。ローカルでは build_and_deploy.bat が `%LOCALAPPDATA%\AEPluginBuild` を設定可能。 |
| **Windows の release** | `cargo build -p {{CrateName}} --release` の後、`.dll` を `.aex` としてコピー。AdobePlugin.just の [windows] release を参照。 |
| **プラグイン名** | Justfile の PluginName / BinaryName が Cargo.toml の name と一致していること。 |

---

## 6. 実行時エラー（MEE_Plugins / MediaCore）が出る場合

- ローカルビルドの .aex で **起動時に** クラッシュする場合、**handle_command より前**（DLL ロード直後）で落ちている可能性が高い（ログが 1 行も出ないため）。
- 対策の候補: CRT 静的リンク（.cargo/config.toml の `crt-static`）、red_noise のみでビルドして環境要因かプラグイン要因かを切り分け。詳細は `docs/ENV_SETUP_REVIEW.md` の「GitHub を使わずにローカルで切り分け・対策を試す場合」を参照。

---

## チェック実施結果の記録例

- [ ] PiPL OutFlags/OutFlags2 と cargo clean 再ビルド
- [ ] pipl = path で crates/pipl 使用
- [ ] AE_Reserved_Info の有無・値の統一
- [ ] SendUpdateParamsUI（動的UI時）PiPL + GlobalSetup 両方
- [ ] BUILD_YEAR / バージョン 3 要素
- [ ] Support URL / Match Name の typo なし
- [ ] cargo fmt / clippy / test 通過
