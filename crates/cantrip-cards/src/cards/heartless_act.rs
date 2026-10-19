use cantrip_api::*;
use cantrip_mechanics::{destroy_target, targets};
use cantrip_oracle as oracle;

/// "Target creature with no counters on it" (CR 115.1, 122.1): a creature on the battlefield with counters of no kind, judged as
/// it is chosen and again as the spell resolves (CR 608.2b), so a counter put on it in response makes the target illegal.
fn creature_with_no_counters(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::CREATURE.filter)(q, asker, t) && matches!(t, Target::Object(id) if !q.state().obj(id).has_counters())}

/// "Destroy target creature with no counters on it."
fn destroy_mode(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

/// "Remove up to three counters from target creature": its controller chooses which, of any kinds, as it resolves (CR 608.2d).
fn remove_mode(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    targets::object(g, item, 0).map(|creature| g.remove_counters_up_to(item.controller, item.object, creature, 3)).transpose()?;
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(destroy_mode, "Destroy target creature with no counters on it.").targets(&[target(creature_with_no_counters)]),
    mode(remove_mode, "Remove up to three counters from target creature.").targets(&[targets::CREATURE]),
]);

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef::MODAL),
    modal: Some(&MODAL),
    ..oracle::HEARTLESS_ACT
};
