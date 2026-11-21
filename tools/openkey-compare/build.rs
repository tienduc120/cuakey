//! Compiles `OpenKey`'s engine from the sources `cargo xtask compare-openkey` fetches into the repository's
//! `target/openkey/`, and the shim that drives it. Nothing of `OpenKey` is kept in this repository.

use std::fs;
use std::path::{Path, PathBuf};

/// Where the fetched checkout keeps the engine, relative to this manifest.
const ENGINE_DIR: &str = "../../target/openkey/Sources/OpenKey/engine";
const SHIM: &str = "shim/shim.cpp";
/// Definitions the engine's sources need to build with MSVC as C++17: no deprecation error for `<codecvt>`, and no
/// `std::byte` to clash with the `byte` of `<windows.h>`.
const DEFINES: [(&str, &str); 2] = [
    ("_SILENCE_CXX17_CODECVT_HEADER_DEPRECATION_WARNING", "1"),
    ("_HAS_STD_BYTE", "0"),
];

fn main() -> Result<(), String> {
    let engine = Path::new(env!("CARGO_MANIFEST_DIR")).join(ENGINE_DIR);
    println!("cargo::rerun-if-changed={SHIM}");
    println!("cargo::rerun-if-changed={}", engine.display());
    let sources = engine_sources(&engine)?;

    cpp_build()
        .files(&sources)
        // The engine is someone else's code, compiled as it is: its warnings are not ours to fix.
        .warnings(false)
        .try_compile("openkey_engine")
        .map_err(|e| format!("compiling OpenKey's engine failed: {e}"))?;
    cpp_build()
        .include(&engine)
        .file(SHIM)
        .warnings_into_errors(true)
        .try_compile("openkey_shim")
        .map_err(|e| format!("compiling the shim failed: {e}"))
}

/// C++17 with exceptions, as the engine is written.
fn cpp_build() -> cc::Build {
    let mut build = cc::Build::new();
    build.cpp(true).std("c++17").flag_if_supported("/EHsc");
    for (name, value) in DEFINES {
        build.define(name, value);
    }
    build
}

/// Every C++ source of the engine, or an error that says how to fetch them.
fn engine_sources(engine: &Path) -> Result<Vec<PathBuf>, String> {
    let missing = || {
        format!(
            "OpenKey's engine sources are not in {}; run `cargo xtask compare-openkey`, which fetches them",
            engine.display()
        )
    };
    let entries = fs::read_dir(engine).map_err(|_| missing())?;
    let mut sources: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "cpp"))
        .collect();
    if sources.is_empty() {
        return Err(missing());
    }
    sources.sort();
    Ok(sources)
}
