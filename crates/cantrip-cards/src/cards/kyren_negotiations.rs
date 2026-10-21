use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_mechanics::TAP_AN_UNTAPPED_CREATURE;
use cantrip_oracle as oracle;

/// This enchantment deals 1 damage to target player or planeswalker.
fn ping(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if g.target_legal(item, 0) {
        g.damage(item.object, item.target(0), 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(TAP_AN_UNTAPPED_CREATURE, ping, "Tap an untapped creature you control: This enchantment deals 1 damage to target player or planeswalker.")
        .targets(&[targets::PLAYER_OR_PLANESWALKER])],
    ..oracle::KYREN_NEGOTIATIONS
};
