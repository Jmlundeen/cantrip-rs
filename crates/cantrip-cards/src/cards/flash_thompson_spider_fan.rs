use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Choose one or both" as the trigger goes on the stack (CR 603.3c, 700.2), each chosen mode with its own target (CR 603.3d,
/// 700.2c); the modes are carried out in the order printed (CR 608.2c). Heckle taps its target (CR 701.26a) and Hero Worship
/// untaps its target (CR 701.26b), each only if still legal as it resolves (CR 608.2b).
static MODAL: ModalDef = modes(&[
    mode(effects::tap_targets, "Heckle \u{2014} Tap target creature.").targets(&[targets::CREATURE]),
    mode(effects::untap_targets, "Hero Worship \u{2014} Untap target creature.").targets(&[targets::CREATURE]),
])
.choose(1, 2);

// Flash is the core's, read from the card's JSON.
pub static CARD: CardDef = CardDef {
    abilities: &[modal_triggered(
        THIS_ENTERS,
        &MODAL,
        "When Flash Thompson enters, choose one or both \u{2014}\n\u{2022} Heckle \u{2014} Tap target creature.\n\u{2022} Hero Worship \u{2014} Untap target creature.",
    )],
    ..oracle::FLASH_THOMPSON_SPIDER_FAN
};
