use cantrip_api::*;
use cantrip_mechanics::{effects, scry, triggers};
use cantrip_oracle as oracle;

/// "Whenever a creature you control enters, scry 1 and put a plan counter on this enchantment." (CR 701.22, 122.6) The
/// counter goes only on the enchantment that triggered, if it is still there (CR 400.7).
fn scry_and_plan(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    scry(g, item.controller, item.object, 1)?;
    g.add_counters(item.object, CounterKind::Plan, 1);
    Ok(())
}

/// "When the fourth plan counter is put on this enchantment, sacrifice it, draw a card, and put a +1/+1 counter on each
/// creature you control." (CR 701.21a) Each part is done whether or not the enchantment is still there to be sacrificed.
fn triumph(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.sacrifice(&[item.object]);
    g.draw(item.controller, 1);
    effects::counters_on_creatures_you_control(g, item.controller, CounterKind::PlusOne, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::CREATURE_YOU_CONTROL_ENTERS, scry_and_plan, "Whenever a creature you control enters, scry 1 and put a plan counter on this enchantment."),
        triggered(
            when(EventMask::COUNTERS, triggers::nth_counter_put_on_this::<{ CounterKind::Plan as u8 }, 4>),
            triumph,
            "When the fourth plan counter is put on this enchantment, sacrifice it, draw a card, and put a +1/+1 counter on each creature you control.",
        ),
    ],
    ..oracle::POLITICAL_TRIUMPH
};
