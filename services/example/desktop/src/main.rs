//! Starts the app. Process-wide things happen here and nowhere else: the subscriber is installed and the runtime that
//! runs use cases is made. A real app hands [`App`] to its UI toolkit here, whose event loop runs on this thread until
//! the last window closes; the witness has no window, so it submits one form and exits.
#![windows_subsystem = "windows"]

use std::process::ExitCode;

use example_desktop::App;
use example_interfaces::{Message, NoteForm};

fn main() -> ExitCode {
    #[expect(clippy::disallowed_methods, reason = "the composition root installs the subscriber")]
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    let runtime = match tokio::runtime::Builder::new_current_thread().build() {
        Ok(runtime) => runtime,
        Err(e) => {
            tracing::error!(error = ?e, "start");
            return ExitCode::FAILURE;
        }
    };
    let app = App::start();
    let mut form = NoteForm {
        id: "n-1".to_owned(),
        name: "first".to_owned(),
        message: None,
    };
    runtime.block_on(app.submit(&mut form));
    match form.message {
        Some(Message::Done(text)) => {
            tracing::info!(%text, "submitted");
            ExitCode::SUCCESS
        }
        Some(Message::Problem(text)) => {
            tracing::error!(%text, "submitted");
            ExitCode::FAILURE
        }
        None => ExitCode::FAILURE,
    }
}
