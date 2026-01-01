use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use chrono::Datelike;
use fs_extra::dir::{copy as copy_dir, CopyOptions};
use std::{fs, path::{Path, PathBuf}};
use toml_edit::{value, DocumentMut};
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(name = "xtask")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Create a new AE plugin crate from template.
    NewPlugin {
        /// Plugin name input. Accepts snake_case / kebab-case / CamelCase.
        name: String,

        /// AE major version (for README and cfg scaffolding)
        #[arg(long, default_value = "25")]
        ae: u32,

        /// Include OpenCV feature (as default feature for the generated plugin crate)
        #[arg(long)]
        opencv: bool,

        /// Include FFT feature (as default feature for the generated plugin crate)
        #[arg(long)]
        fft: bool,

        /// Include GPU feature (as default feature for the generated plugin crate)
        #[arg(long)]
        gpu: bool,

        /// Template directory
        #[arg(long, default_value = "templates/plugin")]
        template_dir: String,

        /// Destination plugins directory
        #[arg(long, default_value = "plugins")]
        plugins_dir: String,
    },

    /// Run formatting, clippy, and tests (and optionally plugin builds).
    Ci {
        /// Also build plugin crates (may require AE SDK env on your machine/CI).
        #[arg(long)]
        build_plugins: bool,
    },

    /// Package one plugin (or all) into dist/ (placeholder implementation).
    Package {
        /// Plugin crate name (e.g. aod_glow_key) or "all"
        #[arg(long, default_value = "all")]
        plugin: String,

        /// AE major version label used in artifact naming
        #[arg(long, default_value = "25")]
        ae: u32,

        /// Comma-separated features to enable (e.g. "opencv,fft")
        #[arg(long)]
        features: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.cmd {
        Cmd::NewPlugin { name, ae, opencv, fft, gpu, template_dir, plugins_dir } => {
            cmd_new_plugin(&name, ae, opencv, fft, gpu, &template_dir, &plugins_dir)
        }
        Cmd::Ci { build_plugins } => cmd_ci(build_plugins),
        Cmd::Package { plugin, ae, features } => cmd_package(&plugin, ae, features.as_deref()),
    }
}

fn cmd_new_plugin(
    input: &str,
    ae: u32,
    opencv: bool,
    fft: bool,
    gpu: bool,
    template_dir: &str,
    plugins_dir: &str,
) -> Result<()> {
    let naming = Naming::from_input(input)?;

    let template_dir = PathBuf::from(template_dir);
    let dest_dir = PathBuf::from(plugins_dir).join(&naming.crate_name);

    if dest_dir.exists() {
        anyhow::bail!("Destination already exists: {}", dest_dir.display());
    }
    if !template_dir.exists() {
        anyhow::bail!("Template directory not found: {}", template_dir.display());
    }

    // 1) copy template dir
    let mut opt = CopyOptions::new();
    opt.copy_inside = true;
    fs::create_dir_all(&dest_dir).context("create destination dir")?;
    copy_dir(&template_dir, &dest_dir, &opt).context("copy template")?;

    // 2) replace placeholders
    let year = chrono::Utc::now().year();
    let pairs = vec![
        ("{{CRATE_NAME}}".to_string(), naming.crate_name.clone()),
        ("{{PLUGIN_CAMEL}}".to_string(), naming.camel.clone()),
        ("{{AE_PLUGIN_NAME}}".to_string(), naming.ae_name.clone()),
        ("{{YEAR}}".to_string(), year.to_string()),
        ("{{AE_VERSION}}".to_string(), ae.to_string()),
    ];
    replace_placeholders_recursively(&dest_dir, &pairs)?;

    // 3) set default features in generated plugin Cargo.toml (optional)
    let mut defaults = Vec::<&str>::new();
    if opencv { defaults.push("opencv"); }
    if fft { defaults.push("fft"); }
    if gpu { defaults.push("gpu"); }

    if !defaults.is_empty() {
        let cargo_toml = dest_dir.join("Cargo.toml");
        set_plugin_default_features(&cargo_toml, &defaults)?;
    }

    println!("Created: {}  (crate: {})", naming.ae_name, naming.crate_name);
    println!("Path: {}", dest_dir.display());
    Ok(())
}

