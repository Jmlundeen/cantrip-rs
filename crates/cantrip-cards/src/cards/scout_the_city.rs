use cantrip_api::*;
use cantrip_mechanics::{effects, library, targets};
use cantrip_oracle as oracle;

/// "Mill three cards. You may put a permanent card from among them into your hand. You gain 3 life." (CR 701.17a, 701.17c,
/// 110.4a, 119.3): the life is gained whether or not a card was taken, or any was milled (a library with fewer than three, or none).
fn look_around(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    library::mill_then_may_take_into_hand(g, item.controller, item.object, 3, "permanent", targets::is_permanent_card)?;
    effects::gain_life(g, item, 3)
}

static MODAL: ModalDef = modes(&[
    mode(look_around, "Look Around \u{2014} Mill three cards. You may put a permanent card from among them into your hand. You gain 3 life."),
    mode(effects::destroy_first_target, "Bring Down \u{2014} Destroy target creature with flying.").targets(&[targets::CREATURE_WITH_FLYING]),
]);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::SCOUT_THE_CITY };
