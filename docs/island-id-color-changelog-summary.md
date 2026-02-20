# Island ID Color プラグイン 処理変更の流れ（要約）

前回の「処理変更の流れ」以降に行った変更の要約です。

---

## 1. Sort Mask / Grad Mask の表示を実マスク名に

**状況**: ポップアップが「Mask 1」～「Mask 4」のまま、レイヤーで付けたマスク名が表示されない。

**対応**:
- `SortMaskIndex` と `GradMaskIndex` を **PopupDef → PathDef** に変更。
- AE の Path パラメータがレイヤーのマスク名をそのまま表示するため、AEGP で名前を書き換える処理は削除。
- レンダー側は `path_id()` を直接使用し、`path_info()` によるインデックス変換を廃止。

**注意**: 既存 .aep では Sort Mask / Grad Mask の値はリセットされるため、再設定が必要。

---

## 2. Sort Angle グループの初期展開

**状況**: 「Island Tracking & Temp Colors」グループが最初から閉じており、Sort Angle にすぐアクセスできない。

**対応**:
- `params_setup` の `add_group(..., true)` を `false` に変更し、グループを**初期表示で開く**ようにした。
- `update_params_ui_visibility` 内で **SortAngle に `START_COLLAPSED` を毎回付けていた処理を削除**。これが親グループの表示に影響していた可能性あり。

**補足**: 既存プロジェクトでは AE がグループの開閉状態を保存するため、一度手動で開いて保存し直すと以降も開いた状態になる。

---

## 3. Final Gradient：全アイランド表示と Enable の意味づけ

**方針**: 基本的に全アイランドを表示し、「Enable Gradient N」は「その島の色・反転・不透明度を個別にいじるかどうか」のスイッチにする。

**対応**:
- **Enable のデフォルトをすべて ON** に変更（従来は 1 のみ ON）。
- **Enable OFF でも非表示にしない**: OFF の島は白→黒のデフォルトグラデーションで表示し、不透明度 100%。
- Enable ON のときだけ、Start/End 色・反転・不透明度（後述）を適用。

---

## 4. グラデーションスロットに不透明度を追加

**追加内容**:
- 各グラデーションスロットに **「Opacity N (%)」**（0～100%、デフォルト 100%）を追加。
- `Params` に `GradientOpacity0`～`GradientOpacity31`、定数配列 `GRADIENT_OPACITY` を追加。
- レンダー時は `(enabled, start, end, invert, opacity)` の 5 要素で扱い、最終アルファに `opacity` を乗算。
- Enable OFF のときは不透明度を 100% 固定で描画。

**UI**: Enable OFF のとき、Start/End/Invert/Opacity は DISABLED（グレーアウト）。

---

## 5. Gradient 1～32 がどの島か分かるように（パラメータ名）

**対応**:
- 各スロットのチェック名を **「Enable Gradient 1 (Island #1)」** ～ **「Enable Gradient 32 (Island #32)」** に変更。
- ソート後の表示順と対応（Island #1 = 1 番目の島、Temp Color の 1 番目と同じ島）。

**補足**: ビューポート上で「いま触っているパラメータがどの島か」を視覚で示すには、後述の「島番号表示」を利用する。

---

## 6. Master Noise：フラクタル → グレイン状ノイズ

**要望**: Master Noise Amount は細かいグレイン状のノイズにしたい。

**対応**:
- グラデーションのノイズを **Perlin 風ノイズから `pixel_noise`（ピクセル単位の擬似乱数）に戻した**。
- コメントを「グレイン状ノイズ」に合わせて修正。Perlin 用の `perlin_noise_2d` と `GRADIENT_NOISE_SCALE` は `#[allow(dead_code)]` で残置。

---

## 7. 島番号表示（チェックで ON/OFF）

**目的**: 複数島があるときに、ビューポート上で「どの島が何番か」を番号で確認できるようにする。

**実装**:
- **パラメータ**: Master Noise Amount の直下に **「Show Island Numbers」** チェックを追加（デフォルト OFF）。
- **描画**: Output Mode が Final Gradient かつ Show Island Numbers が ON のとき、各島の**重心**を算出し、その位置に島 ID（1, 2, 3, …）を **5×7 ビットマップフォント**で白描画。
- **定数**: `ISLAND_NUM_FONT`（0～9）、`island_number_pixel(px, py, centroids)`、重心リスト `island_centroids` を追加。
- レンダーでは、通常のグラデーション描画の後に、番号ピクセルに該当する座標を白で上書き。

**補足**: パラメータ選択時に該当島をハイライトする機能は、AE SDK に「現在フォーカス中のパラメータ」を返す API がなく、標準範囲では未実装。

---

## 8. CI 対応（cargo fmt）

**事象**: GitHub CI で `cargo fmt --check` が差分で失敗。

**対応**:
- `ISLAND_NUM_FONT` の配列を、`cargo fmt` のスタイル（内側の配列を複数行に分割）に合わせて整形。
- `island_number_pixel` のシグネチャを 1 行にまとめた。
- ルートで `cargo fmt --all` を実行し、`cargo fmt --all -- --check` が通る状態に統一。

---

## ファイル・箇所の目安

| 内容           | 主な対象 |
|----------------|----------|
| PathDef / マスク名 | `Params` enum, `params_setup`, `update_params_ui_visibility`, `do_render`（sort_path_id / grad_path_id）, `sort_by_mask_path`, `island_grad_tangent` |
| グループ初期展開 / SortAngle | `params_setup` の `add_group`, `update_params_ui_visibility` の SortAngle ブロック |
| 全島表示・Opacity・Enable 名 | `Params` / 定数配列, `params_setup`, `update_params_ui_visibility`, `do_render`（grad_slots, 描画分岐） |
| グレインノイズ   | グラデーション計算のノイズ加算部分（`pixel_noise` 使用） |
| 島番号表示       | `Params::ShowIslandNumbers`, `params_setup`, `ISLAND_NUM_FONT` / `island_number_pixel`, `island_centroids`, レンダー末尾の上書き |
| fmt             | `lib.rs` のフォント定数と `island_number_pixel` 付近 |

---

## 注意事項

- **既存プロジェクト**: Sort Mask / Grad Mask を PathDef に変えたため、既存 .aep ではこれらの値が失われます。GradientOpacity を 32 個追加したため、パラメータ構成が変わり、古いプロジェクトでエフェクトが読み込めない場合もあります。新規またはエフェクト付け直しでの確認を推奨します。
- **CI**: コミット前に `cargo fmt --all` と `cargo clippy --workspace` を実行しておくと安全です。
