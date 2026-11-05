use cantrip_api::*;
use cantrip_mechanics::{tokens, triggers};
use cantrip_oracle as oracle;

fn create_borg(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::BORG, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(triggers::THIS_DIES, create_borg, "When this creature dies, create a 1/1 colorless Borg artifact creature token.")],
    ..oracle::TENACIOUS_DRONE
};

pub static EXTRA: &[&CardDef] = &[&tokens::BORG];
