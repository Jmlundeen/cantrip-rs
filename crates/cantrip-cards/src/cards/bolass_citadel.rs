use cantrip_api::*;
use cantrip_mechanics::targets::filters::nonland;
use cantrip_oracle as oracle;

/// Each opponent loses 10 life.
fn drain(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.each_opponent_loses_life(item.controller, 10);
    Ok(())
}

// "You may look at the top card of your library any time" (CR 401.2), which the host shows its controller, and the
// permission to play from it, for life.
pub static CARD: CardDef = CardDef {
    abilities: &[ability(TAP.and(sacrifice(10, nonland)), drain, "{T}, Sacrifice ten nonland permanents: Each opponent loses 10 life.")],
    play_rules: &[PlayRule::LookAtLibraryTop, PlayRule::LibraryTop],
    ..oracle::BOLASS_CITADEL
};
