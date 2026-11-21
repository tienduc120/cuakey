//! Clean twin: delivery maps an outcome to what the window shows and sees only the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Message {
    Drafted,
    Refused,
}

pub fn draft(id: &str, total_cents: u64) -> Message {
    if billing_application::draft(id, total_cents).is_some() { Message::Drafted } else { Message::Refused }
}
