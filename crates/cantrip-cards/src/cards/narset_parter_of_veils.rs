use cantrip_api::*;
use cantrip_mechanics::look_for_noncreature_nonland;
use cantrip_mechanics::statics::opponents_cant_draw_more_than_one_card_each_turn;
use cantrip_oracle as oracle;

/// -2: Look at the top four cards of your library. You may reveal a noncreature, nonland card from among them and put
/// it into your hand. Put the rest on the bottom of your library in a random order. With no such card among them
/// nobody is asked.
fn look(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    look_for_noncreature_nonland(g, item.controller, item.object, 4)
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(loyalty(-2), look, "\u{2212}2: Look at the top four cards of your library. You may reveal a noncreature, nonland card from among them and put it into your hand. Put the rest on the bottom of your library in a random order.")],
    // "Each opponent can't draw more than one card each turn" (CR 121.2b): each draw past the first is not made, and no
    // replacement effect replaces it (CR 614.17c).
    cant_draw: &[opponents_cant_draw_more_than_one_card_each_turn],
    ..oracle::NARSET_PARTER_OF_VEILS
};
