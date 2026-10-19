use cantrip_api::*;
use cantrip_mechanics::{choose, targets, triggers};
use cantrip_oracle as oracle;

/// "Target opponent reveals their hand. You may choose a nonland card from it. If you do, that player discards that
/// card, then draws a card." The whole hand is revealed and the card is chosen by the controller, up to one, among the
/// nonland cards; choosing none ends the ability.
fn reveal(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Player(opponent) = item.targets[0] else { return Ok(()) };
    let hand = g.state().zone(Zone::hand(opponent)).to_vec();
    if hand.is_empty() {
        return Ok(());
    }
    g.reveal(opponent, &hand);
    let eligible: Vec<ObjectId> = hand.iter().copied().filter(|&c| !g.chars(c).is(CardType::Land)).collect();
    let Some(&card) = choose::cards_from(g, item.controller, item.object, Purpose::Discard, hand, eligible, 1, true)?.first() else {
        return Ok(());
    };
    g.discard(card);
    g.draw(opponent, 1);
    Ok(())
}

pub static BACK: CardDef = CardDef {
    abilities: &[triggered(triggers::THIS_TRANSFORMS, reveal, "When this creature transforms into Revealing Eye, target opponent reveals their hand. You may choose a nonland card from it. If you do, that player discards that card, then draws a card.")
        .targets(&[targets::OPPONENT])],
    ..oracle::REVEALING_EYE
};

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{2}{B}"), cantrip_mechanics::effects::transform_self, "{2}{B}: Transform Concealing Curtains. Activate only as a sorcery.")
        .sorcery_speed()],
    back: Some(&BACK),
    ..oracle::CONCEALING_CURTAINS
};
