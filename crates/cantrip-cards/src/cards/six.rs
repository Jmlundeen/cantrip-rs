use cantrip_api::*;
use cantrip_mechanics::{choose, retrace};
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

/// Mill three cards. You may put a land card from among them into your hand (CR 701.17c); nobody is asked when no
/// land is among them.
fn mill_and_take(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    let among = g.mill(player, 3);
    let eligible: Vec<ObjectId> = among.iter().copied().filter(|&c| g.chars(c).is(CardType::Land)).collect();
    let chosen = choose::cards_from(g, player, item.object, Purpose::PutIntoHand, among, eligible, 1, true)?;
    g.return_to_hand(&chosen);
    Ok(())
}

/// During your turn, nonland permanent cards in your graveyard have retrace (CR 702.81a): the permission is Six's, so
/// it reaches the cards in its controller's graveyard whatever they are, and only on that player's own turn.
fn your_nonland_permanent_cards(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    let controller = q.state().obj(source).controller;
    q.state().turn.active == controller
        && q.state().obj(affected).zone == Zone::graveyard(controller)
        && !so_far.is(CardType::Land)
        && [CardType::Artifact, CardType::Battle, CardType::Creature, CardType::Enchantment, CardType::Planeswalker]
            .into_iter()
            .any(|t| so_far.is(t))
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(triggers::THIS_ATTACKS, mill_and_take, "Whenever Six attacks, mill three cards. You may put a land card from among them into your hand.")],
    statics: &[StaticDef {
        text: "During your turn, nonland permanent cards in your graveyard have retrace.",
        layer: Layer::Ability,
        affects: your_nonland_permanent_cards,
        apply: retrace::give,
        everywhere: true,
        ..StaticDef::BLANK
    }],
    ..oracle::SIX
};
