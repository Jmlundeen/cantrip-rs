use cantrip_api::*;
use cantrip_mechanics::{copy, triggers};
use cantrip_oracle as oracle;

/// "At the beginning of your upkeep, create a token that's a copy of this creature" (CR 707.2, 111.1): the token copies
/// what this creature is as the ability resolves, including the +1/+1 counters on it.
fn another_tribble(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    copy::create_token_copy_of_it(g, item, |_, values| values)
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::YOUR_UPKEEP,
        another_tribble,
        "At the beginning of your upkeep, create a token that's a copy of this creature.",
    )],
    keywords: Keywords::of(&[Keyword::CantBlock]),
    ..oracle::TROUBLING_TRIBBLES
};
