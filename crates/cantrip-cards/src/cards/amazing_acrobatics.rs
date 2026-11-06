use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Choose one or both" (CR 700.2): the modes are carried out in the order printed (CR 608.2c).
static MODAL: ModalDef = modes(&[
    mode(effects::counter_first_target, "Counter target spell.").targets(&[targets::SPELL]),
    mode(effects::tap_targets, "Tap one or two target creatures.").targets(&[targets::CREATURE.between(1, 2)]),
])
.choose(1, 2);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::AMAZING_ACROBATICS };
