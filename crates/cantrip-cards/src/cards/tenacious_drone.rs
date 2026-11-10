use cantrip_api::*;
use cantrip_mechanics::{tokens, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    // "When this creature dies, create a 1/1 colorless Borg artifact creature token" (CR 603.6a, 111.1, 700.4): the token is
    // created as the death trigger resolves. The shared body.
    abilities: &[triggered(
        triggers::THIS_DIES,
        tokens::create_borg::<1>,
        "When this creature dies, create a 1/1 colorless Borg artifact creature token.",
    )],
    ..oracle::TENACIOUS_DRONE
};

pub static EXTRA: &[&CardDef] = &[&tokens::BORG];
