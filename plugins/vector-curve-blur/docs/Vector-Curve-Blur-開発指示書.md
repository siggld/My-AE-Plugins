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

## 現行主要パラメータ
- `Use All Paths (path / path_[n])`
  - ブラー対象はマスク名が `path` または `path_[n]`（例: `path_1`, `path_2`）のものだけを採用する
  - OFF 時は一致した最初の 1 本のみ使用する
- `Normal Range`
  - 選択した片側へ広げる法線帯域の幅
- `CenterLine (%)`
  - `0% = パス上`、`100% = NormalRange の外縁`
  - `Normal Falloff` / `Normal Falloff Bias` が収束する中心位置を法線帯域内でオフセットする
- `Normal Side`
  - `Positive` / `Negative` の片側だけに法線処理を適用する
- `Start Taper Curve` / `End Taper Curve`
  - 初期値は `0.5`
- `Profile Taper (Curve)`
  - `Enable Profile Curve (Curve)` を ON にすると、マスク名 `Curve` のパスをプロファイル用カーブとして使用する
  - `Positive Scale` / `Negative Scale` / `Link Scales` で片側倍率を制御する

## コアロジック仕様
- マスク名フィルタ（ターゲット/プロファイル分離）:
  - `PF_PathQuerySuite1` / `PF_PathDataSuite` で全マスク走査
  - マスク名 `path` または `path_[n]` 一致を描画ソースパスとして採用
  - マスク名 `Curve` 一致をプロファイル用カーブとして別抽出
- パス解析:
  - `PF_PathGetSegLength` / `PF_PathEvalSegLengthDeriv1` を利用
  - 接線 `T` / 法線 `N` を算出
- 法線帯域:
  - `Normal Side` で `Positive` / `Negative` の片側だけを処理対象にする
  - 法線距離を `0.0 = パス上`、`1.0 = NormalRange 外縁` に正規化する
  - `CenterLine (%)` を中心に、`Normal Falloff` / `Normal Falloff Bias` はパス側と外縁側の両端から収束する
- ブラー:
  - 画素から最短サンプル点を取得
  - 正規化位置 `t` を算出して `Path Blur Offset` 適用
- Taper:
  - `Start/End Taper Length` と `Curve` でパス始端/終端の厚みを制御する
- Fractal:
  - シードに `t` は使わず、法線距離 `d` と `Evolution` を使用
- Profile Curve:
  - カーブ始点X=0.0、終点X=1.0 の正規化空間でサンプリング
  - Y は始点/終点の上下関係から上側=1.0、下側=0.0 へ正規化
  - `Positive/Negative Scale`（`Link Scales` 時は同値）で倍率化

## バージョン/アーカイブ運用
- 形式: `vMajor.Minor.Patch`
- 作業完了時:
  - `archives/vX.Y.Z/` を作成し、主要ソースを複製
  - `CHANGELOG.md` 先頭に追記
- Minor/Major 更新時:
  - 旧 Patch フォルダは整理対象
  - ただし `CHANGELOG.md` の履歴は削除禁止

