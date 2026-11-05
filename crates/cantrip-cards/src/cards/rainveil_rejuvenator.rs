use cantrip_api::*;
use cantrip_mechanics::{effects, mana};
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// "When this creature enters, you may mill three cards." (CR 701.17a, 701.17b)
fn may_mill_three(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::may_mill(g, item, 3)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, may_mill_three, "When this creature enters, you may mill three cards. (You may put the top three cards of your library into your graveyard.)"),
        mana_ability(TAP, add_by(mana::add_green_equal_to_its_power), "{T}: Add an amount of {G} equal to this creature's power."),
    ],
    ..oracle::RAINVEIL_REJUVENATOR
};
