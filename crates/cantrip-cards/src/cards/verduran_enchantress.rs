use cantrip_api::*;
use cantrip_mechanics::{effects, triggers::ENCHANTMENT_SPELL_YOU_CAST};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        ENCHANTMENT_SPELL_YOU_CAST,
        effects::may_draw_a_card,
        "Whenever you cast an enchantment spell, you may draw a card.",
    )],
    ..oracle::VERDURAN_ENCHANTRESS
};
