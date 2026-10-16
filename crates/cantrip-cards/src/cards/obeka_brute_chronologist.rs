use cantrip_api::*;
use cantrip_mechanics::effects::ask_may;
use cantrip_oracle as oracle;

/// "The player whose turn it is may end the turn." That player, whoever controls Obeka, chooses as the ability resolves
/// (CR 608.2d); if they do, the turn ends (CR 724.1) and the ability is exiled with everything else on the stack (724.1b).
fn may_end_the_turn(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let active = g.state().turn.active;
    if ask_may(g, active, item.object, Purpose::EndTurn)? {
        g.end_the_turn()?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(TAP, may_end_the_turn, "{T}: The player whose turn it is may end the turn.")],
    ..oracle::OBEKA_BRUTE_CHRONOLOGIST
};
