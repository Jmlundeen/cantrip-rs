use cantrip_api::*;
use cantrip_mechanics::{effects, flurry};
use cantrip_oracle as oracle;

/// "This creature deals 1 damage to each opponent" (CR 120.3, 102.2).
fn ping(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::damage_each_opponent(g, item, 1);
    Ok(())
}

/// Flurry (an ability word, CR 207.2c): the trigger counts the spells its controller cast this turn, those cast
/// before the Duelist entered included, so the Duelist cast as the second spell does not trigger itself.
pub static CARD: CardDef = CardDef {
    abilities: &[flurry::trigger("Whenever you cast your second spell each turn, this creature deals 1 damage to each opponent.", ping)],
    ..oracle::DEVOTED_DUELIST
};
