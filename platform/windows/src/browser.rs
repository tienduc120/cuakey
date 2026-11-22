//! Opening a web address, such as the project page in the about section, in the user's browser.

/// The scheme of the only addresses the app opens.
#[cfg(any(windows, test))]
const HTTPS: &str = "https://";

/// Whether `url` is an address the app opens: `https://` followed by a host, with no whitespace or control character.
/// The shell would run anything else as a program or open it as a file, which a link must never do.
#[cfg(any(windows, test))]
fn openable(url: &str) -> bool {
    url.strip_prefix(HTTPS)
        .is_some_and(|rest| !rest.is_empty() && !rest.starts_with('/'))
        && !url
            .chars()
            .any(|character| character.is_whitespace() || character.is_control())
}

#[cfg(windows)]
pub use self::shell::{OpenUrlError, open_url};

#[cfg(windows)]
mod shell {
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
    use windows::core::{HSTRING, PCWSTR, w};

    use super::openable;
    use crate::com::ComScope;

    /// The highest value `ShellExecuteW` returns for a failure; anything above it is a success.
    const LAST_ERROR_CODE: isize = 32;

    /// Why an address was not opened.
    #[derive(Debug, thiserror::Error)]
    pub enum OpenUrlError {
        /// Only `https://` addresses with a host are opened.
        #[error("only https addresses are opened")]
        NotHttps,
        #[error("could not prepare to open the address")]
        Com(#[source] windows::core::Error),
        /// The shell's own code for why it could not open the address.
        #[error("the shell could not open the address (code {code})")]
        Shell { code: isize },
    }

    /// Opens `url` in the user's browser. Only an `https://` address is handed to the shell; anything else is refused
    /// before it gets there.
    pub fn open_url(url: &str) -> Result<(), OpenUrlError> {
        if !openable(url) {
            return Err(OpenUrlError::NotHttps);
        }
        // The shell may hand the address to an extension it reaches through COM.
        let _com = ComScope::enter().map_err(OpenUrlError::Com)?;
        // SAFETY: the verb and the address are terminated strings that outlive the call, the other strings are null as
        // the call allows, and no owner window is given; the address is an https one, which the shell gives to the
        // browser rather than running it.
        let result = unsafe {
            ShellExecuteW(
                None,
                w!("open"),
                &HSTRING::from(url),
                PCWSTR::null(),
                PCWSTR::null(),
                SW_SHOWNORMAL,
            )
        };
        // The call answers in a handle-shaped value that only says whether it worked.
        let code = result.0 as isize;
        if code > LAST_ERROR_CODE {
            Ok(())
        } else {
            Err(OpenUrlError::Shell { code })
        }
    }
}

#[cfg(test)]
mod tests;
