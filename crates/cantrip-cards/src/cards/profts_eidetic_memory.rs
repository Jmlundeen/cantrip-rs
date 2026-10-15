use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::{history::drawn_this_turn, targets, triggers};
use cantrip_oracle as oracle;

/// The intervening "if you've drawn more than one card this turn" (CR 603.4): asked as it triggers and as it resolves.
fn drawn_more_than_one(q: &dyn Query, asker: &Asker) -> bool {
    drawn_this_turn(q, asker.controller) > 1
}

/// Put X +1/+1 counters on target creature you control, X being the cards you've drawn this turn minus one, counted as the
/// ability resolves (CR 608.2h).
fn grow(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let x = drawn_this_turn(g, item.controller) as i16 - 1;
    if x > 0
        && let Some(creature) = targets::object(g, item, 0)
    {
        g.add_counters(creature, CounterKind::PlusOne, x);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "When Proft's Eidetic Memory enters, draw a card.",
            listens: EventMask::ZONE_CHANGE,
            matches: triggers::this_enters,
            effect: effects::draw::<1>,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "At the beginning of combat on your turn, if you've drawn more than one card this turn, put X +1/+1 counters on target creature you control, where X is the number of cards you've drawn this turn minus one.",
            listens: EventMask::STEP_BEGAN,
            matches: triggers::beginning_of_your_combat,
            condition: Some(drawn_more_than_one),
            targets: &[targets::CREATURE_YOU_CONTROL],
            effect: grow,
            ..TriggeredDef::BLANK
        }),
    ],
    play_rules: &[PlayRule::NoMaximumHandSize],
    ..oracle::PROFTS_EIDETIC_MEMORY
};
