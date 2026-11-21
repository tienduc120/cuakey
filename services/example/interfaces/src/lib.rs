//! What this service shows the user. Only this layer maps an outcome to a message on screen, with an exhaustive
//! `match` (ADR-0011, ADR-0014). Its state is plain data: a UI toolkit draws it, and a test reads it.

mod note_form;

pub use note_form::{Message, NoteForm};
