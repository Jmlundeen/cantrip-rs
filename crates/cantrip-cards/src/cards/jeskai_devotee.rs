use cantrip_api::*;
use cantrip_mechanics::{effects, flurry};
use cantrip_oracle as oracle;

/// Flurry (an ability word, CR 207.2c): +1/+1 until end of turn to this creature. The mana ability is an
/// activated one that costs {1} and adds one mana of a color chosen as it resolves (CR 605.1a, 106.1); "activate only once each
/// turn" counts activations by this object (CR 602.5b).
pub static CARD: CardDef = CardDef {
    abilities: &[
        flurry::trigger("Whenever you cast your second spell each turn, this creature gets +1/+1 until end of turn.", effects::this_gets::<1, 1>),
        mana_ability(cost("{1}"), add_one_of(&[Color::Blue, Color::Red, Color::White]), "{1}: Add {U}, {R}, or {W}. Activate only once each turn.")
            .once_each_turn(),
    ],
    ..oracle::JESKAI_DEVOTEE
};
