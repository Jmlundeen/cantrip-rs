use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, effects::scry_two, "When this artifact enters, scry 2."),
        ability(cost("{2}, {T}").and(SACRIFICE_THIS), effects::gain_life_then_draw::<3, 1>, "{2}, {T}, Sacrifice this artifact: You gain 3 life and draw a card."),
    ],
    ..oracle::CANDY_TRAIL
};
