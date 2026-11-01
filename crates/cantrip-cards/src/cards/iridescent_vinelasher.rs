use cantrip_api::*;
use cantrip_mechanics::triggers::LANDFALL;
use cantrip_mechanics::{effects, offspring, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    kicker: Some(offspring::kicker(mana("{2}"))),
    abilities: &[
        offspring::trigger("Offspring {2} (You may pay an additional {2} as you cast this spell. If you do, when this creature enters, create a 1/1 token copy of it.)"),
        triggered(LANDFALL, effects::damage_first_target::<1>, "Landfall \u{2014} Whenever a land you control enters, this creature deals 1 damage to target opponent.")
            .targets(&[targets::OPPONENT]),
    ],
    ..oracle::IRIDESCENT_VINELASHER
};
