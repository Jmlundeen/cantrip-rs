use cantrip_api::*;
use cantrip_mechanics::{delayed, library, sacrifice_chosen, select, triggers};
use cantrip_oracle as oracle;

/// I: "Search your library for a basic Mountain card, put it onto the battlefield tapped, then shuffle." (CR 701.23): the
/// player finds one or none (CR 701.23b), and the library is shuffled either way. "Mountain" is a word of the Saga's text
/// (CR 612.2), which the selection rule reads.
fn chapter_one(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    library::find_to_battlefield(g, item.controller, item.object, select::BASIC_MOUNTAIN_CARD, true)?;
    Ok(())
}

/// II: "Whenever a creature you control enters this turn, put a +1/+1 counter on it and it gains haste until end of turn."
/// (CR 603.7b): one delayed ability for the rest of the turn, the card's number 0.
fn chapter_two(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    delayed::this_turn(g, item, item.object);
    Ok(())
}

/// The delayed ability itself: the counter and the haste are one instruction about the creature that entered, which is the
/// trigger's subject (CR 603.6a, 608.2h) - not a batch of `event_objects`, which a trigger that is not "one or more" does
/// not carry.
fn counter_and_haste(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Object(creature) = item.subject else { return Ok(()) };
    instruction(g, |g| {
        g.add_counters(creature, CounterKind::PlusOne, 1);
        cantrip_mechanics::haste_until_end_of_turn(g, item.object, creature);
    });
    Ok(())
}

/// III: "Sacrifice a land." (CR 701.21a): chosen and sacrificed as the chapter resolves; a player with no land sacrifices
/// nothing.
fn chapter_three(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    sacrifice_chosen(g, item.controller, item.object, 1, |c| c.is(CardType::Land))?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        chapter(1, chapter_one, "I \u{2014} Search your library for a basic Mountain card, put it onto the battlefield tapped, then shuffle."),
        chapter(2, chapter_two, "II \u{2014} Whenever a creature you control enters this turn, put a +1/+1 counter on it and it gains haste until end of turn."),
        chapter(3, chapter_three, "III \u{2014} Sacrifice a land."),
    ],
    delayed: &[triggered(
        triggers::CREATURE_YOU_CONTROL_ENTERS,
        counter_and_haste,
        "Whenever a creature you control enters this turn, put a +1/+1 counter on it and it gains haste until end of turn.",
    )],
    ..oracle::THE_NAKED_TIME
};
