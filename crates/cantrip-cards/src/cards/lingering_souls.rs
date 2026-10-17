use cantrip_api::*;
use cantrip_mechanics::{flashback, tokens};
use cantrip_oracle as oracle;

fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::WHITE_FLYING_SPIRIT, 2)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], effect),
    ways: &[flashback::way(cost("{1}{B}"), "Flashback {1}{B}")],
    ..oracle::LINGERING_SOULS
};

pub static EXTRA: &[&CardDef] = &[&tokens::WHITE_FLYING_SPIRIT];
