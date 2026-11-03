use cantrip_api::*;
use cantrip_oracle as oracle;

/// "This creature gets +1/+1 as long as an opponent controls a nontoken white permanent." (CR 604.1).
fn creature_gets_1_1_as_affects(
    q: &dyn LayerQuery,
    source: ObjectId,
    affected: ObjectId,
    _: &Chars,
) -> bool {
    let asker = &Asker::new(source, q.controller(source));
    (q
        .state()
        .opponents(asker.controller)
        .collect::<Vec<PlayerId>>()
        .into_iter()
        .any(|bound_0: PlayerId| {
            q
                .state()
                .battlefield()
                .iter()
                .copied()
                .filter(|&id| {
                    q.controller(id) == Some(bound_0).unwrap_or(PlayerId(u8::MAX))
                        && (!(q.state().obj(id).token)
                            && q
                                .chars(id)
                                .colors
                                .has(q.words(asker.source).color(Color::White)))
                })
                .count() >= 1
        })) && (source == affected)
}

pub static CARD: CardDef = CardDef {
    statics: &[
        cantrip_mechanics::protection::from::<{ Color::Red as u8 }>("Protection from red"),
        static_ability(
            Layer::PtModify,
            creature_gets_1_1_as_affects,
            cantrip_mechanics::statics::plus_pt::<1, 1>,
            "This creature gets +1/+1 as long as an opponent controls a nontoken white permanent.",
        ),
    ],
    ..oracle::BEASTS_OF_BOGARDAN
};
