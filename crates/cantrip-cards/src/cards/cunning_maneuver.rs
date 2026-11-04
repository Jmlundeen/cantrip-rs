use cantrip_api::*;
use cantrip_mechanics::{pump, targets, tokens};
use cantrip_oracle as oracle;

/// "Target creature gets +3/+1 until end of turn. Create a Clue token." (CR 611.2c, 111.10f.) If the target became illegal the
/// spell does not resolve at all (CR 608.2b), the Clue included. Creating a Clue is not investigating (the ruling).
fn pump_and_clue(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    pump(g, item.object, creature, 3, 1);
    tokens::create(g, item.controller, &tokens::CLUE, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], pump_and_clue),
    ..oracle::CUNNING_MANEUVER
};

pub static EXTRA: &[&CardDef] = &[&tokens::CLUE];
