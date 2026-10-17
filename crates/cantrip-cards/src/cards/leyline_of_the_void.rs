use cantrip_api::*;
use cantrip_mechanics::statics;
use cantrip_oracle as oracle;

/// "If a card would be put into an opponent's graveyard from anywhere, exile it instead" (CR 614.1a, 400.3): a card, not a token
/// or a copy of a spell (CR 108.2), going to the graveyard of a player who is an opponent of this permanent's controller, from any
/// zone - a spell resolving or countered, a creature dying, a discard, a mill.
fn card_to_an_opponents_graveyard(q: &dyn Query, asker: &Asker, proposal: &Proposal) -> bool {
    matches!(proposal, Proposal::ZoneChange(z)
        if z.is_card
            && z.to.is(ZoneKind::Graveyard)
            && q.state().opponents(asker.controller).any(|opponent| opponent == z.to.player))
}

pub static CARD: CardDef = CardDef {
    opening_hand: Some(statics::BEGIN_ON_THE_BATTLEFIELD),
    replacements: &[replacement(
        ReplacementKind::ZoneChange,
        card_to_an_opponents_graveyard,
        cantrip_mechanics::replacements::exile_instead_apply,
        "exiles a card an opponent's graveyard would receive",
    )],
    ..oracle::LEYLINE_OF_THE_VOID
};
