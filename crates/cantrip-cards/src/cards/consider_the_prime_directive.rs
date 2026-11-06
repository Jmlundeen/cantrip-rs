use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

static MODAL: ModalDef = modes(&[
    mode(effects::counter_first_target_unless_pays::<3>, "Interfere — Counter target spell unless its controller pays {3}.")
        .targets(&[targets::SPELL]),
    mode(effects::tap_and_stun_then_draw, "Observe — Tap up to one target creature and put a stun counter on it. Draw a card.")
        .targets(&[targets::CREATURE.optional()]),
]);

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef::MODAL),
    modal: Some(&MODAL),
    ..oracle::CONSIDER_THE_PRIME_DIRECTIVE
};
