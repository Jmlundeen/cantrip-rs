use cantrip_api::*;
use cantrip_mechanics::{effects, history::an_opponent_lost_life, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "Add {B}{R}": mana from a triggered ability, which uses the stack (CR 605.1b says it is not a mana ability) and
/// empties at the end of the step (CR 500.5).
fn add_black_and_red(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_mana(item.controller, Mana::B, 1);
    g.add_mana(item.controller, Mana::R, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, add_black_and_red, "When this creature enters, if an opponent lost life this turn, add {B}{R}.")
            .condition(an_opponent_lost_life),
        ability(cost("{1}{R}").and(discard_cards(1)), |g, item| effects::draw(g, item, 1), "{1}{R}, Discard a card: Draw a card."),
    ],
    ..oracle::FLAMECACHE_GECKO
};
