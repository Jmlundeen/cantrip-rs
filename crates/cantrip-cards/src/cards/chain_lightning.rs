use cantrip_api::*;
use cantrip_mechanics::{damage_target, targets};
use cantrip_oracle as oracle;

const COPY: Question = Question::new("CopySpell", "pay {cost} to copy {subject}?", "You choose a new target for the copy", "Pay {cost}", "Don't copy");

/// Three damage to any target; then that player, or that permanent's controller, may pay {R}{R}, and if they do, copy
/// this spell and choose a new target for the copy (CR 707.10, 707.10c). Paying is the choice to copy. A target that
/// has become illegal makes the whole spell fizzle (CR 608.2b), so nobody is offered the chain.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 3);
    let target = item.targets[0];
    let Some(copier) = target.player().or_else(|| target.object().map(|id| g.controller(id))) else { return Ok(()) };
    if g.may_pay(copier, item.object, item.object, cost("{R}{R}"), &COPY)? {
        g.copy_spell(item.object, copier, true)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::ANY], effect),
    ..oracle::CHAIN_LIGHTNING
};
