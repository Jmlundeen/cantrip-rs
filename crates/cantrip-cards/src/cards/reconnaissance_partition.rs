use cantrip_api::*;
use cantrip_mechanics::{effects, library, select};
use cantrip_oracle as oracle;

const TEXT: &str = "{T}, Remove a counter from this creature: Choose one —\n• Search your library for a basic land card, put it onto the battlefield tapped, then shuffle.\n• Draw a card.";
const SEARCH: &str = "Search your library for a basic land card, put it onto the battlefield tapped, then shuffle.";

/// "This creature enters with two charge counters on it" (CR 614.1c, 122.6): the counters it enters with, put as it enters.
fn enters_with_two_charge_counters(_: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    entering.counters[CounterKind::Charge as usize] = 2;
    Ok(())
}

/// "Search your library for a basic land card, put it onto the battlefield tapped, then shuffle" (CR 701.23): the card may
/// be left unfound and the library is shuffled either way.
fn search_a_basic_land(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    library::find_to_battlefield(g, item.controller, item.object, select::BASIC_LAND_CARD, true)?;
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(search_a_basic_land, SEARCH),
    mode(|g, item| effects::draw(g, item, 1), "Draw a card."),
]);

pub static CARD: CardDef = CardDef {
    enters: Some(enters_with_two_charge_counters),
    abilities: &[modal_ability(TAP.and(REMOVE_A_COUNTER), &MODAL, TEXT)],
    ..oracle::RECONNAISSANCE_PARTITION
};
