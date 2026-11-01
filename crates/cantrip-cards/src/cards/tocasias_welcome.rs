use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Whenever one or more creatures you control with mana value 3 or less enter" (CR 603.2c, 603.6a): one trigger for the
/// batch that entered together, which the subjects of the matching events make up. The mana value is the entering
/// permanent's own (CR 202.3: a token that is a copy has its copied value, a token with no mana cost 0).
fn small_creatures_enter(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { new, to, .. }
            if to.is(ZoneKind::Battlefield)
                && q.controller(*new) == me.controller
                && q.chars(*new).is(CardType::Creature)
                && q.mana_value(*new) <= 3 =>
        {
            Fired::about(Target::Object(*new))
        }
        _ => None,
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(when_asked(EventMask::ZONE_CHANGE, small_creatures_enter), |g, item| cantrip_mechanics::effects::draw(g, item, 1), "Whenever one or more creatures you control with mana value 3 or less enter, draw a card. This ability triggers only once each turn.")
        .triggers_each_turn(1)
        .once_per_batch()],
    ..oracle::TOCASIAS_WELCOME
};
