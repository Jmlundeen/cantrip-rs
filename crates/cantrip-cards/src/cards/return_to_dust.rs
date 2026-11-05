use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "If you cast this spell during your main phase" (CR 601.2i, 500.5a): the spell is still on the stack, so the step it
/// is in is the step it was cast in - nothing advances a step while a spell waits there.
fn cast_in_a_main_phase(g: &mut dyn Game) -> bool {
    matches!(g.state().turn.step(), Step::PrecombatMain | Step::PostcombatMain)
}

/// "Exile target artifact or enchantment. If you cast this spell during your main phase, you may exile up to one other
/// target artifact or enchantment" (CR 701.13a, 608.2b): the first target is exiled whatever the step; the second only
/// when cast in a main phase and one was chosen (ruling 2021-03-19: the slot may always be targeted, and the card simply
/// is not exiled outside a main phase).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(first) = targets::object(g, item, 0) {
        g.exile(first);
    }
    if cast_in_a_main_phase(g)
        && let Some(second) = targets::object(g, item, 1)
    {
        g.exile(second);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::ARTIFACT_OR_ENCHANTMENT, targets::UP_TO_ONE_OTHER_ARTIFACT_OR_ENCHANTMENT], effect),
    ..oracle::RETURN_TO_DUST
};
