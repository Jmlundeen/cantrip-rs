use cantrip_api::*;
use cantrip_mechanics::{effects, targets, triggers};
use cantrip_oracle as oracle;

static MODES: ModalDef = modes(&[
    mode(effects::damage_first_target::<1>, "This Aura deals 1 damage to target player or planeswalker.")
        .targets(&[targets::PLAYER_OR_PLANESWALKER]),
    mode(effects::target_player_gains::<1>, "Target player gains 1 life.").targets(&[targets::PLAYER]),
]).choose(1, 1);

pub static CARD: CardDef = CardDef {
    abilities: &[modal_triggered(
        when(EventMask::TAPPED, triggers::attached_permanent_becomes_tapped),
        &MODES,
        "Whenever enchanted artifact becomes tapped, choose one — This Aura deals 1 damage to target player or planeswalker; or Target player gains 1 life.",
    )],
    enchant: Some(targets::ARTIFACT_AN_OPPONENT_CONTROLS),
    ..oracle::RELIC_BIND
};
