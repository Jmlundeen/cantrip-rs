use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "This creature deals 1 damage to each opponent": one damage event to each, dealt at once (CR 120.2), by this creature as it last
/// was if it has left the battlefield (CR 113.7a), so lifelink still gains the life (CR 702.15b).
fn ping_each_opponent(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::damage_each_opponent(g, item, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{3}{B}"), ping_each_opponent, "{3}{B}: This creature deals 1 damage to each opponent.")],
    ..oracle::MERCILESS_ENFORCERS
};
