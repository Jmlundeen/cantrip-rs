use cantrip_api::*;
use cantrip_mechanics::{effects, harmonize};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    spell: spell(&[], |g, item| effects::draw(g, item, 1)),
    ways: &[harmonize::way(mana("{5}{U}"), "Harmonize {5}{U}")],
    ..oracle::UNENDING_WHISPER
};
