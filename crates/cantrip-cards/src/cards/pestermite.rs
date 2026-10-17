use cantrip_api::*;
use cantrip_mechanics::{choose, targets, triggers};
use cantrip_oracle as oracle;

/// "When this creature enters, you may tap or untap target permanent." The target is chosen as the trigger goes on the stack
/// (CR 603.3d), any permanent, and as it resolves its controller chooses Tap, Untap or Do nothing (`choose::may_tap_or_untap_target`,
/// CR 608.2d, 701.26); nothing if the target is gone (CR 608.2b).
fn tap_or_untap(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let card = g.card_id(&CARD);
    choose::may_tap_or_untap_target(g, item, card)
}

pub static CARD: CardDef = CardDef {
    labels: choose::TAP_OR_UNTAP,
    abilities: &[triggered(triggers::THIS_ENTERS, tap_or_untap, "When this creature enters, you may tap or untap target permanent.").targets(&[targets::PERMANENT])],
    ..oracle::PESTERMITE
};
