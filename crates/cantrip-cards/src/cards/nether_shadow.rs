use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

/// The intervening condition is checked both at upkeep and on resolution (CR 603.4). Graveyards are ordered oldest first.
fn three_creatures_above(q: &dyn Query, asker: &Asker) -> bool {
    let cards = q.state().zone(Zone::graveyard(asker.controller));
    cards.iter().position(|&card| card == asker.source).is_some_and(|at| {
        cards[at + 1..].iter().filter(|&&card| q.chars(card).is(CardType::Creature)).count() >= 3
    })
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::YOUR_UPKEEP,
            effects::may_return_this_card_to_battlefield,
            "At the beginning of your upkeep, if this card is in your graveyard with three or more creature cards above it, you may put this card onto the battlefield.",
        )
        .condition(three_creatures_above)
        .functions_in(IN_A_GRAVEYARD),
    ],
    reads_graveyard_order: true,
    ..oracle::NETHER_SHADOW
};
