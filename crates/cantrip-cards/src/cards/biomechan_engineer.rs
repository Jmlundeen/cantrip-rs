use cantrip_api::*;
use cantrip_mechanics::tokens::{self, LANDER, ROBOT_2_2};
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// "Draw two cards and create a 2/2 colorless Robot artifact creature token" (CR 608.2c): the cards are drawn, then the Robot is made.
fn draw_and_make_a_robot(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    g.draw(you, 2);
    let robot = g.card_id(&ROBOT_2_2);
    g.create_tokens(robot, you, 1, false)?;
    Ok(())
}

/// "When this creature enters, create a Lander token."
fn lander(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create_lander(g, item.controller)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, lander, "When this creature enters, create a Lander token."),
        ability(cost("{8}"), draw_and_make_a_robot, "{8}: Draw two cards and create a 2/2 colorless Robot artifact creature token."),
    ],
    ..oracle::BIOMECHAN_ENGINEER
};

pub static EXTRA: &[&CardDef] = &[&LANDER, &ROBOT_2_2];
