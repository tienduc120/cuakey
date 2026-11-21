use cuakey_domain::{InputMethod, KeyCode};

use super::{Rng, combo, sequence};
use crate::corpus::Stroke;

#[test]
fn the_same_seed_gives_the_same_sequences_and_settings() {
    let draw = |seed| {
        let mut rng = Rng::new(seed);
        (0..50)
            .map(|_| (combo(&mut rng), sequence(&mut rng, InputMethod::Telex)))
            .collect::<Vec<_>>()
    };

    assert_eq!(draw(7), draw(7));
    assert_ne!(draw(7), draw(8), "another seed, other sequences");
}

#[test]
fn sequences_hold_one_to_twelve_strokes_then_a_space_that_ends_the_last_word() {
    let mut rng = Rng::new(1);

    let sequences: Vec<Vec<Stroke>> = (0..500).map(|_| sequence(&mut rng, InputMethod::Vni)).collect();

    let lengths: Vec<usize> = sequences.iter().map(Vec::len).collect();
    assert_eq!(lengths.iter().min(), Some(&2));
    assert_eq!(lengths.iter().max(), Some(&13));
    let ends_with_space =
        |strokes: &Vec<Stroke>| matches!(strokes.last(), Some(Stroke::Key(key)) if key.code() == KeyCode::Space);
    assert!(sequences.iter().all(ends_with_space));
}
