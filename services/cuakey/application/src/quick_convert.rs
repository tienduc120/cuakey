use cuakey_domain::{ConvertSettings, convert};

use crate::{Clipboard, ClipboardError, PreferencesStore, StoreError};

/// What the caller does once the clipboard holds the converted text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuickConvertResult {
    /// The user asked to be told when a conversion is done.
    pub notify: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum QuickConvertError {
    #[error("the clipboard holds no text")]
    NoText,
    #[error("could not use the clipboard")]
    Clipboard(#[source] ClipboardError),
    #[error("could not load the conversion options")]
    Storage(#[source] StoreError),
}

/// Converts the text on the clipboard in place, with the options the conversion window last saved.
#[derive(Debug)]
pub struct QuickConvert<C, S> {
    clipboard: C,
    store: S,
}

impl<C: Clipboard, S: PreferencesStore> QuickConvert<C, S> {
    pub const fn new(clipboard: C, store: S) -> Self {
        Self { clipboard, store }
    }

    pub async fn execute(&self) -> Result<QuickConvertResult, QuickConvertError> {
        let stored = self.store.load().await.map_err(QuickConvertError::Storage)?;
        let settings = stored.map_or_else(ConvertSettings::default, |preferences| {
            preferences.settings().convert().clone()
        });
        let text = self
            .clipboard
            .read_text()
            .await
            .map_err(QuickConvertError::Clipboard)?
            .filter(|text| !text.is_empty())
            .ok_or(QuickConvertError::NoText)?;
        let converted = convert(&text, settings.options());
        self.clipboard
            .write_text(&converted)
            .await
            .map_err(QuickConvertError::Clipboard)?;
        Ok(QuickConvertResult {
            notify: settings.notify_when_done().is_on(),
        })
    }

    #[cfg(test)]
    const fn clipboard(&self) -> &C {
        &self.clipboard
    }
}

#[cfg(test)]
mod tests;
