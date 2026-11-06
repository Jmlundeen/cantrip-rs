use cantrip_api::*;
use cantrip_mechanics::cast_from_your_graveyard;
use cantrip_mechanics::statics::plus_pt;
use cantrip_oracle as oracle;

/// "As long as there are three or more different kinds of counters among creatures you control, this creature gets +2/+4"
/// (CR 613.4c, 122.1): itself, while the creatures its controller controls (as the earlier layers made them) carry counters of
/// at least three kinds between them.
fn itself_with_three_kinds_of_counters(view: &dyn LayerQuery, source: ObjectId, affected: ObjectId, _: &Chars) -> bool {
    let state = view.state();
    let you = state.obj(source).controller;
    affected == source
        && CounterKind::ALL
            .iter()
            .filter(|&&kind| {
                state.battlefield().iter().any(|&id| state.obj(id).controller == you && view.chars(id).is(CardType::Creature) && state.obj(id).counters(kind) > 0)
            })
            .count()
            >= 3
}

pub static CARD: CardDef = CardDef {
    enters: Some(cast_from_your_graveyard::enters_with_finality_if_cast_by_it),
    statics: &[static_ability(
        Layer::PtModify,
        itself_with_three_kinds_of_counters,
        plus_pt::<2, 4>,
        "As long as there are three or more different kinds of counters among creatures you control, this creature gets +2/+4.",
    )],
    exile_casts: &[cast_from_your_graveyard::PERMISSION],
    ..oracle::HUNDRED_BATTLE_VETERAN
};
