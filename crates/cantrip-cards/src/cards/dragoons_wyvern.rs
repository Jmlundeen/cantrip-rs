use cantrip_api::*;
use cantrip_mechanics::tokens::HERO_1_1_COLORLESS;
use cantrip_mechanics::{tokens, triggers};
use cantrip_oracle as oracle;

/// "When this creature enters, create a 1/1 colorless Hero creature token." (CR 603.6a, 111.1): the controller of the ability
/// creates the token, the 1/1 colorless Hero that job select makes too.
fn hero(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &HERO_1_1_COLORLESS, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::THIS_ENTERS,
        hero,
        "When this creature enters, create a 1/1 colorless Hero creature token.",
    )],
    ..oracle::DRAGOONS_WYVERN
};

pub static EXTRA: &[&CardDef] = &[&HERO_1_1_COLORLESS];
