use cuakey_domain::InputMethod;

use super::{Rng, combo, sequence};

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
fn sequences_hold_one_to_twelve_strokes() {
    let mut rng = Rng::new(1);

    let lengths: Vec<usize> = (0..500).map(|_| sequence(&mut rng, InputMethod::Vni).len()).collect();

    assert_eq!(lengths.iter().min(), Some(&1));
    assert_eq!(lengths.iter().max(), Some(&12));
}
