use cantrip_api::*;
use cantrip_mechanics::{targets, tokens};
use cantrip_oracle as oracle;

/// "Return target nonland permanent to its owner's hand. If it was attacking, create a 1/1 colorless Drone artifact creature token with
/// flying and 'This token can block only creatures with flying.'" (CR 400.3, 506.3, 608.2b): whether it was attacking is read as the
/// spell resolves, before it leaves combat (CR 506.4); a target that became illegal does nothing (CR 608.2b).
fn return_it_and_drone_if_it_was_attacking(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(permanent) = targets::object(g, item, 0) else { return Ok(()) };
    let was_attacking = g.is_attacking(permanent);
    g.return_to_hand(&[permanent]);
    if was_attacking {
        tokens::create(g, item.controller, &tokens::DRONE, 1)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::NONLAND_PERMANENT], return_it_and_drone_if_it_was_attacking),
    ..oracle::DESCULPTING_BLAST
};

pub static EXTRA: &[&CardDef] = &[&tokens::DRONE];
