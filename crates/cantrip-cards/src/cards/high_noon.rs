use cantrip_api::*;
use cantrip_mechanics::{damage_target, targets};
use cantrip_oracle as oracle;

/// "{4}{R}, Sacrifice this enchantment: It deals 5 damage to any target." The enchantment is the source of the damage
/// (last known information, CR 608.2h).
fn damage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 5);
    Ok(())
}

/// "Each player can't cast more than one spell each turn." (CR 101.2.)
pub static CARD: CardDef = CardDef {
    cast_limit: 1,
    abilities: &[ability(
        cost("{4}{R}").and(SACRIFICE_THIS),
        damage,
        "{4}{R}, Sacrifice High Noon: It deals 5 damage to any target.",
    )
    .targets(&[targets::ANY])],
    ..oracle::HIGH_NOON
};
