use cantrip_api::*;
use cantrip_mechanics::tokens::{self, RABBIT};
use cantrip_oracle as oracle;

/// "Create three 1/1 white Rabbit creature tokens."
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &RABBIT, 3)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], effect),
    ..oracle::HOP_TO_IT
};

pub static EXTRA: &[&CardDef] = &[&RABBIT];
