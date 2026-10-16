use cantrip_api::*;
use cantrip_mechanics::{damage_target, effects, targets};
use cantrip_oracle as oracle;

/// "Broadside Barrage deals 5 damage to target creature or planeswalker. Draw a card, then discard a card." The spell
/// deals the damage (CR 120.3); its only target gone, it doesn't resolve at all (CR 608.2b) and nothing is drawn.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 5);
    effects::loot(g, item)
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE_OR_PLANESWALKER], effect),
    ..oracle::BROADSIDE_BARRAGE
};
