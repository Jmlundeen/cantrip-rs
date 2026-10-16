use cantrip_api::*;
use cantrip_mechanics::{delayed, haste_until_end_of_turn, targets};
use cantrip_oracle as oracle;

/// "Create X tokens that are copies of target artifact or creature you control" (CR 707.2, 111.1): X is the value chosen as
/// the spell was cast (a copy of the spell has an X of 0, CR 107.3), and the tokens enter together. "Those tokens gain
/// haste until end of turn." "Sacrifice them at the beginning of the next end step" (CR 603.7c): a delayed ability about
/// exactly these tokens. If the target is no longer legal the spell does not resolve (CR 608.2b) and nothing is made.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(target) = targets::object(g, item, 0) else { return Ok(()) };
    let tokens = g.create_token_copies(target, item.controller, item.x as u32)?;
    if tokens.is_empty() {
        return Ok(());
    }
    for &token in &tokens {
        haste_until_end_of_turn(g, item.object, token);
    }
    delayed::at_next_end_step_about(g, item, 0, &tokens);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::ARTIFACT_OR_CREATURE_YOU_CONTROL], effect),
    delayed: &[delayed::SACRIFICE_CREATED],
    ..oracle::DEVASTATING_ONSLAUGHT
};
