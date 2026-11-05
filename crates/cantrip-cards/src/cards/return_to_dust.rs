use cantrip_api::*;
use cantrip_mechanics::{history, targets};
use cantrip_oracle as oracle;

/// "Exile target artifact or enchantment. If you cast this spell during your main phase, you may exile up to one other
/// target artifact or enchantment" (CR 701.13a, 608.2b, 601.2i): the first target is exiled whatever the step; the second only
/// when this spell was cast during its caster's main phase - the cast record's fact, so a copy never has it, and an
/// opponent's main phase does not count - and one was chosen (ruling 2021-03-19: the slot may always be targeted, and the
/// card simply is not exiled otherwise).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(first) = targets::object(g, item, 0) {
        g.exile(first);
    }
    if g.history().fact_as_cast(item.object) != 0
        && let Some(second) = targets::object(g, item, 1)
    {
        g.exile(second);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::ARTIFACT_OR_ENCHANTMENT, targets::UP_TO_ONE_OTHER_ARTIFACT_OR_ENCHANTMENT], effect),
    cast_fact: Some(history::cast_during_your_main_phase),
    ..oracle::RETURN_TO_DUST
};
