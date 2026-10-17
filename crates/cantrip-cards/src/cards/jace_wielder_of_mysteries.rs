use cantrip_api::*;
use cantrip_mechanics::replacements::WIN_INSTEAD_OF_DRAWING_FROM_AN_EMPTY_LIBRARY;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// +1: "Target player mills two cards. Draw a card."
fn plus_one(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if g.target_legal(item, 0)
        && let Some(player) = item.targets[0].player()
    {
        g.mill(player, 2);
    }
    g.draw(item.controller, 1);
    Ok(())
}

/// -8: "Draw seven cards. Then if your library has no cards in it, you win the game."
fn minus_eight(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 7);
    if g.state().zone(Zone::library(item.controller)).is_empty() {
        g.win_game(item.controller);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(loyalty(1), plus_one, "+1: Target player mills two cards. Draw a card.")
            .targets(&[targets::PLAYER]),
        ability(loyalty(-8), minus_eight, "\u{2212}8: Draw seven cards. Then if your library has no cards in it, you win the game."),
    ],
    // "If you would draw a card while your library has no cards in it, you win the game instead."
    replacements: &[WIN_INSTEAD_OF_DRAWING_FROM_AN_EMPTY_LIBRARY],
    ..oracle::JACE_WIELDER_OF_MYSTERIES
};
