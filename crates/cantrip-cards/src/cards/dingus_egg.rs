use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "Whenever a land is put into a graveyard from the battlefield" (CR 603.2).
fn when_land_put_into_graveyard_battlefield(
    q: &dyn Query,
    _: &Asker,
    event: &Event,
) -> Option<Fired> {
    let Event::ZoneChange { old, from, to, .. } = event else { return None };
    let id = *old;
    if to.is(ZoneKind::Graveyard) && from.is(ZoneKind::Battlefield)
        && (q.chars(id).is(CardType::Land))
    {
        return Fired::about(Target::Object(id));
    }
    None
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            when_asked(EventMask::ZONE_CHANGE, when_land_put_into_graveyard_battlefield),
            effects::ping_subject_controller::<2>,
            "Whenever a land is put into a graveyard from the battlefield, this artifact deals 2 damage to that land's controller.",
        ),
    ],
    ..oracle::DINGUS_EGG
};
