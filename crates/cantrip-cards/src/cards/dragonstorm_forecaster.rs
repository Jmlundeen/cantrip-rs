use cantrip_api::select::part;
use cantrip_api::*;
use cantrip_mechanics::library;
use cantrip_mechanics::select::{up_to, Part, Selection};
use cantrip_oracle as oracle;

fn named_globe_or_dragon(q: &dyn Query, choice: &Choice, _: u32) -> Verdict {
    Verdict::each(choice, |card| {
        let names = library::object_names(q, card);
        names.contains(&"Dragonstorm Globe") || names.contains(&"Boulderborn Dragon")
    })
}

/// "A card named Dragonstorm Globe or Boulderborn Dragon" (CR 201.2a): a card with either name, as it is in the library.
const NAMED_GLOBE_OR_DRAGON: Part = part(&(named_globe_or_dragon as SelectFn), 0, "");

/// One such card, or none: a search for a quality may find nothing (CR 701.23b).
const GLOBE_OR_DRAGON_CARD: Selection = Selection::of(up_to(1), &[NAMED_GLOBE_OR_DRAGON]).called("card named Dragonstorm Globe or Boulderborn Dragon");

/// "Search your library for a card named Dragonstorm Globe or Boulderborn Dragon, reveal it, put it into your hand, then
/// shuffle." (CR 701.23): the controller's own library, shuffled whether or not a card is found.
fn search(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    library::find_to_hand(g, item.controller, item.object, GLOBE_OR_DRAGON_CARD, true)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        cost("{2}, {T}"),
        search,
        "{2}, {T}: Search your library for a card named Dragonstorm Globe or Boulderborn Dragon, reveal it, put it into your hand, then shuffle.",
    )],
    ..oracle::DRAGONSTORM_FORECASTER
};
