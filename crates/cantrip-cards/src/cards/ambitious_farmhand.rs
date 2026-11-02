use cantrip_api::*;
use cantrip_mechanics::library::{ask_may_search, find_to_hand};
use cantrip_mechanics::{coven, effects, select, triggers};
use cantrip_oracle as oracle;

pub static BACK: CardDef = CardDef {
    ..oracle::SEASONED_CATHAR
};

const ASK: Question = Question::new("SearchLibrary", "search your library for a basic Plains card?", "Reveal it and put it into your hand, then shuffle", "Search", "Don't search");

/// "When this creature enters, you may search your library for a basic Plains card, reveal it, put it into your hand, then
/// shuffle." (CR 701.23): asked first, and a declined search shuffles nothing.
fn search_library_basic_plains_card(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if ask_may_search(g, item.controller, item.object, &ASK)? {
        find_to_hand(g, item.controller, item.object, select::BASIC_PLAINS_CARD, true)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::THIS_ENTERS,
            search_library_basic_plains_card,
            "When this creature enters, you may search your library for a basic Plains card, reveal it, put it into your hand, then shuffle.",
        ),
        // "Coven — {1}{W}{W}: Transform this creature. Activate only if you control three or more creatures with different powers."
        ability(
                cost("{1}{W}{W}"),
                effects::transform_self,
                "Coven — {1}{W}{W}: Transform this creature. Activate only if you control three or more creatures with different powers.",
            )
            .condition(coven::you_control_asked),
    ],
    back: Some(&BACK),
    ..oracle::AMBITIOUS_FARMHAND
};
