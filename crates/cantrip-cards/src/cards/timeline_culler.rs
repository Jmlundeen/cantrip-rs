use cantrip_api::*;
use cantrip_mechanics::warp;
use cantrip_oracle as oracle;

/// "Warp - {B}, Pay 2 life" (CR 702.185a, 118.8): the warp cost, from the hand. The life is paid with the mana (CR 601.2b, 601.2f-h).
const WARP_COST: Cost = cost("{B}").and(life(2));

/// "You may cast this card from your graveyard using its warp ability": the card's own sentence, not warp's (warp casts only from the
/// hand, CR 702.185a). It is the same warp cost paid for the card cast from its owner's graveyard, and only from there, and it counts
/// as cast for its warp cost (CR 702.185a-c): the permanent is exiled at the next end step and the spell was warped.
const FROM_THE_GRAVEYARD: WayDef =
    way(WARP_COST, ZoneKind::Graveyard, "Warp\u{2014}{B}, Pay 2 life (from your graveyard)").kind(WayKind::Warp).only_from();

pub static CARD: CardDef = CardDef {
    ways: &[warp::way_with_cost(WARP_COST, "Warp\u{2014}{B}, Pay 2 life"), FROM_THE_GRAVEYARD],
    // Both ways (the first from the hand, the second from the graveyard) create the card's first delayed ability.
    end_step_riders: &[(1, 0), (2, 0)],
    delayed: &[warp::EXILE],
    ..oracle::TIMELINE_CULLER
};
