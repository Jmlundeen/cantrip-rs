use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

/// "You may pay {1}{W}. If you do, return this card from your graveyard to your hand."
fn pay_to_return_to_hand(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::may_pay_to_return_this_to_hand(g, item, cost("{1}{W}"))
}

pub static CARD: CardDef = CardDef {
    // A leaves-the-battlefield ability (CR 603.10a): it triggers only from a card that was in the graveyard before the death,
    // so not for this card's own death, nor for creatures dying with it.
    abilities: &[triggered(
        triggers::CREATURE_YOU_CONTROL_DIES,
        pay_to_return_to_hand,
        "Whenever a creature you control dies while this card is in your graveyard, you may pay {1}{W}. If you do, return this card from your graveyard to your hand.",
    )
    // It returns this card from the graveyard, so it functions only there (CR 113.6m).
    .functions_in(IN_A_GRAVEYARD)],
    ..oracle::FURIOUS_FOREBEAR
};
