use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{effects, mayhem};
use cantrip_oracle as oracle;

/// "You may discard a card. If you do, put a +1/+1 counter on him" (CR 701.9, 122.1): the discard is chosen as the ability resolves
/// and is not a cost, so an empty hand does nothing; Kaine gets the counter only if a card was discarded, and only if he is still
/// the same object on the battlefield (CR 400.7).
fn discard_for_counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if effects::may_discard_a_card(g, item)? {
        g.add_counters(item.object, CounterKind::PlusOne, 1);
    }
    Ok(())
}

/// Menace is a core keyword.
pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, discard_for_counter, "When Scarlet Spider enters, you may discard a card. If you do, put a +1/+1 counter on him.")],
    ways: &[mayhem::way(mana("{B/R}"), "Mayhem {B/R}")],
    ..oracle::SCARLET_SPIDER_KAINE
};
