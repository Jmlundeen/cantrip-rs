use cantrip_api::*;
use cantrip_mechanics::{exile_target_until_leaves, targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "Exile target creature an opponent controls until this Aura leaves the battlefield" (CR 610.3). With the Aura already gone
/// as the ability resolves, nothing is exiled (CR 610.3b).
fn exile_until_it_leaves(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    exile_target_until_leaves(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enchant: Some(targets::BASIC_LAND_YOU_CONTROL),
    abilities: &[triggered(
        THIS_ENTERS,
        exile_until_it_leaves,
        "When this Aura enters, exile target creature an opponent controls until this Aura leaves the battlefield.",
    )
    .targets(&[targets::CREATURE_AN_OPPONENT_CONTROLS])],
    ..oracle::DIMENSIONAL_EXILE
};
