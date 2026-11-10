use cantrip_api::*;
use cantrip_mechanics::{tokens, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    // "When this creature enters, create a 1/1 colorless Borg artifact creature token" (CR 603.6a, 111.1): the controller of
    // the ability creates the token. The shared body.
    abilities: &[triggered(
        triggers::THIS_ENTERS,
        tokens::create_borg::<1>,
        "When this creature enters, create a 1/1 colorless Borg artifact creature token.",
    )],
    ..oracle::ASSEMBLY_SPECIALIST
};

pub static EXTRA: &[&CardDef] = &[&tokens::BORG];
