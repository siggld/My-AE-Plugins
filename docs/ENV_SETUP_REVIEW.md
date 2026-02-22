# 開発環境セットアップの精査（オブザーバー指摘事項）

このリポジトリは **Rust 製** の AE プラグイン集です。C/C++ + CMake の一般的な AE プラグイン構成とは異なるため、外部オブザーバーの指摘事項を本リポジトリに合わせて精査しました。

---

## 1. Adobe After Effects SDK（必須）

| オブザーバー指摘 | 本リポジトリでの扱い |
|-----------------|----------------------|
| SDK を Adobe Developer Console から取得し、CMakeLists.txt または `AE_SDK_PATH` でパス指定 | **該当しません。** 本リポジトリは **CMake を使いません**。AE の API は [after-effects](https://github.com/virtualritz/after-effects) クレートがラップしており、ワークスペースの `Cargo.toml` で git 依存として参照しています。SDK をローカルに配置する必要はありません。 |

---

## 2. コンパイラとビルドツール

| オブザーバー指摘 | 本リポジトリでの扱い |
|-----------------|----------------------|
| Windows: Visual Studio 2019/2022（C++ デスクトップ開発） | **適用。** 本リポジトリでは **Visual Studio 2022** を指定しています（GitHub CI の windows-latest と一致）。`build_and_deploy.bat` が `vcvars64.bat` で MSVC 環境を読み込みます。VS 2022 Community または Build Tools（C++ ワークロード）をインストールしてください。 |
| macOS: Xcode / Apple Silicon 対応 | **適用。** `just build` / `just release` で macOS 向けにビルドします。ユニバーサルバイナリはテンプレート・Justfile で必要に応じて設定します。 |

---

## 3. Cursor 拡張機能の最適化

| オブザーバー指摘 | 本リポジトリでの扱い |
|-----------------|----------------------|
| C/C++ Extension Pack, CMake Tools, clangd | **Rust 主体のため C/C++ 系は必須ではありません。** Rust 開発には **rust-analyzer**（Rust 拡張）を推奨。CMake は使わないため CMake Tools は不要。AE SDK の C ヘッダを直接触らない限り、C/C++ 拡張や clangd はオプションです。**AI ペアプロ**で AE の型（PF_Handle 等）を参照したい場合は、`after-effects` クレートの公開 API や `AGENTS.md` をコンテキストに渡すとよいです。 |

---

## 4. デバッグ環境の整備

| オブザーバー指摘 | 本リポジトリでの扱い |
|-----------------|----------------------|
| .vscode/launch.json で .aex を Plug-ins にコピー／シンボリックリンク、AE をデバッガ起動または Attach | **対応済み。** `.vscode/launch.json` に以下を追加しています。**Build, Deploy & Launch AE**: 事前タスクで `build_and_deploy.bat` を実行し、続けて After Effects を起動。**Attach to After Effects**: 起動中の AE プロセスにアタッチ（`${command:pickProcess}` でプロセス選択）。Windows では `cppvsdbg` を使用するため、C/C++ 拡張（ms-vscode.cpptools）を入れておくと利用しやすいです。AE のインストール先が異なる場合は `launch.json` の `program` パスを編集してください。 |

---

## 5. シンボリックリンクの活用

| オブザーバー指摘 | 本リポジトリでの扱い |
|-----------------|----------------------|
| 開発用プラグインフォルダから AE の Plug-ins へ mklink / ln -s でシンボリックリンク | **対応済み。** `create_plugin_symlink.bat` を用意しています。初回は `build_and_deploy.bat` でビルドしたあと、**管理者として** `create_plugin_symlink.bat` を実行すると、`%LOCALAPPDATA%\AEPluginBuild\release\island_id_color.dll` へ向かうシンボリックリンクが AE の Plug-ins フォルダに作成されます。以降はビルドするだけで .aex が更新されます。通常のコピー運用の場合は `build_and_deploy.bat` のまま利用してください。 |

---

## 本リポジトリで追加で必要な環境

- **Rust**: `rust-toolchain.toml` で channel 指定（例: 1.93）。`rustup` でインストール。
- **cargo-generate**: 新規プラグイン作成時に使用（`cargo new-plugin`）。
- **just**: 推奨。`just build` / `just release` で一括ビルド・インストール。
- **Visual Studio 2022**（Windows）: MSVC 用。Community または Build Tools の「C++ によるデスクトップ開発」ワークロード。

---

## ローカルビルドの .aex で AE が起動時にクラッシュする場合

**現象**: ローカルでビルドした .aex を配置すると、AE 起動時に `<MEE_Plugins> <5> … MediaCore` でクラッシュする。プラグインを外すと AE は起動する。

**ログ計装の結果**: プラグイン内の `params_setup` / `GlobalSetup` / `UpdateParamsUi` のいずれにも**到達前に**クラッシュしている（指定したログファイルに 1 行も出力されない）。  
→ 原因は **DLL ロード直後** または **AE がプラグインに初回呼び出しする前** の段階と判断できる。

**当面の対処**:
- **リリース用 .aex は [GitHub Releases](https://github.com/Aodaruma/aod-AE-plugin/releases) の成果物を使用する**（CI は windows-latest / VS 2022 でビルドしており、こちらではクラッシュしない報告あり）。
- ローカルでは `cargo build` / `just build` でビルドし、**デプロイと実機確認には CI でビルドされた .aex を配置する**運用を推奨。
- `build_and_deploy.bat` 実行時に表示される `Using: ...\vcvars64.bat` で、実際に VS 2022 が使われているか確認できる。

---

## 開発〜リリースの工程（ローカルビルドで AE が落ちる場合）

| 段階 | やること | 補足 |
|------|----------|------|
| **1. 開発** | コードを編集する | いつも通り。 |
| **2. ビルド確認** | ローカルで `cargo build --release -p island_id_color` または `.\build_and_deploy.bat` を実行する | コンパイルが通るか確認。ここで作った .aex は **AE の Plug-ins に置かない**（起動クラッシュするため）。 |
| **3. コミット〜CI** | 変更を push し、CI（`just release`）でビルドさせる | main / dev などブランチに push すると GitHub Actions が動く。 |
| **4. .aex の取得** | [Releases](https://github.com/Aodaruma/aod-AE-plugin/releases) から Windows 用 zip をダウンロードし、中身の `island_id_color.aex` を取り出す | プレリリースや main-release など、対象ブランチの最新を選ぶ。 |
| **5. AE で確認** | 取得した `island_id_color.aex` を `Plug-ins\AOD\` に配置し、After Effects を起動して動作確認する | 実機で使う .aex は **必ず CI ビルドのもの**にする。 |

**まとめ**: 「書く → ローカルでビルド（通るかだけ確認）→ push → CI で .aex ができる → その .aex をダウンロードして AE に配置して試す」という流れになる。

---

## まとめ

- **AE SDK の直接配置・CMake・AE_SDK_PATH**: 本リポジトリでは不要（Rust + after-effects クレートで完結）。
- **VS 2022・Rust・just**: 必須に近い。
- **デバッグ用 launch.json・シンボリックリンク**: `.vscode/launch.json` と `create_plugin_symlink.bat` で対応済み。
- **C/C++ 拡張・CMake Tools**: Rust 主体のため通常は不要。launch.json の「Build, Deploy & Launch AE」「Attach to After Effects」を使う場合は Windows で C/C++ 拡張（cppvsdbg）があると便利。
