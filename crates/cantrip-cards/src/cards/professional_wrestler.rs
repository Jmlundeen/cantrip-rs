use cantrip_api::*;
use cantrip_mechanics::{tokens, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

pub static EXTRA: &[&CardDef] = &[&tokens::TREASURE];

pub static CARD: CardDef = CardDef {
    // "This creature can't be blocked by more than one creature" (CR 509.1b): the most blockers it may have is one.
    keywords: Keywords::of(&[Keyword::CantBeBlockedByMoreThanOne]),
    abilities: &[triggered(THIS_ENTERS, tokens::create_a_treasure, "When this creature enters, create a Treasure token.")],
    ..oracle::PROFESSIONAL_WRESTLER
};
