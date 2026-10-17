use cantrip_api::*;
use cantrip_mechanics::{mana, scry, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "When this land enters, scry 1." (CR 701.22)
fn scry_one(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    scry(g, item.controller, item.object, 1)
}

pub static CARD: CardDef = CardDef {
    enters: Some(mana::enters_tapped),
    abilities: &[
        triggered(THIS_ENTERS, scry_one, "When this land enters, scry 1."),
        mana_ability(TAP, add_one_of(&[Color::Blue, Color::Black]), "{T}: Add {U} or {B}."),
    ],
    ..oracle::TEMPLE_OF_DECEIT
};
