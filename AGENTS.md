# aod-AE-plugin 開発ガイド（AGENTS）

## 概要

- 本リポジトリは Rust 製の Adobe After Effects エフェクトプラグイン集（Cargo workspace）です。
- 主要ディレクトリ: `plugins/`（各プラグイン）、`crates/utils/`（共通）、`templates/plugin/`（テンプレート）、`tester/`（手動テスト用 .aep）。

## 実装前の確認順（必須）

- 新規実装や修正の前に、まず `templates/plugin/`、類似プラグイン、`crates/utils/` を確認し、既存の設計・命名・責務分割を優先的に踏襲すること。
- 単発用途のためだけに新しい抽象化を増やさず、まずは追跡しやすい明快な実装を選ぶこと。
- バグ修正や CI 不一致の調査では、修正前に再現手順、期待値、実際の結果、実行コマンド/ログを揃えてから着手すること。

## プラグイン命名規則（必須）

テンプレート（`templates/plugin/`）と既存実装の設計に合わせて統一してください。

- ディレクトリ名: kebab-case（例: `distance-generate`）
- crate 名（`Cargo.toml` の `package.name`）: snake_case（例: `distance_generate`）
- AE 表示名（`build.rs` の `Property::Name`）: `AOD_` + PascalCase（例: `AOD_DistanceGenerate`）
- AE Match Name（`Property::AE_Effect_Match_Name`）: PascalCase（例: `DistanceGenerate`）
- Category: `Aodaruma`

注意:

- `AE_Effect_Match_Name` は AE プロジェクトの識別に使われるため、既存プラグインの Match Name を変更すると互換性が壊れる可能性があります。変更は慎重に。

## 説明文の書き方（必須）

同一の説明文を以下に反映してください（テンプレートが連動しています）。

- `plugins/<name>/Cargo.toml` の `description`
- `plugins/<name>/src/lib.rs` の `PLUGIN_DESCRIPTION`
- `plugins/<name>/README.md` の本文
- ルート `README.md` の「Plugins / プラグイン説明」一覧

ルール:

- 英語で 1 文の短い説明（例: “Generates distance images from contours of colored regions.”）
- ルート README の一覧は「日本語 / English」のペアにする（既存スタイルに合わせる）
- 文末はピリオドで統一

## 新規プラグイン作成（cargo new-plugin）

エイリアスは `.cargo/config.toml` に定義済みです。

1) ルートで `cargo new-plugin`  
   - 内部的に `cargo generate --path templates/plugin --destination plugins` を実行
2) 生成された `plugins/<name>/` の以下を確認・調整  
   - `Cargo.toml` の `description`
   - `build.rs` の Name / Match Name（命名規則と一致しているか）
   - `src/lib.rs` の `PLUGIN_DESCRIPTION`
3) ルート `Cargo.toml` の `[workspace].members` に対象プラグインが含まれていることを確認
4) ルート `README.md` のプラグイン一覧に追加

## ビルド・インストール

- 全プラグイン: `just build` / `just release`
- 単体プラグイン: `just -f plugins/<name>/Justfile build`
- インストールをスキップ: `NO_INSTALL=1 just build`

## GitHub Desktop / Google Drive 対策

- このリポジトリは同期ドライブ配下のため、Windows や Google Drive により `.git/` 配下へ `desktop.ini` が混入し、GitHub Desktop が `fatal: git show-ref: bad ref refs/tags/desktop.ini` で壊れることがある。
- 各マシンの初回セットアップ時に、ルートで `powershell -ExecutionPolicy Bypass -File .\install_git_hooks.ps1` を実行し、`core.hooksPath=.githooks` を設定すること。
- GitHub Desktop がすでに壊れた場合は `powershell -ExecutionPolicy Bypass -File .\cleanup_git_desktop_ini.ps1` または `repair_git_metadata.bat` を実行してから再起動すること。
- hook 本体では `.git/` 配下の `desktop.ini` だけを削除し、作業ツリーのファイルは触らない。
## テスト（CI 互換）

`.github/workflows/ci.yml` に合わせ、ローカルでも以下を実行してください。

- `cargo fmt --all -- --check`
- `cargo clippy --workspace`
- `cargo test`

手動テスト:

- `tester/` の .aep を After Effects で開き、挙動を確認
- 挙動変更がある場合は、手動確認手順と実結果を作業ログまたは PR に残す

