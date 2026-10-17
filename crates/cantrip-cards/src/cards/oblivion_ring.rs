use cantrip_api::*;
use cantrip_mechanics::{effects, targets, triggers};
use cantrip_oracle as oracle;

// Two triggered abilities, linked (CR 607.2a): the second returns only what the first exiled, and the card is exiled
// for good, not "until" this leaves (CR 610.3), so if this leaves before its enters ability resolves, the leaves ability
// returns nothing and the enters ability then exiles its target forever (the Oblivion Ring ruling).
pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::THIS_ENTERS, effects::exile_first_target_with_this, "When this enchantment enters, exile another target nonland permanent.")
            .targets(&[target(targets::other_nonland_permanent_filter)]),
        triggered(triggers::THIS_LEAVES, effects::return_exiled_to_battlefield, "When this enchantment leaves the battlefield, return the exiled card to the battlefield under its owner's control.")
            .look_back(),
    ],
    ..oracle::OBLIVION_RING
};
