use super::Sentence;

fn after(text: &str) -> Sentence {
    text.chars().fold(Sentence::default(), Sentence::after)
}

#[test]
fn a_full_stop_question_or_exclamation_mark_and_a_space_end_the_sentence() {
    for text in ["tôi. ", "tôi? ", "tôi! ", "tôi.\n", "tôi.  \t"] {
        assert!(after(text).starts_next(), "{text:?}: the next letter starts a sentence");
    }
}

#[test]
fn anything_else_continues_it() {
    for text in ["", "tôi", "tôi ", "tôi,", "tôi.", "tôi. a", "3.5"] {
        assert!(
            !after(text).starts_next(),
            "{text:?}: the next letter continues the sentence"
        );
    }
}
