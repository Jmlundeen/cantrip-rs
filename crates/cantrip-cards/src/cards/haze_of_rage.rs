use cantrip_api::*;
use cantrip_mechanics::{effects, storm};
use cantrip_mechanics::replacements::BUYBACK;
use cantrip_oracle as oracle;

/// Creatures you control get +1/+0 until end of turn: those there as it resolves (CR 611.2c).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::pump_creatures_you_control(g, item, 1, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], effect),
    kicker: kicker(mana("{2}")),
    replacements: &[BUYBACK],
    abilities: &[storm::trigger("Storm (When you cast Haze of Rage, copy it for each spell cast before it this turn.)")],
    ..oracle::HAZE_OF_RAGE
};
