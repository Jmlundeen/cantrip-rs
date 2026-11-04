use cantrip_api::*;
use cantrip_mechanics::effects::gain_until_end_of_turn;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Target creature you control gains double strike until end of turn. If it has a +1/+1 counter on it, draw a card." (CR 702.4,
/// 611.2a, 608.2c): two instructions in turn, the second asked of the creature as it is then. A target no longer legal
/// as the spell resolves means the spell does nothing at all (CR 608.2b).
fn gain_double_strike_and_draw_if_countered(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    gain_until_end_of_turn(g, item.object, creature, Keyword::DoubleStrike);
    if g.obj(creature).counters(CounterKind::PlusOne) > 0 {
        g.draw(item.controller, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE_YOU_CONTROL], gain_double_strike_and_draw_if_countered),
    ..oracle::DUAL_SUN_TECHNIQUE
};
