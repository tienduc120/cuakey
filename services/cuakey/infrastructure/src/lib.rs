//! Adapters for this service's ports. Each one turns its storage's conditions into the application's errors.

#[cfg(windows)]
mod clipboard;
mod files;
mod key_names;
mod macro_file;
mod preferences_file;
#[cfg(test)]
mod test_dir;
mod window_places_file;

#[cfg(windows)]
pub use clipboard::WindowsClipboard;
pub use macro_file::TextMacroStore;
pub use preferences_file::JsonPreferencesStore;
pub use window_places_file::JsonWindowPlacesStore;
