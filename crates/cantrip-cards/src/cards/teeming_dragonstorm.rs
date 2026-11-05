use cantrip_api::*;
use cantrip_mechanics::tokens::{self, SOLDIER_2_2};
use cantrip_mechanics::triggers::{DRAGON_YOU_CONTROL_ENTERS, THIS_ENTERS};
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "Create two 2/2 white Soldier creature tokens" (CR 111.1): both made together, as one event.
fn two_soldiers(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &SOLDIER_2_2, 2)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, two_soldiers, "When this enchantment enters, create two 2/2 white Soldier creature tokens."),
        triggered(DRAGON_YOU_CONTROL_ENTERS, effects::return_this_to_hand, "When a Dragon you control enters, return this enchantment to its owner's hand."),
    ],
    ..oracle::TEEMING_DRAGONSTORM
};

pub static EXTRA: &[&CardDef] = &[&SOLDIER_2_2];
