use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// Exile target creature you control, then return it to the battlefield under its owner's control. It returns as a
/// new object (CR 400.7); a token exiled this way ceases to exist and does not return (CR 111.7). Nothing happens if
/// the creature is no longer a legal target (CR 608.2b).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    cantrip_mechanics::exile_and_return(g, &[creature])
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE_YOU_CONTROL], effect),
    rebound: true,
    ..oracle::EPHEMERATE
};
