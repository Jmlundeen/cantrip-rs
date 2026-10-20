use cantrip_api::*;
use cantrip_mechanics::{pump, targets, tokens};
use cantrip_oracle as oracle;

/// "Target creature gets +2/+0 until end of turn. Create a Monster Role token attached to it."
/// Both stop if the target is no longer legal (CR 608.2b). The Role is attached by the effect, not targeted (the Role
/// rulings); one that could not enchant the creature is not created (CR 303.4i), and a Role the caster already controls
/// on it is put into its owner's graveyard (CR 704.5z).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    pump(g, item.object, creature, 2, 0);
    tokens::create_role(g, &tokens::MONSTER_ROLE, item.controller, creature)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], effect),
    ..oracle::MONSTROUS_RAGE
};

pub static EXTRA: &[&CardDef] = &[&tokens::MONSTER_ROLE];
