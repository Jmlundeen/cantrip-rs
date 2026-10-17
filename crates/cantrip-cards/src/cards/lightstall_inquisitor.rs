use cantrip_api::*;
use cantrip_mechanics::choose;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// "When this creature enters, each opponent exiles a card from their hand and may play that card for as long as it
/// remains exiled. Each spell cast this way costs {1} more to cast. Each land played this way enters tapped."
///
/// Each opponent, in turn order from the active player (CR 101.4), chooses a card of their own hand (CR 608.2d); the
/// chosen cards are then exiled together. Each is its owner's to play for as long as it stays in exile (CR 611.2, 400.7):
/// a spell for its costs and {1} more (CR 601.2f), a land as their land play, at the times the rules allow (CR 305.1,
/// 305.2, ruling 2025-07-25), entering tapped (CR 614.1c).
fn each_opponent_exiles(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let opponents: Vec<PlayerId> = g.state().apnap().filter(|&p| p != item.controller).collect();
    let mut chosen = Vec::new();
    for opponent in opponents {
        let hand = g.state().zone(Zone::hand(opponent)).to_vec();
        chosen.extend(choose::card(g, opponent, item.object, Purpose::Exile, hand)?);
    }
    // A card a replacement sent elsewhere than exile (CR 614.1a) was not exiled, and is no card "that remains exiled".
    for moved in g.exile_all(&chosen) {
        if g.obj(moved).zone.is(ZoneKind::Exile) {
            let owner = g.obj(moved).owner;
            g.allow_play(moved, owner, PlayGrant { lands: true, more: 1, enters_tapped: true, ..PlayGrant::CAST });
        }
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ENTERS,
        each_opponent_exiles,
        "When this creature enters, each opponent exiles a card from their hand and may play that card for as long as it remains exiled. Each spell cast this way costs {1} more to cast. Each land played this way enters tapped.",
    )],
    ..oracle::LIGHTSTALL_INQUISITOR
};
