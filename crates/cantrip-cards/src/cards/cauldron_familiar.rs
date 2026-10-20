use cantrip_api::*;
use cantrip_mechanics::effects::{each_opponent_loses_1_life_you_gain_1, return_this_card_to_battlefield};
use cantrip_mechanics::targets::filters;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// "Sacrifice a Food: Return this card from your graveyard to the battlefield": it functions only from the graveyard (CR 113.6b);
/// the Food is the cost (CR 602.2b), chosen as it is paid, so a player with none can't activate it.
pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, each_opponent_loses_1_life_you_gain_1, "When this creature enters, each opponent loses 1 life and you gain 1 life."),
        ability(Cost::sacrifice(1, filters::food), return_this_card_to_battlefield, "Sacrifice a Food: Return this card from your graveyard to the battlefield.").from(ZoneKind::Graveyard),
    ],
    ..oracle::CAULDRON_FAMILIAR
};
