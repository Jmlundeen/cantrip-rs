use cantrip_api::*;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

/// "Draw a card and you get {E}{E}." Energy counters belong to the player, and persist between steps (CR 121.1).
fn draw_and_energy(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 1);
    g.change_energy(item.controller, 2);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(triggers::THIS_ENTERS, draw_and_energy, "When this creature enters, draw a card and you get {E}{E}.")],
    ..oracle::ROGUE_REFINER
};
