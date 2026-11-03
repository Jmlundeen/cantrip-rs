use cantrip_api::*;
use cantrip_mechanics::crew::crew;
use cantrip_mechanics::{effects, firebending, targets, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        firebending::trigger::<1>("Firebending 1 (Whenever this creature attacks, add {R}. This mana lasts until end of combat.)"),
        // The target is chosen as the trigger goes on the stack (CR 603.3d); a target gone by resolution gives nothing (CR 608.2b).
        triggered(triggers::THIS_ENTERS, effects::target_creature_gains_indestructible, "When this Vehicle enters, target creature you control gains indestructible until end of turn.")
            .targets(&[targets::CREATURE_YOU_CONTROL]),
        crew(1, "Crew 1 (Tap any number of creatures you control with total power 1 or more: This Vehicle becomes an artifact creature until end of turn.)"),
    ],
    ..oracle::TUNDRA_TANK
};
