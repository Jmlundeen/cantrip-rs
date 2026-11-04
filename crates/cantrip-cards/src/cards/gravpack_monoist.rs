use cantrip_api::*;
use cantrip_mechanics::tokens::ROBOT_2_2;
use cantrip_mechanics::triggers::THIS_DIES;
use cantrip_oracle as oracle;

/// "When this creature dies, create a tapped 2/2 colorless Robot artifact creature token."
fn make_robot(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let robot = g.card_id(&ROBOT_2_2);
    g.create_tokens(robot, item.controller, 1, true)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_DIES, make_robot, "When this creature dies, create a tapped 2/2 colorless Robot artifact creature token.")],
    ..oracle::GRAVPACK_MONOIST
};

pub static EXTRA: &[&CardDef] = &[&ROBOT_2_2];
