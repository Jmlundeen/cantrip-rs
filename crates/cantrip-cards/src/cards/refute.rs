use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Counter target spell" (CR 701.5a): the targeted spell is removed from the stack and put into its owner's graveyard. Any
/// spell is a legal target, the caster's own included (CR 115.1a), and no target means nothing is countered (CR 608.2b).
/// The draw follows the counter, so a Refute whose target became illegal does nothing at all.
fn counter_then_loot(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::counter_first_target(g, item)?;
    effects::loot(g, item)
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::SPELL], counter_then_loot),
    ..oracle::REFUTE
};
