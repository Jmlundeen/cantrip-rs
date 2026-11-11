use cantrip_api::*;
use cantrip_mechanics::{tokens, triggers};
use cantrip_oracle as oracle;

/// "When The Duras Sisters die, create a 2/1 red Klingon Warrior creature token with haste." (CR 603.6a, 111.1): the
/// ability's controller creates the token, and it is theirs - an opponent who controlled The Duras Sisters gets it.
fn create_klingon(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create_klingons(g, item.controller, 1)
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::THIS_DIES,
        create_klingon,
        "When The Duras Sisters die, create a 2/1 red Klingon Warrior creature token with haste.",
    )],
    ..oracle::THE_DURAS_SISTERS
};

pub static EXTRA: &[&CardDef] = &[&tokens::KLINGON];
