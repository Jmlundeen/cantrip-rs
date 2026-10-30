use cantrip_api::*;
use cantrip_mechanics::{creatures_have_total_power_at_least, effects, targets, triggers};
use cantrip_oracle as oracle;

/// "Whenever you attack while creatures you control have total power 8 or greater": the intervening condition is checked
/// only as the trigger event happens and again as it resolves (CR 508.3d, 603.4), counting the creatures the attacking
/// player controls then.
fn eight_power(q: &dyn Query, asker: &Asker) -> bool {
    creatures_have_total_power_at_least(q, asker.controller, 8)
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::YOU_ATTACK,
        effects::first_target_cant_block_this_turn,
        "Whenever you attack while creatures you control have total power 8 or greater, target creature can't block this turn.",
    )
    .only_if(eight_power)
    .targets(&[targets::CREATURE])],
    ..oracle::LAAN_NOONIEN_SINGH_SECURITY
};
