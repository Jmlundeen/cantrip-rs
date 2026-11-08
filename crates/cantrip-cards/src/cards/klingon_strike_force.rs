use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

/// "When this creature dies, it deals 2 damage to each opponent" (CR 700.4, 603.10a, 120.3): one damage event for each
/// opponent, dealt together by the resolving ability.
fn two_damage_to_each_opponent(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    Ok(effects::damage_each_opponent(g, item, 2))
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::THIS_DIES,
        two_damage_to_each_opponent,
        "When this creature dies, it deals 2 damage to each opponent.",
    )],
    ..oracle::KLINGON_STRIKE_FORCE
};
