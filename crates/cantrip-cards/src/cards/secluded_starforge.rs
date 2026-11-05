use cantrip_api::*;
use cantrip_mechanics::tokens::{self, ROBOT_2_2};
use cantrip_mechanics::{targets, untapped_artifact_you_control};
use cantrip_oracle as oracle;

/// "Target creature gets +X/+0 until end of turn." X is the number of artifacts tapped to pay the cost, announced as the
/// ability was activated (CR 107.3a, 601.2b by way of 602.2b; `tap_x_untapped`).
fn target_creature_gets_x_0(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    cantrip_mechanics::pump(g, item.object, creature, item.x as i32, 0);
    Ok(())
}

/// "Create a 2/2 colorless Robot artifact creature token." (CR 111.2)
fn make_robot(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &ROBOT_2_2, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        ability(
            cost("{2}, {T}").and(tap_x_untapped(untapped_artifact_you_control)),
            target_creature_gets_x_0,
            "{2}, {T}, Tap X untapped artifacts you control: Target creature gets +X/+0 until end of turn. Activate only as a sorcery.",
        )
        .targets(&[targets::CREATURE])
        .sorcery_speed(),
        ability(cost("{5}, {T}"), make_robot, "{5}, {T}: Create a 2/2 colorless Robot artifact creature token."),
    ],
    ..oracle::SECLUDED_STARFORGE
};

pub static EXTRA: &[&CardDef] = &[&ROBOT_2_2];
