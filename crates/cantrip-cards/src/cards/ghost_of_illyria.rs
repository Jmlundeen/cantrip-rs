use cantrip_api::*;
use cantrip_mechanics::{delayed, targets, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    // "When this creature enters, exile up to one other target nonland permanent you control. Return that card to the
    // battlefield under its owner's control at the beginning of the next end step." The exile and the delayed ability that
    // brings it back are one effect (CR 603.7c): the card returns under its owner's control, whoever controls it by then, and
    // a token exiled this way has ceased to exist and stays gone (CR 111.7). The shared slot-0 body.
    abilities: &[triggered(
        triggers::THIS_ENTERS,
        delayed::exile_target_then_return_at_next_end_step_0,
        "When this creature enters, exile up to one other target nonland permanent you control. Return that card to the battlefield under its owner's control at the beginning of the next end step.",
    )
    .targets(&[target(targets::other_nonland_permanent_you_control_filter).optional()])],
    delayed: &[delayed_trigger(
        delayed::return_subject_to_battlefield,
        "Return that card to the battlefield under its owner's control at the beginning of the next end step.",
    )],
    ..oracle::GHOST_OF_ILLYRIA
};
