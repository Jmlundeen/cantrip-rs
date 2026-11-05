use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// Target player mills two cards; if two cards that share a color were milled this way, repeat this process. Each pass
/// is one simultaneous mill of the top two cards (CR 701.13a), and every card milled counts wherever it went (CR 701.17c: Rest in
/// Peace exiles it, it is still milled): a pass with fewer than two cannot share a color, and colorless cards share none (CR 105.2a). The pass that
/// does not shorten the library stops the loop rather than spinning. A replacement may mill more than two (The Water
/// Crystal); any two of the cards milled that share a color repeat it (CR 701.17d).
fn grind(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(who) = item.target(0).player() else { return Ok(()) };
    loop {
        let library = g.state().zone(Zone::library(who)).to_vec();
        if library.is_empty() {
            return Ok(());
        }
        let milled = g.mill(who, 2);
        let shares = |i: usize, j: usize| g.chars(milled[i]).colors.intersects(g.chars(milled[j]).colors);
        if !(0..milled.len()).any(|i| (i + 1..milled.len()).any(|j| shares(i, j))) {
            return Ok(());
        }
        if g.state().zone(Zone::library(who)).len() >= library.len() {
            return Ok(());
        }
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{3}, {T}"), grind, "{3}, {T}: Target player mills two cards. If two cards that share a color were milled this way, repeat this process.")
        .targets(&[targets::PLAYER])],
    ..oracle::GRINDSTONE
};
