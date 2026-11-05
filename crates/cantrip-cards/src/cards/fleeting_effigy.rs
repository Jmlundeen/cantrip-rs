use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

/// "At the beginning of your end step, return this creature to its owner's hand" (CR 603.2, 400.3): a trigger of the
/// permanent, so it exists only while the Effigy is on the battlefield at the beginning of its controller's end step. If the
/// Effigy has left by the time the trigger resolves, it is a new object and is left alone (CR 400.7).
pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::YOUR_END_STEP, effects::return_this_to_hand, "At the beginning of your end step, return this creature to its owner's hand."),
        ability(cost("{2}{R}"), effects::this_gets::<2, 0>, "{2}{R}: This creature gets +2/+0 until end of turn."),
    ],
    ..oracle::FLEETING_EFFIGY
};
