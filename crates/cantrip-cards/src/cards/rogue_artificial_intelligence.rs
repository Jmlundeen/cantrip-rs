use cantrip_api::*;
use cantrip_mechanics::{effects, triggers::THIS_ENTERS, typecycling};
use cantrip_oracle as oracle;

/// "When this creature enters, choose one —" (CR 603.6a, 603.3c).
static MODAL: ModalDef = modes(&[
    mode(|g, item| effects::gain_life(g, item, 3), "You gain 3 life."),
    mode(|g, item| effects::surveil(g, item, 2), "Surveil 2."),
]);

pub static CARD: CardDef = CardDef {
    abilities: &[
        modal_triggered(
            THIS_ENTERS,
            &MODAL,
            "When this creature enters, choose one — • You gain 3 life. • Surveil 2.",
        ),
        typecycling::basic_landcycling(mana("{2}"), "Basic landcycling {2}"),
    ],
    ..oracle::ROGUE_ARTIFICIAL_INTELLIGENCE
};
