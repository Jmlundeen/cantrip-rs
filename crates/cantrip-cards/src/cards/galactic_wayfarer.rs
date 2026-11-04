use cantrip_api::*;
use cantrip_mechanics::tokens::{self, LANDER};
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// "When this creature enters, create a Lander token." (CR 111.10u)
fn lander(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create_lander(g, item.controller)
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, lander, "When this creature enters, create a Lander token.")],
    ..oracle::GALACTIC_WAYFARER
};

pub static EXTRA: &[&CardDef] = &[&LANDER];
