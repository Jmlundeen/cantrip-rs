use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

/// "This creature deals 1 damage to each opponent" (CR 120.3).
fn damage_each_opponent(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::damage_each_opponent(g, item, 1);
    Ok(())
}

/// Reach (CR 702.17, a core keyword) and "Landfall \u{2014} Whenever a land you control enters, this creature deals 1 damage to
/// each opponent" (CR 603.2, 120.3): a land of its controller's entering, played or put onto the battlefield.
pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::LANDFALL,
        damage_each_opponent,
        "Landfall \u{2014} Whenever a land you control enters, this creature deals 1 damage to each opponent.",
    )],
    ..oracle::SABOTENDER
};
