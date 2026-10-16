use cantrip_api::*;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

/// "Each opponent loses 1 life and you gain 1 life."
fn drain(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.each_opponent_loses_life(item.controller, 1);
    g.gain_life(item.controller, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(triggers::YOU_COMMIT_A_CRIME, drain, "Whenever you commit a crime, each opponent loses 1 life and you gain 1 life. This ability triggers only once each turn. (Targeting opponents, anything they control, and/or cards in their graveyards is a crime.)")
        .triggers_each_turn(1)],
    ..oracle::RAVEN_OF_FELL_OMENS
};
