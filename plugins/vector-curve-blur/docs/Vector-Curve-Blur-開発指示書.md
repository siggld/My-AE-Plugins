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
- `Antialiasing Quality`
  - `Normal Side` の上に置き、接線/法線処理より前にサンプリング品質を決める
  - `Non`: point sample
  - `Low`: bilinear + Fractal マットの軽い平滑化
  - `High`: マット評価・色サンプル・Fractal マットに効く多点 supersample
  - 初期値は `High`
- `Normal Side`
  - `Positive` / `Negative` の片側だけに法線処理を適用する
- `TangentAmount(+)` / `TangentAmount(-)` / `TangentOffset`
  - `NormalControls` グループの外に置き、その直上で接線方向の基本量を管理する
  - `TangentAmount(+)` は接線正方向の基準量で、ブラー量だけでなく Displace の基準量にも使う
  - `TangentAmount(-)` は接線負方向の基準量で、常時有効かつ既定値は `0`
  - `TangentOffset` は Displace / Blur 後の像全体を接線 `+/-` 方向へ戻し込む後段オフセット
- `NormalControls`
  - `Normal Range` から `Normal Falloff Bias` までの法線帯域制御を 1 グループとして扱う
  - `Normal Range` は選択した片側へ広げる法線帯域の幅
  - `CenterLine (%)` は `0% = パス上`、`100% = NormalRange の外縁`
  - `CenterLine (%)` の初期値は `50`
  - `Normal Falloff` / `Normal Falloff Bias` が収束する中心位置を法線帯域内でオフセットする
  - `Taper Mode` で `SimpleTaper` / `ProfileTaper(Curve)` を切り替えた時も、この値を軸に内外から収束する
- `TangentOffset` / `Subtraction Alpha`
  - `TangentOffset` は Displace / Blur 後の像を接線 `+/-` 方向へずらす後段オフセット
  - 端から薄くする効果は `Subtraction Alpha` で別制御する
- `View Mode`
  - `Final`
  - `NormalMat`: NormalBand 側の最終ウェイトをグレースケール表示
  - `TangentMat`: TangentFalloff 側の最終ウェイトをグレースケール表示
  - `Fractal`: 共通 Fractal マット表示
  - `Taper`: Taper / Profile / CenterLine 反映後の幅マット表示
- `Start Taper Curve` / `End Taper Curve`
  - 初期値は `0.5`
- `Taper Mode`
  - `Non`（初期値） / `SimpleTaper` / `ProfileTaper(Curve)` を切り替える
  - `SimpleTaper` の時だけ `Simple Taper` グループ内の項目を編集できる
  - `Non` と `ProfileTaper(Curve)` の時は `Simple Taper` グループ全体をグレーアウトする
  - `ProfileTaper(Curve)` はマスク名 `Curve` のパスを自動取得し、追加パラメータなしで `Curve` 形状と `NormalRange` だけを使って帯域幅を決める
- `Master Intensity`
  - `Fractal Amount` はこのグループに含め、共通 Fractal マットの混入率を制御する
  - `Displace Multiplier` は元絵側 Displace の強さ
  - `Blur Multiplier` は元絵側 Blur の強さで、初期値は `10`
  - `Ghost Multiplier` は Ghost 側 Displace / Blur の強さ
  - `Ghost Alpha` は Ghost 結果を最終像へ重ねる不透明度で、初期値は `0`
- `Fractal`
  - `Fractal Scale` / `Fractal Tangent Scale` / `Fractal Tangent Offset` / `Fractal Complexity` / `Evolution`
  - Fractal グループはテクスチャ形状だけを調整する

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
- Taper モード:
  - `Taper Mode = Non` の時は追加の帯域変形を行わない
  - `Taper Mode = SimpleTaper` の時は `Start/End Taper Length` と `Start/End Taper Curve` を使う
  - `Taper Mode = ProfileTaper(Curve)` の時はマスク名 `Curve` の Y 形状をそのまま帯域乗数として使う
- ブラー:
  - 画素から最短サンプル点を取得
  - 共通 Fractal マットを `normal/taper/profile` 後の帯域に対して評価する
  - a. `TangentAmount(+)` / `TangentAmount(-)` と `Displace Multiplier` で、接線の正負両側を基準に変位像を作る
  - b. `TangentAmount(+)` / `TangentAmount(-)` と `Blur Multiplier` で、正負量から決まる中心と幅を使って接線方向ブラーを掛ける
  - c. `Ghost Multiplier` で Ghost 用の Displace / Blur を別結果として生成する
  - d. `Ghost Alpha` で Ghost 結果を最終像へ上から重ね、`TangentOffset` は各結果を接線 `+/-` 方向へ後段オフセットする
  - `Normal Falloff` / `Normal Falloff Bias` / `Tangent Falloff` / Taper で作られた最終マットは、Displace / Blur / Ghost / AddColor の処理量に共通適用する
  - `NormalFalloff` の減衰は元絵との後段 lerp ではなく処理量そのものへ掛け、境界で原画とずれた像が二重に見えにくいようにする
- Taper:
  - `Start/End Taper Length` と `Curve` でパス始端/終端の厚みを制御する
  - 帯域の収束軸は `CenterLine (%)` と同期し、パス固定ではなく CenterLine を中心に縮む
- Fractal:
  - シードに `t` は使わず、法線距離 `d` と `Evolution` を使用
  - `Fractal Scale` 初期値は `15`
  - `Fractal Tangent Scale` 初期値は `5`
  - `Fractal Complexity` 初期値は `15`
  - `CenterLine (%)` と Taper / Profile Taper で帯域が収束した場合、Fractal も同じ収束後帯域を基準に評価する
  - `Fractal Amount = 0` の時は Fractal 形状を掛けず、帯域全体へ均一に作用する
- Profile Curve:
  - カーブ始点X=0.0、終点X=1.0 の正規化空間でサンプリング
  - Y は始点/終点の上下関係から上側=1.0、下側=0.0 へ正規化
  - `Taper Mode = ProfileTaper(Curve)` の時だけ `CenterLine (%)` 基準の帯域変形として適用する
  - 個別の `Profile Amount` / `Invert Profile` / `Profile Min Width` は持たず、`Curve` 形状そのものを使う
- 境界保持 / AA:
  - `Master Intensity` は `Fractal Amount` / `Displace Multiplier` / `Blur Multiplier` / `Ghost Multiplier` / `Ghost Alpha` の 5 項目で構成する
  - 単純な代表色 1 点選抜ではなく、変位像を基準にした加重平均で近似色のディテールを残す
  - `Antialiasing Quality` は `Non / Low / High` を切り替え、`Low` 以上では Fractal 由来の Displace 量も平滑化し、`High` は効果全体の supersample で細線のドット欠け低減を優先する

## バージョン/アーカイブ運用
- 形式: `vMajor.Minor.Patch`
- 作業完了時:
  - `archives/vX.Y.Z/` を作成し、主要ソースを複製
  - `CHANGELOG.md` 先頭に追記
- Minor/Major 更新時:
  - 旧 Patch フォルダは整理対象
  - ただし `CHANGELOG.md` の履歴は削除禁止

