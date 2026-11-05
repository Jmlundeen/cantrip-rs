use cantrip_api::*;
use cantrip_mechanics::{targets, triggers};
use cantrip_oracle as oracle;

/// "you may return target permanent card with mana value 3 or less from your graveyard to the battlefield" (CR 608.2b,
/// 110.2a): the target is chosen as the trigger goes on the stack and the return is optional; the card enters under its
/// controller's control, untapped, and nothing happens if the target is no longer there.
fn return_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(card) = targets::object(g, item, 0) else { return Ok(()) };
    g.put_onto_battlefield(card, item.controller, false)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::THIS_ENTERS_OR_ATTACKS,
        return_it,
        "Whenever this creature enters or attacks, you may return target permanent card with mana value 3 or less from your graveyard to the battlefield.",
    )
    .targets(&[targets::PERMANENT_CARD_IN_YOUR_GRAVEYARD_WITH_MANA_VALUE_3_OR_LESS.optional()])],
    ..oracle::SUN_TITAN
};
