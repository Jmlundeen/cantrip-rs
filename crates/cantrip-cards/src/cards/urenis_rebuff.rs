use cantrip_api::*;
use cantrip_mechanics::{effects, harmonize, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], effects::return_first_target_to_hand),
    ways: &[harmonize::way(mana("{5}{U}"), "Harmonize {5}{U}")],
    ..oracle::URENIS_REBUFF
};
