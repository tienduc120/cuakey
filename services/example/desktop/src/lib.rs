//! The composition root: the one place that knows which adapter backs each port. A desktop app's root also starts the
//! UI toolkit's event loop on the main thread, which hands each user action to [`App`] (ADR-0014).

use example_application::CreateNote;
use example_infrastructure::MemoryNoteRepository;
use example_interfaces::NoteForm;
use platform_clock::SystemClock;

#[derive(Debug)]
pub struct App {
    create: CreateNote<MemoryNoteRepository, SystemClock>,
}

impl App {
    /// Wires every port to its adapter, once, before the event loop starts.
    pub fn start() -> Self {
        Self {
            create: CreateNote::new(MemoryNoteRepository::default(), SystemClock),
        }
    }

    /// What the window calls when the user submits the form.
    pub async fn submit(&self, form: &mut NoteForm) {
        form.submit(&self.create).await;
    }
}

#[cfg(test)]
mod tests;
