use cantrip_api::*;
use cantrip_mechanics::{library, select};
use cantrip_oracle as oracle;

/// "Forest card" (CR 109.2, 205.3i): a card with the Forest subtype as it is in the library, basic or not (Temple
/// Garden is found through its Forest subtype even though it is nonbasic and also a Plains; an Island alone is not).
fn forest_card(q: &dyn Query, choice: &Choice, _: u32) -> Verdict {
    Verdict::each(choice, |card| q.chars(card).has_subtype(q.words(choice.source).subtype(Subtype::Forest)))
}

const FOREST_CARD: Selection = Selection::of(
    select::up_to(1),
    &[select::part(&(forest_card as SelectFn), 0, "")],
).called("Forest");

/// "Search your library for a Forest card, put that card onto the battlefield, then shuffle."
/// The search may be left unfound (CR 701.23b); the shared search wrapper puts the found card onto the battlefield
/// untapped and then shuffles (CR 701.23, 701.24).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    library::find_to_battlefield(g, item.controller, item.object, FOREST_CARD, false)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], effect),
    ..oracle::NATURES_LORE
};
