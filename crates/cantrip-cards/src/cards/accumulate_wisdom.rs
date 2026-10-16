use cantrip_api::*;
use cantrip_mechanics::library::{look_at_top, Rest};
use cantrip_oracle as oracle;

/// "Look at the top three cards of your library. Put one of those cards into your hand and the rest on the bottom of your
/// library in any order. Put each of those cards into your hand instead if there are three or more Lesson cards in your
/// graveyard." The graveyard is read as this resolves, so Accumulate Wisdom itself, still on the stack, is not one of the
/// Lessons (CR 608.2c). A library of fewer than three cards shows what it has (CR 609.3), and all of them go to the hand
/// when the count is met.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    let lessons = g.state().zone(Zone::graveyard(player)).iter().filter(|&&c| g.chars(c).has_subtype(Subtype::Lesson)).count();
    let taken = if lessons >= 3 { 3 } else { 1 };
    look_at_top(g, player, item.object, 3, (taken, taken), false, |_| true, Rest::BottomChosen)?;
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::ACCUMULATE_WISDOM };
