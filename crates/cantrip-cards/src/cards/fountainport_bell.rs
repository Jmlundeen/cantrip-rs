use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::lands::may_search_basic_land_to_top;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// "When this artifact enters, you may search your library for a basic land card, reveal it, then shuffle and put that card on
/// top" is `lands::may_search_basic_land_to_top`; "{1}, Sacrifice this artifact: Draw a card" costs {1} (the sacrifice is a cost,
/// so the draw happens even with the Bell gone).
pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, |g, item| may_search_basic_land_to_top(g, item.controller, item.object), "When this artifact enters, you may search your library for a basic land card, reveal it, then shuffle and put that card on top."),
        ability(cost("{1}").and(SACRIFICE_THIS), |g, item| effects::draw(g, item, 1), "{1}, Sacrifice this artifact: Draw a card."),
    ],
    ..oracle::FOUNTAINPORT_BELL
};
