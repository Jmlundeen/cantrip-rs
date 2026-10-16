use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// Exile target creature. Its controller may search their library for a basic land card, put that card onto the
/// battlefield tapped, then shuffle. Its controller is the one it last had (CR 608.2h); nothing happens at all if
/// the creature is no longer a legal target. The "may" is asked of that player, and declining skips the search and
/// the shuffle.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    let owner = g.controller(creature);
    g.exile(creature);
    cantrip_mechanics::lands::may_fetch_basic(g, owner, item.object, true)
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], effect),
    ..oracle::PATH_TO_EXILE
};
