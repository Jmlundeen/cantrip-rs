use cantrip_api::*;
use cantrip_oracle as oracle;
use cantrip_mechanics::{tokens, triggers};

/// "When this creature enters, create a 1/1 colorless Borg artifact creature token." (CR 603.6a, 111.1): the controller
/// of the ability creates the token.
fn borg(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::BORG, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(triggers::THIS_ENTERS, borg, "When this creature enters, create a 1/1 colorless Borg artifact creature token.")],
    ..oracle::ASSEMBLY_SPECIALIST
};

pub static EXTRA: &[&CardDef] = &[&tokens::BORG];
