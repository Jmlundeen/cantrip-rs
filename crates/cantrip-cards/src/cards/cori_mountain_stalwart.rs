use cantrip_api::*;
use cantrip_mechanics::{effects, flurry};
use cantrip_oracle as oracle;

/// "This creature deals 2 damage to each opponent and you gain 2 life." Both happen even if it has left the battlefield by
/// the time the trigger resolves (CR 113.7a, 608.2h).
fn ping_and_gain(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::damage_each_opponent_and_gain_life(g, item, 2)
}

pub static CARD: CardDef = CardDef {
    abilities: &[flurry::trigger("Flurry \u{2014} Whenever you cast your second spell each turn, this creature deals 2 damage to each opponent and you gain 2 life.", ping_and_gain)],
    ..oracle::CORI_MOUNTAIN_STALWART
};
