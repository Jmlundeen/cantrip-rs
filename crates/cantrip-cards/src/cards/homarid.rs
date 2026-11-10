use cantrip_api::*;
use cantrip_mechanics::{effects, statics, tide};
use cantrip_oracle as oracle;

/// "As long as there is exactly one tide counter on this creature, it gets -1/-1." (CR 604.1).
fn exactly_one_tide_counter(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, _: &Chars) -> bool {
    q.state().obj(source).counters(CounterKind::Tide) == 1 && source == affected
}

/// "As long as there are exactly three tide counters on this creature, it gets +1/+1." (CR 604.1).
fn exactly_three_tide_counters(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, _: &Chars) -> bool {
    q.state().obj(source).counters(CounterKind::Tide) == 3 && source == affected
}

pub static CARD: CardDef = CardDef {
    abilities: &tide::cycle(
        "At the beginning of your upkeep, put a tide counter on this creature.",
        "Whenever there are four or more tide counters on this creature, remove all tide counters from it.",
    ),
    statics: &[
        static_ability(
            Layer::PtModify,
            exactly_one_tide_counter,
            statics::plus_pt::<-1, -1>,
            "As long as there is exactly one tide counter on this creature, it gets -1/-1.",
        ),
        static_ability(
            Layer::PtModify,
            exactly_three_tide_counters,
            statics::plus_pt::<1, 1>,
            "As long as there are exactly three tide counters on this creature, it gets +1/+1.",
        ),
    ],
    enters: Some(effects::enters_with_a_counter::<{ CounterKind::Tide as usize }>),
    ..oracle::HOMARID
};
