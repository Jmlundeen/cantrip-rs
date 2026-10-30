use cantrip_api::*;
use cantrip_mechanics::effects::{self, this_gains_indestructible_then_tap};
use cantrip_mechanics::greatest_power;
use cantrip_mechanics::TAP_ANOTHER_UNTAPPED_CREATURE;
use cantrip_oracle as oracle;

/// "Creatures you control get +X/+X until end of turn, where X is the greatest power among creatures you control": X is
/// worked out once, as the ability resolves (CR 608.2h, the ruling); with no creature it is 0, and a negative greatest power
/// gives +0/+0 (CR 107.1b).
fn pump_by_greatest_power(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let x = greatest_power::among_creatures_you_control(g, item.controller) as i16;
    effects::pump_creatures_you_control(g, item, x, x);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(
            TAP_ANOTHER_UNTAPPED_CREATURE,
            this_gains_indestructible_then_tap,
            "Tap another untapped creature you control: This creature gains indestructible until end of turn. Tap it.",
        ),
        ability(
            cost("{3}{G}{G}"),
            pump_by_greatest_power,
            "{3}{G}{G}: Creatures you control get +X/+X until end of turn, where X is the greatest power among creatures you control.",
        ),
    ],
    ..oracle::TYVAR_THE_PUMMELER
};
