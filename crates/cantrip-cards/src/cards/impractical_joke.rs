use cantrip_api::*;
use cantrip_mechanics::damage_target;
use cantrip_oracle as oracle;

/// "Damage can't be prevented this turn" happens whether or not a target was chosen (CR 615.12); the damage only to
/// a target still legal. A spell whose every chosen target is illegal does not resolve at all (CR 608.2b): the core.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.damage_cant_be_prevented();
    if item.target_count > 0 {
        damage_target(g, item, 0, 3);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[cantrip_mechanics::targets::CREATURE_OR_PLANESWALKER.between(0, 1)], effect),
    ..oracle::IMPRACTICAL_JOKE
};
