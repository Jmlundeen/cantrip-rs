use cantrip_api::*;
use cantrip_mechanics::fight::targets_fight;
use cantrip_mechanics::library::{search_to_hand, Find};
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Search your library for a basic land card, reveal it, put it into your hand, then shuffle" (CR 701.23): the card may
/// be left unfound (CR 701.23b), and the library is shuffled either way.
fn search(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    search_to_hand(g, item.controller, item.object, Find::ONE.revealed(), |q, c| {
        let chars = q.chars(c);
        chars.is(CardType::Land) && chars.supertypes.basic
    })?;
    Ok(())
}

/// "Target creature you control fights target creature you don't control" (CR 701.14).
fn fight(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    targets_fight(g, item, 0, 1);
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(search, "Search your library for a basic land card, reveal it, put it into your hand, then shuffle."),
    mode(fight, "Target creature you control fights target creature you don't control.")
        .targets(&[targets::CREATURE_YOU_CONTROL, targets::CREATURE_YOU_DONT_CONTROL]),
]);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::BUSHWHACK };
