use cantrip_api::*;
use cantrip_mechanics::statics::have_hexproof;
use cantrip_oracle as oracle;

/// "Ruric Thar has hexproof as long as they haven't dealt combat damage yet" (CR 702.11, 604.1, 611.3a): only this
/// creature, and only while no combat damage has been dealt by this object anywhere in the game's event history. "Yet" is
/// the whole game, not the turn: once they have dealt combat damage the hexproof is gone for good as this object (a new
/// object after a zone change has not dealt any, CR 400.7). Damage that was prevented was not dealt (CR 615.6), so it is
/// no combat damage here; noncombat damage does not count.
fn itself_until_it_deals_combat_damage(view: &dyn LayerQuery, source: ObjectId, affected: ObjectId, _: &Chars) -> bool {
    // Newest first, up to the zone change that made this object: it can have dealt damage only since then.
    source == affected
        && !view
            .history()
            .all()
            .iter()
            .rev()
            .take_while(|record| !matches!(record.event, Event::ZoneChange { new, .. } if new == source))
            .any(|record| matches!(record.event, Event::Damage { source: from, combat: true, amount, .. } if from == source && amount > 0))
}

pub static CARD: CardDef = CardDef {
    statics: &[static_ability(
        Layer::Ability,
        itself_until_it_deals_combat_damage,
        have_hexproof,
        "Ruric Thar has hexproof as long as they haven't dealt combat damage yet.",
    )],
    ..oracle::RURIC_THAR_MAGECRUSHER
};
