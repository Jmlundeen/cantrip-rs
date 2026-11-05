use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{effects, lands, library, replacements, select};
use cantrip_oracle as oracle;

/// "When this enchantment enters, search your library for a basic land card or Planet card, reveal it, put it into your
/// hand, then shuffle." (CR 701.23b, 701.20, 701.24): the found card is revealed, and the search itself shuffles.
fn fetch(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    library::find_to_hand(g, item.controller, item.object, select::BASIC_LAND_OR_PLANET_CARD, true)?;
    Ok(())
}

/// "This ability costs {X} less to activate, where X is the number of differently named lands you control" (CR 601.2f by
/// way of 602.2b): counted as the ability is offered.
fn cost_reduction(q: &dyn Query, asker: &Asker, _: Costed, _: &[Target]) -> CostDelta {
    CostDelta::less(lands::differently_named_lands(q, asker.controller) as u32)
}

/// "Strange new worlds — {7}{U}{U}, Sacrifice this enchantment: Draw two cards." "Lands you control enter untapped." is
/// the shared replacement (CR 614.1d).
pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            THIS_ENTERS,
            fetch,
            "When this enchantment enters, search your library for a basic land card or Planet card, reveal it, put it into your hand, then shuffle.",
        ),
        ability(
            cost("{7}{U}{U}").and(SACRIFICE_THIS),
            |g, item| effects::draw(g, item, 2),
            "Strange new worlds — {7}{U}{U}, Sacrifice this enchantment: Draw two cards. This ability costs {X} less to activate, where X is the number of differently named lands you control.",
        )
        .cost_change(cost_reduction),
    ],
    replacements: &[replacements::LANDS_YOU_CONTROL_ENTER_UNTAPPED],
    ..oracle::WARP_SPEED
};
