use cantrip_api::*;
use cantrip_mechanics::{effects, empower, triggers::{CREATURE_YOU_CONTROL_DIES, THIS_ENTERS}};
use cantrip_oracle as oracle;

/// "When Way of the Necromancer enters, empower Jace 2" (CR 701.71a).
fn empower_jace(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    empower::jace(g, item.controller, item.object, 2)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, empower_jace, "When Way of the Necromancer enters, empower Jace 2."),
        triggered(
            CREATURE_YOU_CONTROL_DIES,
            effects::put_loyalty_counter_on_each_planeswalker_you_control,
            "Whenever a creature you control dies, put a loyalty counter on each planeswalker you control.",
        ),
    ],
    ..oracle::WAY_OF_THE_NECROMANCER
};

pub static EXTRA: &[&CardDef] = &[&empower::JACE];
