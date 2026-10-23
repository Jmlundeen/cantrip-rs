use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// "You gain 3 life and draw a card." (CR 119.3, 121.1.) The artifact was sacrificed as a cost.
fn gain_and_draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::gain_life(g, item, 3)?;
    effects::draw(g, item, 1)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, effects::scry_two, "When this artifact enters, scry 2."),
        ability(cost("{2}, {T}").and(SACRIFICE_THIS), gain_and_draw, "{2}, {T}, Sacrifice this artifact: You gain 3 life and draw a card."),
    ],
    ..oracle::CANDY_TRAIL
};
