use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{effects, pump, targets};
use cantrip_oracle as oracle;

/// Mode 1: "Target creature gets -X/-0 until end of turn, where X is the number of cards in your graveyard." X is counted as
/// the ability resolves (CR 608.2h) in its controller's graveyard (CR 109.5: "your"), and never the opponent's. The effect is
/// a fixed amount from then on (CR 611.2a). With the target illegal the core does not resolve the ability (CR 608.2b).
fn shrink_power(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        let x = g.state().zone(Zone::graveyard(item.controller)).len() as i16;
        pump(g, item.object, creature, -x, 0);
    }
    Ok(())
}

/// Mode 2: "Surveil 2." (CR 701.25).
fn surveil_two(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::surveil(g, item, 2)
}

static MODAL: ModalDef = modes(&[
    mode(
        shrink_power,
        "Target creature gets -X/-0 until end of turn, where X is the number of cards in your graveyard.",
    )
    .targets(&[targets::CREATURE]),
    mode(surveil_two, "Surveil 2."),
]);

pub static CARD: CardDef = CardDef {
    abilities: &[modal_triggered(
        THIS_ENTERS,
        &MODAL,
        "When Yuriko enters, choose one \u{2014} Target creature gets -X/-0 until end of turn, where X is the number of cards in your graveyard; or surveil 2.",
    )],
    ..oracle::YURIKO_HOPE_FROM_THE_SHADOWS
};
