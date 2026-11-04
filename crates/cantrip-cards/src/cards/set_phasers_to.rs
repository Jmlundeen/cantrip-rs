use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Choose one -- Stun ... Kill ..." (CR 700.2): one mode, chosen as the spell is cast, with the target of its own slot
/// (CR 700.2c). Stun taps the creature and puts a stun counter on it (CR 122.1d); Kill deals 3 damage to an attacker, and
/// nothing to a creature that has stopped attacking or left the battlefield (CR 608.2b).
static MODAL: ModalDef = modes(&[
    mode(effects::tap_and_stun::<1>, "Stun \u{2014} Tap target creature and put a stun counter on it.")
        .targets(&[targets::CREATURE]),
    mode(
        effects::damage_first_target::<3>,
        "Kill \u{2014} This spell deals 3 damage to target attacking creature.",
    )
    .targets(&[targets::ATTACKING_CREATURE]),
]);

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef::MODAL),
    modal: Some(&MODAL),
    ..oracle::SET_PHASERS_TO
};
