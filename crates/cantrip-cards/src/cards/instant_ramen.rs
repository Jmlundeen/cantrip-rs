use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, |g, item| effects::draw(g, item, 1), "When this artifact enters, draw a card."),
        ability(cost("{2}, {T}").and(SACRIFICE_THIS), |g, item| effects::gain_life(g, item, 3), "{2}, {T}, Sacrifice this artifact: You gain 3 life."),
    ],
    ..oracle::INSTANT_RAMEN
};
