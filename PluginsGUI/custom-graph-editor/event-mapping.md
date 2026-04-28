# Custom Graph Editor Event Mapping

## 基本方針

- 入力イベントは UI 層で解釈し、状態変更はすべて Model API 経由で行う
- UI 層で不変条件を再実装しない

## 入力イベント一覧

## Mouse Down

- Left + アンカーヒット:
  - アクティブターゲットを Anchor に設定
- Left + アウトハンドルヒット:
  - アクティブターゲットを OutHandle に設定
- Left + インハンドルヒット:
  - アクティブターゲットを InHandle に設定
- Left + 曲線ヒット:
  - `AddNodeOnCurve(hitX)` 実行
  - 追加ノードをアクティブ Anchor としてドラッグ開始
- Shift + Left + 内部アンカーヒット:
  - `RemoveNode(index)` 実行
- Right + 内部アンカーヒット:
  - `RemoveNode(index)` 実行

## Mouse Move（ドラッグ中）

- Active=Anchor:
  - `MoveAnchor(index, normalizedPoint)`
- Active=InHandle:
  - `MoveInHandle(index, normalizedPoint)`
- Active=OutHandle:
  - `MoveOutHandle(index, normalizedPoint)`

## Mouse Up

- アクティブターゲットをクリア

## ヒットテスト優先順

1. アンカー
2. ハンドル
3. 曲線

## ピッキング

- 半径は px ベースで管理
- 実画面スケールに応じて調整可能にする（デフォルト値あり）
