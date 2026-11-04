use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        TAP.and(sacrifice(1, targets::filters::creature)),
        effects::return_first_target_to_battlefield,
        "{T}, Sacrifice a creature: Return target creature card from your graveyard to the battlefield. Activate only during your upkeep.",
    )
    .targets(&[targets::CREATURE_CARD_IN_YOUR_GRAVEYARD])
    .condition(cantrip_mechanics::your_upkeep)],
    ..oracle::HELLS_CARETAKER
};
