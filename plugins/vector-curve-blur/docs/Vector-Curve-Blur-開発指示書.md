# Vector Curve Blur - 開発指示書

## 目的
After Effects 用 Rust プラグイン `Vector Curve Blur` の仕様・運用ルールを記録し、以後の改修時に履歴更新と同期してメンテナンスする。

## プロジェクト概要
- プラグイン名: `Vector Curve Blur`
- ディレクトリ: `plugins/vector-curve-blur/`
- 機能概要:
  - マスクパス形状をベクトルソースとして解析
  - パス沿いブラー、オフセット、Taper、スリット状フラクタルノイズを適用

## AE アーキテクチャ要件
- MFR 対応:
  - グローバル/静的ミューテーション禁止
  - `OutFlags2::SupportsThreadedRendering` を有効化
- SmartFX と全深度対応:
  - 8/16/32-bit 対応
  - `OutFlags2::SupportsSmartRender` を有効化
  - `SmartPreRender` / `SmartRender` を実装
- 動的 UI:
  - トリガーに `ParamFlag::SUPERVISE`
  - `UpdateParamsUi` で `ParamUIFlags::DISABLED` などを制御

## パラメータ ID 固定仕様
1. View Mode (Pop-up)
2. Target Mask Name (Arbitrary)
3. All Masks (Checkbox)
4. Normal Range
5. Normal Falloff
6. Normal Falloff Bias
7. Path Blur Amount
8. Path Blur Offset
9. Enable Taper (Checkbox, SUPERVISE)
10. Taper Group Start
11. Start Taper Length
12. Start Taper Curve
13. End Taper Length
14. End Taper Curve
15. Taper Group End
16. Fractal Amount
17. Fractal Scale
18. Fractal Complexity
19. Evolution
20. Profile Group Start
21. Enable Profile Curve (Checkbox, SUPERVISE)
22. Profile Mask Name (Arbitrary, default: `curve`)
23. Positive Scale
24. Negative Scale
25. Link Scales (Checkbox, SUPERVISE)
26. Invert Curve X (None / Positive / Negative / Both)
27. Swap Normal (+/-)
28. Profile Group End

## コアロジック仕様
- マスク名フィルタ（ターゲット/プロファイル分離）:
  - `PF_PathQuerySuite1` / `PF_PathDataSuite` で全マスク走査
  - `Target Mask Name`（または `All Masks`）一致を描画ソースパスとして採用
  - `Profile Mask Name` 一致をプロファイル用カーブとして別抽出
- パス解析:
  - `PF_PathGetSegLength` / `PF_PathEvalSegLengthDeriv1` を利用
  - 接線 `T` / 法線 `N` を算出
- ブラー:
  - 画素から最短サンプル点を取得
  - 正規化位置 `t` を算出して `Path Blur Offset` 適用
- Taper:
  - `Start/End Taper Length` と `Curve` で減衰
- Fractal:
  - シードに `t` は使わず、法線距離 `d` と `Evolution` を使用
- Profile Curve:
  - カーブ始点X=0.0、終点X=1.0 の正規化空間でサンプリング
  - Y は始点/終点の上下関係から上側=1.0、下側=0.0 へ正規化
  - `Invert Curve X` に従い正負側独立で `t` 反転
  - `Positive/Negative Scale`（`Link Scales` 時は同値）で倍率化
  - `Swap Normal (+/-)` で正負適用先を入替

## バージョン/アーカイブ運用
- 形式: `vMajor.Minor.Patch`
- 作業完了時:
  - `archives/vX.Y.Z/` を作成し、主要ソースを複製
  - `CHANGELOG.md` 先頭に追記
- Minor/Major 更新時:
  - 旧 Patch フォルダは整理対象
  - ただし `CHANGELOG.md` の履歴は削除禁止

