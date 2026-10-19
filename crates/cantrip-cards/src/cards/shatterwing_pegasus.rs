use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "Flying" is a core keyword read from its characteristics. "{4}{W}: Creatures you control get +1/+1 until end of
/// turn." The creatures are those its controller controls as the ability resolves (CR 611.2c).
fn pump_team(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::pump_creatures_you_control(g, item, 1, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{4}{W}"), pump_team, "{4}{W}: Creatures you control get +1/+1 until end of turn.")],
    ..oracle::SHATTERWING_PEGASUS
};
