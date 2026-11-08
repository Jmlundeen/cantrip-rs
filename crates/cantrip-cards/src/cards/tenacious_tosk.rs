use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

/// "This creature can't be blocked by more than one creature" is the core keyword CantBeBlockedByMoreThanOne (CR 509.1b),
/// which crates/cantrip-oracle/build.rs reads from the printed sentence, so the card writes no code for it. The
/// combat-damage trigger is the shared DEALS_COMBAT_DAMAGE_TO_PLAYER_SUBJECT and the shared counter effect.
pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::DEALS_COMBAT_DAMAGE_TO_PLAYER_SUBJECT,
        effects::plus_one_counter_on_this,
        "Whenever this creature deals combat damage to a player, put a +1/+1 counter on it.",
    )],
    ..oracle::TENACIOUS_TOSK
};
