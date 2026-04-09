# TKG Path Line Hatching スケルトン

- [x] workspace-member
- [x] params-setup
- [x] global-setup-aegp
- [x] update-params-ui
- [x] smartfx-pipeline
- [x] render-skeleton
- [x] verification

# AE実行時エラー・警告修正（2026-04-09）

- [x] `build.rs` の `AE_Effect_Global_OutFlags` を `GlobalSetup` 側と一致する値（警告のコード側）へ同期
- [x] `build.rs` の `AE_Effect_Global_OutFlags_2` を `GlobalSetup` 側と一致する値（警告のコード側）へ同期
- [x] `src/lib.rs` の `SmartPreRender` で `Params::RefLayer` の実インデックスを `params.index(...)` から取得して `checkout_layer` するよう修正
- [x] `src/lib.rs` の `SmartRender` でも `RefLayer` の実インデックスを使って `checkout_layer_pixels` / `checkin_layer_pixels` するよう修正
- [x] `params_setup` の Popup 選択肢（Algorithm / RefMode / Align_U / Align_V / Ease_U / Ease_V）のみ英語へ置換（UIラベルの日本語は維持）
- [x] ビルド確認（対象クレート）でコンパイルエラーがないことを検証

# Phase 2 実装（2026-04-09）

- [ ] `Unknown checkout id` 対策として `SmartPreRender`/`SmartRender` の checkout/checkin ID=1 利用条件を整合（RefLayer が存在する時のみ要求・使用）
- [ ] `Params` 先頭に `RenderMode` を追加し、`params_setup` 冒頭で「描画モード」Popupを登録
- [ ] パス名一致（`U_1`/`U_2`/`V_1`/`V_2`）で4パスを厳密取得する処理を実装（不足時は安全フォールバック）
- [ ] Coons Patch 計算ヘルパー（`calculate_coons_patch`）と、パス評価ヘルパーを実装
- [ ] `RenderMode::Assignment` で4パス始点に色付き 5x5 マーカー描画を実装
- [ ] `RenderMode::UV+Grid` で格子状ポイント描画を実装
- [ ] `RenderMode::Final Result` / `Distribution map` は現時点フォールバック（入力コピー/TODO）を実装
- [ ] `cargo check -p tkg_path_line_hatching` でコンパイル確認
- [x] `Unknown checkout id` 対策として `SmartPreRender`/`SmartRender` の checkout/checkin ID=1 利用条件を整合（RefLayer が存在する時のみ要求・使用）
- [x] `Params` 先頭に `RenderMode` を追加し、`params_setup` 冒頭で「描画モード」Popupを登録
- [x] パス名一致（`U_1`/`U_2`/`V_1`/`V_2`）で4パスを厳密取得する処理を実装（不足時は安全フォールバック）
- [x] Coons Patch 計算ヘルパー（`calculate_coons_patch`）と、パス評価ヘルパーを実装
- [x] `RenderMode::Assignment` で4パス始点に色付き 5x5 マーカー描画を実装
- [x] `RenderMode::UV+Grid` で格子状ポイント描画を実装
- [x] `RenderMode::Final Result` / `Distribution map` は現時点フォールバック（入力コピー/TODO）を実装
- [x] `cargo check -p tkg_path_line_hatching` でコンパイル確認
