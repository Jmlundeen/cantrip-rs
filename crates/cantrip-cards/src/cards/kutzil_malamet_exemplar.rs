use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::statics;
use cantrip_mechanics::triggers::creature_you_control_deals_combat_damage_to_player;
use cantrip_oracle as oracle;

/// "Whenever one or more creatures you control each with power greater than its base power deals combat damage to a
/// player" (CR 603.2c, 120.4b): the power is read as the damage is dealt, before it is processed into its results
/// (CR 120.4c), so neither that damage's lifelink gain nor its wither or infect counters count. One trigger for each player
/// the batch damaged.
fn creature_above_base_power_deals_combat_damage_to_player(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::Damage { source, .. } if q.chars(*source).power_above_base() => {
            creature_you_control_deals_combat_damage_to_player(q, me, event)
        }
        _ => None,
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when_asked(EventMask::DAMAGE, creature_above_base_power_deals_combat_damage_to_player),
        |g, item| effects::draw(g, item, 1),
        "Whenever one or more creatures you control each with power greater than its base power deals combat damage to a player, draw a card.",
    )
    .sums_per_subject()],
    cast_rules: &[statics::opponents_cant_cast_spells_during_your_turn],
    ..oracle::KUTZIL_MALAMET_EXEMPLAR
};
