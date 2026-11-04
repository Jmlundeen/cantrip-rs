use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

/// "Other creatures you control get +1/+0 and gain deathtouch until end of turn." The creatures are those its controller controls as
/// it resolves, not Syr Vondam, and none that enters later (CR 611.2c).
fn others_get_plus_one_and_deathtouch(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    instruction(g, |g| {
        for creature in effects::pump_other_creatures_you_control(g, item, 1, 0) {
            effects::gain_until_end_of_turn(g, item.object, creature, Keyword::Deathtouch);
        }
    });
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::THIS_ENTERS_OR_ATTACKS,
        others_get_plus_one_and_deathtouch,
        "Whenever Syr Vondam enters or attacks, other creatures you control get +1/+0 and gain deathtouch until end of turn.",
    )],
    ..oracle::SYR_VONDAM_THE_LUCENT
};
