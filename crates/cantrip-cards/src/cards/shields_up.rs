use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Target artifact or creature you control gains hexproof and indestructible until end of turn. If it's a creature, put a
/// +1/+1 counter on it." Both parts are read of the target as the spell resolves (CR 608.2): the counter goes on it if it
/// is a creature then. Nothing happens if the target is no longer legal (CR 608.2b).
fn shield_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(permanent) = targets::object(g, item, 0) else { return Ok(()) };
    effects::gain_hexproof_and_indestructible_until_end_of_turn(g, item.object, permanent);
    if g.chars(permanent).is(CardType::Creature) {
        g.add_counters(permanent, CounterKind::PlusOne, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::ARTIFACT_OR_CREATURE_YOU_CONTROL], shield_it),
    ..oracle::SHIELDS_UP
};
