use cantrip_api::*;
use cantrip_mechanics::{destroy_target, targets};
use cantrip_oracle as oracle;

/// "Destroy target nonland permanent" (CR 701.8): the artifact is already sacrificed as the cost, so it may destroy only another.
fn destroy(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

/// "This ability costs {3} less to activate if you attacked with a Spacecraft this turn" (CR 601.2f, 508.1): a creature declared as an
/// attacker this turn by the ability's controller that was a Spacecraft as it was declared (`Attacked::chars`), whatever it has
/// become or wherever it has gone since.
fn cost_reduction(q: &dyn Query, asker: &Asker, _: Costed, _: &[Target]) -> CostDelta {
    if q.history().attacked_with(asker.controller).any(|a| a.chars(q).has_subtype(Subtype::Spacecraft)) {
        CostDelta::less(3)
    } else {
        CostDelta::less(0)
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        cost("{6}").and(TAP).and(SACRIFICE_THIS),
        destroy,
        "{6}, {T}, Sacrifice this artifact: Destroy target nonland permanent. This ability costs {3} less to activate if you attacked with a Spacecraft this turn.",
    )
    .targets(&[targets::NONLAND_PERMANENT])
    .cost_change(cost_reduction)],
    ..oracle::THAUMATON_TORPEDO
};
