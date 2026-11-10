use cantrip_api::*;
use cantrip_mechanics::{delayed, destroy_target, targets};
use cantrip_oracle as oracle;

/// I: "Destroy up to one other target artifact or enchantment" (CR 701.8a): the chapter's target, if one was chosen and
/// it is still legal as the chapter resolves (CR 608.2b).
fn chapter_one(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    // II, III: "Exile up to one other target permanent you control. Return that card to the battlefield under its owner's
    // control at the beginning of the next end step" (CR 603.7): the card exiled is a new object when it returns (CR 400.7),
    // and nothing returns if no target was chosen or it left exile first - the shared slot-0 body.
    abilities: &[
        chapter(1, chapter_one, "I \u{2014} Destroy up to one other target artifact or enchantment.")
            .targets(&[targets::UP_TO_ONE_OTHER_ARTIFACT_OR_ENCHANTMENT]),
        chapter(2, delayed::exile_target_then_return_at_next_end_step_0, "II \u{2014} Exile up to one other target permanent you control. Return that card to the battlefield under its owner's control at the beginning of the next end step.")
            .targets(&[targets::UP_TO_ONE_ANOTHER_PERMANENT_YOU_CONTROL]),
        chapter(3, delayed::exile_target_then_return_at_next_end_step_0, "III \u{2014} Exile up to one other target permanent you control. Return that card to the battlefield under its owner's control at the beginning of the next end step.")
            .targets(&[targets::UP_TO_ONE_ANOTHER_PERMANENT_YOU_CONTROL]),
    ],
    delayed: &[delayed_trigger(
        delayed::return_subject_to_battlefield,
        "Return that card to the battlefield under its owner's control at the beginning of the next end step.",
    )],
    ..oracle::BALANCE_OF_TERROR
};
