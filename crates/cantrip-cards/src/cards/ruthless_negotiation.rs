use cantrip_api::*;
use cantrip_mechanics::{choose, effects, flashback, targets};
use cantrip_oracle as oracle;

/// "Target opponent exiles a card from their hand": the opponent chooses it (CR 608.2d), asked even when it is their only
/// card, and none is asked of an empty hand. "If this spell was cast from a graveyard, draw a card" (CR 601.2a): the zone the
/// spell was cast from, by any permission and not only by its own flashback; a copy of it was never cast (CR 707.10), so it
/// draws nothing.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(opponent) = targets::player(g, item, 0) else { return Ok(()) };
    let hand = g.state().zone(Zone::hand(opponent)).to_vec();
    if let Some(card) = choose::card(g, opponent, item.object, Purpose::Exile, hand)? {
        g.exile_all(&[card]);
    }
    if g.obj(item.object).cast_from == Some(ZoneKind::Graveyard) {
        effects::draw(g, item, 1)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::OPPONENT], effect),
    ways: &[flashback::way(cost("{4}{B}"), "Flashback {4}{B}")],
    ..oracle::RUTHLESS_NEGOTIATION
};
