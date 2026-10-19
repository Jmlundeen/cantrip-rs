use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

/// "Whenever you cast a noncreature spell, creatures you control get +1/+0 until end of turn" (CR 611.2c): the
/// creatures its controller controls as the trigger resolves, and no creature that enters later.
fn pump_team(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::pump_creatures_you_control(g, item, 1, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::NONCREATURE_SPELL_YOU_CAST,
        pump_team,
        "Whenever you cast a noncreature spell, creatures you control get +1/+0 until end of turn.",
    )],
    ..oracle::WINTER_TEAM_PLAYER
};
