//! Writes the icons, the version information and the manifest into the Windows executable. The tray shows icon 2 for
//! Vietnamese and icon 3 for English; icon 1 is the executable's own.

#[cfg(windows)]
fn main() -> std::io::Result<()> {
    // Paths from the manifest folder, fixed when this script is built, so the resource compiler finds them from any
    // folder it runs in.
    winresource::WindowsResource::new()
        .set_icon(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../icon/icon_V.ico"))
        .set_icon_with_id(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../icon/icon_V.ico"), "2")
        .set_icon_with_id(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../icon/icon_E.ico"), "3")
        .set("ProductName", "CuaKey")
        .set("FileDescription", "CuaKey - Bộ gõ Tiếng Việt")
        .set("OriginalFilename", "cuakey.exe")
        .set("LegalCopyright", "MIT")
        .set_manifest_file(concat!(env!("CARGO_MANIFEST_DIR"), "/cuakey.manifest"))
        .compile()
}

/// Only a Windows executable carries these resources.
#[cfg(not(windows))]
fn main() {}
