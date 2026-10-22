use cantrip_api::*;
use cantrip_mechanics::typecycling;
use cantrip_mechanics::triggers::{THIS_DIES, THIS_ENTERS};
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "When this creature enters or dies, you gain 2 life." (CR 603.6a, 603.10a.)
fn gain_two_life(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::gain_life(g, item, 2)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, gain_two_life, "When this creature enters, you gain 2 life."),
        triggered(THIS_DIES, gain_two_life, "When this creature dies, you gain 2 life."),
        typecycling::basic_landcycling(mana("{2}"), "Basic landcycling {2}"),
    ],
    ..oracle::APEX_WITCHSTALKER
};
