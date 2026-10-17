use cantrip_api::*;
use cantrip_mechanics::{sacrifice_chosen, targets};
use cantrip_oracle as oracle;

/// "Target player sacrifices a creature of their choice." (CR 701.21a): the targeted player chooses when they control
/// more than one creature and the creature is taken without asking when there is only one. Only the player is targeted,
/// so a creature's hexproof does not matter. A target that is no longer legal leaves the effect nothing to do (CR 608.2b).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(victim) = targets::player(g, item, 0) else { return Ok(()) };
    sacrifice_chosen(g, victim, item.object, 1, |c| c.is(CardType::Creature))
}

pub static CARD: CardDef = CardDef { spell: spell(&[targets::PLAYER], effect), ..oracle::DIABOLIC_EDICT };
