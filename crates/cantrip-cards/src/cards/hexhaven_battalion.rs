use cantrip_api::*;
use cantrip_mechanics::{empower, tokens, typecycling};
use cantrip_oracle as oracle;

/// "Create three 2/2 colorless Wizard Soldier creature tokens named Cadet. Empower Jace 2." The Cadets enter together, then
/// Jace is empowered (CR 608.2c, 701.71a).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::CADET, 3)?;
    empower::jace(g, item.controller, item.object, 2)
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], effect),
    abilities: &[typecycling::basic_landcycling(mana("{2}"), "Basic landcycling {2}")],
    ..oracle::HEXHAVEN_BATTALION
};

pub static EXTRA: &[&CardDef] = &[&tokens::CADET];
