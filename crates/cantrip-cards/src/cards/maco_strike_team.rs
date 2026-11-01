use cantrip_api::*;
use cantrip_mechanics::{tokens, triggers};
use cantrip_oracle as oracle;

/// "Create a tapped 1/1 red Officer creature token" (CR 111.1): one, tapped, under this creature's controller.
fn create_tapped_officer(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create_by_rule(g, item.controller, &tokens::OFFICER, 1, true)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::YOUR_CREATURE_TARGETED_BY_AN_OPPONENT,
        create_tapped_officer,
        "Whenever a creature you control becomes the target of a spell or ability an opponent controls, create a tapped 1/1 red Officer creature token.",
    )],
    ..oracle::MACO_STRIKE_TEAM
};
