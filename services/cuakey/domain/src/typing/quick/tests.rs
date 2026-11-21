use super::{first_pair, last_pair, second_of_doubled};
use crate::BaseLetter;

#[test]
fn a_consonant_typed_twice_types_the_pair_it_starts() {
    let pairs = [
        (BaseLetter::C, BaseLetter::H),
        (BaseLetter::G, BaseLetter::I),
        (BaseLetter::K, BaseLetter::H),
        (BaseLetter::N, BaseLetter::G),
        (BaseLetter::P, BaseLetter::H),
        (BaseLetter::Q, BaseLetter::U),
        (BaseLetter::T, BaseLetter::H),
    ];

    for (first, second) in pairs {
        assert_eq!(second_of_doubled(first), Some(second), "{first:?} twice");
    }
    assert_eq!(second_of_doubled(BaseLetter::M), None, "mm stays mm");
}

#[test]
fn f_j_w_start_ph_gi_qu() {
    assert_eq!(first_pair(BaseLetter::F), Some([BaseLetter::P, BaseLetter::H]), "f");
    assert_eq!(first_pair(BaseLetter::J), Some([BaseLetter::G, BaseLetter::I]), "j");
    assert_eq!(first_pair(BaseLetter::W), Some([BaseLetter::Q, BaseLetter::U]), "w");
    assert_eq!(first_pair(BaseLetter::Z), None, "z stands for nothing");
}

#[test]
fn g_h_k_end_ng_nh_ch() {
    assert_eq!(last_pair(BaseLetter::G), Some([BaseLetter::N, BaseLetter::G]), "g");
    assert_eq!(last_pair(BaseLetter::H), Some([BaseLetter::N, BaseLetter::H]), "h");
    assert_eq!(last_pair(BaseLetter::K), Some([BaseLetter::C, BaseLetter::H]), "k");
    assert_eq!(last_pair(BaseLetter::T), None, "t stands for itself");
}
