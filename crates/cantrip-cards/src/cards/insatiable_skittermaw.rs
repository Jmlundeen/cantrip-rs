use cantrip_api::*;
use cantrip_mechanics::effects::plus_one_counter_on_this;
use cantrip_mechanics::history;
use cantrip_mechanics::triggers::YOUR_END_STEP;
use cantrip_oracle as oracle;

/// "Void — At the beginning of your end step, if a nonland permanent left the battlefield this turn or a spell was warped this
/// turn, put a +1/+1 counter on this creature" (CR 207.2c, 603.4): an intervening "if", checked as the end step begins and
/// again as the ability resolves. Menace is in the content JSON.
pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        YOUR_END_STEP,
        plus_one_counter_on_this,
        "Void \u{2014} At the beginning of your end step, if a nonland permanent left the battlefield this turn or a spell was warped this turn, put a +1/+1 counter on this creature.",
    )
    .condition(|q, _| history::void(q))],
    ..oracle::INSATIABLE_SKITTERMAW
};
