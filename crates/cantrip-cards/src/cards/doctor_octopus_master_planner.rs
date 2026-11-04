use cantrip_api::*;
use cantrip_mechanics::statics::{other_of_subtype_you_control, plus_pt};
use cantrip_mechanics::{lands, triggers};
use cantrip_oracle as oracle;

/// The cards in the ability controller's hand, for "fewer than eight" (CR 603.4).
fn hand_size(q: &dyn Query, you: PlayerId) -> usize {
    q.state().zone(Zone::hand(you)).len()
}

/// "if you have fewer than eight cards in hand" (CR 603.4): true as the trigger fires and again as it resolves.
fn fewer_than_eight_cards_in_hand(q: &dyn Query, asker: &Asker) -> bool {
    hand_size(q, asker.controller) < 8
}

/// "draw cards equal to the difference" (CR 121.1): eight minus the cards in hand as the ability resolves; no card if the hand
/// has since reached eight.
fn draw_the_difference(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let difference = 8usize.saturating_sub(hand_size(g, item.controller)) as u32;
    g.draw(item.controller, difference);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    statics: &[static_ability(
        Layer::PtModify,
        other_of_subtype_you_control::<{ lands::sub(Subtype::Villain) }>,
        plus_pt::<2, 2>,
        "Other Villains you control get +2/+2.",
    )],
    play_rules: &[PlayRule::MaximumHandSize(Some(8))],
    abilities: &[triggered(
        triggers::YOUR_END_STEP,
        draw_the_difference,
        "At the beginning of your end step, if you have fewer than eight cards in hand, draw cards equal to the difference.",
    )
    .condition(fewer_than_eight_cards_in_hand)],
    ..oracle::DOCTOR_OCTOPUS_MASTER_PLANNER
};
