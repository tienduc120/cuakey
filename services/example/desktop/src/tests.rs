use example_interfaces::{Message, NoteForm};

use super::App;

fn form(id: &str, name: &str) -> NoteForm {
    NoteForm {
        id: id.to_owned(),
        name: name.to_owned(),
        message: None,
    }
}

#[tokio::test]
async fn the_assembled_app_creates_a_note_and_refuses_the_same_id_twice() {
    let app = App::start();
    let (mut first, mut again) = (form("n-1", "groceries"), form("n-1", "groceries"));

    app.submit(&mut first).await;
    app.submit(&mut again).await;

    assert_eq!(
        (first.message, again.message),
        (
            Some(Message::Done("created n-1".to_owned())),
            Some(Message::Problem("n-1 already exists".to_owned()))
        ),
        "created, then the stored note makes the same id a conflict"
    );
}
