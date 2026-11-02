use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Target opponent reveals their hand. Exile all noncreature, nonland cards from that player's hand and graveyard." (CR 701.20a,
/// 701.13a): the hand is shown first, and the sentence ends before the exile (CR 608.2c). The exile is simultaneous (CR 101.4).
fn reveal_hand_then_exile(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(opponent) = item.target(0).player() else { return Ok(()) };
    let hand = g.state().zone(Zone::hand(opponent)).to_vec();
    g.reveal(opponent, &hand);
    g.end_sentence();
    let graveyard = g.state().zone(Zone::graveyard(opponent)).to_vec();
    let exiled: Vec<ObjectId> =
        hand.into_iter().chain(graveyard).filter(|&c| !g.chars(c).is(CardType::Creature) && !g.chars(c).is(CardType::Land)).collect();
    g.exile_all(&exiled);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::OPPONENT], reveal_hand_then_exile),
    ..oracle::THOUGHT_DISTORTION
};
