use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_mechanics::choose_from_hand;
use cantrip_mechanics::triggers::DEALS_COMBAT_DAMAGE_TO_PLAYER;
use cantrip_oracle as oracle;

/// Put a chorus counter on it (if it is still on the battlefield), draw a card, then discard a card; with four or more
/// chorus counters on it - as it last was, when it has left (CR 608.2h) - the discarded card may be cast for free from
/// where it went, as the ability resolves (CR 608.2g). A land, or a card with no legal target, cannot be cast.
fn chorus(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let malcolm = item.object;
    let player = item.controller;
    if targets::permanent(g, malcolm) {
        g.add_counters(malcolm, CounterKind::Chorus, 1);
    }
    g.draw(player, 1);
    let chosen = choose_from_hand(g, player, malcolm, 1)?;
    let Some(&card) = chosen.first() else { return Ok(()) };
    g.discard(card);
    // A card a replacement effect put somewhere hidden cannot be found there (CR 400.7j).
    let moved = g.obj(card).became.unwrap_or(card);
    let zone = g.obj(moved).zone;
    if zone.is(ZoneKind::Hand) || zone.is(ZoneKind::Library) {
        return Ok(());
    }
    if g.obj(malcolm).counters(CounterKind::Chorus) < 4 || !g.could_cast_during_resolution(moved) {
        return Ok(());
    }
    if cantrip_mechanics::effects::ask_may(g, player, moved, Purpose::Cast)? {
        g.cast_during_resolution(moved, Some(ManaCost::ZERO))?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(DEALS_COMBAT_DAMAGE_TO_PLAYER, chorus, "Whenever Malcolm deals combat damage to a player, put a chorus counter on it. Draw a card, then discard a card. If there are four or more chorus counters on Malcolm, you may cast the discarded card without paying its mana cost.")],
    ..oracle::MALCOLM_ALLURING_SCOUNDREL
};
