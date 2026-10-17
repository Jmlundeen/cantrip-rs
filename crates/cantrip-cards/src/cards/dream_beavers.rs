use cantrip_api::*;
use cantrip_mechanics::{effects, scry, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "Each opponent loses 1 life and you gain 1 life. Scry 1." (CR 701.22)
fn drain_and_scry(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::each_opponent_loses_1_life_you_gain_1(g, item)?;
    scry(g, item.controller, item.object, 1)
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ENTERS,
        drain_and_scry,
        "When this creature enters, each opponent loses 1 life and you gain 1 life. Scry 1.",
    )],
    ..oracle::DREAM_BEAVERS
};
