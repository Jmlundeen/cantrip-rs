use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

/// "Whenever you cast an instant or sorcery spell, creatures you control get +1/+0 and gain trample until end of turn." Only the
/// creatures there as the ability resolves are affected, Balmor included (CR 611.2c); the ability resolves before the spell that
/// caused it, and whether or not that spell is still on the stack (CR 603.3).
fn pump_creatures(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::pump_and_trample_creatures_you_control(g, item, 1, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::INSTANT_OR_SORCERY_YOU_CAST,
        pump_creatures,
        "Whenever you cast an instant or sorcery spell, creatures you control get +1/+0 and gain trample until end of turn.",
    )],
    ..oracle::BALMOR_BATTLEMAGE_CAPTAIN
};
