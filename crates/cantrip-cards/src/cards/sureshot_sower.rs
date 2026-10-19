use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "{3}{G}, Discard this card: Destroy target creature with flying." (CR 118.3: discarding it is the cost, paid as the
/// ability is activated, so the card is already in the graveyard when it resolves; the target is chosen first, CR 602.2b.)
fn destroy_flyer(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::destroy_target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        cost("{3}{G}").and(DISCARD_THIS),
        destroy_flyer,
        "{3}{G}, Discard this card: Destroy target creature with flying.",
    )
    .targets(&[targets::CREATURE_WITH_FLYING])
    .from(ZoneKind::Hand)],
    ..oracle::SURESHOT_SOWER
};
