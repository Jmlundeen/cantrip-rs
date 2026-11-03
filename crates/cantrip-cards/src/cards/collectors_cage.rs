use cantrip_api::*;
use cantrip_mechanics::{coven, hideaway};
use cantrip_mechanics::hideaway_play::play_exiled_card;
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

fn hide_a_card(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    hideaway(g, item, 5)
}

/// Put a +1/+1 counter on target creature you control. Then, if you control three or more creatures with different powers (judged
/// after the counter, CR 608.2c), you may play the exiled card without paying its mana cost.
fn counter_then_play(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        g.add_counters(creature, CounterKind::PlusOne, 1);
    }
    if coven::you_control(g, item.controller) {
        play_exiled_card(g, item, item.controller)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, hide_a_card, "Hideaway 5 (When this artifact enters, look at the top five cards of your library, exile one face down, then put the rest on the bottom in a random order.)"),
        ability(cost("{1}, {T}"), counter_then_play, "{1}, {T}: Put a +1/+1 counter on target creature you control. Then if you control three or more creatures with different powers, you may play the exiled card without paying its mana cost.")
            .targets(&[targets::CREATURE_YOU_CONTROL]),
    ],
    ..oracle::COLLECTORS_CAGE
};
