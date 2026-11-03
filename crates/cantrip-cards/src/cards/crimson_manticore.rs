use cantrip_api::*;
use cantrip_mechanics::{damage_target, targets};
use cantrip_oracle as oracle;

/// This creature deals 1 damage to target attacking or blocking creature (CR 608.2b, 120.3).
fn damage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(cost("{R}").and(TAP), damage, "{R}, {T}: This creature deals 1 damage to target attacking or blocking creature.")
            .targets(&[targets::ATTACKING_OR_BLOCKING_CREATURE]),
    ],
    ..oracle::CRIMSON_MANTICORE
};
