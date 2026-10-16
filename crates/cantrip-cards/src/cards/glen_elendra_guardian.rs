use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "This creature enters with a -1/-1 counter on it" (CR 614.1c).
fn enters_with_a_counter(_: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    entering.counters[CounterKind::MinusOne as usize] += 1;
    Ok(())
}

/// Counter target noncreature spell. Its controller draws a card: the controller is read while the spell is still on the
/// stack, and draws whether or not the spell could be countered, since the ability resolves as long as its target is
/// still legal (CR 608.2b).
fn counter_and_draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(spell) = targets::object(g, item, 0) else { return Ok(()) };
    let controller = g.controller(spell);
    g.counter(spell);
    g.draw(controller, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(enters_with_a_counter),
    abilities: &[ability(cost("{1}{U}").and(REMOVE_A_COUNTER), counter_and_draw, "{1}{U}, Remove a counter from this creature: Counter target noncreature spell. Its controller draws a card.")
        .targets(&[targets::NONCREATURE_SPELL])],
    ..oracle::GLEN_ELENDRA_GUARDIAN
};
