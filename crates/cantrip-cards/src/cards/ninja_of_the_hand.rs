use cantrip_api::*;
use cantrip_mechanics::{choose_from_hand, power_up};
use cantrip_oracle as oracle;

/// "Each opponent discards a card. Put a +1/+1 counter on this creature." Each opponent chooses in turn order (CR
/// 101.4) and the cards are discarded together (CR 701.9).
fn discard_and_counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let mut chosen = Vec::new();
    for opponent in g.state().opponents(item.controller).collect::<Vec<_>>() {
        if !cantrip_mechanics::players::reached(g, opponent) {
            continue;
        }
        chosen.extend(choose_from_hand(g, opponent, item.object, 1)?);
    }
    g.discard_together(&chosen);
    g.add_counters(item.object, CounterKind::PlusOne, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[power_up::ability(
        cost("{4}{B}"),
        discard_and_counter,
        "Power-up \u{2014} {4}{B}: Each opponent discards a card. Put a +1/+1 counter on this creature. (Activate each power-up ability only once. Reduce the cost by its mana cost if it entered this turn.)",
    )],
    ..oracle::NINJA_OF_THE_HAND
};
