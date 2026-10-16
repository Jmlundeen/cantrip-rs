use cantrip_api::*;
use cantrip_mechanics::{behold, counter_unless_pays, targets};
use cantrip_oracle as oracle;

/// "Counter target spell unless its controller pays {2}. If a Dragon was beheld, counter that spell unless its
/// controller pays {4} instead." (CR 118.12, 701.6.) A copy of the spell has the cost paid when the original did
/// (CR 707.10).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let ransom = if behold::beheld(item) { mana(4, 0, 0, 0, 0, 0) } else { mana(2, 0, 0, 0, 0, 0) };
    counter_unless_pays(g, item, 0, ransom)?;
    Ok(())
}

/// "As an additional cost to cast this spell, you may behold a Dragon" (CR 701.4a, 601.2b): an optional cost announced
/// before the target, which is chosen after it. Caustic Exhale, Osseous Exhale and Piercing Exhale also behold a Dragon.
pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[targets::SPELL], effect }),
    late_targets: true,
    optional_cost: Some(behold::optional(behold::dragon)),
    labels: &["Cast without beholding", "Behold a Dragon: choose a Dragon you control or reveal one from your hand"],
    ..oracle::DISPELLING_EXHALE
};
