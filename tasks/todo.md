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
