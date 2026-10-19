use cantrip_api::*;
use cantrip_mechanics::{targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "Return up to one other target creature you control to its owner's hand." A target that became illegal is not
/// returned (CR 608.2b).
fn bounce(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        g.return_to_hand(&[creature]);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, bounce, "When this creature enters, return up to one other target creature you control to its owner's hand.")
        .targets(&[up_to_one(targets::other_creature_you_control_filter)])],
    ..oracle::FLOCK_IMPOSTOR
};
