use cantrip_api::*;
use cantrip_mechanics::{effects, lands, library, select, triggers};
use cantrip_oracle as oracle;

fn basic_land_or_planet(q: &dyn Query, choice: &Choice, _: u32) -> Verdict {
    let planet = q.words(choice.source).subtype(Subtype::Planet);
    Verdict::each(choice, |card| {
        lands::is_basic_land_card(q, card) || q.chars(card).has_subtype(planet)
    })
}

const BASIC_LAND_OR_PLANET: Part =
    select::part(&(basic_land_or_planet as SelectFn), 0, "or Planet");
const SEARCH: Selection =
    Selection::of(select::up_to(1), &[BASIC_LAND_OR_PLANET]).called("basic land card");

/// "When Genesis Device enters, search your library for a basic land card or Planet card, put it onto the battlefield
/// tapped, then shuffle" (CR 701.23, 205.3i, 205.4c): the searcher finds one matching card or none; the opponent's library is not
/// part of the search.
fn enters_search(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    library::find_to_battlefield(g, item.controller, item.object, SEARCH, true)?;
    Ok(())
}

/// "Sacrifice any number of lands you control, then search your library for up to that many land cards, put them onto the
/// battlefield tapped, then shuffle" (CR 608.2c-d, 701.21a, 701.23b). The shared effect derives the search limit from the
/// lands actually sacrificed.
fn sacrifice_and_search(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::sacrifice_any_number_of_lands_for_lands(g, item)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::THIS_ENTERS,
            enters_search,
            "When Genesis Device enters, search your library for a basic land card or Planet card, put it onto the battlefield tapped, then shuffle.",
        ),
        // "Activate only as a sorcery" (CR 602.5d): the activated ability uses sorcery timing.
        ability(
            cost("{3}{G}{G}, {T}").and(SACRIFICE_THIS),
            sacrifice_and_search,
            "{3}{G}{G}, {T}, Sacrifice Genesis Device: Sacrifice any number of lands you control, then search your library for up to that many land cards, put them onto the battlefield tapped, then shuffle. Activate only as a sorcery.",
        )
        .sorcery_speed(),
    ],
    ..oracle::GENESIS_DEVICE
};
