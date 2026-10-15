use cantrip_api::*;
use cantrip_mechanics::hideaway;
use cantrip_mechanics::hideaway_play::play_exiled_card;
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers::this_enters;
use cantrip_oracle as oracle;

fn hide_a_card(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    hideaway(g, item.controller, item.object, 5)
}

/// How many different powers there are among the creatures this player controls.
fn different_powers(g: &dyn Game, player: PlayerId) -> usize {
    let mut powers: Vec<i16> = Vec::new();
    for id in g.permanents_of(player) {
        let c = g.chars(id);
        if c.is(CardType::Creature)
            && let Some(p) = c.power
            && !powers.contains(&p)
        {
            powers.push(p);
        }
    }
    powers.len()
}

/// Put a +1/+1 counter on target creature you control. Then, if you control three or more creatures with different powers (judged
/// after the counter, CR 608.2c), you may play the exiled card without paying its mana cost.
fn counter_then_play(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        g.add_counters(creature, CounterKind::PlusOne, 1);
    }
    if different_powers(g, item.controller) >= 3 {
        play_exiled_card(g, item.object, item.controller)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "Hideaway 5 (When this artifact enters, look at the top five cards of your library, exile one face down, then put the rest on the bottom in a random order.)",
            listens: EventMask::ZONE_CHANGE,
            matches: this_enters,
            effect: hide_a_card,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "{1}, {T}: Put a +1/+1 counter on target creature you control. Then if you control three or more creatures with different powers, you may play the exiled card without paying its mana cost.",
            cost: Cost { mana: mana(1, 0, 0, 0, 0, 0), tap: true, ..Cost::NONE },
            targets: &[targets::CREATURE_YOU_CONTROL],
            effect: counter_then_play,
            ..ActivatedDef::BLANK
        }),
    ],
    ..oracle::COLLECTORS_CAGE
};
