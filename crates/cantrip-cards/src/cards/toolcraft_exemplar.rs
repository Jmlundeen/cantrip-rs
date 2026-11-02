use cantrip_api::*;
use cantrip_mechanics::effects::gain_until_end_of_turn;
use cantrip_mechanics::{artifacts_you_control, pump, triggers};
use cantrip_oracle as oracle;

/// "At the beginning of combat on your turn, if you control an artifact, ..." (CR 603.4).
fn if_control_artifact(q: &dyn Query, asker: &Asker) -> bool {
    artifacts_you_control(q, asker.controller) >= 1
}

/// "This creature gets +2/+1 until end of turn. If you control three or more artifacts, it also gains first strike until
/// end of turn." (CR 608.2, 611.2a).
fn gets_pump_and_first_strike(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    pump(g, item.object, item.object, 2, 1);
    g.end_sentence();
    if artifacts_you_control(g, item.controller) >= 3 {
        gain_until_end_of_turn(g, item.object, item.object, Keyword::FirstStrike);
        g.end_sentence();
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::BEGINNING_OF_YOUR_COMBAT,
        gets_pump_and_first_strike,
        "At the beginning of combat on your turn, if you control an artifact, this creature gets +2/+1 until end of turn. If you control three or more artifacts, it also gains first strike until end of turn.",
    )
    .condition(if_control_artifact)],
    ..oracle::TOOLCRAFT_EXEMPLAR
};
