use cantrip_api::*;
use cantrip_mechanics::tokens::{self, LANDER};
use cantrip_mechanics::triggers::THIS_DIES;
use cantrip_oracle as oracle;

/// "When this creature dies, create a Lander token."
fn lander(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create_lander(g, item.controller)
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_DIES, lander, "When this creature dies, create a Lander token.")],
    ..oracle::BEAMSAW_PROSPECTOR
};

pub static EXTRA: &[&CardDef] = &[&LANDER];
