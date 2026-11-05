use cantrip_api::*;
use cantrip_mechanics::{delayed, library, triggers};
use cantrip_oracle as oracle;

/// "At the beginning of combat on your turn, reveal cards from the top of your library until you reveal two land cards. Put
/// those land cards onto the battlefield and the rest on the bottom of your library in a random order. Sacrifice those lands
/// at the beginning of your next end step." (CR 701.20, 401.4, 603.7c): both lands are revealed before either is put out,
/// and they enter together (CR 110.2a), so neither is a land the other sees as it enters (two Blackcleave Cliffs both enter untapped); the
/// rest goes back in a random order, and the delayed ability is about the permanents those lands became, not a later
/// object with the same name (CR 400.7).
fn reveal_two_lands(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let (found, rest) = library::reveal_until_n(g, you, 2, &|q, card| q.chars(card).is(CardType::Land));
    let lands: Vec<(ObjectId, PlayerId)> = found.iter().map(|&card| (card, you)).collect();
    let placed = g.put_all_onto_battlefield(&lands)?;
    library::rest_to_bottom_in_random_order(g, you, &rest);
    delayed::at_your_next_end_step_about(g, item, 0, &placed);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::BEGINNING_OF_YOUR_COMBAT,
        reveal_two_lands,
        "At the beginning of combat on your turn, reveal cards from the top of your library until you reveal two land cards. Put those land cards onto the battlefield and the rest on the bottom of your library in a random order. Sacrifice those lands at the beginning of your next end step.",
    )],
    delayed: &[delayed_trigger(delayed::sacrifice_created, "Sacrifice those lands at the beginning of your next end step.")],
    ..oracle::WILL_RIKER_FIRST_IN_THE_FIELD
};
