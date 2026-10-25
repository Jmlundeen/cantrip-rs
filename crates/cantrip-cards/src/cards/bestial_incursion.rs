use cantrip_api::*;
use cantrip_mechanics::tokens;
use cantrip_mechanics::{flashback, tokens::BEAST_4_4_TRAMPLE};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    spell: spell(&[], tokens::create_a_beast_4_4_trample),
    ways: &[flashback::way(cost("{5}{G}"), "Flashback {5}{G}")],
    ..oracle::BESTIAL_INCURSION
};

pub static EXTRA: &[&CardDef] = &[&BEAST_4_4_TRAMPLE];
