use cantrip_api::*;
use cantrip_mechanics::effects::ask_may;
use cantrip_mechanics::{library, targets};
use cantrip_oracle as oracle;

/// "Whenever a card is put into your graveyard from anywhere" (CR 603.2): any zone change into the graveyard of
/// this enchantment's controller ("your" is its controller, CR 109.5). "From anywhere" is no leaves-the-battlefield
/// trigger (CR 603.6c), so Quest, sacrificed, does not see itself arrive. Each card of a group put there at once is its own event and its
/// own trigger (CR 603.2c). A token is no card (CR 111.7), so it does not count.
fn card_put_into_your_graveyard(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { new, to, .. }
            if *to == Zone::graveyard(me.controller) && !q.obj(*new).token =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

const PUT_COUNTER: Question = Question::new("QuestCounter", "put a quest counter on it?", "", "Put a counter", "Don't");

/// "You may put a quest counter on this enchantment" (CR 122.1): asked as the trigger resolves; an enchantment that has
/// left the battlefield gets nothing, the core does not put a counter on it.
fn quest_counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if ask_may(g, item.controller, item.object, Purpose::Ask(&PUT_COUNTER))? {
        g.add_counters(item.object, CounterKind::Quest, 1);
    }
    Ok(())
}

/// "Target player shuffles their graveyard into their library" (CR 701.24a). The cost, five counters and the sacrifice,
/// was paid on activation, so the enchantment itself is in its owner's graveyard and is shuffled in when it is the target
/// player's. Nothing happens for a target that is no longer legal (CR 608.2b).
fn shuffle_graveyard(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(player) = targets::player(g, item, 0) {
        library::shuffle_graveyard_into_library(g, player);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            when_asked(EventMask::ZONE_CHANGE, card_put_into_your_graveyard),
            quest_counter,
            "Whenever a card is put into your graveyard from anywhere, you may put a quest counter on this enchantment.",
        ),
        ability(
            remove_counters(CounterKind::Quest, 5).and(SACRIFICE_THIS),
            shuffle_graveyard,
            "Remove five quest counters from this enchantment and sacrifice it: Target player shuffles their graveyard into their library.",
        )
        .targets(&[targets::PLAYER]),
    ],
    ..oracle::QUEST_FOR_ANCIENT_SECRETS
};
