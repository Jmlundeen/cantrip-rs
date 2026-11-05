use cantrip_api::*;
use cantrip_mechanics::{endure, triggers};
use cantrip_oracle as oracle;

/// "When this creature enters, it endures 1." (CR 701.63a, 603.6a): the choice is made as the ability resolves.
fn endures(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    endure::endure(g, item.object, item.object, 1)
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(triggers::THIS_ENTERS, endures, "When this creature enters, it endures 1. (Put a +1/+1 counter on it or create a 1/1 white Spirit creature token.)")],
    ..oracle::FORTRESS_KIN_GUARD
};

pub static EXTRA: &[&CardDef] = &[&endure::SPIRIT];
