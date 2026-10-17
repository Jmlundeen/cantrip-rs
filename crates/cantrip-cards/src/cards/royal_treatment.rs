use cantrip_api::*;
use cantrip_mechanics::{effects, targets, tokens};
use cantrip_oracle as oracle;

/// "Target creature you control gains hexproof until end of turn. Create a Royal Role token attached to that creature."
/// Both stop if the target is no longer legal (CR 608.2b). The Role is attached by the effect, not targeted, so the
/// hexproof just given does not stop it (the Role rulings); one that could not enchant the creature is not created
/// (CR 303.4i), and a Role the caster already controls on it is put into its owner's graveyard (CR 704.5z).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    effects::gain_hexproof_until_end_of_turn(g, item.object, creature);
    tokens::create_role(g, &tokens::ROYAL_ROLE, item.controller, creature)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE_YOU_CONTROL], effect),
    ..oracle::ROYAL_TREATMENT
};

pub static EXTRA: &[&CardDef] = &[&tokens::ROYAL_ROLE];
