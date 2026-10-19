use cantrip_api::*;
use cantrip_mechanics::{empower, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "When this creature enters, empower Jace 2."
fn empower_jace(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    empower::jace(g, item.controller, item.object, 2)
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, empower_jace, "When this creature enters, empower Jace 2.")],
    ..oracle::ARCANE_AMPHISBAENA
};

pub static EXTRA: &[&CardDef] = &[&empower::JACE];
