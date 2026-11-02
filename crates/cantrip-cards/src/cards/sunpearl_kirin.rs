use cantrip_api::*;
use cantrip_mechanics::{targets, triggers};
use cantrip_oracle as oracle;

/// "When this creature enters, return up to one other target nonland permanent you control to its owner's hand. If it was a token, draw a card." (CR 603.1, 608.2, 111.7).
fn return_up_one_other_target(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(permanent) = targets::object(g, item, 0) else { return Ok(()) };
    let was_token = g.obj(permanent).token;
    g.return_to_hand(&[permanent]);
    g.end_sentence();
    if was_token {
        g.draw(item.controller, 1);
        g.end_sentence();
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::THIS_ENTERS,
        return_up_one_other_target,
        "When this creature enters, return up to one other target nonland permanent you control to its owner's hand. If it was a token, draw a card.",
    )
    .targets(&[up_to_one(targets::other_nonland_permanent_you_control_filter)])],
    ..oracle::SUNPEARL_KIRIN
};
