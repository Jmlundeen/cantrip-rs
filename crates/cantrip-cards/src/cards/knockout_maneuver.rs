use cantrip_api::*;
use cantrip_mechanics::{fight, targets};
use cantrip_oracle as oracle;

/// "Put a +1/+1 counter on target creature you control. Then it deals damage equal to its power to target creature an opponent
/// controls." The counter is put first, so the damage is the power it has now (CR 608.2c); with either target illegal the creature
/// deals no damage, and with the first one illegal there is no counter either.
fn knockout_maneuver(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    g.add_counters(creature, CounterKind::PlusOne, 1);
    // A sentence ends (CR 608.2c): the state triggers judge what it did before the next (`Game::end_sentence`).
    g.end_sentence();
    fight::targets_bite(g, item, 0, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE_YOU_CONTROL, targets::CREATURE_AN_OPPONENT_CONTROLS], knockout_maneuver),
    ..oracle::KNOCKOUT_MANEUVER
};
