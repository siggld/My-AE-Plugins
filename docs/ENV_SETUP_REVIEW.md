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

**red_noise 単体での確認**: **red_noise.aex のみ**を Plug-ins に配置して AE を起動した場合も**同様のクラッシュ**が発生することを確認済み。  
→ 原因は **island-id-color 固有ではなく、この環境でビルドした .aex 全般**（ローカルビルド環境：VS・CRT・ドライブ等）にあると結論できる。

**「ローカル環境の改善では解決できない」かについて**: **不可能とは言い切れません**。今回のデバッグで分かったのは「クラッシュは DLL ロード時」「特定プラグインではなく環境要因」であり、CRT 静的リンクや PiPL の一部変更は試したが未解消だった、という点までです。**まだ試していない環境要因**（下記「CI を使わない場合にローカルで試せること」）が残っているため、それらを試すことで解決する可能性はあります。CI が使えない場合は、以下を順に試す価値があります。

**GitHub を使わずにローカルで切り分け・対策を試す場合**:
1. **クラッシュ位置**  
   - ログ計装の結果、**handle_command には到達していない**ことが判明している（ログファイルは 1 行も出力されず）。クラッシュは DLL ロード直後〜コールバック前の段階。
2. **CRT を静的リンクして試す**  
   - AE と DLL の MSVC ランタイムの不一致が疑われる場合、Rust 側で CRT を静的リンク（`crt-static`）にすると解消することがある。  
   - ルートの `.cargo/config.toml` に以下を追加してビルドし直し、同じ .aex で AE を起動してクラッシュするか確認する。  
     ```toml
     [target.x86_64-pc-windows-msvc]
     rustflags = ["-C", "target-feature=+crt-static"]
     ```  
   - 試したあと問題が続く、または別の不具合が出た場合はこの設定を外す。
3. **最小プラグイン（red_noise）で同じ環境か確認する**  
   - 同じ環境で `cargo build --release -p red_noise` を実行し、`target\release\red_noise.dll` を `Plug-ins\AOD\red_noise.aex` としてコピーして AE を起動する。  
   - **red_noise でもクラッシュする** → ローカル環境（VS バージョン・CRT・ドライブ等）の問題の可能性が高い。上記の CRT 静的リンクや、ビルド出力を C: の `target` に出すなどの変更を試す。  
   - **red_noise ではクラッシュしない** → island-id-color 固有の要因（依存関係やコード量・最適化など）を疑い、プラグイン側の変更やビルドオプションを検討する。

