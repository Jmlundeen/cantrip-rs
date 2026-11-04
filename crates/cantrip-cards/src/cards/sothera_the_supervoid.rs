use cantrip_api::*;
use cantrip_mechanics::triggers::{CREATURE_YOU_CONTROL_DIES, YOUR_END_STEP};
use cantrip_mechanics::{choose, creatures_you_control, select};
use cantrip_oracle as oracle;

/// "Each opponent chooses a creature they control and exiles it" (CR 101.4, 607.2a): in turn order from the active player each
/// opponent still in the game chooses, in view of the board as it is before anything is exiled (a player with one creature has no
/// choice, one with none exiles nothing), and then every chosen creature is exiled at once, linked to Sothera for her other
/// ability. Linked to the ability's source, which a trigger of Sothera dying with a creature still knows (CR 603.10a, 607.2a).
fn opponents_exile_a_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let opponents: Vec<PlayerId> = g.state().apnap().filter(|&p| p != item.controller).collect();
    let mut chosen = Vec::new();
    for who in opponents {
        let creatures: Vec<ObjectId> = g.permanents_of(who).into_iter().filter(|&id| g.chars(id).is(CardType::Creature)).collect();
        if creatures.len() <= 1 {
            chosen.extend(creatures);
        } else if let Some(pick) = choose::select(g, who, item.object, Purpose::Exile, creatures, select::creature_cards(select::exactly(1)))?.first() {
            chosen.push(*pick);
        }
    }
    g.exile_with(&chosen, g.linked(item), false);
    Ok(())
}

/// "If a player controls no creatures" (CR 603.4): any player still in the game, Sothera's controller or not. Checked as the
/// end step begins, and again as the ability resolves.
fn a_player_controls_no_creatures(q: &dyn Query, _: &Asker) -> bool {
    q.state().apnap().any(|p| creatures_you_control(q, p) == 0)
}

/// "Sacrifice Sothera, then put a creature card exiled with it onto the battlefield under your control with two additional
/// +1/+1 counters on it" (CR 701.21a, 607.2a, 122.6): the sacrifice comes first, and the creature cards exiled with this ability's
/// source are still its own (Memory Jar's way); the counters are on it as it enters, not put on afterwards. The card is chosen by Sothera's controller, none puts nothing. If Sothera could not be sacrificed (she has gone already) the second step is made all
/// the same: "then" is no "if you do".
fn sacrifice_then_put_an_exiled_creature_card(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let linked = g.linked(item);
    g.sacrifice(item.controller, &[item.object]);
    let eligible: Vec<ObjectId> = g.exiled_with(linked).into_iter().filter(|&id| g.chars(id).is(CardType::Creature)).collect();
    if let Some(&card) = choose::select(g, item.controller, item.object, Purpose::PutOntoBattlefield, eligible, select::creature_cards(select::exactly(1)))?.first() {
        g.put_onto_battlefield_with_counters(card, item.controller, false, &[(CounterKind::PlusOne, 2)])?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(CREATURE_YOU_CONTROL_DIES, opponents_exile_a_creature, "Whenever a creature you control dies, each opponent chooses a creature they control and exiles it."),
        triggered(
            YOUR_END_STEP,
            sacrifice_then_put_an_exiled_creature_card,
            "At the beginning of your end step, if a player controls no creatures, sacrifice Sothera, then put a creature card exiled with it onto the battlefield under your control with two additional +1/+1 counters on it.",
        )
        .condition(a_player_controls_no_creatures),
    ],
    ..oracle::SOTHERA_THE_SUPERVOID
};
