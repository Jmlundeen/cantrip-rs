use cantrip_api::*;
use cantrip_mechanics::{effects::damage_each_opponent, triggers::NONCREATURE_SPELL_YOU_CAST};
use cantrip_oracle as oracle;

/// "This creature deals 2 damage to each opponent" (CR 120.3): one damage event to each, dealt at once (CR 120.2).
fn two_damage_to_each_opponent(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_each_opponent(g, item, 2);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        NONCREATURE_SPELL_YOU_CAST,
        two_damage_to_each_opponent,
        "Whenever you cast a noncreature spell, Black Waltz No. 3 deals 2 damage to each opponent.",
    )],
    ..oracle::BLACK_WALTZ_NO_3
};
