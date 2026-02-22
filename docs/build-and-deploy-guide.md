# ビルド・デプロイスクリプト 社内配布用説明書

After Effects 用プラグイン **island_id_color** を、開発環境からビルドして After Effects の Plug-ins フォルダへ自動で配置するための手順書です。  
社内でプロジェクトを共有・配布する際のセットアップと運用方法をまとめています。

---

## 1. 概要

### 1.1 このスクリプトでできること

- **一括実行**で次を行います。
  1. Visual Studio の x64 ビルド環境を有効化（vcvars64.bat）
  2. `cargo clean` の後に `cargo build --release` でワークスペース全体をビルド
  3. 既存の `.aex` を削除したうえで、ビルド成果物（`.dll`）を AE の Plug-ins フォルダへ `island_id_color.aex` としてコピー

手動で「x64 ツール起動 → cargo build → リネーム → コピー」を行う必要がなくなります。  
※ スクリプト内のメッセージはコマンドプロンプトの文字化けを避けるため英語表記のみです。

### 1.2 配布物の構成

社内で配布する場合は、少なくとも次のファイル・フォルダを含めてください。

| 対象 | 説明 |
|------|------|
| `build_and_deploy.bat` | ビルド＆デプロイ用バッチ（プロジェクトルート） |
| `.vscode/tasks.json` | 任意。Cursor / VS Code から「Run Task」で実行する場合に使用 |
| `docs/build-and-deploy-guide.md` | 本説明書 |
| プロジェクト一式 | `plugins/island-id-color/` および `crates/`、`Cargo.toml` など、`cargo build` に必要な全体 |

※ ビルド成果物（`target/`）は配布しなくてかまいません。受け取り側でスクリプトを実行すれば生成されます。

---

## 2. 必要環境

スクリプトを実行する PC に、以下がインストールされている必要があります。

