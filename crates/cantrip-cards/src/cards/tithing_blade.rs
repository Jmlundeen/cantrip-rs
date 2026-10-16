use cantrip_api::*;
use cantrip_mechanics::triggers::{THIS_ENTERS, YOUR_UPKEEP};
use cantrip_mechanics::{craft, each_opponent_sacrifices};
use cantrip_oracle as oracle;

/// "When this artifact enters, each opponent sacrifices a creature of their choice." (CR 701.21a, 101.4)
fn edict(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    each_opponent_sacrifices(g, item.controller, item.object, 1, |q, id| q.chars(id).is(CardType::Creature))
}

/// "At the beginning of your upkeep, each opponent loses 1 life and you gain 1 life."
fn drain(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.each_opponent_loses_life(item.controller, 1);
    g.gain_life(item.controller, 1);
    Ok(())
}

pub static BACK: CardDef = CardDef {
    abilities: &[triggered(YOUR_UPKEEP, drain, "At the beginning of your upkeep, each opponent loses 1 life and you gain 1 life.")],
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
