use cantrip_api::*;
use cantrip_oracle as oracle;

fn nonland(q: &dyn Query, _: &Asker, id: ObjectId) -> bool {
    !q.chars(id).is(CardType::Land)
}

/// Each opponent loses 10 life.
fn drain(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.each_opponent_loses_life(item.controller, 10);
    Ok(())
}

// "You may look at the top card of your library any time" (CR 401.2) is what a client shows its controller; the rules
// part is the permission to play from it, which is `PlayRule::LibraryTop`.
pub static CARD: CardDef = CardDef {
    abilities: &[ability(TAP.and(sacrifice(10, nonland)), drain, "{T}, Sacrifice ten nonland permanents: Each opponent loses 10 life.")],
    play_rules: &[PlayRule::LibraryTop],
    ..oracle::BOLASS_CITADEL
};
