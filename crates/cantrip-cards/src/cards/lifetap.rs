use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Whenever a Forest an opponent controls becomes tapped" (CR 603.2).
fn when_forest_opponent_controls_becomes_tapped(
    q: &dyn Query,
    asker: &Asker,
    event: &Event,
) -> Option<Fired> {
    let Event::Tapped { permanent } = event else { return None };
    let id = *permanent;
    let subject = Target::Object(id);
    if q.chars(id).has_subtype(q.words(asker.source).subtype(Subtype::Forest))
        && q
            .state()
            .opponents(asker.controller)
            .collect::<Vec<PlayerId>>()
            .contains(&q.controller(id))
    {
        return Fired::about(subject);
    }
    None
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            when_asked(EventMask::TAPPED, when_forest_opponent_controls_becomes_tapped),
            cantrip_mechanics::effects::gain_one_life,
            "Whenever a Forest an opponent controls becomes tapped, you gain 1 life.",
        ),
    ],
    ..oracle::LIFETAP
};
