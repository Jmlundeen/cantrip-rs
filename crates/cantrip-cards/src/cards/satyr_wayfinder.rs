use cantrip_api::*;
use cantrip_mechanics::library::reveal_top_take_one_rest_to_graveyard;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// Reveal the top four cards of your library; you may put a land card from among them into your hand; put the rest
/// into your graveyard, in the order they were revealed. With no land among them there is nothing to decide.
fn rummage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    reveal_top_take_one_rest_to_graveyard(g, item.controller, item.object, 4, |chars| chars.is(CardType::Land))
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, rummage, "When Satyr Wayfinder enters, reveal the top four cards of your library. You may put a land card from among them into your hand. Put the rest into your graveyard.")],
    ..oracle::SATYR_WAYFINDER
};
