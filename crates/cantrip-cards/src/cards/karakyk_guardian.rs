use cantrip_api::*;
use cantrip_mechanics::statics::have_hexproof;
use cantrip_oracle as oracle;

/// "This creature has hexproof as long as it hasn't dealt damage yet" (CR 702.11, 604.1, 611.3a, 120.2): only the Guardian itself,
/// and only while the event history, over the whole game, holds no damage it dealt - to anything, combat or not, an amount of
/// zero or damage that was prevented not being dealt (CR 615.6), which the history never records as damage. The source is this object, so a Guardian that left and came back is
/// a new one with none dealt (CR 400.7). The layers ask again whenever characteristics are computed, so the hexproof stops the
/// moment the damage is dealt.
fn itself_while_it_has_dealt_no_damage(view: &dyn LayerQuery, source: ObjectId, affected: ObjectId, _: &Chars) -> bool {
    source == affected
        && !view.history().all().iter().any(|r| matches!(r.event, Event::Damage { source: dealer, .. } if dealer == source))
}

pub static CARD: CardDef = CardDef {
    statics: &[static_ability(
        Layer::Ability,
        itself_while_it_has_dealt_no_damage,
        have_hexproof,
        "This creature has hexproof as long as it hasn't dealt damage yet.",
    )],
    ..oracle::KARAKYK_GUARDIAN
};
