//! Where each window was on screen when it last closed, so it opens there again. A place is plain values, not part of
//! the typing domain: a window without one only opens where it opens by default.

use std::collections::BTreeMap;

use crate::{StoreError, WindowPlacesStore};

/// A window that opens where it last closed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PlacedWindow {
    Settings,
    Macros,
    Convert,
    AlwaysOn,
}

/// Where a window's frame was: its top-left corner in whole physical pixels of the virtual screen, which mean the same
/// on every monitor, and the scale of the monitor it was on, which turns them into the points a window opens at.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowPlace {
    pub x: f32,
    pub y: f32,
    pub scale: f32,
}

/// The last place of each window that has one.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct WindowPlaces {
    places: BTreeMap<PlacedWindow, WindowPlace>,
}

impl WindowPlaces {
    pub fn get(&self, window: PlacedWindow) -> Option<WindowPlace> {
        self.places.get(&window).copied()
    }

    pub fn set(&mut self, window: PlacedWindow, place: WindowPlace) {
        self.places.insert(window, place);
    }

    /// Each window's place, in the order the windows are listed.
    pub fn iter(&self) -> impl Iterator<Item = (PlacedWindow, WindowPlace)> + '_ {
        self.places.iter().map(|(window, place)| (*window, *place))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LoadWindowPlacesError {
    #[error("could not load the window places")]
    Storage(#[source] StoreError),
}

#[derive(Debug, thiserror::Error)]
pub enum SaveWindowPlacesError {
    #[error("could not save the window places")]
    Storage(#[source] StoreError),
}

/// Reads where each window last closed.
#[derive(Debug)]
pub struct LoadWindowPlaces<S> {
    store: S,
}

impl<S: WindowPlacesStore> LoadWindowPlaces<S> {
    pub const fn new(store: S) -> Self {
        Self { store }
    }

    pub async fn execute(&self) -> Result<WindowPlaces, LoadWindowPlacesError> {
        self.store.load().await.map_err(LoadWindowPlacesError::Storage)
    }
}

/// Stores where each window last closed.
#[derive(Debug)]
pub struct SaveWindowPlaces<S> {
    store: S,
}

impl<S: WindowPlacesStore> SaveWindowPlaces<S> {
    pub const fn new(store: S) -> Self {
        Self { store }
    }

    pub async fn execute(&self, places: &WindowPlaces) -> Result<(), SaveWindowPlacesError> {
        self.store.save(places).await.map_err(SaveWindowPlacesError::Storage)
    }

    #[cfg(test)]
    const fn store(&self) -> &S {
        &self.store
    }
}

#[cfg(test)]
mod tests;
