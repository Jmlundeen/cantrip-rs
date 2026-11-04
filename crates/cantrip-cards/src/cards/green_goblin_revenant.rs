use cantrip_api::*;
use cantrip_mechanics::{history, triggers};
use cantrip_oracle as oracle;

/// "Discard a card. Then draw a card for each card you've discarded this turn." The count is taken after this discard
/// (CR 701.9), so it includes the discard just made and any made earlier in the turn; an empty hand discards nothing.
fn discard_then_draw_per_discard(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::discard_chosen(g, item.controller, item.object, 1)?;
    let n = history::cards_discarded_this_turn(g, item.controller);
    g.draw(item.controller, n);
    Ok(())
}

/// Flying and deathtouch are core keywords.
pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::THIS_ATTACKS,
        discard_then_draw_per_discard,
        "Whenever Green Goblin attacks, discard a card. Then draw a card for each card you've discarded this turn.",
    )],
    ..oracle::GREEN_GOBLIN_REVENANT
};