| 項目 | 内容 |
|------|------|
| **OS** | Windows（x64） |
| **Rust** | [rustup](https://www.rust-lang.org/tools/install) でインストールした Rust（`cargo` がパスを通った状態） |
| **Visual Studio** | 2022 の **Build Tools** または **Community / Professional / Enterprise**。  
「C++ によるデスクトップ開発」または「C++ 用ビルドツール」で **x64 用のツール** が入っていること。  
※ PiPL 埋め込み用に `rc.exe` 等を使用するため必要です。 |
| **After Effects** | プラグインを配置する After Effects（2023 / 2024 / 2025 など）がインストールされていること。 |

---

## 3. 初回設定（パスの変更）

`build_and_deploy.bat` を**テキストエディタで開き**、次の 2 か所を**各 PC の環境に合わせて**書き換えてください。

### 3.1 Visual Studio のパス（vcvars64.bat）

```batch
set "VCVARS=C:\Program Files\Microsoft Visual Studio\18\Community\VC\Auxiliary\Build\vcvars64.bat"
```

- スクリプトでは上記のように `18` や `2022` など、実際のインストール先のフォルダ名が使われている場合があります。
- **Visual Studio 2022** の場合は `2022`、**Professional** の場合は `Professional`、**Enterprise** の場合は `Enterprise` に合わせて変更してください。
- **別ドライブや別パス**にインストールしている場合は、その PC 上の  
  `VC\Auxiliary\Build\vcvars64.bat` のフルパスに合わせてください。

### 3.2 After Effects の Plug-ins フォルダ

```batch
set "AE_PLUGINS=C:\Program Files\Adobe\Common\Plug-ins\7.0\MediaCore\AOD"
```

- スクリプトでは **Adobe Common** の共通 Plug-ins フォルダ（上記）を例にしています。
- 単体の After Effects を使う場合は、  
  `C:\Program Files\Adobe\Adobe After Effects 2024\Support Files\Plug-ins` のように、使用する **AE のバージョン**（2023 / 2024 / 2025 など）に合わせたパスに変更してください。
- インストール先が違う場合（例: D ドライブ）は、その PC の実際のパスに合わせてください。

保存後、スクリプトは何度でも同じ設定で実行できます。

---

## 4. 実行方法

### 4.1 コマンドプロンプトまたは PowerShell から実行

1. プロジェクトの**ルートフォルダ**（`build_and_deploy.bat` があるフォルダ）を開く。
2. コマンドプロンプトまたは PowerShell で、そのフォルダに移動する。
3. 次を実行する。**先頭の `.\` を必ず付ける**（付けないと「認識されません」エラーになります）。

   ```batch
   .\build_and_deploy.bat
   ```

   PowerShell で実行ポリシーエラーになる場合や、確実に cmd で実行したい場合:

   ```powershell
   cmd /c build_and_deploy.bat
   ```

### 4.2 Cursor / VS Code のタスクから実行（一撃で実行）

1. Cursor または VS Code でプロジェクトを開く。
2. **Ctrl + Shift + P** を押し、「**Tasks: Run Task**」を選択。
3. 一覧から「**Build & Deploy island_id_color**」を選ぶ。

または **Ctrl + Shift + B**（ビルドタスクの実行）でも同じタスクが実行されます。

※ `.vscode/tasks.json` がプロジェクトに含まれている場合のみ利用できます。

---

## 5. 実行結果の確認

- 成功時は、最後に **`DONE!`** と表示され、キー入力待ち（pause）で終了します。
- 失敗時は、その時点で表示されたエラー内容を確認し、次の「トラブルシューティング」を参照して対応してください。

After Effects でプラグインを利用する場合は、**AE を一度終了してから**スクリプトを実行し、その後 AE を起動し直すと確実です（既に AE が起動中のままコピーしても、次回起動時には読み込まれます）。

---

## 6. トラブルシューティング

| 現象 | 確認・対処 |
|------|------------|
| `vcvars64.bat not found` | 3.1 の Visual Studio パスが、その PC のインストール先と一致しているか確認し、`build_and_deploy.bat` の `VCVARS` を修正する。 |
| `cargo build failed` | Rust がインストールされ、`cargo` がパスを通しているか確認する。  
  ターミナルで `cargo --version` が動くか試す。  
  ビルドエラー内容に従ってコードや依存関係を修正する。 |
| `AE Plug-ins folder not found` | 3.2 の `AE_PLUGINS` が、その PC の After Effects 用 Plug-ins フォルダ（Common または各 AE の Support Files\Plug-ins）の実際のパスと一致しているか確認し、`build_and_deploy.bat` を修正する。フォルダが無い場合はスクリプトが自動作成します。 |
| `Copy failed` / コピーできない | Plug-ins フォルダへの書き込みには**管理者権限**が必要な場合があります。  
  コマンドプロンプトを「管理者として実行」してから、同じ手順で `build_and_deploy.bat` を実行する。 |
| AE でプラグインが表示されない | AE のバージョンが x64 か確認する。  
  配置先が正しく「Support Files\Plug-ins」直下か確認する。  
  AE を再起動してからもう一度「エフェクト」メニュー等を確認する。 |
| **「必要なPiPLプロパティが見つかりません (25, 31)」** | PiPL の OutFlags/OutFlags2 が DLL に埋め込まれていない状態です。  
  **1)** After Effects を終了する。  
  **2)** プロジェクトルートで `cargo clean -p island_id_color` を実行。  
  **3)** `cargo build --release -p island_id_color` で再ビルド。  
  **4)** 生成された `target\release\island_id_color.dll` を Plug-ins フォルダに `island_id_color.aex` として上書きコピー。  
  **5)** After Effects を起動し直す。  
  （build.rs の PiPL 変更後は必ずクリーンビルドが必要です。）  
  **それでも (25, 31) が出る場合:** Windows では PiPL を**ビッグエンディアン**で埋め込む必要があります。  
  **`docs/pipl-windows-byteorder-fix.md`** の手順に従い、pipl を 1 行修正したフォークを [patch] で差し替えてから再ビルドしてください。 |
| **「実行中にエラーが発生しました」＋ &lt;17000&gt; &lt;MEE_Plugins&gt; &lt;5&gt; … MediaCore** | AE が Common の `Plug-ins\7.0\MediaCore` をスキャン／読み込みする際に失敗しています。  
  **対処 A:** **バージョン別の Plug-ins フォルダに配置して試す**（推奨）。  
  `build_and_deploy.bat` の `AE_PLUGINS` を、使用中の AE のフォルダに変更する。  
  例: `set "AE_PLUGINS=C:\Program Files\Adobe\Adobe After Effects 2024\Support Files\Plug-ins"`  
  その後、AE を終了 → バッチを再実行 → AE を起動し直す。  
  **対処 B:** 上記で直る場合、その PC／AE では Common\MediaCore ではなくバージョン別フォルダの利用を推奨する。  
  **対処 C:** エラーが「プラグイン適用時」なら、Visual C++ 2015–2022 再頒布可能パック（x64）のインストールを確認する。 |

---

## 7. 注意事項（社内利用向け）

- **権限**  
  After Effects の Plug-ins フォルダは `C:\Program Files\...` 以下のことが多く、コピー時に管理者権限が必要な場合があります。  
  社内のセキュリティポリシーに合わせて、管理者実行の可否を確認してください。

- **AE のバージョン**  
  スクリプトは「指定した 1 つの Plug-ins フォルダ」にしかコピーしません。  
  複数バージョンの AE（2023 / 2024 / 2025）を使う場合は、都度 `AE_PLUGINS` を書き換えるか、バッチをコピーしてバージョン別に用意するなどの運用を推奨します。

- **上書き**  
  実行のたびに、指定した Plug-ins フォルダ内の既存の `island_id_color.aex` は削除されたうえで、新しいビルドで上書きされます。  
  必要な場合は、実行前に手動でバックアップを取ってください。

- **ウイルス対策**  
  社内のリアルタイムスキャンが `.dll` / `.aex` を検知することがあります。  
  誤検知の場合は、プロジェクトフォルダや Plug-ins フォルダを除外する等、社内ルールに従って対応してください。

---

## 8. 変更履歴

| 日付 | 内容 |
|------|------|
| 初版 | 社内配布用説明書として作成（ビルド・デプロイ手順、初回設定、トラブルシューティング） |
| 改訂 | スクリプトを英語メッセージ化・cargo clean フルビルド・パス例（VCVARS=18 / AE_PLUGINS=Common）に合わせてガイドを更新 |

---

不明点や社内環境に合わせた追加の手順が必要な場合は、開発担当までお問い合わせください。
