use cantrip_api::*;
use cantrip_mechanics::{library, select};
use cantrip_oracle as oracle;

/// "A Plains, Island, Swamp, or Mountain card" (CR 205.3i, 305.6): a card with any one of the four land subtypes as it
/// is in the library, basic or not (Taiga is found through its Mountain subtype even though it is nonbasic and also a
/// Forest; a Forest alone is not). The subtypes are read through this card's words (CR 612.2).
fn listed_land(q: &dyn Query, choice: &Choice, _: u32) -> Verdict {
    let words = q.words(choice.source);
    let plains = words.subtype(Subtype::Plains);
    let island = words.subtype(Subtype::Island);
    let swamp = words.subtype(Subtype::Swamp);
    let mountain = words.subtype(Subtype::Mountain);
    Verdict::each(choice, |card| {
        let chars = q.chars(card);
        chars.has_subtype(plains)
            || chars.has_subtype(island)
            || chars.has_subtype(swamp)
            || chars.has_subtype(mountain)
    })
}

const LISTED_LAND_CARD: Selection = Selection::of(
    select::up_to(1),
    &[select::part(&(listed_land as SelectFn), 0, "")],
).called("Plains, Island, Swamp, or Mountain");

/// "Search your library for a Plains, Island, Swamp, or Mountain card, put it onto the battlefield tapped, then shuffle."
/// The search may be left unfound (CR 701.23b); the shared search wrapper puts the found card onto the battlefield tapped
/// and then shuffles (CR 701.23, 701.24).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    library::find_to_battlefield(g, item.controller, item.object, LISTED_LAND_CARD, true)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], effect),
    ..oracle::FARSEEK
};
