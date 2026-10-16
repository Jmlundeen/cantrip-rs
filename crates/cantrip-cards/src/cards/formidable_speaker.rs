use cantrip_api::*;
use cantrip_mechanics::choose;
use cantrip_mechanics::targets;
use cantrip_mechanics::library::{search_to_hand, Find};
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// When this creature enters, you may discard a card. If you do, search your library for a creature card, reveal it,
/// put it into your hand, then shuffle. An empty hand asks nothing; choosing no card declines.
fn discard_and_search(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    let hand = g.state().zone(Zone::hand(player)).to_vec();
    let cards = choose::cards(g, player, item.object, Purpose::Discard, hand, 1, true)?;
    let Some(&card) = cards.first() else { return Ok(()) };
    g.discard(card);
    search_to_hand(g, player, item.object, Find::ONE.revealed(), |q, c| q.chars(c).is(CardType::Creature))?;
    Ok(())
}

/// "Another target permanent": any permanent but the source.
fn other_permanent(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if id != asker.source && targets::permanent(q, id))
}

fn untap_target(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(permanent) = targets::object(g, item, 0) {
        g.untap(permanent);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, discard_and_search, "When this creature enters, you may discard a card. If you do, search your library for a creature card, reveal it, put it into your hand, then shuffle."),
        ability(cost("{1}, {T}"), untap_target, "{1}, {T}: Untap another target permanent.")
            .targets(&[TargetSpec { min: 1, max: 1, filter: other_permanent }]),
    ],
    ..oracle::FORMIDABLE_SPEAKER
};
