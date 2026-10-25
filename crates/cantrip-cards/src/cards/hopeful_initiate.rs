use cantrip_api::*;
use cantrip_mechanics::{destroy_target, targets, training};
use cantrip_oracle as oracle;

/// "Destroy target artifact or enchantment." (CR 701.8)
fn destroy(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        training::TRIGGER,
        ability(
            // Only +1/+1 counters pay it, from among creatures you control (CR 118.3, 122.1).
            cost("{2}{W}").and(remove_kind_counters_among(CounterKind::PlusOne, 2, targets::filters::creature)),
            destroy,
            "{2}{W}, Remove two +1/+1 counters from among creatures you control: Destroy target artifact or enchantment.",
        )
        .targets(&[targets::ARTIFACT_OR_ENCHANTMENT]),
    ],
    ..oracle::HOPEFUL_INITIATE
};
