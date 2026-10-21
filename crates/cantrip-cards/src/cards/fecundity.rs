use cantrip_api::*;
use cantrip_mechanics::triggers::CREATURE_DIES;
use cantrip_oracle as oracle;

const ASK: Question = Question::new("Draw", "draw a card?", "", "Draw", "Don't draw");

/// That creature's controller may draw a card; a player who has left the game is asked nothing, nor one who can't draw
/// (CR 121.3).
fn may_draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Player(who) = item.subject else { return Ok(()) };
    if !g.state().in_game(who) {
        return Ok(());
    }
    cantrip_mechanics::effects::may_draw(g, who, item.object, 1, &ASK)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(CREATURE_DIES, may_draw, "Whenever a creature dies, that creature's controller may draw a card.")
        .look_back()],
    ..oracle::FECUNDITY
};
