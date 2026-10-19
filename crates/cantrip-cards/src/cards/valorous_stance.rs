use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Target creature gains indestructible until end of turn." Nothing happens if the target is no longer legal (CR 608.2b).
fn gain_indestructible(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        effects::gain_indestructible_until_end_of_turn(g, item.object, creature);
    }
    Ok(())
}

/// "Target creature with toughness 4 or greater": the computed toughness (CR 115.1), so a counter or an effect moves a
/// creature in and out of range.
fn big_toughness_creature(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::CREATURE.filter)(q, asker, t)
        && matches!(t, Target::Object(id) if q.chars(id).toughness.is_some_and(|v| v >= 4))
}

static MODAL: ModalDef = modes(&[
    mode(gain_indestructible, "Target creature gains indestructible until end of turn.").targets(&[targets::CREATURE]),
    mode(effects::destroy_first_target, "Destroy target creature with toughness 4 or greater.").targets(&[target(big_toughness_creature)]),
]);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::VALOROUS_STANCE };