**当面の対処**（CI を利用する場合）:
- **リリース用 .aex は [GitHub Releases](https://github.com/Aodaruma/aod-AE-plugin/releases) の成果物を使用する**（CI は windows-latest / VS 2022 でビルドしており、こちらではクラッシュしない報告あり）。
- ローカルでは `cargo build` / `just build` でビルドし、**デプロイと実機確認には CI でビルドされた .aex を配置する**運用を推奨。
- `build_and_deploy.bat` 実行時に表示される `Using: ...\vcvars64.bat` で、実際に VS 2022 が使われているか確認できる。

**CI を使わない場合にローカルで試せること**（有料 CI を使わず、ローカル環境の改善で解決を狙う場合）:
- **リポジトリとビルド出力を同期ドライブ以外に置く**  
  現在のワークスペースが G:（Google Drive 等の**同期ドライブ**）の場合、DLL の読み込みやパス長・ロックの影響でクラッシュすることがあります。**C: や W: など、同期しないローカルドライブ**にリポジトリを置き、そこでビルドしてできた .aex を Plug-ins に置いて AE を起動して試す。C: の容量を圧迫したくない場合は **W: を移行先**にするとよい（手順は下記「W: ドライブに移行してビルドする」参照）。
- **CARGO_TARGET_DIR を同期ドライブ以外の短いパスにする**  
  `build_and_deploy.bat` では `%LOCALAPPDATA%\AEPluginBuild` を使っていますが、リポジトリを W: に移した場合は `W:\work\My-AE-Plugins\target` のように W: 上のパスにすると、ソースとビルド出力を同じドライブにまとめられる。
- **Rust を CI と同じ stable に合わせる**  
  `rust-toolchain.toml` は 1.93 指定です。CI では `dtolnay/rust-toolchain@stable` で stable が使われています。`rustup default stable` で stable に切り替えてビルドし直し、.aex で起動を試す。
- **VS 2022 が確実に使われているか確認する**  
  `build_and_deploy.bat` 実行時の `Using: ...\vcvars64.bat` が **2022** のパスになっているか確認。別の VS が選ばれていないか確認する。
- **別 PC やクリーンな環境でビルドする**  
  同じソースを別マシン（または VM）で VS 2022 + Rust のみ入れた状態でビルドし、その .aex をこの PC の AE に置いて起動する。起動すれば「この PC のビルド環境」に原因があると切り分けできる。

上記はいずれも「まだ試していない」または「十分に試せていない」項目です。CI が使えずローカルで .aex を動かしたい場合は、上から順に試すとよいです。

---

## W: ドライブに移行してビルドする（問題解決の試行）

**目的**: 同期ドライブ（G: 等）でのビルドがクラッシュの要因になっている可能性を避けるため、**W: にリポジトリとビルド出力を置いて**ビルドし、できた .aex で AE が起動するか試す。C: の容量を圧迫したくない場合の移行先として W: を推奨。

**前提**: W: は同期ドライブではなく、ローカル（またはネットワークで常時マウントされた）ドライブであること。

**手順**:

1. **W: に作業用フォルダを作成し、リポジトリをコピーまたはクローンする**
   - 例: `W:\work\My-AE-Plugins`  
   - 既存の G: のリポジトリをそのままコピーするか、git でクローンし直す。  
   - コピーする場合: `.git` フォルダを含めてコピーすれば、そのまま git の履歴も使える。

2. **W: 上でビルド環境を整える**
   - W: のリポジトリルートで **VS 2022 の開発者コマンドプロンプト** を開く（または `vcvars64.bat` を実行したうえで同じフォルダをカレントにする）。
   - `cargo clean` のあと、次いずれかでビルドする。
     - **build_and_deploy_W.bat を使う場合**: リポジトリルートに `build_and_deploy_W.bat` を用意してある。W: に移したリポジトリのルートでこのバッチを実行すると、`CARGO_TARGET_DIR` が「そのドライブのリポジトリ直下の `target`」になり、island_id_color がビルドされ、AE の Plug-ins に配置される。まずは **red_noise** で試したい場合は、手動で `set CARGO_TARGET_DIR=W:\work\My-AE-Plugins\target` と `cargo build --release -p red_noise` を実行し、`target\release\red_noise.dll` を `red_noise.aex` として Plug-ins にコピーする。
     - **手動でビルドする場合**:
       ```batch
       set CARGO_TARGET_DIR=W:\work\My-AE-Plugins\target
       cargo build --release -p red_noise
       ```
       または island-id-color の場合:
       ```batch
       set CARGO_TARGET_DIR=W:\work\My-AE-Plugins\target
       cargo build --release -p island_id_color
       ```
   - ビルドが成功すると、`W:\work\My-AE-Plugins\target\release\` に `.dll` ができる。これを `.aex` としてコピーして使う。

3. **.aex を AE の Plug-ins に配置して起動確認**
   - 例: `target\release\red_noise.dll` を `red_noise.aex` にリネームしてコピーし、`C:\Program Files\Adobe\Adobe After Effects 2024\Support Files\Plug-ins\AOD\` に配置（パスは環境に合わせる）。
   - After Effects を起動し、同じ MEE_Plugins / MediaCore のクラッシュが出ないか確認する。

4. **結果の解釈**
   - **W: でビルドした .aex で AE が起動した** → 同期ドライブ（G:）上のビルドが要因だった可能性が高い。以降は W: を開発・ビルドの作業場所にする。
   - **依然クラッシュする** → **W: に移してビルドしてもクラッシュした場合は、ドライブ要因ではなかった**と判断できる。VS バージョン・CRT・Rust バージョン・この PC の環境要因を疑い、下記「W: でもクラッシュした場合に次に試すこと」を試す。

**W: でもクラッシュした場合に次に試すこと**:
- **Rust を stable に合わせる**: `rustup default stable` でビルドし直し、.aex で起動を試す（CI は stable を使用）。**→ 試したがクラッシュは解消しなかった。**
- **VS 2022 が確実に使われているか確認**: `build_and_deploy_W.bat` 実行時に表示される `Using: ...\vcvars64.bat` のパスが **`...\2022\...`** か確認。vcvars64.bat の場所は `C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat`（Community）または `...\2022\BuildTools\...`（Build Tools）。エクスプローラーで上記パスにファイルがあるか開いて確認できる。
- **別 PC や VM でビルド**: 同じソースを別マシンで VS 2022 + Rust のみの環境でビルドし、その .aex をこの PC の AE に置いて起動する。起動すれば「この PC のビルド環境」に原因があると切り分けできる。（別 PC が無い場合はスキップ。）
- **CI が使えない場合の運用**: ローカルではコンパイル・静的解析までとし、実機で動かす .aex は CI ビルドのものを別手段で取得するか、上記を試したうえで別環境ビルドの .aex を使う。

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

## ローカル環境を継続して使う場合の次のステップ

この環境では**ローカルでビルドした .aex を Plug-ins に置くと AE が起動時にクラッシュする**ため、次の流れで進めます。

| やること | 手順 |
|----------|------|
| **1. 開発** | コードを編集する。ローカルで `cargo build --release -p <crate名>` または `just build` でコンパイルが通るか確認する。**ここでできた .aex は AE の Plug-ins に置かない。** |
| **2. 品質チェック** | push 前に `cargo fmt --all -- --check` と `cargo clippy --workspace`（と必要なら `cargo test`）を実行し、CI で落ちないようにする。 |
| **3. コミット・push** | 変更をコミットし、main / dev などブランチに push する。CI が走り、Create Release ワークフローで .aex がビルドされる。 |
| **4. .aex の取得** | [GitHub Releases](https://github.com/Aodaruma/aod-AE-plugin/releases) から対象ブランチの Windows 用 zip をダウンロードし、中身の .aex を取り出す。 |
| **5. AE で確認** | 取得した .aex を `Plug-ins\AOD\`（または使用中の AE の Plug-ins フォルダ）に配置し、After Effects を起動して動作確認する。 |

**ポイント**: ローカルは「コンパイル・静的解析」まで。**実機で動かす .aex は必ず CI ビルドのもの**を使う。PiPL やビルド設定を変えたときは `cargo clean` してから再ビルドする（`docs/GITHUB_CI_CHECKLIST.md` 参照）。

---

## まとめ

- **AE SDK の直接配置・CMake・AE_SDK_PATH**: 本リポジトリでは不要（Rust + after-effects クレートで完結）。
- **VS 2022・Rust・just**: 必須に近い。
- **デバッグ用 launch.json・シンボリックリンク**: `.vscode/launch.json` と `create_plugin_symlink.bat` で対応済み。
- **C/C++ 拡張・CMake Tools**: Rust 主体のため通常は不要。launch.json の「Build, Deploy & Launch AE」「Attach to After Effects」を使う場合は Windows で C/C++ 拡張（cppvsdbg）があると便利。
