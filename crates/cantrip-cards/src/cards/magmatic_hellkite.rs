use cantrip_api::*;
use cantrip_mechanics::{search_library, targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "When this creature enters, destroy target nonbasic land an opponent controls. Its controller searches their library
/// for a basic land card, puts it onto the battlefield tapped with a stun counter on it, then shuffles."
///
/// The land's controller (the one it last had, CR 608.2h) searches whether or not the land was destroyed (indestructible,
/// regenerated: the card's ruling), and the search may find nothing (CR 701.23b); the library is shuffled either way
/// (CR 701.23). The land enters with the stun counter (CR 122.1d), so it is a replacement of how it enters, not a counter
/// added afterwards. With the target illegal the ability does not resolve at all (CR 608.2b), so nobody searches.
fn destroy_and_fetch(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(land) = targets::object(g, item, 0) else { return Ok(()) };
    let player = g.controller(land);
    g.destroy(land);
    let found = search_library(g, player, player, item.object, SearchDestination::BattlefieldTapped, 1, true, |q, c| {
        let chars = q.chars(c);
        chars.is(CardType::Land) && chars.supertypes.basic
    })?;
    for card in found {
        g.put_onto_battlefield_with_counters(card, player, true, &[(CounterKind::Stun, 1)])?;
    }
    g.shuffle(player);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ENTERS,
        destroy_and_fetch,
        "When Magmatic Hellkite enters, destroy target nonbasic land an opponent controls. Its controller searches their library for a basic land card, puts it onto the battlefield tapped with a stun counter on it, then shuffles.",
    )
    .targets(&[targets::NONBASIC_LAND_AN_OPPONENT_CONTROLS])],
    ..oracle::MAGMATIC_HELLKITE
};
