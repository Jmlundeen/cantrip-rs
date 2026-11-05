use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, effects::scry_two, "When this creature enters, scry 2."),
        mana_ability(cost("{1}"), add_one_of(&[Color::Red, Color::White, Color::Black]), "{1}: Add {R}, {W}, or {B}. Activate only once each turn.").once_each_turn(),
    ],
    ..oracle::MARDU_DEVOTEE
};
