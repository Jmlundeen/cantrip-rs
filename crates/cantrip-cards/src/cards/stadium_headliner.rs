use cantrip_api::*;
use cantrip_mechanics::{damage_target, mobilize, targets};
use cantrip_oracle as oracle;

/// "{1}{R}, Sacrifice this creature: It deals damage equal to the number of creatures you control to target creature."
/// The number is counted as the ability resolves (official ruling 2025-04-04), so the sacrificed creature is not in it.
fn blast(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let creatures = cantrip_mechanics::creatures_you_control(g, item.controller);
    damage_target(g, item, 0, creatures as u16);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mobilize::trigger::<1>("Mobilize 1 (Whenever this creature attacks, create a tapped and attacking 1/1 red Warrior creature token. Sacrifice it at the beginning of the next end step.)"),
        ability(cost("{1}{R}").and(SACRIFICE_THIS), blast, "{1}{R}, Sacrifice this creature: It deals damage equal to the number of creatures you control to target creature.")
            .targets(&[targets::CREATURE]),
    ],
    delayed: &[mobilize::SACRIFICE],
    ..oracle::STADIUM_HEADLINER
};

pub static EXTRA: &[&CardDef] = &[&mobilize::WARRIOR];
