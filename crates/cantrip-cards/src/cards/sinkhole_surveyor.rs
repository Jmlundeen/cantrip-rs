use cantrip_api::*;
use cantrip_mechanics::{endure, triggers};
use cantrip_oracle as oracle;

/// "You lose 1 life and this creature endures 1." (CR 119.3, 701.63a): the endure choice is made as the ability resolves.
fn lose_life_and_endure(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.lose_life(item.controller, 1);
    endure::endure(g, item.object, item.object, 1)
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::THIS_ATTACKS,
        lose_life_and_endure,
        "Whenever this creature attacks, you lose 1 life and this creature endures 1. (Put a +1/+1 counter on it or create a 1/1 white Spirit creature token.)",
    )],
    ..oracle::SINKHOLE_SURVEYOR
};

pub static EXTRA: &[&CardDef] = &[&endure::SPIRIT];
