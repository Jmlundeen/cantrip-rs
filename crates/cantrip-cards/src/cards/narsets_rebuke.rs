use cantrip_api::*;
use cantrip_mechanics::replacements::{EXILE_INSTEAD_OF_DYING, exile_if_it_would_die};
use cantrip_mechanics::{damage_target, targets};
use cantrip_oracle as oracle;

/// "Narset's Rebuke deals 5 damage to target creature. Add {U}{R}{W}. If that creature would die this turn, exile it
/// instead": the replacement is the spell's own effect on that creature for the turn (CR 611.2c, 614.1a), as Bot Bashing
/// Time's is. With its one target gone the spell does not resolve at all, so no mana comes (CR 608.2b).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    exile_if_it_would_die(g, item, 0, creature);
    damage_target(g, item, 0, 5);
    for mana in [Mana::U, Mana::R, Mana::W] {
        g.add_mana(item.controller, mana, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], effect),
    granted_replacements: &[EXILE_INSTEAD_OF_DYING.text("If that creature would die this turn, exile it instead.")],
    ..oracle::NARSETS_REBUKE
};
