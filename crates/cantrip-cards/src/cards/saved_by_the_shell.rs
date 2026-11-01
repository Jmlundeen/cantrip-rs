use cantrip_api::*;
use cantrip_mechanics::effects::plus_one_counters_on_first_target_that_gains_until_end_of_turn;
use cantrip_mechanics::{cost_reduction, lands, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    spell: spell(
        &[targets::CREATURE_YOU_CONTROL],
        plus_one_counters_on_first_target_that_gains_until_end_of_turn::<
            1,
            { Keywords::of(&[Keyword::Trample, Keyword::Hexproof, Keyword::Indestructible]).0 },
        >,
    ),
    own_cost_changes: &[cost_reduction::less_if_you_control_subtype::<1, { lands::sub(Subtype::Turtle) }>],
    ..oracle::SAVED_BY_THE_SHELL
};
