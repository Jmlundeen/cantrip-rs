use cantrip_api::*;
use cantrip_mechanics::{empower, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "When this creature enters, empower Jace 2." Creates the Jace planeswalker token if its controller controls none, then
/// puts two loyalty counters on the chosen Jace token (CR 701.71a).
fn empower_jace(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    empower::jace(g, item.controller, item.object, 2)
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, empower_jace, "When this creature enters, empower Jace 2.")],
    ..oracle::KEEPER_OF_THE_QUIET_HOUR
};

pub static EXTRA: &[&CardDef] = &[&empower::JACE];
