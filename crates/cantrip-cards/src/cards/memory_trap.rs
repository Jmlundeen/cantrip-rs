use cantrip_api::*;
use cantrip_mechanics::{targets, triggers};
use cantrip_oracle as oracle;

/// "Exile target nonland permanent an opponent controls until this enchantment leaves the battlefield." (CR 610.3b) The
/// card exiled this way returns as this enchantment leaves.
fn exile_until_leaves(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(permanent) = targets::object(g, item, 0) {
        g.exile_with(&[permanent], g.linked(item), true);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::THIS_ENTERS,
        exile_until_leaves,
        "When this enchantment enters, exile target nonland permanent an opponent controls until this enchantment leaves the battlefield.",
    )
    .targets(&[targets::NONLAND_PERMANENT_AN_OPPONENT_CONTROLS])],
    ..oracle::MEMORY_TRAP
};
