---
name: handling-errors
description: Use when a function can fail, when adding a use-case outcome or the message a window shows for one, or when an error crosses a layer. Follows ADR-0011 and ADR-0014.
---

# Handle errors

Failure is a returned `Result`, and each layer owns the errors it raises. `services/example` shows the whole shape in four files: `domain/src/note.rs`, `application/src/ports.rs`, `application/src/create_note.rs`, `interfaces/src/note_form.rs`.

## What each layer does

| Layer | Declares | Does with an error |
| --- | --- | --- |
| domain | a `thiserror` enum of broken invariants (`NoteError::EmptyName`) | returns it from `new` |
| application | per use case, an enum of the outcomes a caller branches on (`CreateNoteError`); per port, the conditions its callers branch on (`SaveError::AlreadyExists`) | wraps a domain error with `#[from]`; turns a port's condition into an outcome; keeps the rest as `#[source]` |
| infrastructure | nothing other layers need | turns its storage's condition into the port's error (a unique-key violation becomes `SaveError::AlreadyExists`) |
| interfaces | nothing | matches every outcome to the message a window shows; logs the cause of an internal one |

## Rules

1. The cause travels: `#[source]` or `#[from]` on the variant that wraps it. A message names the operation that failed ("save note"), and the source chain says why.
2. Absence is the use case's call: a repository lookup answers `Option<T>`, and the use case decides whether `None` is an outcome.
3. Only `interfaces` decides what the user sees, with an exhaustive `match` and no wildcard arm: a problem with what the user entered says what to fix, a conflict names what is taken, and anything internal shows a generic message with the cause in the log, never on screen.
4. Name an error enum after the operation (`CreateNoteError`) and a variant after the condition (`AlreadyExists`).
5. A state the code makes impossible is not an outcome, so it gets no error variant. When `expect` is the honest statement, put `#[expect(clippy::expect_used, reason = "<why it cannot happen>")]` on that one item and tell the user: it is debt, and the reason is what a reviewer checks (Microsoft M-PANIC-ON-BUG; Google).

## Adding an outcome

1. Write the `interfaces` test first: the use case returns the new outcome, expect the message the window shows, which leaks nothing internal. Watch it go red.
2. Add the variant to the use case's error enum and return it.
3. The `match` in `interfaces` stops compiling until you add the arm: give it its message.
4. **Done** when you can walk one failing path from where it starts to the screen and point at each layer's translation.
