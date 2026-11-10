use cantrip_api::*;
use cantrip_mechanics::{commander, effects};
use cantrip_oracle as oracle;

/// "Sacrifice this artifact: Draw a card." (CR 701.21a, 121.1): the sacrifice is the whole cost, with no tap.
fn sacrifice_and_draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::draw(g, item, 1)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        commander::MANA,
        ability(SACRIFICE_THIS, sacrifice_and_draw, "Sacrifice this artifact: Draw a card."),
    ],
    ..oracle::COMMANDERS_SPHERE
};
