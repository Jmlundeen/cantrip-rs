use cantrip_api::*;
use cantrip_mechanics::{targets, tokens};
use cantrip_oracle as oracle;

/// "Orbital Plunge deals 6 damage to target creature. If excess damage was dealt this way, create a Lander token." Excess damage
/// is the damage beyond lethal damage, damage already marked on the creature counted (CR 120.10), judged by the damage procedure
/// after any replacement or prevention; nothing is created when the creature is gone or the damage was prevented (CR 608.2b).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let target = item.target(0);
    if let Some(result) = g.damage_result(item.object, target, 6)
        && result.target == target
        && result.excess > 0
    {
        tokens::create_lander(g, item.controller)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], effect),
    ..oracle::ORBITAL_PLUNGE
};

pub static EXTRA: &[&CardDef] = &[&tokens::LANDER];
