use cantrip_api::*;
use cantrip_mechanics::{destroy_target, targets};
use cantrip_oracle as oracle;

/// "Target spell or ability with a single target" (CR 115.1): exactly one target across every slot, counted with repeats, as the
/// item was announced; re-asked as the mode resolves (CR 608.2b).
fn spell_or_ability_with_a_single_target(q: &dyn Query, _: &Asker, t: Target) -> bool {
    let stack = &q.state().stack;
    match t {
        Target::Object(id) => stack.iter().any(|i| i.kind == StackKind::Spell && i.object == id && i.target_count == 1),
        Target::Ability(serial) => stack.iter().any(|i| i.kind != StackKind::Spell && i.serial == serial && i.target_count == 1),
        _ => false,
    }
}

/// Destroy target artifact.
fn destroy_artifact(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

/// Change the target of target spell or ability with a single target (CR 115.7a): its caster chooses the new one.
fn change_target(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if g.target_legal(item, 0) {
        g.change_target(item.targets[0], item.controller)?;
    }
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

static MODAL: ModalDef = ModalDef {
    modes: &[
        ModeDef {
            text: "Destroy target artifact.",
            targets: &[targets::ARTIFACT],
            effect: destroy_artifact,
            cost: ManaCost::ZERO,
        },
        ModeDef {
            text: "Change the target of target spell or ability with a single target.",
            targets: &[TargetSpec { min: 1, max: 1, filter: spell_or_ability_with_a_single_target }],
            effect: change_target,
            cost: ManaCost::ZERO,
        },
        ModeDef {
            text: "One or two target creatures can't block this turn.",
            targets: &[TargetSpec { max: 2, ..targets::CREATURE }],
            effect: cant_block,
            cost: ManaCost::ZERO,
        },
    ],
    min: 1,
    max: 1,
    escalate: None,
    repeat: false, allowed: None,
};

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::UNTIMELY_MALFUNCTION };
