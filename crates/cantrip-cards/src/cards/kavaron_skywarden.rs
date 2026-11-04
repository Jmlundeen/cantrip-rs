use cantrip_api::*;
use cantrip_mechanics::triggers::YOUR_END_STEP;
use cantrip_mechanics::{effects, history};
use cantrip_oracle as oracle;

/// Reach is a core keyword (CR 702.17), read from the card's JSON. Void is an ability word (CR 207.2c): the intervening "if"
/// (CR 603.4) is `history::void`, checked as the end step begins and again as the ability resolves.
pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        YOUR_END_STEP,
        effects::plus_one_counter_on_this,
        "Void \u{2014} At the beginning of your end step, if a nonland permanent left the battlefield this turn or a spell was warped this turn, put a +1/+1 counter on this creature.",
    )
    .condition(|q, _| history::void(q))],
    ..oracle::KAVARON_SKYWARDEN
};
