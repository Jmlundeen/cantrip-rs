use cantrip_api::*;
use cantrip_mechanics::{targets, triggers};
use cantrip_oracle as oracle;

/// "Put a +1/+1 counter on this creature": the creature the ability triggered on, not a new object it became (CR 400.7).
fn grow(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_counters(item.object, CounterKind::PlusOne, 1);
    Ok(())
}

/// "Target opponent loses 1 life and you gain 1 life." With its target illegal it does not resolve (CR 608.2b), so
/// nothing is gained.
fn drain(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Player(opponent) = item.targets[0] else { return Ok(()) };
    g.lose_life(opponent, 1);
    g.gain_life(item.controller, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::YOU_COMMIT_A_CRIME, grow, "Whenever you commit a crime, put a +1/+1 counter on this creature. This ability triggers only once each turn. (Targeting opponents, anything they control, and/or cards in their graveyards is a crime.)")
            .triggers_each_turn(1),
        ability(cost("{3}{B}"), drain, "{3}{B}: Target opponent loses 1 life and you gain 1 life.").targets(&[targets::OPPONENT]),
    ],
    ..oracle::BLOOD_HUSTLER
};
