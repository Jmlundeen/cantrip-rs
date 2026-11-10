use cantrip_api::*;
use cantrip_mechanics::triggers::LANDFALL;
use cantrip_mechanics::{delayed, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    // "Landfall — Whenever a land you control enters, exile up to one other target creature you control. Return that card to
    // the battlefield under its owner's control at the beginning of the next end step." (CR 603.7c, 400.7): the delayed
    // ability is about the card it became in exile, so a token exiled this way has ceased to exist and nothing returns
    // (CR 111.7).
    abilities: &[triggered(
        LANDFALL,
        delayed::exile_target_then_return_at_next_end_step_0,
        "Landfall \u{2014} Whenever a land you control enters, exile up to one other target creature you control. Return that card to the battlefield under its owner's control at the beginning of the next end step.",
    )
    .targets(&[targets::OTHER_CREATURE_YOU_CONTROL.optional()])],
    delayed: &[delayed_trigger(
        delayed::return_subject_to_battlefield,
        "Return that card to the battlefield under its owner's control at the beginning of the next end step.",
    )],
    ..oracle::CHIEF_KYLE_TRANSPORTER_TECH
};
