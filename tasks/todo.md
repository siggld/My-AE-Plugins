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

# Phase 3 実装（2026-04-09）

- [x] パス全長ベース評価関数 `evaluate_path_at_ratio` を実装し、中間頂点を含むパス評価に対応
- [x] Uのみ/Vのみのフォールバック仕様を `calculate_coons_patch` に実装（仮想境界線生成）
- [x] 全パス未設定時に安全フォールバック（入力画像コピー）で終了する動作を維持
- [x] `RenderMode::UV+Grid` を 300x300 サンプルの UV グラデーション + 0.1 間隔グリッド + 2x2 相当スプラット描画へ拡張
- [x] `RenderMode::Assignment` に 3x5 ビットマップ文字（U/V/1/2）描画を追加し、始点マーカー横へラベル表示
- [x] `do_render` の入力コピーをベースに、各モード描画を上書きする構成を保持
- [x] `cargo fmt -p tkg_path_line_hatching` / `cargo check -p tkg_path_line_hatching` で整形・コンパイル確認

# vector-curve-blur 調整（2026-04-23）

- [ ] 再現条件を固定する: Fast Box Blur が白黒グラデーション境界/アルファ境界で効かない理由、`Radius=0` で端の見え方を比較できない理由、ヘヤピンカーブ時のドット抜け発生条件を `src/lib.rs` の現状ロジックに対応付けて整理する
- [ ] Fast Box Blur の責務を見直す: 画像ブラー・アルファ境界・端フェード確認に寄与するよう、`smoothed_normal_buffer` の適用先と端点処理（start/end skip 含む）を再設計する
- [ ] ヘヤピン時のドット抜けを軽減する: `nearest_sample` / 接線平滑化 / 分岐曖昧度の扱いを見直し、`vcb_0.1.29` ベースから改善差分を作る
- [ ] `addColor` を元コンポのアルファ基準へ拡張する: 元画像アルファから透明度を加算する軌跡マップを作り、既存の色加算と両立させる
- [ ] Tangent 方向の軌跡追加を実装する: パス軌跡から `NormalRange` 形状のマップを作成し、`PathBlurAmount` 分前後へ拡張後、Tangent 方向へ伸ばす新規項目を追加する
- [x] UI とドキュメントを同期する: 新規パラメータ名・説明・挙動を `docs/Vector-Curve-Blur-開発指示書.md` と必要な README/CHANGELOG に反映する
- [ ] 検証する: `cargo fmt --all -- --check`、`cargo clippy --workspace`、`cargo test`、必要なら対象クレートのビルドと手動確認手順を残す
