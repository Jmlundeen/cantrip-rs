use cantrip_api::*;
use cantrip_mechanics::{choose, targets};
use cantrip_oracle as oracle;

/// "Target opponent reveals their hand. You may choose a nonland card from it. If you do, they discard it. Otherwise, you
/// may put a face-up exiled card they own into their graveyard." The hand is revealed (an empty one has nothing to show),
/// and the choice from it may be no card. Only when no card was chosen is the exile looked at: any face-up card the
/// opponent owns there, whoever exiled it (CR 406.3: a face-down exiled card is not one), and again the choice may be none.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(opponent) = item.target(0).player() else { return Ok(()) };
    let you = item.controller;
    let hand = g.state().zone(Zone::hand(opponent)).to_vec();
    if !hand.is_empty() {
        g.reveal(opponent, &hand);
    }
    let nonland: Vec<ObjectId> = hand.iter().copied().filter(|&c| !g.chars(c).is(CardType::Land)).collect();
    let chosen = choose::cards_from_kind(g, you, item.object, Purpose::TakeFromHand, "nonland", hand, nonland, 1, true)?;
    if !chosen.is_empty() {
        g.discard_revealed(&chosen);
        return Ok(());
    }
    let exiled: Vec<ObjectId> = g.state().zone(Zone::exile(opponent)).iter().copied().filter(|&c| !g.obj(c).face_down).collect();
    let put = choose::cards(g, you, item.object, Purpose::PutIntoGraveyard, exiled, 1, true)?;
    g.put_into_graveyard(&put);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::OPPONENT], effect),
    ..oracle::BINDING_NEGOTIATION
};
