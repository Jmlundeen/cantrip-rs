use cantrip_api::*;
use cantrip_mechanics::{behold, effects, targets};
use cantrip_oracle as oracle;

/// "This spell deals 5 damage to target attacking or blocking creature. If a Dragon was beheld, you gain 2 life." (CR 120.3,
/// 701.4a.) The second sentence is followed after the first (CR 608.2c). When the target has become illegal the spell does not
/// resolve at all (CR 608.2b), so nothing is gained.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::damage_first_target::<5>(g, item)?;
    if behold::beheld(item) {
        effects::gain_life(g, item, 2)?;
    }
    Ok(())
}

/// "As an additional cost to cast this spell, you may behold a Dragon" (CR 701.4a, 601.2b): an optional cost announced before
/// the target, which is chosen after it. Caustic Exhale, Dispelling Exhale and Piercing Exhale also behold a Dragon.
pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::ATTACKING_OR_BLOCKING_CREATURE], effect),
    late_targets: true,
    optional_cost: Some(behold::optional(behold::dragon, "Dragon")),
    labels: &["Cast without beholding", "Behold a Dragon: choose a Dragon you control or reveal one from your hand"],
    ..oracle::OSSEOUS_EXHALE
};
