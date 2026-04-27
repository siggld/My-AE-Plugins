# GUI Development Flow (Shared + Feature-specific)

このドキュメントは、GUI開発を「共通基盤」と「個別機能」に分離して進めるための運用フローです。

## 目的

- 複数GUIで再利用できる実装を `crates/ae_ui` に集約する
- CustomGraphEditor など個別機能の開発速度を落とさない
- 共通化候補を継続的に取り込める開発サイクルを固定する

## 役割分担

### 共通基盤Agentで実施すること

- `crates/ae_ui` の共通ロジック実装・改善
- `templates/plugin` / `README.md` / `docs` の共通運用更新
- GUI検証用サンプルpluginでの回帰確認
- `cargo fmt --all -- --check` / `cargo clippy --workspace` / `cargo test` の品質確認

### 個別GUI Agentで実施すること

- CustomGraphEditor など固有仕様の実装
- 個別パラメータ設計、UI挙動、描画ロジック
- 共通化できる候補の抽出（後で共通基盤Agentへ移管）

## 判断基準

- 他GUIにも使える可能性が高い実装: `crates/ae_ui` へ
- そのGUIでのみ意味を持つ実装: 個別pluginへ

## 標準ワークフロー

1. 個別GUI Agentで機能を実装する
2. 共通化候補（重複するUI制御）を抽出する
3. 共通基盤Agentで `crates/ae_ui` へ最小移植する
4. GUI検証用サンプルpluginへ適用して動作確認する
5. 問題なければ対象pluginへ横展開する
6. docs/README の運用記述を更新する

## GUI検証用サンプルpluginの扱い

- 用途は「共通UI機能の検証」に限定する
- 個別プロダクト仕様を入れすぎない
- 壊しやすい host 差分（AE/Premiere）を優先的に検証する

## 依頼テンプレート（共通基盤Agent向け）

以下の形式で依頼すると、共通化作業が最短で進みます。

```md
### 背景
- 対象GUI: CustomGraphEditor
- 現在の実装先: plugins/<name>/src/lib.rs

### 共通化したい内容
- 例: UpdateParamsUi での表示/非表示切替
- 例: AE/Premiere の host 差分吸収

### 期待する成果物
- crates/ae_ui への関数追加
- サンプルpluginでの検証実装
- README/docs の更新

### 非対象
- CustomGraphEditor固有の描画仕様は変更しない
```

## Push前チェックリスト

- `cargo fmt --all -- --check`
- `cargo clippy --workspace`
- `cargo test`
- 変更点が「共通化」と「個別仕様」で混ざっていないことを確認

## CustomGraphEditor Agent依頼プロンプト

以下をそのまま個別GUI Agentへ渡してください。

```md
あなたは CustomGraphEditor 専任の実装Agentです。

## 目的
- CustomGraphEditor の個別GUI実装を進める。
- 共通化可能な処理は抽出して報告するが、このタスクでは共通基盤 (`crates/ae_ui`) へは直接移植しない。

## 実装ルール
- 対象は CustomGraphEditor 固有仕様のみ。
- `crates/ae_ui` の既存APIは利用してよい。
- 他GUIにも使える抽象化は新規で作りすぎない。
- 共通化候補を見つけたら「候補リスト」として最後に報告する。
- 既存の命名規則・workspace構成に従う。

## 今回やってほしいこと
1. CustomGraphEditor の ECW パラメータを実装する
2. `UpdateParamsUi` で必要な表示切替/活性制御を実装する
3. AE/Premiere差分が必要な箇所は現在のrepoパターンに沿って実装する
4. ビルド/静的チェック/テストを実行する
   - `cargo fmt --all -- --check`
   - `cargo clippy --workspace`
   - `cargo test`
5. 変更内容を要約し、共通化候補を分離して提示する

## 最終報告フォーマット
- 実装した内容（CustomGraphEditor固有）
- 変更ファイル一覧
- 検証結果（fmt/clippy/test）
- 共通化候補（このタスクでは未実施）
  - 候補名
  - 想定配置先 (`crates/ae_ui`)
  - 理由
```
