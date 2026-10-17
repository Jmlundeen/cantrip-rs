use cantrip_api::*;
use cantrip_mechanics::{effects, offspring};
use cantrip_mechanics::triggers::YOU_GAIN_LIFE;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    keywords: Keywords::of(&[Keyword::Flying, Keyword::CantBlock]),
    kicker: Some(offspring::kicker(mana("{2}{B}"))),
    abilities: &[
        offspring::trigger("Offspring {2}{B} (You may pay an additional {2}{B} as you cast this spell. If you do, when this creature enters, create a 1/1 token copy of it.)"),
        triggered(YOU_GAIN_LIFE, effects::each_opponent_loses_1_life, "Whenever you gain life, each opponent loses 1 life."),
    ],
    ..oracle::STARSCAPE_CLERIC
};
