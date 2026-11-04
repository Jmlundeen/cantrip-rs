use cantrip_api::*;
use cantrip_mechanics::{effects, targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "Target player draws two cards, then discards two cards" (CR 121.1, 701.9): the target player does both.
fn target_player_loots_two(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::target_player_draws_then_discards(g, item, 0, 2)
}

/// "Choose two" (CR 700.2): each mode has its own target (CR 700.2c), done in printed order (CR 700.2d); a mode whose
/// target is gone is left alone and the other still does its part (CR 608.2b).
static MODAL: ModalDef = modes(&[
    mode(
        effects::counter_first_target_unless_pays::<2>,
        "Counter target noncreature spell unless its controller pays {2}.",
    )
    .targets(&[targets::NONCREATURE_SPELL]),
    mode(
        effects::return_first_target_to_hand,
        "Return target creature with mana value 2 or less to its owner's hand.",
    )
    .targets(&[target(targets::creature_with_mana_value_at_most::<2>)]),
    mode(
        effects::target_creature_cant_be_blocked_this_turn,
        "Target creature can't be blocked this turn.",
    )
    .targets(&[targets::CREATURE]),
    mode(
        target_player_loots_two,
        "Target player draws two cards, then discards two cards.",
    )
    .targets(&[targets::PLAYER]),
])
.choose(2, 2);

pub static CARD: CardDef = CardDef {
    abilities: &[modal_triggered(
        THIS_ENTERS,
        &MODAL,
        "When Wesley Crusher enters, choose two \u{2014}\n\u{2022} Counter target noncreature spell unless its controller pays {2}.\n\u{2022} Return target creature with mana value 2 or less to its owner's hand.\n\u{2022} Target creature can't be blocked this turn.\n\u{2022} Target player draws two cards, then discards two cards.",
    )],
    ..oracle::WESLEY_CRUSHER_STARFLEET_CADET
};
