use cantrip_api::*;
use cantrip_mechanics::triggers::YOUR_END_STEP;
use cantrip_mechanics::{effects, history, targets};
use cantrip_oracle as oracle;

/// "If a creature died this turn" (CR 603.4, 700.4): any creature's, a token's included, asked of the history as the step begins and
/// again as the ability resolves.
fn a_creature_died_this_turn(q: &dyn Query, _: &Asker) -> bool {
    history::creatures_died_this_turn(q, None) > 0
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        YOUR_END_STEP,
        effects::target_connives,
        "At the beginning of your end step, if a creature died this turn, target creature you control connives.",
    )
    .targets(&[targets::CREATURE_YOU_CONTROL])
    .condition(a_creature_died_this_turn)],
    ..oracle::SCORPION_SEETHING_STRIKER
};
