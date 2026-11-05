use cantrip_api::*;
use cantrip_mechanics::{endure, triggers};
use cantrip_oracle as oracle;

/// "When this creature enters, it endures 2." (CR 701.63a, 603.6a): the choice is made as the ability resolves.
fn endures(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    endure::endure(g, item.object, item.object, 2)
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(triggers::THIS_ENTERS, endures, "When this creature enters, it endures 2. (Put two +1/+1 counters on it or create a 2/2 white Spirit creature token.)")],
    ..oracle::SANDSKITTER_OUTRIDER
};

pub static EXTRA: &[&CardDef] = &[&endure::SPIRIT];
