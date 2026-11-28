//! `cargo xtask msix`: packs the release build into the MSIX package the Microsoft Store signs and delivers
//! (ADR-0017). Nothing is signed here. `--register` registers the staged files with Windows instead, which Developer
//! Mode allows, to try the packaged app on this PC.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context as _, Result, bail};

const MANIFEST: &str = "services/cuakey/desktop/package/AppxManifest.xml";
const ASSETS: &str = "services/cuakey/desktop/package/Assets";
const RELEASE_EXE: &str = "target/release/cuakey.exe";
const OUT_DIR: &str = "target/msix";
/// Where the package's files are gathered, and where `--register` leaves them for Windows to run from.
const STAGING: &str = "target/msix/layout";
/// The Windows SDK's tools, one folder per SDK version, on a developer's PC and on the CI runner alike.
const SDK_BIN: &str = r"C:\Program Files (x86)\Windows Kits\10\bin";
/// The text in the manifest the version is written over, once.
const VERSION_PLACEHOLDER: &str = r#"Version="0.0.0.0""#;

/// Stages the package from `exe` (the release build when `None`), then packs it, or registers it when `register`.
pub(crate) fn run(root: &Path, exe: Option<&Path>, register: bool, out: &mut impl Write) -> Result<bool> {
    let cargo_version = workspace_version(root)?;
    let version = package_version(&cargo_version)?;
    let exe = exe.map_or_else(|| root.join(RELEASE_EXE), Path::to_path_buf);
    if !exe.is_file() {
        bail!(
            "{} is missing; build it first with `cargo build --release --locked -p cuakey-desktop`",
            exe.display()
        );
    }
    let staging = root.join(STAGING);
    stage(root, &exe, &version, &staging)?;
    if register {
        register_layout(&staging)?;
        writeln!(out, "registered CuaKey {version} from {}", staging.display())?;
    } else {
        let package = root.join(OUT_DIR).join(format!("cuakey-v{cargo_version}-x64.msix"));
        pack(&staging, &package)?;
        writeln!(out, "packed {}", package.display())?;
    }
    Ok(true)
}

/// The version every crate inherits from the root manifest.
fn workspace_version(root: &Path) -> Result<String> {
    let path = root.join("Cargo.toml");
    let text = fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    let table: toml::Table = text.parse().with_context(|| format!("parsing {}", path.display()))?;
    table
        .get("workspace")
        .and_then(|workspace| workspace.get("package"))
        .and_then(|package| package.get("version"))
        .and_then(toml::Value::as_str)
        .map(str::to_owned)
        .context("the root Cargo.toml has no [workspace.package] version")
}

/// Gathers the manifest, stamped with `version`, the executable and the logos in `staging`, emptied first.
fn stage(root: &Path, exe: &Path, version: &str, staging: &Path) -> Result<()> {
    if staging.exists() {
        fs::remove_dir_all(staging).with_context(|| format!("emptying {}", staging.display()))?;
    }
    let assets = staging.join("Assets");
    fs::create_dir_all(&assets).with_context(|| format!("creating {}", assets.display()))?;
    let manifest = fs::read_to_string(root.join(MANIFEST)).with_context(|| format!("reading {MANIFEST}"))?;
    fs::write(staging.join("AppxManifest.xml"), stamp_version(&manifest, version)?)?;
    fs::copy(exe, staging.join("cuakey.exe")).with_context(|| format!("copying {}", exe.display()))?;
    for entry in fs::read_dir(root.join(ASSETS)).with_context(|| format!("reading {ASSETS}"))? {
        let entry = entry?;
        fs::copy(entry.path(), assets.join(entry.file_name()))?;
    }
    Ok(())
}

/// Packs `staging` into `package` with the newest Windows SDK's `makeappx`.
fn pack(staging: &Path, package: &Path) -> Result<()> {
    let makeappx = makeappx()?;
    let status = Command::new(&makeappx)
        .arg("pack")
        .arg("/o")
        .arg("/d")
        .arg(staging)
        .arg("/p")
        .arg(package)
        .status()
        .with_context(|| format!("running {}", makeappx.display()))?;
    if !status.success() {
        bail!("makeappx could not pack the package ({status})");
    }
    Ok(())
}

fn makeappx() -> Result<PathBuf> {
    let folders: Vec<String> = fs::read_dir(SDK_BIN)
        .with_context(|| format!("no Windows SDK in {SDK_BIN}; install it with Visual Studio Build Tools"))?
        .filter_map(|entry| entry.ok()?.file_name().into_string().ok())
        .collect();
    let sdk = newest_sdk(folders.iter().map(String::as_str)).context("no versioned Windows SDK in its bin folder")?;
    Ok(Path::new(SDK_BIN).join(sdk).join("x64").join("makeappx.exe"))
}

/// Registers the package staged in `staging`, replacing a registration of any version.
fn register_layout(staging: &Path) -> Result<()> {
    let manifest = staging.join("AppxManifest.xml");
    // Windows PowerShell, since the Appx module does not load in PowerShell 7 everywhere.
    let status = Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command"])
        .arg(format!(
            "Add-AppxPackage -Register '{}' -ForceApplicationShutdown -ForceUpdateFromAnyVersion",
            manifest.display()
        ))
        .status()
        .context("running Windows PowerShell")?;
    if !status.success() {
        bail!("Windows refused to register the package ({status}); is Developer Mode on?");
    }
    Ok(())
}

/// The package version for the Cargo version `cargo_version`: its three parts and a fourth of zero, which the Store
/// requires.
pub(crate) fn package_version(cargo_version: &str) -> Result<String> {
    let parts: Vec<&str> = cargo_version.split('.').collect();
    if parts.len() != 3 {
        bail!("the version {cargo_version} needs exactly three parts to become a package version");
    }
    for part in &parts {
        if part.parse::<u16>().is_err() {
            bail!("the version {cargo_version} has the part `{part}`, and a package version holds numbers up to 65535");
        }
    }
    Ok(format!("{cargo_version}.0"))
}

/// `manifest` with `version` written over its placeholder.
pub(crate) fn stamp_version(manifest: &str, version: &str) -> Result<String> {
    let found = manifest.matches(VERSION_PLACEHOLDER).count();
    if found != 1 {
        bail!(
            "the manifest must hold {VERSION_PLACEHOLDER} exactly once to take the version; it holds it {found} times"
        );
    }
    Ok(manifest.replace(VERSION_PLACEHOLDER, &format!(r#"Version="{version}""#)))
}

/// The newest Windows SDK among the folder names in its `bin` folder.
pub(crate) fn newest_sdk<'a>(folders: impl IntoIterator<Item = &'a str>) -> Option<&'a str> {
    folders
        .into_iter()
        .filter_map(|name| sdk_version(name).map(|version| (version, name)))
        .max()
        .map(|(_, name)| name)
}

/// The four numbers of an SDK folder name such as `10.0.26100.0`, or `None` for any other folder.
fn sdk_version(name: &str) -> Option<[u32; 4]> {
    let mut numbers = name.split('.').map(|part| part.parse::<u32>().ok());
    let version = [numbers.next()??, numbers.next()??, numbers.next()??, numbers.next()??];
    numbers.next().is_none().then_some(version)
}

#[cfg(test)]
mod tests;
