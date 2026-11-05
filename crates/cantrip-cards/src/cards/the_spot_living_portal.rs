use cantrip_api::*;
use cantrip_mechanics::targets::{self, is_nonland_permanent_card};
use cantrip_mechanics::triggers::{self, THIS_ENTERS};
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "Up to one target nonland permanent card from a graveyard" (CR 115.1, 110.4a): a nonland permanent card in any player's graveyard,
/// not a spell or token.
fn nonland_permanent_card_in_a_graveyard(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::CARD_IN_A_GRAVEYARD.filter)(q, asker, t) && matches!(t, Target::Object(id) if is_nonland_permanent_card(&q.chars(id)))
}

/// "Exile up to one target nonland permanent and up to one target nonland permanent card from a graveyard" (CR 608.2b, 607.2a): the
/// targets still legal as it resolves are exiled together and remembered as exiled with this ability's object, for the second ability to
/// return. They are not tied to The Spot staying on the battlefield.
fn exile_the_targets(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let legal = targets::legal_objects(g, item);
    if !legal.is_empty() {
        g.exile_with(&legal, g.linked(item), false);
    }
    Ok(())
}

/// "Put him on the bottom of his owner's library. If you do, return the exiled cards to their owners' hands" (CR 401.4, 608.2c): The Spot
/// only goes if he is still in the graveyard as the card that died (CR 400.7), and a replacement may send him elsewhere; the exiled cards
/// return only if he arrived in the library.
fn bottom_of_library_then_return_exiled(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Object(card) = item.subject else { return Ok(()) };
    if g.put_into_library(&[card], LibraryPlace::Bottom).is_empty() { return Ok(()) }
    effects::return_exiled_to_hand(g, item)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, exile_the_targets, "When The Spot enters, exile up to one target nonland permanent and up to one target nonland permanent card from a graveyard.")
            .targets(&[targets::NONLAND_PERMANENT.optional(), target(nonland_permanent_card_in_a_graveyard).optional()]),
        triggered(when(EventMask::ZONE_CHANGE, triggers::this_dies_about_the_card), bottom_of_library_then_return_exiled, "When The Spot dies, put him on the bottom of his owner's library. If you do, return the exiled cards to their owners' hands."),
    ],
    ..oracle::THE_SPOT_LIVING_PORTAL
};
