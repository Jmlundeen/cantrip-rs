use cantrip_api::*;
use cantrip_mechanics::{effects, escape};
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        escape::sacrifice_unless_escaped("When Phlage enters, sacrifice it unless it escaped."),
        triggered(triggers::THIS_ENTERS_OR_ATTACKS, |g, item| effects::damage_target_and_gain_life(g, item, 3, 3), "Whenever Phlage enters or attacks, it deals 3 damage to any target and you gain 3 life.")
            .targets(&[targets::ANY]),
    ],
    ways: &[escape::way(mana("{W}{W}{R}{R}"), 5, "Escape—{R}{R}{W}{W}, Exile five other cards from your graveyard")],
    ..oracle::PHLAGE_TITAN_OF_FIRES_FURY
};
