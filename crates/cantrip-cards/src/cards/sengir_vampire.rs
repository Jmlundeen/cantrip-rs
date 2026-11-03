use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "Whenever a creature dealt damage by this creature this turn dies" (CR 603.2).
fn when_creature_dealt_damage_by_creature(
    q: &dyn Query,
    asker: &Asker,
    event: &Event,
) -> Option<Fired> {
    let Event::ZoneChange { old, from, to, .. } = event else { return None };
    if from.is(ZoneKind::Battlefield) && to.is(ZoneKind::Graveyard)
        && q.chars(*old).is(CardType::Creature)
        && q.history().this_turn().iter().any(|r| matches!(r.event,
            Event::Damage { source, target: Target::Object(target), amount, .. }
                if source == asker.source && target == *old && amount > 0))
    {
        return Fired::about(Target::Object(*old));
    }
    None
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            when_asked(EventMask::ZONE_CHANGE, when_creature_dealt_damage_by_creature),
            effects::plus_one_counter_on_this,
            "Whenever a creature dealt damage by this creature this turn dies, put a +1/+1 counter on this creature.",
        ),
    ],
    ..oracle::SENGIR_VAMPIRE
};
