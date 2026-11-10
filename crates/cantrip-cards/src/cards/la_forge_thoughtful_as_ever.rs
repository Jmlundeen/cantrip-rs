use cantrip_api::*;
use cantrip_mechanics::{history, tokens};
use cantrip_oracle as oracle;

/// "For each permanent you control that you've put a counter on this turn, create a 1/1 blue Faerie creature token with
/// flying." (CR 122.6, 107.1b, 101.4): counted as the ability resolves, over the permanents its controller has on the
/// battlefield then - a counter of any kind that player put on it this turn, on the battlefield or as it entered, however
/// many and by whatever effect (CR 122.6). A permanent an opponent put counters on is not counted, nor one whose counters
/// came in some earlier turn, and a permanent that left and came back has had none put on it (CR 400.7). All the tokens are
/// made at once, as one event; none are made for a count of 0.
fn faeries(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let count = g
        .permanents_of(you)
        .into_iter()
        .filter(|&permanent| history::put_a_counter_on_this_turn(g.history(), you, permanent))
        .count() as u32;
    if count > 0 {
        tokens::create(g, you, &tokens::FAERIE, count)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        TAP,
        faeries,
        "{T}: For each permanent you control that you've put a counter on this turn, create a 1/1 blue Faerie creature token with flying.",
    )],
    ..oracle::LA_FORGE_THOUGHTFUL_AS_EVER
};

pub static EXTRA: &[&CardDef] = &[&tokens::FAERIE];
