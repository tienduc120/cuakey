use example_application::{CreateNote, CreateNoteCommand, CreateNoteError, CreateNoteResult, NoteRepository};
use platform_clock::Clock;

/// The form a window draws to create a note: what the user typed, and the message shown under it after a submit.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NoteForm {
    pub id: String,
    pub name: String,
    pub message: Option<Message>,
}

/// What the window tells the user after an action; a problem is drawn as an error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    Done(String),
    Problem(String),
}

impl NoteForm {
    /// Runs the use case on what the user typed and keeps the message to show.
    pub async fn submit<R: NoteRepository, C: Clock>(&mut self, create: &CreateNote<R, C>) {
        let outcome = create
            .execute(CreateNoteCommand {
                id: self.id.clone(),
                name: self.name.clone(),
            })
            .await;
        self.message = Some(message(outcome));
    }
}

/// Exhaustive on purpose: a new outcome does not compile until someone decides what the user sees.
fn message(outcome: Result<CreateNoteResult, CreateNoteError>) -> Message {
    match outcome {
        Ok(created) => Message::Done(format!("created {}", created.id)),
        Err(e @ (CreateNoteError::Invalid(_) | CreateNoteError::AlreadyExists(_))) => Message::Problem(e.to_string()),
        Err(e @ CreateNoteError::Storage(_)) => {
            tracing::error!(error = ?e, "create note");
            Message::Problem("could not save; try again".to_owned())
        }
    }
}

#[cfg(test)]
mod tests;
