use cantrip_api::*;
use cantrip_mechanics::{scry, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "When this land enters, scry 1." (CR 701.22)
fn scry_one(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    scry(g, item.controller, item.object, 1)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        mana_ability(cost("{1}, {T}"), add_any_color(1), "{1}, {T}: Add one mana of any color."),
        triggered(THIS_ENTERS, scry_one, "When this land enters, scry 1."),
    ],
    ..oracle::CRYSTAL_GROTTO
};
