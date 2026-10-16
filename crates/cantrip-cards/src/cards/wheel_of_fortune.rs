use cantrip_api::*;
use cantrip_oracle as oracle;

/// Each player discards their hand, then draws seven cards. Every player, in turn order from the active player
/// (CR 101.4); nobody is asked, a whole hand leaving nothing to choose.
fn effect(g: &mut dyn Game, _: &StackItem) -> Result<(), Suspend> {
    let players: Vec<PlayerId> = g.state().apnap().collect();
    let hands: Vec<ObjectId> = players.iter().flat_map(|&p| g.state().zone(Zone::hand(p)).to_vec()).collect();
    g.discard_together(&hands);
    for p in players {
        g.draw(p, 7);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::WHEEL_OF_FORTUNE };
