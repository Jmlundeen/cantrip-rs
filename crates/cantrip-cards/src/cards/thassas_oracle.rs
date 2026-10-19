use cantrip_api::*;
use cantrip_mechanics::{top_of_library, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// Mana symbols of `color` (blue as the Oracle prints it, or what a text change made it) in a permanent's mana cost (CR 700.5): hybrid ({U/R}, {2/U}) and Phyrexian blue symbols each count once. A copy
/// counts the cost it copies (CR 707.2), none if it has none (CR 707.9b).
fn symbols_of(q: &dyn Query, id: ObjectId, color: Color) -> u32 {
    let mana = color.mana();
    let values = q.copiable_values(id);
    if values.no_cost {
        return 0;
    }
    let cost = q.card(values.card).cost;
    cost.symbols[mana as usize] as u32
        + cost.hybrid.iter().flatten().filter(|(a, b)| *a == mana || *b == mana).count() as u32
        + cost.phyrexian[mana as usize] as u32
        + cost.twobrid[mana as usize] as u32
}

/// "Look at the top X cards of your library, where X is your devotion to blue. Put up to one of them on top of your
/// library and the rest on the bottom of your library in a random order. If X is greater than or equal to the number of
/// cards in your library, you win the game."
fn look_and_win(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let color = g.words(item.object).color(Color::Blue);
    let devotion: u32 = g.permanents_of(you).into_iter().map(|id| symbols_of(g, id, color)).sum();
    let looked = top_of_library(g, you, devotion as usize);
    if !looked.is_empty() {
        let ask = RequestKind::DistributeCards {
            source: item.object,
            cards: looked.clone(),
            destinations: vec![
                Destination { kind: DestinationKind::LibraryTop, min: 0, max: 1, eligible: looked.clone() },
                Destination { kind: DestinationKind::LibraryBottom, min: 0, max: u8::MAX, eligible: looked },
            ],
            may_shuffle: false,
        };
        if let Answer::Placements { placements, .. } = g.ask(you, ask)? {
            g.arrange_library(you, &placements[0], &[]);
            g.bottom_in_random_order(you, &placements[1]);
        }
    }
    if devotion as usize >= g.state().zone(Zone::library(you)).len() {
        g.win_game(you);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, look_and_win, "When this creature enters, look at the top X cards of your library, where X is your devotion to blue. Put up to one of them on top of your library and the rest on the bottom of your library in a random order. If X is greater than or equal to the number of cards in your library, you win the game.")],
    ..oracle::THASSAS_ORACLE
};
