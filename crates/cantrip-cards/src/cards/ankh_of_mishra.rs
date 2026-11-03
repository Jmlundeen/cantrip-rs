use cantrip_api::*;
use cantrip_mechanics::effects::ping_subject_controller;
use cantrip_oracle as oracle;

/// "Whenever a land enters" (CR 603.2).
fn when_land_enters(q: &dyn Query, _: &Asker, event: &Event) -> Option<Fired> {
    let Event::ZoneChange { new, to, .. } = event else { return None };
    let id = *new;
    let subject = Target::Object(id);
    if to.is(ZoneKind::Battlefield) && q.chars(id).is(CardType::Land) {
        return Fired::about(subject);
    }
    None
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            when_asked(EventMask::ZONE_CHANGE, when_land_enters),
            ping_subject_controller::<2>,
            "Whenever a land enters, this artifact deals 2 damage to that land's controller.",
        ),
    ],
    ..oracle::ANKH_OF_MISHRA
};
