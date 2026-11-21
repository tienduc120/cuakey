use super::{Mode, Only, parse_mode};
use crate::corpus::CORPUS_FILES;

fn args(text: &str) -> Vec<String> {
    text.split_whitespace().map(str::to_owned).collect()
}

#[test]
fn no_arguments_compare_the_whole_corpus_and_a_filter_one_file() {
    assert_eq!(parse_mode(&[]), Ok(Mode::Corpus(None)));
    assert_eq!(
        parse_mode(&args("--filter vni")),
        Ok(Mode::Corpus(Some(Only::File(CORPUS_FILES[1]))))
    );
}

#[test]
fn random_mode_takes_a_count_and_a_seed_that_defaults_to_one() {
    assert_eq!(
        parse_mode(&args("--random 20 --seed 9")),
        Ok(Mode::Random { count: 20, seed: 9 })
    );
    assert_eq!(
        parse_mode(&args("--random 20")),
        Ok(Mode::Random { count: 20, seed: 1 })
    );
}

#[test]
fn a_filter_names_the_macro_file_or_the_conversion_texts_too() {
    assert_eq!(
        parse_mode(&args("--filter macros")),
        Ok(Mode::Corpus(Some(Only::File(CORPUS_FILES[5]))))
    );
    assert_eq!(
        parse_mode(&args("--filter convert")),
        Ok(Mode::Corpus(Some(Only::Convert)))
    );
}

#[test]
fn random_mode_needs_at_least_one_sequence() {
    assert_eq!(
        parse_mode(&args("--random 0")),
        Err("--random needs at least one sequence; with none it would pass without comparing anything".to_owned())
    );
}

#[test]
fn arguments_that_mean_nothing_together_are_refused() {
    assert_eq!(
        parse_mode(&args("--seed 3")),
        Err("--seed only applies with --random".to_owned())
    );
    assert_eq!(
        parse_mode(&args("--random 3 --filter vni")),
        Err("--filter chooses corpus files; --random types random sequences instead".to_owned())
    );
    assert_eq!(
        parse_mode(&args("--filter latin")),
        Err(
            "no corpus file is called `latin`; there are telex, vni, simple_telex, english, sentences, macros, convert"
                .to_owned()
        )
    );
}
