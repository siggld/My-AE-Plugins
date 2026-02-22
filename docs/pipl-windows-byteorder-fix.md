# PiPL (25, 31) エラー：Windows でのバイトオーダー修正

After Effects で「必要なPiPLプロパティが見つかりません (25, 31)」が出る場合、**PiPL が Windows でリトルエンディアンで埋め込まれている**ことが原因です。  
Adobe の仕様では PiPL は **常に macOS（ビッグエンディアン）のバイトオーダー**で記述する必要があります。

## 現在の対応（フォーク不要）

このプロジェクトでは **`crates/pipl`** に修正版 pipl を同梱しています。  
- Windows でも **BigEndian** で PiPL を出力するよう変更（`LittleEndian` → `BigEndian`）。  
- Windows 用の先頭 2 バイト（Reserved）を**出力しない**ように変更。PiPL は macOS と同じバイナリ形式（先頭が kPIPropertiesVersion）にし、「PiPLバージョンが一致しません」を防ぐ。  
- ルート `Cargo.toml` の `[workspace.dependencies]` で `pipl = { path = "crates/pipl" }` を指定。

**手順:** プロジェクトルートで次を実行し、生成された DLL を AE の Plug-ins に `.aex` としてコピーしてください。

```bash
cargo clean -p island_id_color
cargo build --release -p island_id_color
```

---

## 原因（参考）

- 依存している [virtualritz/after-effects](https://github.com/virtualritz/after-effects) の **pipl** クレートが、Windows で `LittleEndian` を使って PiPL を出力している。
- AE は PiPL をビッグエンディアンとして解釈するため、プロパティ (25) OutFlags と (31) OutFlags2 を正しく認識できない。

## 対処：pipl をビッグエンディアンに差し替える

pipl のソースを **Windows でも BigEndian を使う**ように 1 行だけ変更した上で、Cargo の [patch] でその変更を反映します。

### 手順

1. **after-effects リポジトリをフォークする**  
   https://github.com/virtualritz/after-effects を自分の GitHub アカウントで Fork する。

2. **フォーク内で pipl の 1 行を修正する**  
   - リポジトリ内の **`pipl/src/lib.rs`** を開く。
   - 先頭付近（9〜10 行目付近）の次の行を探す：
     ```rust
     #[cfg(target_os = "windows")]
     use byteorder::LittleEndian as ByteOrder;
     ```
   - 2 行目を次のように変更する：
     ```rust
     #[cfg(target_os = "windows")]
     use byteorder::BigEndian as ByteOrder;
     ```
   - 変更をコミットし、任意のブランチ名（例: `pipl-windows-bigendian`）でプッシュする。

3. **このプロジェクトのルート `Cargo.toml` に [patch] を追加する**  
   次のブロックを **`[workspace.dependencies]` の前** に追加する（`YOUR_GITHUB_USERNAME` を自分のユーザ名に置き換える）。

   ```toml
   [patch."https://github.com/virtualritz/after-effects.git"]
   after-effects = { git = "https://github.com/YOUR_GITHUB_USERNAME/after-effects.git", branch = "pipl-windows-bigendian" }
   pipl = { git = "https://github.com/YOUR_GITHUB_USERNAME/after-effects.git", branch = "pipl-windows-bigendian" }
   ```

4. **クリーンビルドとデプロイ**  
   - After Effects を終了する。
   - プロジェクトルートで実行：
     ```bash
     cargo clean -p island_id_color
     cargo build --release -p island_id_color
     ```
   - 生成された `target\release\island_id_color.dll` を、AE の Plug-ins フォルダに `island_id_color.aex` としてコピーする。
   - After Effects を起動し直す。

これで PiPL がビッグエンディアンで埋め込まれ、(25, 31) エラーが解消される想定です。

## 参考

- Adobe: “PiPL properties must always be in **macOS-specific byte order**”  
  （Windows 用ビルドでも PiPL の中身はビッグエンディアンで記述する必要がある）
