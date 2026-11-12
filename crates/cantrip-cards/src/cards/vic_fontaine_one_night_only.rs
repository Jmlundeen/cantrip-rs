use cantrip_api::*;
use cantrip_mechanics::{effects, seek_out, targets};
use cantrip_oracle as oracle;

const SEEK_OUT: &str = "Seek out — Whenever you draw your second card each turn, one or two target creatures each get +2/+0 and gain lifelink until end of turn.";

/// "Seek out — Whenever you draw your second card each turn, one or two target creatures each get +2/+0 and gain lifelink
/// until end of turn" (CR 603.2, 121.1, 611.2a): once a turn, on the second card drawn, whoever's turn it is; one or two
/// creatures are chosen as the ability goes on the stack (CR 601.2c), an opponent's among them.
pub static CARD: CardDef = CardDef {
    abilities: &[seek_out::seek_out(
        |g, item| effects::targets_get_and_gain(g, item, 2, 0, Keyword::Lifelink),
        &[targets::CREATURE.between(1, 2)],
        SEEK_OUT,
    )],
    ..oracle::VIC_FONTAINE_ONE_NIGHT_ONLY
};
