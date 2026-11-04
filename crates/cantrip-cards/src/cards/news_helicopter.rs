use cantrip_api::*;
use cantrip_mechanics::tokens::{self, HUMAN_CITIZEN};
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// "Create a 1/1 green and white Human Citizen creature token."
fn citizen(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &HUMAN_CITIZEN, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, citizen, "When this creature enters, create a 1/1 green and white Human Citizen creature token.")],
    ..oracle::NEWS_HELICOPTER
};

pub static EXTRA: &[&CardDef] = &[&HUMAN_CITIZEN];
