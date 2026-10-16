use cantrip_api::*;
use cantrip_mechanics::choose;
use cantrip_oracle as oracle;

/// Search your library and graveyard for five cards and exile the rest. Put the chosen cards on top of your library
/// in any order. You lose half your life, rounded up (CR 701.23, 401.4, 119.3).
///
/// One ordered choice over the library (bottom first, as the search lists it) and the graveyard together, not
/// revealed: exactly five, or all of them when there are fewer (CR 701.23d). The last picked ends on top, as with
/// Brainstorm. Everything else in both zones is exiled at once, the chosen graveyard cards move to the library, and
/// the five end on top in order; nothing is shuffled. A life total of zero or less has no half to lose (CR 107.1b).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let who = item.controller;
    let mut all: Vec<ObjectId> = g.state().zone(Zone::library(who)).to_vec();
    all.reverse();
    all.extend_from_slice(g.state().zone(Zone::graveyard(who)));
    // The library is searched whatever is in it or chosen.
    g.searched(who, who);
    if !all.is_empty() {
        let chosen = choose::cards(g, who, item.object, Purpose::PutOnTop, all.clone(), 5, false)?;
        let rest: Vec<ObjectId> = all.iter().copied().filter(|c| !chosen.contains(c)).collect();
        g.exile_all(&rest);

        let order: Vec<ObjectId> = chosen.iter().rev().copied().collect();
        let from_graveyard: Vec<ObjectId> = order.iter().copied().filter(|&c| g.obj(c).zone == Zone::graveyard(who)).collect();
        let moved = g.put_into_library(&from_graveyard, LibraryPlace::Top);
        let arranged: Vec<ObjectId> = order
            .iter()
            .map(|&c| match from_graveyard.iter().position(|&from| from == c) {
                Some(i) => moved.get(i).copied().unwrap_or(c),
                None => c,
            })
            .collect();
        g.arrange_library(who, &arranged, &[]);
    }
    let life = g.state().player(who).life;
    if life > 0 {
        g.lose_life(who, (life + 1) / 2);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::DOOMSDAY };
