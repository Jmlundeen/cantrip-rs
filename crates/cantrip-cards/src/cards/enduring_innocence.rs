use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Whenever one or more other creatures you control with power 2 or less enter" (CR 603.2c): one trigger for the
/// batch that entered together, which the subjects of the matching events make up.
fn small_creatures_enter(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { new, to, .. }
            if *new != source
                && to.is(ZoneKind::Battlefield)
                && q.controller(*new) == q.controller(source)
                && q.chars(*new).is(CardType::Creature)
                && q.chars(*new).power.is_some_and(|p| p <= 2) =>
        {
            Fired::about(Target::Object(*new))
        }
        _ => None,
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(when(EventMask::ZONE_CHANGE, small_creatures_enter), |g, item| cantrip_mechanics::effects::draw(g, item, 1), "Whenever one or more other creatures you control with power 2 or less enter, draw a card. This ability triggers only once each turn.")
            .triggers_each_turn(1)
            .once_per_batch(),
        triggered(when(EventMask::ZONE_CHANGE, cantrip_mechanics::enduring::dies_as_a_creature), cantrip_mechanics::enduring::return_as_enchantment, "When Enduring Innocence dies, if it was a creature, return it to the battlefield under its owner's control. It's an enchantment. (It's not a creature.)")
            .look_back(),
    ],
    ..oracle::ENDURING_INNOCENCE
};
