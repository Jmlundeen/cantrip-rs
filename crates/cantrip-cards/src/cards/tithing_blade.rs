use cantrip_api::*;
use cantrip_mechanics::triggers::{THIS_ENTERS, YOUR_UPKEEP};
use cantrip_mechanics::{craft, each_opponent_sacrifices};
use cantrip_oracle as oracle;

/// "When this artifact enters, each opponent sacrifices a creature of their choice." (CR 701.21a, 101.4)
fn edict(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    each_opponent_sacrifices(g, item.controller, item.object, 1, |q, id| q.chars(id).is(CardType::Creature))
}

pub static BACK: CardDef = CardDef {
    abilities: &[triggered(YOUR_UPKEEP, cantrip_mechanics::effects::each_opponent_loses_1_life_you_gain_1, "At the beginning of your upkeep, each opponent loses 1 life and you gain 1 life.")],
    ..oracle::CONSUMING_SEPULCHER
};

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, edict, "When this artifact enters, each opponent sacrifices a creature of their choice."),
        craft::ability(mana("{4}{B}"), &craft::CREATURE, "Craft with creature {4}{B}"),
    ],
    back: Some(&BACK),
    ..oracle::TITHING_BLADE
};
