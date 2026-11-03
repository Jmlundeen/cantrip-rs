use cantrip_api::*;
use cantrip_mechanics::{effects, targets, triggers, typecycling};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        // The target is chosen as the trigger goes on the stack (CR 603.3d); the Lizard deals the damage (CR 120.3).
        triggered(triggers::THIS_ENTERS, effects::damage_first_target::<1>, "When this creature enters, it deals 1 damage to any target.")
            .targets(&[targets::ANY]),
        typecycling::ability(Subtype::Mountain, mana("{2}"), "Mountaincycling {2} ({2}, Discard this card: Search your library for a Mountain card, reveal it, put it into your hand, then shuffle.)"),
    ],
    ..oracle::MONGOOSE_LIZARD
};
