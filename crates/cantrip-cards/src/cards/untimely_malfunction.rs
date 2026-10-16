use cantrip_api::*;
use cantrip_mechanics::{destroy_target, effects, targets};
use cantrip_oracle as oracle;

/// Destroy target artifact.
fn destroy_artifact(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

/// One or two target creatures can't block this turn.
fn cant_block(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    for slot in 0..item.target_count as usize {
        if let Some(creature) = targets::object(g, item, slot) {
            g.add_effect(item.object, creature, EffectKind::CantBlock, Duration::EndOfTurn);
        }
    }
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(destroy_artifact, "Destroy target artifact.")
        .targets(&[targets::ARTIFACT]),
    mode(effects::change_first_target, "Change the target of target spell or ability with a single target.")
        .targets(&[targets::SPELL_OR_ABILITY_WITH_A_SINGLE_TARGET]),
    mode(cant_block, "One or two target creatures can't block this turn.")
        .targets(&[targets::CREATURE.between(1, 2)]),
]);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::UNTIMELY_MALFUNCTION };
