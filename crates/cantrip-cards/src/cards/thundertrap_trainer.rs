use cantrip_api::*;
use cantrip_mechanics::{look_for_noncreature_nonland, offspring};
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// "Look at the top four cards of your library. You may reveal a noncreature, nonland card from among them and put it
/// into your hand. Put the rest on the bottom of your library in a random order."
fn look(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    look_for_noncreature_nonland(g, item.controller, item.object, 4)
}

pub static CARD: CardDef = CardDef {
    kicker: Some(offspring::kicker(mana(4, 0, 0, 0, 0, 0))),
    abilities: &[
        triggered(THIS_ENTERS, look, "When this creature enters, look at the top four cards of your library. You may reveal a noncreature, nonland card from among them and put it into your hand. Put the rest on the bottom of your library in a random order."),
        offspring::trigger("Offspring {4} (You may pay an additional {4} as you cast this spell. If you do, when this creature enters, create a 1/1 token copy of it.)"),
    ],
    ..oracle::THUNDERTRAP_TRAINER
};
