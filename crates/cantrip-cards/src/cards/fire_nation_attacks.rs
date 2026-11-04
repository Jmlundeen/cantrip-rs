use cantrip_api::*;
use cantrip_mechanics::tokens::{self, FIREBENDING_SOLDIER};
use cantrip_mechanics::flashback;
use cantrip_oracle as oracle;

/// "Create two 2/2 red Soldier creature tokens with firebending 1" (CR 111.1, 701.7): both at once.
fn two_soldiers(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &FIREBENDING_SOLDIER, 2)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], two_soldiers),
    ways: &[flashback::way(cost("{8}{R}"), "Flashback {8}{R}")],
    ..oracle::FIRE_NATION_ATTACKS
};

pub static EXTRA: &[&CardDef] = &[&FIREBENDING_SOLDIER];
