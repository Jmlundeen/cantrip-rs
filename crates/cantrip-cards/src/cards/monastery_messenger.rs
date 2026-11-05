use cantrip_api::*;
use cantrip_mechanics::{targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "Put up to one target ... card from your graveyard on top of your library": with no target chosen, or the card gone, nothing happens (CR 608.2b).
fn put_on_top(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(card) = targets::object(g, item, 0) else { return Ok(()) };
    g.put_into_library(&[card], LibraryPlace::Top);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ENTERS,
        put_on_top,
        "When this creature enters, put up to one target noncreature, nonland card from your graveyard on top of your library.",
    )
    .targets(&[up_to_one(targets::noncreature_nonland_card_in_your_graveyard)])],
    ..oracle::MONASTERY_MESSENGER
};
