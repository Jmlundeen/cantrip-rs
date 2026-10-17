use cantrip_api::*;
use cantrip_mechanics::{choose, targets, triggers};
use cantrip_oracle as oracle;

/// "You may put a permanent card from your hand onto the battlefield." Up to one permanent card (CR 110.4b); nothing is
/// asked when the hand has none. An Aura chooses what it enchants as it enters (CR 303.4f).
fn put_permanent_card(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    let hand = g.state().zone(Zone::hand(player)).to_vec();
    let eligible: Vec<ObjectId> = hand.iter().copied().filter(|&c| targets::is_permanent_card(&g.chars(c))).collect();
    if let Some(&card) = choose::cards_from(g, player, item.object, Purpose::PutOntoBattlefield, hand, eligible, 1, true)?.first() {
        g.put_onto_battlefield(card, player, false)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::YOUR_SECOND_MAIN_PHASE,
        put_permanent_card,
        "Survival \u{2014} At the beginning of your second main phase, if Kona is tapped, you may put a permanent card from your hand onto the battlefield.",
    )
    .condition(triggers::this_is_tapped)],
    ..oracle::KONA_RESCUE_BEASTIE
};
