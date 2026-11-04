use cantrip_api::*;
use cantrip_mechanics::{library::{look_at_top_select, Rest}, select};
use cantrip_mechanics::tokens;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// "Look at the top five cards of your library. You may reveal up to two creature cards from among them and put them into your
/// hand. Put the rest on the bottom of your library in a random order." (CR 401.4, 701.20): the five are shown and chosen
/// from even with no creature card among them, and none need be taken.
fn look(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    look_at_top_select(g, item.controller, item.object, 5, select::creature_cards(select::up_to(2)), true, Rest::BottomRandom)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            THIS_ENTERS,
            look,
            "When this artifact enters, look at the top five cards of your library. You may reveal up to two creature cards from among them and put them into your hand. Put the rest on the bottom of your library in a random order.",
        ),
        ability(
            cost("{1}, {T}").and(SACRIFICE_THIS),
            tokens::create_a_treasure,
            "{1}, {T}, Sacrifice this artifact: Create a Treasure token.",
        ),
    ],
    ..oracle::PICTURES_OF_SPIDER_MAN
};

pub static EXTRA: &[&CardDef] = &[&tokens::TREASURE];
