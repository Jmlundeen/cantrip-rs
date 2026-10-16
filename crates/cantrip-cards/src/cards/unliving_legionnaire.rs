use cantrip_api::*;
use cantrip_mechanics::{power_up, targets};
use cantrip_oracle as oracle;

/// "Return up to one target creature card from your graveyard to your hand. Put two +1/+1 counters on this creature."
/// A target no longer there is left alone (CR 608.2b); the counters are put on all the same.
fn return_and_counters(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(card) = targets::object(g, item, 0) {
        g.return_to_hand(&[card]);
    }
    g.add_counters(item.object, CounterKind::PlusOne, 2);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[power_up::ability(
        cost("{5}{B}{B}"),
        return_and_counters,
        "Power-up \u{2014} {5}{B}{B}: Return up to one target creature card from your graveyard to your hand. Put two +1/+1 counters on this creature. (Activate each power-up ability only once. Reduce the cost by its mana cost if it entered this turn.)",
    )
    .targets(&[targets::CREATURE_CARD_IN_YOUR_GRAVEYARD.optional()])],
    ..oracle::UNLIVING_LEGIONNAIRE
};
