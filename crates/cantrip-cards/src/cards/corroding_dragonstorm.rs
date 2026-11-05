use cantrip_api::*;
use cantrip_mechanics::triggers::{DRAGON_YOU_CONTROL_ENTERS, THIS_ENTERS};
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "Each opponent loses 2 life and you gain 2 life. Surveil 2" (CR 119.3, 701.25), in that order.
fn drain_then_surveil(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::each_opponent_loses_life_you_gain_life::<2>(g, item)?;
    effects::surveil(g, item, 2)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, drain_then_surveil, "When this enchantment enters, each opponent loses 2 life and you gain 2 life. Surveil 2."),
        triggered(DRAGON_YOU_CONTROL_ENTERS, effects::return_this_to_hand, "When a Dragon you control enters, return this enchantment to its owner's hand."),
    ],
    ..oracle::CORRODING_DRAGONSTORM
};
