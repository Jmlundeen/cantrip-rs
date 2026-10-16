use cantrip_api::*;
use cantrip_oracle as oracle;

/// "When you play another land" (CR 305.1): the special action of playing a land only, not a land put onto the
/// battlefield another way.
fn played_another_land(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { new, to, reason: ZoneReason::PlayLand, .. }
            if *new != source && to.is(ZoneKind::Battlefield) && q.controller(*new) == q.controller(source) =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add_n(Mana::C, 2), "{T}: Add {C}{C}."),
        triggered(when(EventMask::ZONE_CHANGE, played_another_land), cantrip_mechanics::effects::sacrifice_self, "When you play another land, sacrifice this land."),
    ],
    ..oracle::CITY_OF_TRAITORS
};
