use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Your life total can't change. (You can't gain or lose life. You can't pay any amount of life except 0.)" (CR 119.7,
/// 119.8): its controller gains no life and loses none - from damage, which is still dealt (the rulings), from an effect,
/// from a life total set to a number, or from a payment, which they can't make for more than 0 life (CR 119.4b). An effect
/// that would replace a gain or a loss of theirs does nothing, and one that would replace an event with a gain or a loss
/// of theirs replaces it with nothing (the rulings).
fn you(_: &dyn Query, asker: &Asker, player: PlayerId) -> bool {
    player == asker.controller
}

pub static CARD: CardDef = CardDef {
    cant_gain_life: &[you],
    cant_lose_life: &[you],
    ..oracle::PLATINUM_EMPERION
};
