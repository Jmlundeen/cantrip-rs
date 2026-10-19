use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::tokens::{self, LANDER};
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// "Exile each opponent's graveyard. Create a Lander token." (CR 111.10u): the exile first, then the token, under the
/// ability's controller.
fn exile_graveyards_and_make_a_lander(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::exile_each_opponents_graveyard(g, item.controller);
    tokens::create_lander(g, item.controller)
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ENTERS,
        exile_graveyards_and_make_a_lander,
        "When this creature enters, exile each opponent's graveyard. Create a Lander token.",
    )],
    ..oracle::DAUNTLESS_SCRAPBOT
};

pub static EXTRA: &[&CardDef] = &[&LANDER];
