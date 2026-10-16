use cantrip_api::*;
use cantrip_mechanics::tokens::{self, investigate};
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    investigate(g, item.controller)
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, effect, "When Thraben Inspector enters, investigate.")],
    ..oracle::THRABEN_INSPECTOR
};

pub static EXTRA: &[&CardDef] = &[&tokens::CLUE];
