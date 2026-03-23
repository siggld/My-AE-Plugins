//! macOS / Linux plugin bundle Info.plist generation.
//! Used when building AE/Pr plugins for .plugin bundle structure.

use std::fs;
use std::path::Path;

use super::PIPLType;

/// XML エスケープ
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// AE/Pr プラグインバンドル用の Info.plist を生成する。
/// path: 出力先の Info.plist パス
/// _kind: プラグイン種別 (PIPLType) — PkgInfo は呼び出し元で生成済み
/// name: プラグインの表示名
pub fn produce_plist(path: impl AsRef<Path>, _kind: &PIPLType, name: &str) {
    let path = path.as_ref();
    let exe_name = std::env::var("CARGO_PKG_NAME").unwrap_or_else(|_| "plugin".into());
    let ident = format!(
        "com.adobe.aftereffects.plugin.{}",
        exe_name.replace('-', "_")
    );
    let name_esc = escape_xml(name);
    let exe_esc = escape_xml(&exe_name);
    let ident_esc = escape_xml(&ident);

    // CFBundlePackageType: PkgInfo と整合させる (kind 4 bytes + FXTC)
    // AE effects では kind が eFKT などのため、plist 側は汎用 BNDL を用いる
    let package_type = "BNDL";

    let plist = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>CFBundleDevelopmentRegion</key>
	<string>en</string>
	<key>CFBundleExecutable</key>
	<string>{exe_esc}</string>
	<key>CFBundleIdentifier</key>
	<string>{ident_esc}</string>
	<key>CFBundleName</key>
	<string>{name_esc}</string>
	<key>CFBundlePackageType</key>
	<string>{package_type}</string>
	<key>CFBundleVersion</key>
	<string>1.0.0</string>
</dict>
</plist>
"#,
        exe_esc = exe_esc,
        ident_esc = ident_esc,
        name_esc = name_esc,
        package_type = package_type
    );

    fs::write(path, plist).expect("Failed to write Info.plist");
}
