use cantrip_api::*;
use cantrip_mechanics::{tokens, triggers};
use cantrip_oracle as oracle;

/// "Create a tapped 1/1 red Officer creature token." (CR 701.7a, 111.2, 612.2a)
fn create_tapped_officer(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let officer = g.card_id(&tokens::OFFICER);
    g.create_tokens(officer, item.controller, 1, true)?;
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

pub static EXTRA: &[&CardDef] = &[&tokens::OFFICER];
