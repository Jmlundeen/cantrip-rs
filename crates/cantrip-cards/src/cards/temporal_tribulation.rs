use cantrip_api::*;
use cantrip_mechanics::{delayed, flashback, targets};
use cantrip_oracle as oracle;

/// "Exile two target permanents you control. Return those cards to the battlefield under their owner's control at the
/// beginning of the next end step." (CR 701.13, 603.7, 400.7): exactly two targets are announced (CR 601.2c) and the shared
/// body exiles them together and sets up the delayed return; both cards return as new objects (CR 400.7).
pub static CARD: CardDef = CardDef {
    spell: spell(
        &[targets::PERMANENT_YOU_CONTROL.between(2, 2)],
        delayed::exile_targets_then_return_at_next_end_step_0,
    ),
    ways: &[flashback::way(
        cost("{5}{U}"),
        "Flashback {5}{U} (You may cast this card from your graveyard for its flashback cost. Then exile it.)",
    )],
    delayed: &[delayed_trigger(
        delayed::return_exiled_to_battlefield,
        "Return those cards to the battlefield under their owner's control at the beginning of the next end step.",
    )],
    ..oracle::TEMPORAL_TRIBULATION
};
