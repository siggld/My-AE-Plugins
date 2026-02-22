# 技術連携メモ：PiPL (25, 31) エラーおよび AE 実行時エラー

シニア・アーキテクト向けの連携用まとめです。  
After Effects プラグイン **AOD_IslandIdColor**（island_id_color）で発生した PiPL 関連エラーと、Cursor 上で実施した対応、および未解決の実行時エラーについて記載しています。

---

## 1. 発生したエラー一覧

| 種類 | メッセージ（要約） | 状態 |
|------|-------------------|------|
| PiPL プロパティ | エフェクト AOD_IslandIdColor に**必要なPiPLプロパティが見つかりません。(25, 31)** | **対応済み**（後述） |
| PiPL バージョン | エフェクト AOD_IslandIdColor の**PiPLバージョンが一致しません。(25, 31)** | **対応済み**（後述） |
| 実行時エラー | **実行中にエラーが発生しました。最後にログされたメッセージ : &lt;39080&gt; &lt;MEE_Plugins&gt; &lt;5&gt; C:\Program Files\Adobe\Common\Plug-ins\7.0\MediaCore** | **未解決**（要調査） |
| 実行時エラー（別件） | 同様に **&lt;17000&gt; &lt;MEE_Plugins&gt; &lt;5&gt; … MediaCore** のログ | バージョン別 Plug-ins への配置で回避可能と案内済み |

※ (25, 31) は Adobe の PiPL におけるプロパティ ID：**25 = OutFlags（AE_Effect_Global_OutFlags）、31 = OutFlags2（AE_Effect_Global_OutFlags_2）** を指します。

---

## 2. PiPL (25, 31) の問題の原因と対応（Cursor で実施した内容）

### 2.1 原因の整理

- **元依存:** [virtualritz/after-effects](https://github.com/virtualritz/after-effects) の **pipl** クレートで PiPL リソースを生成。
- **仕様:** Adobe の PiPL は「**常に macOS（ビッグエンディアン）のバイトオーダー**」で記述する必要があるとされている。
- **実態:**
  - 上記 pipl は **Windows で LittleEndian** を使って PiPL を出力していた。
  - さらに **Windows 用に先頭 2 バイト（Reserved）** を付加しており、PiPL の先頭が macOS と異なっていた。

このため、

1. **「必要なPiPLプロパティが見つかりません (25, 31)」**  
   → バイトオーダーの不一致で、AE が OutFlags/OutFlags2 を正しく認識できなかった。
2. **「PiPLバージョンが一致しません (25, 31)」**  
   → 先頭 2 バイトの有無により、AE が「バージョン」を誤ったオフセットで解釈した。

### 2.2 Cursor で実施した対応（要約）

フォークや外部パッチに頼らず、**プロジェクト内に修正版 pipl を置く**形で対応しました。

| 対応 | 内容 |
|------|------|
| **ローカル pipl の追加** | `crates/pipl` に virtualritz/after-effects の pipl をベースにした修正版を配置。 |
| **バイトオーダー** | `pipl/src/lib.rs` で Windows 時も **BigEndian** を使用するよう変更（`LittleEndian` → `BigEndian`）。 |
| **先頭 2 バイトの削除** | Windows 用に出力していた **先頭 2 バイト（Reserved）を出力しない**ように変更。PiPL バイナリを macOS と同じ形（先頭が kPIPropertiesVersion）に統一。 |
| **ワークスペースの差し替え** | ルート `Cargo.toml` の `[workspace.dependencies]` で `pipl = { path = "crates/pipl" }` を指定し、git 依存の pipl をローカルに差し替え。 |
| **build.rs** | `plugins/island-id-color/build.rs` では、必須フラグ（`PiplOverridesOutdataOutflags`、`DeepColorAware`、`SendUpdateParamsUI` および OutFlags2）を従来どおり指定。変更は pipl 側のみ。 |

この結果、**「必要なPiPLプロパティが見つかりません」** および **「PiPLバージョンが一致しません」** は解消される想定です。

### 2.3 参照ドキュメント・ファイル

- **詳細手順・背景:** `docs/pipl-windows-byteorder-fix.md`
- **修正版 pipl:** `crates/pipl/`（`src/lib.rs` の ByteOrder と先頭 2 バイト出力の削除）
- **ビルド・デプロイ:** `docs/build-and-deploy-guide.md`（クリーンビルド手順、トラブルシューティング）

---

## 3. 今回の実行時エラー（未解決）

### 3.1 ログ内容

- **メッセージ:**  
  「実行中にエラーが発生しました。最後にログされたメッセージ : **&lt;39080&gt; &lt;MEE_Plugins&gt; &lt;5&gt; C:\Program Files\Adobe\Common\Plug-ins\7.0\MediaCore**」
- **解釈（推測）:**  
  - エラーコード／メッセージ ID: **39080**（および以前の同種ログでは **17000**）  
  - コンテキスト: **MEE_Plugins**、サブコード **5**  
  - パス: **Adobe Common** の **Plug-ins\7.0\MediaCore**（共通プラグイン配置先）

### 3.2 想定される要因（調査のたたき台）

- **Common\MediaCore のスキャン／読み込み時の問題**  
  AE が `C:\Program Files\Adobe\Common\Plug-ins\7.0\MediaCore`（およびその配下）をスキャンまたは読み込む段階でエラーを記録している可能性。
- **配置場所の違い**  
  同一プラグインを **バージョン別の Plug-ins**（例: `Adobe After Effects 2024\Support Files\Plug-ins`）に配置した場合は発生せず、Common\MediaCore に配置した場合にのみ発生するかどうかの確認が有効。
- **権限・パス・他プラグイン**  
  当該フォルダへのアクセス権、パスの長さ・文字、他プラグインとの競合などの要因もあり得る。

### 3.3 これまでに案内した回避策

- `build_and_deploy.bat` の **AE_PLUGINS** を、Common ではなく **使用中の AE のバージョン別 Plug-ins フォルダ**に変更してデプロイする。  
  例: `set "AE_PLUGINS=C:\Program Files\Adobe\Adobe After Effects 2024\Support Files\Plug-ins"`
- 上記で解消する場合、その環境では **Common\MediaCore ではなくバージョン別フォルダの利用**を推奨する旨を `docs/build-and-deploy-guide.md` に記載済み。

---

## 4. 連携時に共有するとよい情報

- **エラーコード:** 39080（および 17000）の **正式な意味**（SDK や Adobe ドキュメント上の定義があれば）。
- **MEE_Plugins / サブコード 5** の仕様や、Common\MediaCore を参照する際の前提条件。
- **推奨デプロイ先:** 社内標準として「Common\MediaCore」と「バージョン別 Plug-ins」のどちらを推奨するか、または併用する場合の注意点。
- **再現手順:** エラーが出る操作（起動直後／エフェクト適用時／特定コンポなど）と、Plug-ins の配置パス。

---

## 5. 変更履歴（本ドキュメント）

| 日付 | 内容 |
|------|------|
| 初版 | PiPL (25, 31) の原因・対応まとめ、実行時エラー 39080 / 17000 の整理、シニア・アーキテクト連携用として作成 |
