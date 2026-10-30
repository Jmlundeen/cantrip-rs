use cantrip_api::*;
use cantrip_mechanics::{effects, library, targets};
use cantrip_oracle as oracle;

/// Mill four cards (CR 701.17a), optionally take a permanent card (CR 110.4a, 701.17c), then gain 2 life (CR 119.3)
/// only when this was your first spell this game. The resolving spell is already in history (CR 601.2i), so it is the
/// sole cast recorded for its controller when the count is one.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    library::mill_then_may_take_into_hand(g, item.controller, item.object, 4, "permanent", targets::is_permanent_card)?;
    if g.history().casts_this_game(item.controller) == 1 {
        effects::gain_life(g, item, 2)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::FIRST_CONTACT };
