//! A system font that covers Vietnamese. egui draws no text without a font, and its bundled fonts carry licences this
//! repository refuses (ADR-0015), so the window borrows one the system already has.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use egui::{FontData, FontDefinitions, FontFamily};

/// In order of preference; all cover Vietnamese.
const CANDIDATES: [&str; 3] = ["segoeui.ttf", "tahoma.ttf", "arial.ttf"];

#[derive(Debug, thiserror::Error)]
pub enum FontError {
    #[error("no font that covers Vietnamese could be read from {}", .folder.display())]
    NoneLoaded { folder: PathBuf },
}

/// Installs the first readable system font as the proportional and monospace font.
///
/// # Errors
///
/// Returns [`FontError::NoneLoaded`] when none of the fonts can be read from `fonts_dir`; the window would draw no
/// text, so the caller has to know.
pub fn install_system_font(ctx: &egui::Context, fonts_dir: &Path) -> Result<(), FontError> {
    let (name, bytes) = read_first_font(fonts_dir)?;
    let mut fonts = FontDefinitions::empty();
    fonts
        .font_data
        .insert(name.clone(), Arc::new(FontData::from_owned(bytes)));
    for family in [FontFamily::Proportional, FontFamily::Monospace] {
        fonts.families.entry(family).or_default().push(name.clone());
    }
    ctx.set_fonts(fonts);
    Ok(())
}

fn read_first_font(fonts_dir: &Path) -> Result<(String, Vec<u8>), FontError> {
    CANDIDATES
        .iter()
        .find_map(|name| {
            // A font that is missing or unreadable is not an error: the next candidate is tried.
            let bytes = std::fs::read(fonts_dir.join(name)).ok()?;
            Some(((*name).to_owned(), bytes))
        })
        .ok_or_else(|| FontError::NoneLoaded {
            folder: fonts_dir.to_path_buf(),
        })
}

#[cfg(test)]
mod tests;
