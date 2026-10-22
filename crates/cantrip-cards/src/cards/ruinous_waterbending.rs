use cantrip_api::*;
use cantrip_mechanics::effects::pump_all_creatures;
use cantrip_mechanics::{delayed, triggers::CREATURE_DIES};
use cantrip_oracle as oracle;

/// "Whenever a creature dies this turn, you gain 1 life." A leaves-the-battlefield ability, looked back on (CR 603.10a): the board as it was before the event is consulted, so a Hushbringer
/// dying in the same event still stops it from triggering. "you" is the spell's controller as it resolved (CR 603.7d).
fn gain_one(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.gain_life(item.controller, 1);
    Ok(())
}

/// "All creatures get -2/-2 until end of turn. If this spell's additional cost was paid, whenever a creature dies this turn,
/// you gain 1 life." The -2/-2 goes to the creatures there as it resolves (CR 611.2c); the creatures it kills die as a
/// state-based action after the spell has resolved (CR 704.5f), so the delayed triggered ability (CR 603.7a, 603.7b) is
/// created first and sees them. The additional cost is the waterbend `kicker` (CR 701.67): paid, `item.kicked` is 1.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    pump_all_creatures(g, item.object, -2, -2);
    g.end_sentence();
    if item.kicked > 0 {
        delayed::this_turn(g, item, item.object);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], effect),
    // "As an additional cost to cast this spell, you may waterbend {4}."
    kicker: waterbend_kicker(mana("{4}")),
    delayed: &[triggered(CREATURE_DIES, gain_one, "Whenever a creature dies this turn, you gain 1 life.")],
    ..oracle::RUINOUS_WATERBENDING
};
