use cantrip_api::*;
use cantrip_mechanics::fight::plus_one_counters_on_first_target_then_it_fights_second;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE_YOU_CONTROL, targets::CREATURE_AN_OPPONENT_CONTROLS], plus_one_counters_on_first_target_then_it_fights_second::<2>),
    ..oracle::TROLL_NEGOTIATIONS
};
