//! Whether the process runs with administrator rights, and starting it again with them.

use std::io;

use windows::Win32::Foundation::{CloseHandle, ERROR_CANCELLED, HANDLE};
use windows::Win32::Security::{GetTokenInformation, TOKEN_ELEVATION, TOKEN_QUERY, TokenElevation};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
use windows::Win32::UI::Shell::{SEE_MASK_NOASYNC, SHELLEXECUTEINFOW, ShellExecuteExW};
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
use windows::core::{HSTRING, PCWSTR, w};

/// Why `CuaKey` could not start itself again with administrator rights.
#[derive(Debug, thiserror::Error)]
pub enum RelaunchError {
    #[error("could not find the program's own path")]
    CurrentExe(#[source] io::Error),
    /// The user refused the administrator prompt.
    #[error("the administrator prompt was refused")]
    Cancelled,
    #[error("could not start the program as administrator")]
    Launch(#[source] windows::core::Error),
}

/// Whether this process runs with administrator rights. A token that cannot be read counts as not elevated.
#[must_use]
pub fn is_elevated() -> bool {
    match read_elevation() {
        Ok(elevated) => elevated,
        Err(error) => {
            tracing::warn!(error = ?error, "could not read whether the process is elevated");
            false
        }
    }
}

fn read_elevation() -> windows::core::Result<bool> {
    let mut token = HANDLE::default();
    // SAFETY: the call takes nothing and returns a pseudo handle for this process.
    let process = unsafe { GetCurrentProcess() };
    // SAFETY: the pseudo handle of the current process is always valid, and `token` is a local the call fills.
    unsafe { OpenProcessToken(process, TOKEN_QUERY, &raw mut token) }?;
    let token = Token(token);
    let mut elevation = TOKEN_ELEVATION::default();
    let mut returned = 0_u32;
    let size = u32::try_from(size_of::<TOKEN_ELEVATION>()).unwrap_or(0);
    let out = (&raw mut elevation).cast();
    // SAFETY: the token is open for querying, and `elevation` is a local of exactly `size` bytes, the structure this
    // information class fills.
    unsafe { GetTokenInformation(token.0, TokenElevation, Some(out), size, &raw mut returned) }?;
    Ok(elevation.TokenIsElevated != 0)
}

/// A process token, closed on drop.
struct Token(HANDLE);

impl Drop for Token {
    fn drop(&mut self) {
        // SAFETY: the handle came from `OpenProcessToken` and is closed once, here.
        if let Err(error) = unsafe { CloseHandle(self.0) } {
            tracing::warn!(error = ?error, "could not close a process token");
        }
    }
}

/// Starts this program again with administrator rights, passing it `args`, after the system's prompt. The caller
/// decides what happens to this process. A refused prompt is [`RelaunchError::Cancelled`].
pub fn relaunch_elevated(args: &[String]) -> Result<(), RelaunchError> {
    let exe = std::env::current_exe().map_err(RelaunchError::CurrentExe)?;
    let file = HSTRING::from(exe.as_os_str());
    let parameters = HSTRING::from(command_line(args));
    let mut info = SHELLEXECUTEINFOW {
        cbSize: u32::try_from(size_of::<SHELLEXECUTEINFOW>()).unwrap_or(0),
        fMask: SEE_MASK_NOASYNC,
        lpVerb: w!("runas"),
        lpFile: PCWSTR(file.as_ptr()),
        lpParameters: PCWSTR(parameters.as_ptr()),
        nShow: SW_SHOWNORMAL.0,
        ..Default::default()
    };
    // SAFETY: `info` is fully set for this call, and the strings it points to outlive it.
    unsafe { ShellExecuteExW(&raw mut info) }.map_err(relaunch_error)
}

/// The failure of the launch call, with a refused prompt told apart.
fn relaunch_error(error: windows::core::Error) -> RelaunchError {
    if error.code() == ERROR_CANCELLED.to_hresult() {
        RelaunchError::Cancelled
    } else {
        RelaunchError::Launch(error)
    }
}

/// `args` as one command line, each quoted as the program receiving it will read it back.
fn command_line(args: &[String]) -> String {
    args.iter().map(|arg| quote(arg)).collect::<Vec<_>>().join(" ")
}

fn quote(arg: &str) -> String {
    if !arg.is_empty() && !arg.contains([' ', '\t', '"']) {
        return arg.to_owned();
    }
    let mut quoted = String::from('"');
    let mut backslashes = 0_usize;
    for character in arg.chars() {
        match character {
            '\\' => backslashes += 1,
            '"' => {
                // Backslashes before a quote are doubled, and the quote itself is escaped.
                quoted.extend(std::iter::repeat_n('\\', backslashes * 2 + 1));
                quoted.push('"');
                backslashes = 0;
            }
            other => {
                quoted.extend(std::iter::repeat_n('\\', backslashes));
                quoted.push(other);
                backslashes = 0;
            }
        }
    }
    // Backslashes before the closing quote are doubled so they do not escape it.
    quoted.extend(std::iter::repeat_n('\\', backslashes * 2));
    quoted.push('"');
    quoted
}

#[cfg(test)]
mod tests;
