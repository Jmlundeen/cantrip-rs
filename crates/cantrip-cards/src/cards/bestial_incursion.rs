use cantrip_api::*;
use cantrip_mechanics::tokens;
use cantrip_mechanics::{flashback, tokens::BEAST_4_4_TRAMPLE};
use cantrip_oracle as oracle;

/// "Create a 4/4 green Beast creature token with trample." (CR 111.1.)
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &BEAST_4_4_TRAMPLE, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], effect),
    ways: &[flashback::way(cost("{5}{G}"), "Flashback {5}{G}")],
    ..oracle::BESTIAL_INCURSION
};

pub static EXTRA: &[&CardDef] = &[&BEAST_4_4_TRAMPLE];
