use cantrip_api::*;
use cantrip_mechanics::{effects, scry, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "You gain 2 life and scry 1." (CR 119.1, 701.22)
fn life_and_scry(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::gain_life(g, item, 2)?;
    scry(g, item.controller, item.object, 1)
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ENTERS,
        life_and_scry,
        "When this creature enters, you gain 2 life and scry 1.",
    )],
    ..oracle::HOLY_COW
};
