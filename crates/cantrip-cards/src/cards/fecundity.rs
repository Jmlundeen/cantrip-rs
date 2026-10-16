use cantrip_api::*;
use cantrip_mechanics::triggers::CREATURE_DIES;
use cantrip_oracle as oracle;

/// That creature's controller may draw a card; a player who has left the game is asked nothing.
fn may_draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Player(who) = item.subject else { return Ok(()) };
    if !g.state().in_game(who) {
        return Ok(());
    }
    if cantrip_mechanics::effects::ask_may(g, who, item.object, Purpose::Optional)? {
        g.draw(who, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(CREATURE_DIES, may_draw, "Whenever a creature dies, that creature's controller may draw a card.")
        .look_back()],
    ..oracle::FECUNDITY
};
