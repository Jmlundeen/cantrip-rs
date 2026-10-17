use cantrip_api::*;
use cantrip_mechanics::creatures_you_control;
use cantrip_mechanics::effects::pump_and_trample_creatures_you_control;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// "When this creature enters, creatures you control gain trample and get +X/+X until end of turn, where X is the number of
/// creatures you control." The creatures are the ones on the battlefield as this resolves and no creature that enters later
/// (CR 611.2c); X is counted once, as it resolves (CR 608.2h), Craterhoof Behemoth itself included if it is still there.
/// Haste is the core keyword of its JSON.
fn pump_and_trample(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let x = creatures_you_control(g, item.controller) as i16;
    pump_and_trample_creatures_you_control(g, item, x, x);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ENTERS,
        pump_and_trample,
        "When this creature enters, creatures you control gain trample and get +X/+X until end of turn, where X is the number of creatures you control.",
    )],
    ..oracle::CRATERHOOF_BEHEMOTH
};
