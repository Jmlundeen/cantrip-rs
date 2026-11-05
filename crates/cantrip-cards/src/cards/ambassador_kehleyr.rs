use cantrip_api::*;
use cantrip_mechanics::parley::parley;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

/// Parley (CR 701.20, 121.1): every player reveals the top card of their library, then each draws a card, and the counters
/// are put on K'Ehleyr as the reveal is read - a card revealed and then drawn is still the card revealed, so a land
/// revealed gives none even though it is in its owner's hand by now (CR 400.7).
fn parley_counters(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let revealed = parley(g);
    let nonland = revealed.into_iter().filter(|&card| !g.chars(card).is(CardType::Land)).count() as i32;
    g.add_counters(item.object, CounterKind::PlusOne, nonland);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::BEGINNING_OF_YOUR_COMBAT,
        parley_counters,
        "Parley — At the beginning of combat on your turn, each player reveals the top card of their library, then draws a card. For each nonland card revealed this way, put a +1/+1 counter on Ambassador K'Ehleyr.",
    )],
    ..oracle::AMBASSADOR_KEHLEYR
};
