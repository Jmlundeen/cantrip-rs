use cantrip_api::*;
use cantrip_mechanics::{choose, players, select};
use cantrip_oracle as oracle;

/// "Each player may discard up to three cards. Mind Bomb deals damage to each player equal to 3 minus the number of cards they discarded this way." (CR 608.2, 120.3, 701.9a, 800.4a).
fn player_discard_up_three_cards(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let players: Vec<PlayerId> = g.state().apnap().collect();
    let mut chosen: Vec<ObjectId> = Vec::new();
    for p in players {
        if !players::reached(g, p) {
            continue;
        }
        let hand = g.state().zone(Zone::hand(p)).to_vec();
        chosen.extend(choose::select(g, p, item.object, Purpose::Discard, hand, select::cards(select::up_to(3)))?);
    }
    let discarded = g.discard_together(&chosen);
    g.end_sentence();
    let damage: Vec<(ObjectId, Target, u32)> = {
        let q: &dyn Query = g;
        q.state()
            .apnap()
            .map(|p| {
                let n = discarded.iter().filter(|&&o| q.state().obj(o).owner == p).count() as u32;
                (item.object, Target::Player(p), 3 - n)
            })
            .collect()
    };
    g.damage_together(&damage);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], player_discard_up_three_cards),
    ..oracle::MIND_BOMB
};
