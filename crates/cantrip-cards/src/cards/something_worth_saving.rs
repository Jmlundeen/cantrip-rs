use cantrip_api::*;
use cantrip_mechanics::{effects, library, targets};
use cantrip_oracle as oracle;

/// "Mill four cards. You may put a permanent card from among them into your hand. You gain 1 life." (CR 701.17, 110.4a): the four
/// cards milled are the ones to take from, a permanent card being an artifact, creature, enchantment, land, planeswalker or battle
/// card; nobody is asked when none of them is, and an empty or short library mills what it has. The life is gained either way.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    library::mill_then_may_take_into_hand(g, item.controller, item.object, 4, "permanent", targets::is_permanent_card)?;
    effects::gain_life(g, item, 1)
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::SOMETHING_WORTH_SAVING };
