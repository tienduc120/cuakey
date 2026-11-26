//! Each window's last place as `window-places.json`, beside the preferences but apart from them, so resetting the
//! preferences or copying them to another computer moves no window.

use std::collections::BTreeMap;
use std::future::ready;
use std::path::PathBuf;

use cuakey_application::{PlacedWindow, StoreError, WindowPlace, WindowPlaces, WindowPlacesStore};
use serde::{Deserialize, Serialize};

use crate::files::{FileError, read_if_present, write_replacing};

const FILE_NAME: &str = "window-places.json";

/// Every window that has a place.
const WINDOWS: [PlacedWindow; 4] = [
    PlacedWindow::Settings,
    PlacedWindow::Macros,
    PlacedWindow::Convert,
    PlacedWindow::AlwaysOn,
];

/// The name a window's place goes under in the file.
const fn name_of(window: PlacedWindow) -> &'static str {
    match window {
        PlacedWindow::Settings => "settings",
        PlacedWindow::Macros => "macros",
        PlacedWindow::Convert => "convert",
        PlacedWindow::AlwaysOn => "always_on",
    }
}

/// One window's place as the file stores it.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct PlaceFile {
    x: f32,
    y: f32,
    scale: f32,
}

impl From<WindowPlace> for PlaceFile {
    fn from(place: WindowPlace) -> Self {
        Self {
            x: place.x,
            y: place.y,
            scale: place.scale,
        }
    }
}

impl From<PlaceFile> for WindowPlace {
    fn from(file: PlaceFile) -> Self {
        Self {
            x: file.x,
            y: file.y,
            scale: file.scale,
        }
    }
}

/// Keeps each window's last place in `window-places.json` in a folder. A file it cannot make sense of gives no place
/// rather than an error, and the next save writes over it: a window without a place only opens where it opens by
/// default, which is not worth the user's attention.
#[derive(Debug)]
pub struct JsonWindowPlacesStore {
    dir: PathBuf,
}

impl JsonWindowPlacesStore {
    pub const fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    fn path(&self) -> PathBuf {
        self.dir.join(FILE_NAME)
    }

    fn load_blocking(&self) -> Result<WindowPlaces, FileError> {
        Ok(read_if_present(&self.path())?
            .map(|bytes| parse_places(&bytes))
            .unwrap_or_default())
    }

    fn save_blocking(&self, places: &WindowPlaces) -> Result<(), FileError> {
        let file: BTreeMap<&str, PlaceFile> = places
            .iter()
            .map(|(window, place)| (name_of(window), place.into()))
            .collect();
        let json = serde_json::to_vec_pretty(&file).map_err(FileError::EncodePlaces)?;
        write_replacing(&self.path(), &json)
    }
}

/// The places in the file's bytes. Bytes that are no JSON object give none, and an entry that is no place, or that
/// names a window this version does not know, is left out; either is logged.
fn parse_places(bytes: &[u8]) -> WindowPlaces {
    let entries = match serde_json::from_slice::<BTreeMap<String, serde_json::Value>>(bytes) {
        Ok(entries) => entries,
        Err(error) => {
            tracing::warn!(%error, "the window places file is broken, so the windows open where they open by default");
            return WindowPlaces::default();
        }
    };
    let mut places = WindowPlaces::default();
    let mut left_out = 0_usize;
    for (name, value) in entries {
        let window = WINDOWS.into_iter().find(|window| name_of(*window) == name);
        if let (Some(window), Ok(place)) = (window, serde_json::from_value::<PlaceFile>(value)) {
            places.set(window, place.into());
        } else {
            left_out = left_out.saturating_add(1);
        }
    }
    if left_out > 0 {
        tracing::warn!(
            left_out,
            "entries of the window places file are no window's place and were left out"
        );
    }
    places
}

// The file is small, and the calls have nothing to await, so each answer is ready at once.
impl WindowPlacesStore for JsonWindowPlacesStore {
    fn load(&self) -> impl Future<Output = Result<WindowPlaces, StoreError>> + Send {
        ready(self.load_blocking().map_err(StoreError::from))
    }

    fn save(&self, places: &WindowPlaces) -> impl Future<Output = Result<(), StoreError>> + Send {
        ready(self.save_blocking(places).map_err(StoreError::from))
    }
}

#[cfg(test)]
mod tests;
