use cantrip_api::*;
use cantrip_mechanics::{effects, lands};
use cantrip_oracle as oracle;

/// "Whenever this creature attacks, creatures you control get +X/+0 until end of turn, where X is the number of
/// differently named lands you control." (CR 201.2a-b, 611.2c): X is counted as the trigger resolves, and the creatures
/// are the ones there then.
fn pump_for_each_differently_named_land(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let x = lands::differently_named_lands(g, item.controller) as i32;
    effects::pump_creatures_you_control(g, item, x, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        cantrip_mechanics::triggers::THIS_ATTACKS,
        pump_for_each_differently_named_land,
        "Strange new worlds \u{2014} Whenever this creature attacks, creatures you control get +X/+0 until end of turn, where X is the number of differently named lands you control.",
    )],
    ..oracle::PAH_WRAITH
};
