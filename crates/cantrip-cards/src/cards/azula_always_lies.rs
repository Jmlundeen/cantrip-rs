use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Choose one or both" (CR 700.2): each mode has its own target (CR 700.2c), which may be the same creature, and the modes are carried out in
/// the order printed (CR 700.2a).
static MODAL: ModalDef = modes(&[
    mode(effects::target_creature_gets::<-1, -1>, "Target creature gets -1/-1 until end of turn.").targets(&[targets::CREATURE]),
    mode(effects::plus_one_counter_on_target, "Put a +1/+1 counter on target creature.").targets(&[targets::CREATURE]),
])
.choose(1, 2);

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef::MODAL),
    modal: Some(&MODAL),
    ..oracle::AZULA_ALWAYS_LIES
};
