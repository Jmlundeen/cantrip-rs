use cantrip_api::*;
use cantrip_mechanics::{blight, destroy_target, targets};
use cantrip_oracle as oracle;

/// Each mode: "Destroy target artifact or enchantment", "Destroy target creature with mana value 3 or greater". A mode's
/// target is its own first slot; one gone or no longer legal by resolution is left alone (CR 608.2b).
fn destroy_its_target(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

/// "Target creature with mana value 3 or greater" (CR 115.1, 202.3), asked as the spell is cast and again as it resolves
/// (CR 608.2b).
fn creature_mv3_or_greater(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::CREATURE.filter)(q, asker, t) && matches!(t, Target::Object(id) if q.mana_value(id) >= 3)
}

/// "Choose one. If this spell's additional cost was paid, choose both instead." (CR 700.2): with the blight paid, both
/// modes, each with its own target (CR 700.2c; the same artifact creature may be the target of both, CR 601.2c), done in
/// printed order (CR 700.2d).
static MODAL: ModalDef = modes(&[
    mode(destroy_its_target, "Destroy target artifact or enchantment.")
        .targets(&[targets::ARTIFACT_OR_ENCHANTMENT]),
    mode(destroy_its_target, "Destroy target creature with mana value 3 or greater.")
        .targets(&[target(creature_mv3_or_greater)]),
])
.when_paid(2, 2);

/// "As an additional cost to cast this spell, you may blight 2" (CR 701.68a, 601.2b): announced before the modes, since
/// how many are chosen is defined by it, and offered only while a creature to blight and targets for both modes exist
/// (CR 700.2a). The counters go on as the costs are paid (CR 601.2h), after the targets are chosen.
pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef::MODAL),
    modal: Some(&MODAL),
    optional_cost: Some(blight::optional(2)),
    labels: &["Cast without blighting (choose one)", "Blight 2: put two -1/-1 counters on a creature you control (choose both)"],
    ..oracle::PYRRHIC_STRIKE
};
