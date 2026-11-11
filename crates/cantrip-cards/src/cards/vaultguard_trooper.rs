use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

const ASK: Question = Question::new("Discard", "discard your hand?", "If you do, draw two cards", "Discard, draw two", "Keep your hand");

/// "You may discard your hand. If you do, draw two cards" (CR 608.2c-d, 701.9): the controller is asked as it resolves; an empty hand
/// may be discarded too, and that counts as having done it.
fn may_discard_hand_to_draw_two(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if effects::may_discard_hand(g, item, &ASK)?.is_some() {
        g.draw(item.controller, 2);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::YOUR_END_STEP,
        may_discard_hand_to_draw_two,
        "At the beginning of your end step, if you control two or more tapped creatures, you may discard your hand. If you do, draw two cards.",
    )
    .condition(triggers::you_control_two_or_more_tapped_creatures)],
    ..oracle::VAULTGUARD_TROOPER
};
