use cantrip_api::*;
use cantrip_oracle as oracle;

/// "White creatures get +1/+1." (CR 604.1).
fn white_creatures_get_1_1_affects(
    q: &dyn LayerQuery,
    source: ObjectId,
    _: ObjectId,
    so_far: &Chars,
) -> bool {
    let asker = &Asker::new(source, q.controller(source));
    (*so_far).colors.has(q.words(asker.source).color(Color::White))
        && (*so_far).is(CardType::Creature)
}

pub static CARD: CardDef = CardDef {
    statics: &[
        static_ability(
            Layer::PtModify,
            white_creatures_get_1_1_affects,
            cantrip_mechanics::statics::plus_pt::<1, 1>,
            "White creatures get +1/+1.",
        ),
    ],
    ..oracle::CRUSADE
};
