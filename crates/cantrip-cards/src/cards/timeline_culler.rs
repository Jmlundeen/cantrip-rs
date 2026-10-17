use cantrip_api::*;
use cantrip_mechanics::warp;
use cantrip_oracle as oracle;

/// "Warp - {B}, Pay 2 life" (CR 702.185a, 118.8): the warp cost, from the hand and, by the card's own sentence "You may cast this card
/// from your graveyard using its warp ability", from its owner's graveyard. Casting it for either way follows the rules for paying
/// alternative costs (CR 601.2b, 601.2f-h): the life is paid with the mana, and the spell counts as warped (CR 702.185c).
const WARP_COST: Cost = cost("{B}").and(life(2));

pub static CARD: CardDef = CardDef {
    ways: &[
        warp::way_with_cost(WARP_COST, "Warp\u{2014}{B}, Pay 2 life"),
        warp::way_from_graveyard(WARP_COST, "Warp\u{2014}{B}, Pay 2 life (from your graveyard)"),
    ],
    end_step_riders: warp::RIDERS_FROM_HAND_AND_GRAVEYARD,
    delayed: &[warp::EXILE],
    ..oracle::TIMELINE_CULLER
};
