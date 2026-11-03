use cantrip_api::*;
use cantrip_mechanics::{learn, targets, triggers};
use cantrip_oracle as oracle;

/// "Target player mills X cards, where X is the number of Lesson cards in your graveyard" (CR 701.17a, 205.3k): "your" is the
/// ability's controller, not the target, and X is counted as the ability resolves (CR 608.2c), after the mill is decided, in
/// that graveyard alone. With its target gone it does not resolve (CR 608.2b).
fn mill_x(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(player) = targets::player(g, item, 0) else { return Ok(()) };
    let x = learn::lesson_cards_in_graveyard(g, item.controller) as u32;
    g.mill(player, x);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggers::PROWESS,
        triggered(
            triggers::THIS_BECOMES_TAPPED,
            mill_x,
            "Whenever Master Pakku becomes tapped, target player mills X cards, where X is the number of Lesson cards in your graveyard. (They put the top X cards of their library into their graveyard.)",
        )
        .targets(&[targets::PLAYER]),
    ],
    ..oracle::MASTER_PAKKU
};
