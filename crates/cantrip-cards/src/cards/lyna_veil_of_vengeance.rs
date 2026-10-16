use cantrip_api::*;
use cantrip_mechanics::{effects, targets, triggers};
use cantrip_oracle as oracle;

/// "Whenever a creature you control with shadow deals combat damage to a player" (CR 510.2, 603.2): one trigger for each
/// creature and player, the creature judged as it is when the damage is dealt - with shadow printed, granted or from a
/// shadow counter (CR 122.1b).
fn shadow_creature_connects(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::Damage { source, .. } if q.chars(*source).keywords.has(Keyword::Shadow) => {
            triggers::creature_you_control_deals_combat_damage_to_player(q, me, event)
        }
        _ => None,
    }
}

/// "Put a shadow counter on another target creature" (CR 122.1b): nothing if the target is no longer legal (CR 608.2b).
fn shadow_counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    g.add_counters(creature, CounterKind::Shadow, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(when_asked(EventMask::DAMAGE, shadow_creature_connects), |g, item| effects::draw(g, item, 1), "Whenever a creature you control with shadow deals combat damage to a player, draw a card."),
        ability(cost("{3}"), shadow_counter, "{3}: Put a shadow counter on another target creature. Activate only as a sorcery.")
            .sorcery_speed()
            .targets(&[targets::OTHER_CREATURE]),
    ],
    ..oracle::LYNA_VEIL_OF_VENGEANCE
};
