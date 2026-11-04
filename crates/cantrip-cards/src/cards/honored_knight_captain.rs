use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{library, select, tokens};
use cantrip_oracle as oracle;

/// "Create a 1/1 white Human Soldier creature token" (CR 111.1, 701.7).
fn create_a_human_soldier(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::HUMAN_SOLDIER, 1)?;
    Ok(())
}

/// "Search your library for an Equipment card, put it onto the battlefield, then shuffle" (CR 701.23): the search may find
/// nothing (CR 701.23b), and the library is shuffled either way. The Equipment enters unattached.
fn fetch_an_equipment(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    library::find_to_battlefield(g, item.controller, item.object, select::EQUIPMENT_CARD, false)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, create_a_human_soldier, "When this creature enters, create a 1/1 white Human Soldier creature token."),
        ability(
            cost("{4}{W}{W}").and(SACRIFICE_THIS),
            fetch_an_equipment,
            "{4}{W}{W}, Sacrifice this creature: Search your library for an Equipment card, put it onto the battlefield, then shuffle.",
        ),
    ],
    ..oracle::HONORED_KNIGHT_CAPTAIN
};

pub static EXTRA: &[&CardDef] = &[&tokens::HUMAN_SOLDIER];
