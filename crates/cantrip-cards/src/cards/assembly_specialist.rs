use cantrip_api::*;
use cantrip_oracle as oracle;
use cantrip_mechanics::{tokens, triggers};

fn borg(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::BORG, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(triggers::THIS_ENTERS, borg, "When this creature enters, create a 1/1 colorless Borg artifact creature token.")],
    ..oracle::ASSEMBLY_SPECIALIST
};