fn cmd_ci(build_plugins: bool) -> Result<()> {
    // simple shell-out strategy; keep logic in one place for CI parity
    run("cargo", &["fmt", "--all", "--", "--check"])?;
    run("cargo", &["clippy", "--workspace", "--all-targets", "--", "-D", "warnings"])?;
    run("cargo", &["test", "--workspace"])?;

    if build_plugins {
        // This assumes the AE SDK is available/configured on the machine.
        run("cargo", &["build", "--workspace"])?;
    }
    Ok(())
}

fn cmd_package(plugin: &str, ae: u32, features: Option<&str>) -> Result<()> {
    // Placeholder packaging flow:
    // - build
    // - gather artifact (.aex/.dll) into dist/
    // - add LICENSE / THIRD_PARTY_NOTICES / SOURCE_OFFER
    //
    // Real packaging depends on how your AE plugin build emits artifacts.
    // Keep the "what to do" here and implement "where the artifact is" later.

    let mut args = vec!["build"];
    if plugin != "all" {
        args.extend(["-p", plugin]);
    }
    if let Some(f) = features {
        args.extend(["--features", f]);
    }
    run("cargo", &args)?;

    fs::create_dir_all("dist")?;
    let marker = PathBuf::from("dist").join(format!("PACKAGED_AE{}_{}.txt", ae, plugin));
    fs::write(marker, "packaging placeholder\n")?;

    println!("Packaged (placeholder): plugin={} ae={} features={:?}", plugin, ae, features);
    Ok(())
}

fn run(cmd: &str, args: &[&str]) -> Result<()> {
    use std::process::Command;
    let status = Command::new(cmd)
        .args(args)
        .status()
        .with_context(|| format!("failed to spawn: {} {:?}", cmd, args))?;
    if !status.success() {
        anyhow::bail!("command failed: {} {:?}", cmd, args);
    }
    Ok(())
}

fn replace_placeholders_recursively(root: &Path, pairs: &[(String, String)]) -> Result<()> {
    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() { continue; }
        let path = entry.path();

        // naive binary skip
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if ["png", "jpg", "jpeg", "zip", "aex", "dll", "lib"].contains(&ext) {
                continue;
            }
        }

        let content = fs::read_to_string(path);
        if let Ok(mut text) = content {
            for (k, v) in pairs {
                text = text.replace(k, v);
            }
            fs::write(path, text)?;
        }
    }
    Ok(())
}

fn set_plugin_default_features(cargo_toml_path: &Path, defaults: &[&str]) -> Result<()> {
    let raw = fs::read_to_string(cargo_toml_path)
        .with_context(|| format!("read {}", cargo_toml_path.display()))?;
    let mut doc: DocumentMut = raw.parse().context("parse Cargo.toml")?;

    // ensure [features] exists
    if !doc.as_table().contains_key("features") {
        doc["features"] = toml_edit::table();
    }
    // set default = ["opencv","fft",...]
    let arr = defaults.iter().map(|s| value(*s)).collect::<Vec<_>>();
    doc["features"]["default"] = toml_edit::Item::Value(toml_edit::Value::Array(arr.into_iter().collect()));

    fs::write(cargo_toml_path, doc.to_string())
        .with_context(|| format!("write {}", cargo_toml_path.display()))?;
    Ok(())
}

struct Naming {
    camel: String,
    crate_name: String,
    ae_name: String,
}

impl Naming {
    fn from_input(input: &str) -> Result<Self> {
        let camel = to_camel_case(input);
        if camel.is_empty() {
            anyhow::bail!("Invalid plugin name input: {}", input);
        }
        let snake = to_snake_case(&camel);
        Ok(Self {
            camel: camel.clone(),
            crate_name: format!("aod_{}", snake),
            ae_name: format!("AOD_{}", camel),
        })
    }
}

fn to_camel_case(s: &str) -> String {
    s.split(|c: char| c == '_' || c == '-' || c == ' ')
        .filter(|p| !p.is_empty())
        .map(|p| {
            let mut ch = p.chars();
            match ch.next() {
                None => String::new(),
                Some(f) => {
                    let rest = ch.as_str();
                    f.to_uppercase().collect::<String>() + &rest.to_lowercase()
                }
            }
        })
        .collect::<Vec<_>>()
        .join("")
}

fn to_snake_case(camel: &str) -> String {
    let mut out = String::new();
    for (i, ch) in camel.chars().enumerate() {
        if ch.is_uppercase() && i != 0 {
            out.push('_');
        }
        out.push(ch.to_ascii_lowercase());
    }
    out
}