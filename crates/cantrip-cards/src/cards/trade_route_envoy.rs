use cantrip_api::*;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

/// "Draw a card if you control a creature with a counter on it. If you don't draw a card this way, put a +1/+1 counter on this
/// creature" (CR 608.2c, 121.1): the condition is judged as the ability resolves, over creatures its controller controls, this one
/// included. A card was drawn this way if the controller's count of draws this turn went up, so a draw that a replacement effect
/// (dredge, CR 702.52a) took the place of did not draw. A counter for a creature no longer on the battlefield goes nowhere (CR 400.7).
fn draw_or_counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    let drawn_before = g.history().drawn(player);
    let controls_a_creature_with_a_counter = g
        .state()
        .battlefield()
        .iter()
        .any(|&id| g.controller(id) == player && g.obj(id).has_counters() && g.chars(id).is(CardType::Creature));
    if controls_a_creature_with_a_counter {
        g.draw(player, 1);
    }
    if g.history().drawn(player) == drawn_before {
        g.add_counters(item.object, CounterKind::PlusOne, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::THIS_ENTERS,
        draw_or_counter,
        "When this creature enters, draw a card if you control a creature with a counter on it. If you don't draw a card this way, put a +1/+1 counter on this creature.",
    )],
    ..oracle::TRADE_ROUTE_ENVOY
};
