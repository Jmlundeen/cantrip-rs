//! "Each player" and "each opponent" steps (CR 101.4): the players are listed in turn order as the step begins, and each is
//! reached in turn. A player who has left the game by the time the step reaches them - a concession while an earlier player
//! was choosing, or as they were about to be asked - is no longer one of "each player" (CR 800.4a): they choose nothing and
//! no one chooses for them. (CR 800.4g, another player making the choice, is for a choice an object asks of a particular
//! player, such as "target opponent chooses".)

use cantrip_api::*;

/// Whether an "each player" step that has come to `player` goes on with them: true while they are still in the game.
pub fn reached(g: &mut dyn Game, player: PlayerId) -> bool {
    g.still_in_game(player)
}
