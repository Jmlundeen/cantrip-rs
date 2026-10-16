use cantrip_api::*;
use cantrip_mechanics::{targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "Return two target lands you control to their owner's hand." Exactly two are chosen (CR 601.2c, 603.3d: with fewer than two
/// legal targets the trigger is removed from the stack); those still legal as it resolves are returned together (CR 608.2b), each
/// to its owner's hand.
fn return_lands(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let lands: Vec<ObjectId> = (0..item.target_count as usize).filter_map(|slot| targets::object(g, item, slot)).collect();
    g.return_to_hand(&lands);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, return_lands, "When this creature enters, return two target lands you control to their owner's hand.")
        .targets(&[targets::LAND_YOU_CONTROL.between(2, 2)])],
    ..oracle::SEA_DRAKE
};
