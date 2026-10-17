use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Players can't get counters" (CR 122.1, 101.2): no player gets any - poison from infect damage or an effect, energy,
/// experience. The counters a player already has stay (the ruling).
fn players_cant_get_counters(_: &dyn Query, _: &Asker, _: PlayerId, _: PlayerCounter) -> bool {
    true
}

/// "Counters can't be put on artifacts, creatures, enchantments, or lands" (CR 122.6, 101.2): of any kind, by an effect, as a
/// cost (which then can't be paid, the ruling) or as the permanent enters with them (the ruling), judged on the permanent
/// as it is, or as it would exist on the battlefield. Planeswalkers and battles that are none of these still get theirs, and
/// cards in other zones do (a suspended card's time counters, the ruling). The counters already there stay.
fn no_counters_on_artifacts_creatures_enchantments_or_lands(q: &dyn Query, _: &Asker, object: ObjectId, _: PlayerId, _: CounterKind) -> bool {
    let chars = q.chars(object);
    [CardType::Artifact, CardType::Creature, CardType::Enchantment, CardType::Land].into_iter().any(|t| chars.is(t))
}

pub static CARD: CardDef = CardDef {
    player_counter_bans: &[players_cant_get_counters],
    counter_bans: &[no_counters_on_artifacts_creatures_enchantments_or_lands],
    ..oracle::SOLEMNITY
};
