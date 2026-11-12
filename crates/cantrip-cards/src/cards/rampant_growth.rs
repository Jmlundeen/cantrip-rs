use cantrip_api::*;
use cantrip_mechanics::lands;
use cantrip_oracle as oracle;

/// "Search your library for a basic land card, put it onto the battlefield tapped, then shuffle" (CR 701.23).
pub static CARD: CardDef = CardDef {
    spell: spell(&[], |g, item| lands::search_basic_land_to_battlefield(g, item.controller, item.object, true)),
    ..oracle::RAMPANT_GROWTH
};
