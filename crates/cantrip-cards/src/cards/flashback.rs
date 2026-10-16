use cantrip_api::*;
use cantrip_mechanics::{grant_flashback, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::INSTANT_OR_SORCERY_IN_YOUR_GRAVEYARD], grant_flashback),
    ..oracle::FLASHBACK
};
