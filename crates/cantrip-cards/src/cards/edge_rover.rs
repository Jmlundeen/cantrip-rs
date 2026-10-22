use cantrip_api::*;
use cantrip_mechanics::tokens::{self, LANDER};
use cantrip_mechanics::triggers::THIS_DIES;
use cantrip_oracle as oracle;

/// "When this creature dies, each player creates a Lander token": each player in turn order starting with the active
/// player (CR 101.4) makes one under their own control; a player who has left the game is not "each player" (CR 800.4a).
fn each_player_creates_a_lander(g: &mut dyn Game, _item: &StackItem) -> Result<(), Suspend> {
    let players: Vec<PlayerId> = g.state().apnap().collect();
    for player in players {
        tokens::create_lander(g, player)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_DIES,
        each_player_creates_a_lander,
        "When this creature dies, each player creates a Lander token.",
    )],
    ..oracle::EDGE_ROVER
};

pub static EXTRA: &[&CardDef] = &[&LANDER];
