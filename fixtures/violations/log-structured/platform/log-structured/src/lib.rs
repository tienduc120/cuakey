//! Fixture: the user id is buried in the message text.
pub fn login(id: u32) {
    tracing::info!("user {id} logged in");
}
