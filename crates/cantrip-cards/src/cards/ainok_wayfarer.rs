use cantrip_api::*;
use cantrip_mechanics::library;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// "Mill three cards. You may put a land card from among them into your hand. If you don't, put a +1/+1 counter on this
/// creature." (CR 701.17, 608.2c): no land to take, or declining, puts nothing in the hand and so gives the counter.
fn mill_and_take_or_grow(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let taken = library::mill_then_may_take_into_hand(g, item.controller, item.object, 3, "land", |c| c.is(CardType::Land))?;
    if taken.is_empty() {
        g.add_counters(item.object, CounterKind::PlusOne, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ENTERS,
        mill_and_take_or_grow,
        "When this creature enters, mill three cards. You may put a land card from among them into your hand. If you don't, put a +1/+1 counter on this creature.",
    )],
    ..oracle::AINOK_WAYFARER
};
