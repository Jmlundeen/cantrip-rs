use cantrip_api::*;
use cantrip_mechanics::{top_of_library, triggers::this_enters};
use cantrip_oracle as oracle;

/// Blue mana symbols in a permanent's mana cost (CR 700.5): hybrid and Phyrexian blue symbols each count once. A copy
/// counts the cost it copies (CR 707.2), none if it has none (CR 707.9b).
fn blue_symbols(q: &dyn Query, id: ObjectId) -> u32 {
    let values = q.copiable_values(id);
    if values.no_cost {
        return 0;
    }
    let cost = q.card(values.card).cost;
    cost.symbols[Mana::U as usize] as u32
        + cost.hybrid.iter().flatten().filter(|(a, b)| *a == Mana::U || *b == Mana::U).count() as u32
        + cost.phyrexian[Mana::U as usize] as u32
}

/// "Look at the top X cards of your library, where X is your devotion to blue. Put up to one of them on top of your
/// library and the rest on the bottom of your library in a random order. If X is greater than or equal to the number of
/// cards in your library, you win the game."
fn look_and_win(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let devotion: u32 = g.permanents_of(you).into_iter().map(|id| blue_symbols(g, id)).sum();
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
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "When this creature enters, look at the top X cards of your library, where X is your devotion to blue. Put up to one of them on top of your library and the rest on the bottom of your library in a random order. If X is greater than or equal to the number of cards in your library, you win the game.",
        listens: EventMask::ZONE_CHANGE,
        matches: this_enters,
        effect: look_and_win,
        ..TriggeredDef::BLANK
    })],
    ..oracle::THASSAS_ORACLE
};
