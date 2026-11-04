use cantrip_api::*;
use cantrip_mechanics::{effects, same_name, statics, tide};
use cantrip_oracle as oracle;

/// "Cast this spell only if no permanents named Tidal Influence are on the battlefield." (CR 601.2e, 201.2).
fn no_permanent_named_this(q: &dyn Query, asker: &Asker) -> bool {
    !q.state().battlefield().iter().any(|&id| same_name(q, id, asker.source))
}

/// "All blue creatures" while there is exactly one tide counter on this enchantment (CR 604.1, 613.1f).
fn blue_creatures_while_one_tide_counter(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    q.state().obj(source).counters(CounterKind::Tide) == 1
        && statics::creatures_of_color::<{ Color::Blue as u8 }>(q, source, affected, so_far)
}

/// "All blue creatures" while there are exactly three tide counters on this enchantment (CR 604.1, 613.1f).
fn blue_creatures_while_three_tide_counters(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    q.state().obj(source).counters(CounterKind::Tide) == 3
        && statics::creatures_of_color::<{ Color::Blue as u8 }>(q, source, affected, so_far)
}

pub static CARD: CardDef = CardDef {
    abilities: &tide::cycle(
        "At the beginning of your upkeep, put a tide counter on this enchantment.",
        "Whenever there are four or more tide counters on this enchantment, remove all tide counters from it.",
    ),
    statics: &[
        static_ability(
            Layer::PtModify,
            blue_creatures_while_one_tide_counter,
            statics::plus_pt::<-2, 0>,
            "As long as there is exactly one tide counter on this enchantment, all blue creatures get -2/-0.",
        ),
        static_ability(
            Layer::PtModify,
            blue_creatures_while_three_tide_counters,
            statics::plus_pt::<2, 0>,
            "As long as there are exactly three tide counters on this enchantment, all blue creatures get +2/+0.",
        ),
    ],
    enters: Some(effects::enters_with_a_counter::<{ CounterKind::Tide as usize }>),
    cast_only_if: Some(no_permanent_named_this),
    ..oracle::TIDAL_INFLUENCE
};
