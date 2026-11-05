use cantrip_api::*;
use cantrip_mechanics::crew::crew;
use cantrip_mechanics::library;
use cantrip_mechanics::triggers::THIS_ATTACKS;
use cantrip_oracle as oracle;

/// "Whenever The Regalia attacks, reveal cards from the top of your library until you reveal a land card. Put that card onto the
/// battlefield tapped and the rest on the bottom of your library in a random order" (CR 701.20, 401.4): the cards are revealed
/// together down to the first land card, or the whole library when it holds none (then nothing enters); the land enters tapped
/// and every other revealed card goes to the bottom together.
fn reveal_until_a_land(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let (lands, others) = library::reveal_until_n(g, you, 1, &|q, card| q.chars(card).is(CardType::Land));
    if let Some(&land) = lands.first() {
        g.put_onto_battlefield(land, you, true)?;
    }
    library::rest_to_bottom_in_random_order(g, you, &others);
    Ok(())
}

/// Haste is a core keyword.
pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            THIS_ATTACKS,
            reveal_until_a_land,
            "Whenever The Regalia attacks, reveal cards from the top of your library until you reveal a land card. Put that card onto the battlefield tapped and the rest on the bottom of your library in a random order.",
        ),
        crew(1, "Crew 1 (Tap any number of creatures you control with total power 1 or more: This Vehicle becomes an artifact creature until end of turn.)"),
    ],
    ..oracle::THE_REGALIA
};
