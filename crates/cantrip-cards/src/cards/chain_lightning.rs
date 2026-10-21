use cantrip_api::*;
use cantrip_mechanics::{damage_target, effects, targets};
use cantrip_oracle as oracle;

const PAY: Question = Question::new("CopySpell", "pay {cost} to copy {subject}?", "Once you have paid, you choose whether to copy it", "Pay {cost}", "Don't copy");
const COPY: Question = Question::new("CopySpell", "copy this spell?", "You may choose a new target for the copy", "Copy it", "Don't copy").no_default();

/// "Chain Lightning deals 3 damage to any target. Then that player or that permanent's controller may pay {R}{R}. If the
/// player does, they may copy this spell and may choose a new target for that copy." Paying and copying are two choices
/// (CR 707.10, 707.10c): a player who has paid may still decline the copy. A target that has become illegal makes the
/// whole spell fizzle (CR 608.2b), so nobody is offered the chain.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 3);
    let target = item.target(0);
    let Some(copier) = target.player().or_else(|| target.object().map(|id| g.controller(id))) else { return Ok(()) };
    if g.may_pay(copier, item.object, item.object, cost("{R}{R}"), &PAY)? && effects::ask_may(g, copier, item.object, Purpose::Ask(&COPY))? {
        g.copy_spell(item.object, copier, true)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::ANY], effect),
    ..oracle::CHAIN_LIGHTNING
};
