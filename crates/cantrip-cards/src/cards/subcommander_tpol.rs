use cantrip_api::*;
use cantrip_mechanics::{counter_target_ability, targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "Counter target activated or triggered ability" (CR 701.6): a mana ability is never on the stack to be targeted
/// (CR 605.3b).
fn counter_ability(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    counter_target_ability(g, item, 0);
    Ok(())
}

/// "Tap up to one target artifact or creature. Put a stun counter on it" (CR 701.26a, 122.1d): the counter is put even on
/// an artifact that is no creature, and on one already tapped.
fn tap_and_stun(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(permanent) = targets::object(g, item, 0) {
        g.tap(permanent);
        g.add_counters(permanent, CounterKind::Stun, 1);
    }
    Ok(())
}

/// "Choose one —" (CR 700.2).
static MODAL: ModalDef = modes(&[
    mode(counter_ability, "Counter target activated or triggered ability.")
        .targets(&[targets::ACTIVATED_OR_TRIGGERED_ABILITY]),
    mode(tap_and_stun, "Tap up to one target artifact or creature. Put a stun counter on it.")
        .targets(&[up_to_one(targets::artifact_or_creature_filter)]),
]);

pub static CARD: CardDef = CardDef {
    abilities: &[modal_triggered(
        THIS_ENTERS,
        &MODAL,
        "When Subcommander T'Pol enters, choose one \u{2014}\n\u{2022} Counter target activated or triggered ability.\n\u{2022} Tap up to one target artifact or creature. Put a stun counter on it.",
    )],
    ..oracle::SUBCOMMANDER_TPOL
};
