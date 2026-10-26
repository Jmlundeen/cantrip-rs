use cantrip_api::*;
use cantrip_mechanics::{lands, targets};
use cantrip_oracle as oracle;

/// "Exile target creature or Spacecraft. Its controller may search their library for a basic land card, reveal it, put it
/// into their hand, then shuffle." Its controller is the one it last had (CR 608.2h), read before it is exiled; the "may"
/// is asked of that player, and declining skips the search and the shuffle. Nothing happens if the target is illegal.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(target) = targets::object(g, item, 0) else { return Ok(()) };
    let controller = g.controller(target);
    g.exile(target);
    g.end_sentence();
    lands::may_search_basic_land_to_hand(g, controller, item.object)
}

pub static CARD: CardDef = CardDef { spell: spell(&[targets::CREATURE_OR_SPACECRAFT], effect), ..oracle::_70_000_LIGHT_YEARS_FROM_HOME };
