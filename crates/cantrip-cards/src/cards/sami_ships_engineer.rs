use cantrip_api::*;
use cantrip_mechanics::tokens::ROBOT_2_2;
use cantrip_mechanics::triggers::{YOUR_END_STEP, you_control_two_or_more_tapped_creatures};
use cantrip_oracle as oracle;

/// "Create a tapped 2/2 colorless Robot artifact creature token."
fn tapped_robot(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let robot = g.card_id(&ROBOT_2_2);
    g.create_tokens(robot, item.controller, 1, true)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        YOUR_END_STEP,
        tapped_robot,
        "At the beginning of your end step, if you control two or more tapped creatures, create a tapped 2/2 colorless Robot artifact creature token.",
    )
    // CR 603.4: an intervening "if", checked as the step begins and again as the ability resolves.
    .condition(you_control_two_or_more_tapped_creatures)],
    ..oracle::SAMI_SHIPS_ENGINEER
};

pub static EXTRA: &[&CardDef] = &[&ROBOT_2_2];
