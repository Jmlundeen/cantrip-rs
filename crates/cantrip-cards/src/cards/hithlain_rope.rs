//! Hithlain Rope. "This artifact can't be sacrificed" is the core's `Keyword::CantBeSacrificed` (CR 101.2, 701.21a): it pays no
//! sacrifice cost, and an instruction or a choice to sacrifice it can't take it.

use cantrip_api::*;
use cantrip_mechanics::lands::search_basic_land_to_battlefield;
use cantrip_oracle as oracle;

/// "The player to your right gains control of this artifact" (CR 103.1: turns go clockwise, to each player's left, so the
/// player to your right is the one before you in turn order; in a duel, the other player). Seats emptied by players who
/// have left the game are passed over (CR 800.4a). The new control has no end (CR 613.1b). Nothing happens once the Rope
/// is no longer the permanent that was activated (CR 400.7).
fn pass_to_the_right(g: &mut dyn Game, item: &StackItem) {
    let right = g.state().turn_order_from(item.controller).last().unwrap_or(item.controller);
    if g.is_on_battlefield(item.object) {
        g.gain_control(item.object, item.object, right, Duration::Indefinite);
    }
}

/// "Search your library for a basic land card, put it onto the battlefield tapped, then shuffle. The player to your right
/// gains control of this artifact."
fn search_then_pass(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    search_basic_land_to_battlefield(g, item.controller, item.object, true)?;
    pass_to_the_right(g, item);
    Ok(())
}

/// "Draw a card. The player to your right gains control of this artifact."
fn draw_then_pass(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 1);
    pass_to_the_right(g, item);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    keywords: Keywords::of(&[Keyword::CantBeSacrificed]),
    abilities: &[
        ability(
            cost("{1}, {T}"),
            search_then_pass,
            "{1}, {T}: Search your library for a basic land card, put it onto the battlefield tapped, then shuffle. The player to your right gains control of this artifact.",
        ),
        ability(cost("{2}, {T}"), draw_then_pass, "{2}, {T}: Draw a card. The player to your right gains control of this artifact."),
    ],
    ..oracle::HITHLAIN_ROPE
};
