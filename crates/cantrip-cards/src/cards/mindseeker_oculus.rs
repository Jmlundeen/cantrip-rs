use cantrip_api::*;
use cantrip_mechanics::{empower, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "When this creature enters, empower Jace 4." Creates the Jace planeswalker token if its controller controls none, then
/// puts four loyalty counters on the chosen Jace token (CR 701.71a).
fn empower_jace(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    empower::jace(g, item.controller, item.object, 4)
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, empower_jace, "When this creature enters, empower Jace 4.")],
    ..oracle::MINDSEEKER_OCULUS
};

pub static EXTRA: &[&CardDef] = &[&empower::JACE];
