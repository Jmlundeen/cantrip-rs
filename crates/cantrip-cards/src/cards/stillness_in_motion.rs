use cantrip_api::*;
use cantrip_mechanics::library::ask_library_order;
use cantrip_mechanics::{choose, select, triggers};
use cantrip_oracle as oracle;

/// "Mill three cards. Then if your library has no cards in it, exile this enchantment and put five cards from your graveyard on
/// top of your library in any order" (CR 701.17a, 608.2c, 401.4). The enchantment is exiled only while it is still the same
/// permanent (CR 400.7: `exile` skips a new object), and the cards go back whether or not it was; the graveyard is read after the mill and after the exile
/// (so the mill's own cards, and nothing else the exile moved, are among the choices), and fewer than five are all put (CR 609.3).
fn mill_and_recycle(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    g.mill(you, 3);
    if !g.state().zone(Zone::library(you)).is_empty() {
        return Ok(());
    }
    g.exile(item.object);
    let graveyard = g.state().zone(Zone::graveyard(you)).to_vec();
    let chosen = choose::select(g, you, item.object, Purpose::PutOnTop, graveyard, select::cards(select::as_many_as_can(5)))?;
    let order = ask_library_order(g, you, item.object, &chosen, DestinationKind::LibraryTop)?;
    g.put_into_library(&order, LibraryPlace::Top);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::YOUR_UPKEEP,
        mill_and_recycle,
        "At the beginning of your upkeep, mill three cards. Then if your library has no cards in it, exile this enchantment and put five cards from your graveyard on top of your library in any order.",
    )],
    ..oracle::STILLNESS_IN_MOTION
};
