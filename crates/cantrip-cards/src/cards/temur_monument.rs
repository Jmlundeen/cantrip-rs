use cantrip_api::*;
use cantrip_mechanics::lands::sub;
use cantrip_mechanics::select::{basic_land_of_types, up_to, Selection};
use cantrip_mechanics::{library, tokens, triggers};
use cantrip_oracle as oracle;

/// "A basic Forest, Island, or Mountain card", which may be left unfound (CR 701.23b).
const BASIC_CARD: Selection =
    Selection::of(up_to(1), &[basic_land_of_types::<{ sub(Subtype::Forest) }, { sub(Subtype::Island) }, { sub(Subtype::Mountain) }>()])
        .called("basic Forest, Island, or Mountain");

/// "When this artifact enters, search your library for a basic Forest, Island, or Mountain card, reveal it, put it into your hand,
/// then shuffle" (CR 701.23).
fn search(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    library::find_to_hand(g, item.controller, item.object, BASIC_CARD, true)?;
    Ok(())
}

/// "Create a 5/5 green Elephant creature token" (CR 111.1).
fn create_an_elephant(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::ELEPHANT_5_5, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::THIS_ENTERS,
            search,
            "When this artifact enters, search your library for a basic Forest, Island, or Mountain card, reveal it, put it into your hand, then shuffle.",
        ),
        ability(
            cost("{3}{G}{U}{R}, {T}").and(SACRIFICE_THIS),
            create_an_elephant,
            "{3}{G}{U}{R}, {T}, Sacrifice this artifact: Create a 5/5 green Elephant creature token. Activate only as a sorcery.",
        )
        .sorcery_speed(),
    ],
    ..oracle::TEMUR_MONUMENT
};

pub static EXTRA: &[&CardDef] = &[&tokens::ELEPHANT_5_5];
