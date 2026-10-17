use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// Mode 1: "Tap target permanent an opponent controls" (CR 701.26a). Nothing if it is no longer a legal target (CR 608.2b).
fn tap_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(permanent) = targets::object(g, item, 0) {
        g.tap(permanent);
    }
    Ok(())
}

/// "Choose one" as the trigger goes on the stack (CR 603.3c, 700.2a), with the chosen mode's target (CR 603.3d). Mode 0,
/// "Untap target permanent you control" (CR 701.26b), untaps it if it is still legal (`effects::untap_targets`).
static MODAL: ModalDef = modes(&[
    mode(effects::untap_targets, "Untap target permanent you control.").targets(&[targets::PERMANENT_YOU_CONTROL]),
    mode(tap_it, "Tap target permanent an opponent controls.").targets(&[targets::PERMANENT_AN_OPPONENT_CONTROLS]),
]);

pub static CARD: CardDef = CardDef {
    abilities: &[modal_triggered(THIS_ENTERS, &MODAL, "When this creature enters, choose one \u{2014}\n\u{2022} Untap target permanent you control.\n\u{2022} Tap target permanent an opponent controls.")],
    ..oracle::DECEIVER_EXARCH
};
