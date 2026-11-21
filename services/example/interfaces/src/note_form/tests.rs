use example_application::CreateNote;
use example_application::testing::{FakeNoteRepository, SaveOutcome};
use platform_clock::{FixedClock, Timestamp};

use super::{Message, NoteForm};

async fn submit(outcome: SaveOutcome, id: &str, name: &str) -> NoteForm {
    let clock = FixedClock(Timestamp::from_second(1_700_000_000).unwrap());
    let create = CreateNote::new(FakeNoteRepository::new(outcome), clock);
    let mut form = NoteForm {
        id: id.to_owned(),
        name: name.to_owned(),
        message: None,
    };

    form.submit(&create).await;

    form
}

#[tokio::test]
async fn a_created_note_says_so_with_its_id() {
    let form = submit(SaveOutcome::Saved, "n-1", "groceries").await;

    assert_eq!(form.message, Some(Message::Done("created n-1".to_owned())));
}

#[tokio::test]
async fn an_invalid_note_is_a_problem_that_says_why() {
    let form = submit(SaveOutcome::Saved, "n-1", " ").await;

    let Some(Message::Problem(text)) = form.message else {
        panic!("{form:?}")
    };
    assert!(text.contains("name must not be empty"), "{text}");
}

#[tokio::test]
async fn a_taken_id_is_a_problem_that_names_it() {
    let form = submit(SaveOutcome::Taken, "n-1", "groceries").await;

    assert_eq!(form.message, Some(Message::Problem("n-1 already exists".to_owned())));
}

#[tokio::test]
async fn a_storage_failure_never_shows_its_cause_to_the_user() {
    let form = submit(SaveOutcome::Broken, "n-1", "groceries").await;

    assert_eq!(
        form.message,
        Some(Message::Problem("could not save; try again".to_owned()))
    );
}
