use cantrip_api::*;
use cantrip_mechanics::{destroy_target, power_up, targets};
use cantrip_oracle as oracle;

/// "Destroy up to one target artifact or enchantment. Put a +1/+1 counter on She-Hulk." The counter is put on whether
/// or not anything was targeted or is still there (CR 608.2b).
fn destroy_and_counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    g.add_counters(item.object, CounterKind::PlusOne, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[power_up::ability(
        cost("{4}{G}{G}"),
        destroy_and_counter,
        "Power-up \u{2014} {4}{G}{G}: Destroy up to one target artifact or enchantment. Put a +1/+1 counter on She-Hulk. (Activate each power-up ability only once. Reduce the cost by her mana cost if she entered this turn.)",
    )
    .targets(&[targets::ARTIFACT_OR_ENCHANTMENT.optional()])],
    ..oracle::SHE_HULK_JADE_DEFENDER
};
