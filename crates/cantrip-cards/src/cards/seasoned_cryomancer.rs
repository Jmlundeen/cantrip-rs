use cantrip_api::*;
use cantrip_mechanics::{choose_from_hand, effects, targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// Draw two cards, then discard two cards (all of a shorter hand) as one action (CR 701.9). When one or more of the
/// cards discarded this way are nonland cards, the reflexive ability triggers (CR 603.12), counting them: its "that
/// many" is the number of nonland cards among the discarded ones, read where they went (a card's types are the same
/// in the graveyard, or in exile by madness, as in the hand).
fn enter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    g.draw(you, 2);
    let chosen = choose_from_hand(g, you, item.object, 2)?;
    let discarded = g.discard_together(&chosen);
    let nonlands = discarded.iter().filter(|&&card| !g.chars(card).is(CardType::Land)).count();
    if nonlands > 0 {
        let this = g.card_id(&CARD);
        g.reflexive_trigger_counted(item.object, you, AbilityRef::delayed(this, 0), Target::None, u16::try_from(nonlands).unwrap_or(u16::MAX));
    }
    Ok(())
}

/// Tap up to that many target creatures and put a stun counter on each of them: each target still legal as it
/// resolves (CR 608.2b), an already tapped one included.
fn tap_and_stun(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let creatures = targets::legal_objects(g, item);
    for &creature in &creatures {
        g.tap(creature);
    }
    for &creature in &creatures {
        g.add_counters(creature, CounterKind::Stun, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, enter, "When this creature enters, draw two cards, then discard two cards. When you discard one or more nonland cards this way, tap up to that many target creatures and put a stun counter on each of them."),
        ability(cost("{3}{U}{U}").and(EXILE_THIS), |g, item| effects::draw(g, item, 2), "{3}{U}{U}, Exile this card from your graveyard: Draw two cards.")
            .from(ZoneKind::Graveyard),
    ],
    delayed: &[delayed_trigger(tap_and_stun, "When you discard one or more nonland cards this way, tap up to that many target creatures and put a stun counter on each of them.")
        .targets(&[targets::CREATURE.between(0, TargetSpec::COUNTED)])],
    ..oracle::SEASONED_CRYOMANCER
};
