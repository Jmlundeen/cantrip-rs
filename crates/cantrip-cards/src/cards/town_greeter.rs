use cantrip_api::*;
use cantrip_mechanics::library;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// Mill four cards. You may put a land card from among them into your hand. If you put a Town card into your hand this
/// way, you gain 2 life.
fn mill_and_take(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    let taken = library::mill_then_may_take_into_hand(g, player, item.object, 4, "land", |c| c.is(CardType::Land))?;
    if let Some(&in_hand) = taken.first()
        && g.obj(in_hand).zone == Zone::hand(player)
        && g.chars(in_hand).subtypes.has(Subtype::Town)
    {
        g.gain_life(player, 2);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, mill_and_take, "When this creature enters, mill four cards. You may put a land card from among them into your hand. If you put a Town card into your hand this way, you gain 2 life.")],
    ..oracle::TOWN_GREETER
};
