use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "Whenever another nontoken creature you control enters" (CR 603.6a): a creature other than Arni, under his
/// controller's control as it entered, that is not a token.
fn another_nontoken_creature_enters(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { new, to, .. }
            if *new != me.source
                && to.is(ZoneKind::Battlefield)
                && q.controller(*new) == me.controller
                && q.chars(*new).is(CardType::Creature)
                && !q.obj(*new).token =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(when_asked(EventMask::ZONE_CHANGE, another_nontoken_creature_enters), effects::untap_self, "Whenever another nontoken creature you control enters, untap Arni."),
        ability(TAP, effects::loot, "{T}: Draw a card, then discard a card."),
    ],
    ..oracle::ARNI_HUMBLE_SCRIBE
};
