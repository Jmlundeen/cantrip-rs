use cantrip_api::*;
use cantrip_mechanics::{effects, triggers::creature_you_control_deals_combat_damage_to_player};
use cantrip_oracle as oracle;

/// "Whenever a creature you control with a counter on it deals combat damage to a player" (CR 510.2, 603.2, 122.1): a counter of
/// any kind, on the creature as it dealt the damage.
fn creature_with_a_counter_deals_combat_damage_to_player(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::Damage { source: from, .. } if q.state().obj(*from).has_counters() => creature_you_control_deals_combat_damage_to_player(q, me, event),
        _ => None,
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when_asked(EventMask::DAMAGE, creature_with_a_counter_deals_combat_damage_to_player),
        effects::draw_a_card_and_lose_1_life,
        "Whenever a creature you control with a counter on it deals combat damage to a player, you draw a card and you lose 1 life.",
    )],
    ..oracle::YATHAN_TOMBGUARD
};
