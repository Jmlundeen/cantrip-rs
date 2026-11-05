use cantrip_api::*;
use cantrip_mechanics::triggers::{DRAGON_YOU_CONTROL_ENTERS, THIS_ENTERS};
use cantrip_mechanics::{discover, effects, library};
use cantrip_oracle as oracle;

/// "Exile cards from the top of your library until you exile a nonland card. You may cast it without paying its mana cost if
/// that spell's mana value is 8 or less. If you don't, put that card into your hand." The lands exiled on the way stay in exile;
/// the cast happens as the ability resolves (CR 608.2g), and the spell's mana value is that of the face it is cast as (CR 601.2b),
/// as Discover reads it (CR 701.57a). Solstice Revelations and Bre of Clan Stoutarm say the same with another limit.
fn exile_and_cast_or_take(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let (_, hit) = library::exile_until_nonland(g, you, None, &|_, _| true);
    let Some(card) = hit else { return Ok(()) };
    discover::cast_free_if_at_most_else_hand(g, you, card, 8)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            THIS_ENTERS,
            exile_and_cast_or_take,
            "When this enchantment enters, exile cards from the top of your library until you exile a nonland card. You may cast it without paying its mana cost if that spell's mana value is 8 or less. If you don't, put that card into your hand.",
        ),
        triggered(DRAGON_YOU_CONTROL_ENTERS, effects::return_this_to_hand, "When a Dragon you control enters, return this enchantment to its owner's hand."),
    ],
    ..oracle::BREACHING_DRAGONSTORM
};
