use cantrip_api::*;
use cantrip_mechanics::effects::pump_creatures_you_control;
use cantrip_mechanics::lands::{enters_tapped_unless_you_control, sub};
use cantrip_oracle as oracle;

/// "{1}{R}{R}, {T}: Creatures you control get +1/+0 until end of turn": the creatures there as it resolves (CR 611.2c), no
/// creature that enters later.
fn pump_team(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    pump_creatures_you_control(g, item, 1, 0);
    Ok(())
}

/// "This land enters tapped unless you control a Mountain." "{T}: Add {R}."
pub static CARD: CardDef = CardDef {
    enters: Some(enters_tapped_unless_you_control::<{ sub(Subtype::Mountain) }>),
    abilities: &[
        mana_ability(TAP, add(Mana::R), "{T}: Add {R}."),
        ability(cost("{1}{R}{R}, {T}"), pump_team, "{1}{R}{R}, {T}: Creatures you control get +1/+0 until end of turn."),
    ],
    ..oracle::CASTLE_EMBERETH
};