## 開発ルール

- 新規プラグインは必ずテンプレートから作成する
- 命名規則（kebab/snake/Pascal）と `AOD_` 接頭辞を統一する
- Match Name の変更は互換性に影響するため原則禁止
- 共有処理は `crates/utils/` を優先利用
- 変更後は fmt/clippy/test を通し、CI と同等の品質を担保する

## 動的UI（AE/Premiere 共通）ルール

- 動的UIを使う場合は `build.rs` の PiPL と `GlobalSetup` の両方で `OutFlags::SendUpdateParamsUI` を有効化する。
- UI更新は `Command::UpdateParamsUi` で行い、値変更は `Command::UserChangedParam` で行う（`UpdateParamsUi` では値を変えない）。
- `UserChangedParam` を受けたいパラメータには `ParamFlag::SUPERVISE` を付与する。
- `set_ui_flag(...); update_param_ui();` は必要時のみ呼ぶ（同値再設定を避ける）。
- 表示/非表示はホスト差分を考慮する:
  - After Effects: AEGP Stream の `DynamicStreamFlags::Hidden` を使う。
  - Premiere: `ParamUIFlags::INVISIBLE` を使う。
- `PF_Param_NO_DATA`（`NullDef`）は未処理のままECWに出すと「Unsupported Effects Control」を表示しうるため、説明表示用途では原則使わない。
- 説明文は既存パラメータ名の動的変更（`set_name`）で表現することを優先する。

## ブランチ・PR 運用（必須）

- `main` / `dev` への直接コミットは禁止（PR 経由のみ）。
- 作業時は必ずトピックブランチを作成し、`dev` へ Pull Request を作成する。
- 原則として1トピック1ブランチで管理し、無関係な変更を混在させない。
- Codex は、ユーザーの明示確認があるまでコミットおよび PR 作成を行わない。
- 実装と検証を先に行い、確認後にコミットと PR 作成を実施する。

## コミットメッセージ規則

- 形式: `TAG: Summary`（英語、1 行）
- TAG は以下を優先:
  - `ADD` 新機能・新規追加
  - `REFACTOR` リファクタリング
  - `CHORE` 依存更新・雑務
  - `FIX` 不具合修正
  - `DOCS` ドキュメント更新

## ワークフロー運用ルール

- 同じバグやエラーに対する自律的な修正の試行は最大3回までとする。
- 3回試しても改善しない場合は、無理にコードを書き換え続けず、再現条件・期待値・実結果・試行履歴を整理して停止する。
- 作業を完了扱いにする前に、変更差分と検証結果を確認する。
- 実機挙動が変わる変更は、After Effects または同等の手動確認結果を残す。
- ユーザーの明示確認があるまで、コミットおよび PR 作成は行わない。

## 調査ログと lessons

- 進行中の作業計画は `tasks/todo.md` に残す。
- 再発防止の知見や指摘は `tasks/lessons.md` に追記する。
- 新しい作業に入る前に、必要なら `tasks/lessons.md` を確認する。

## Context and token efficiency

- 構造探索には CodeGraph を優先する。
- CodeGraph の結果は編集対象を絞るための地図として扱う。
- コード変更前には対象ファイルと該当箇所を直接読む。
- Rust のマクロ、trait、FFI、build.rs、PiPL はグラフ結果だけで判断しない。
- 同じ内容を grep、CodeGraph、全ファイル読込で重複調査しない。
- target、生成物、アーカイブ、長大な生ログを通常コンテキストへ入れない。
- 長大なログは原本を保持し、関連箇所だけを抽出する。
- 圧縮されたログからコード変更する前に、原文の該当箇所を確認する。
- AGENTS.md、Git差分、クラッシュ原文、編集対象コードを自動圧縮しない。
- Headroom は全体プロキシとしてではなく、必要時の長大ログ圧縮に限定して扱う。
## Genshijin default

- このプロジェクトは、明示的な上書きが無い限り genshijin v1.4.0 の `extreme` mode を既定とする。
- 技術内容、コード、コマンド、識別子、パス、数値、エラーメッセージは省略しない。
- 破壊的操作の確認、セキュリティ警告、誤解しやすい手順は通常の日本語に戻して明確化する。
- `.md`、`.html`、`.json`、`.yaml`、`.csv` などのユーザー向けテキスト成果物には、自動で genshijin 調を適用しない。

