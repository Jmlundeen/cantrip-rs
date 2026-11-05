use cantrip_api::*;
use cantrip_mechanics::{targets, tokens, triggers, your_turn};
use cantrip_oracle as oracle;

/// "Whenever a card leaves your graveyard during your turn" (CR 603.2, 603.10a): cards leaving the source's controller's graveyard
/// (`triggers::cards_leave_your_graveyard`), while it is that controller's turn.
fn cards_leave_your_graveyard_during_your_turn(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    your_turn(q, me).then(|| triggers::cards_leave_your_graveyard(q, me, event)).flatten()
}

/// "Put two +1/+1 counters and a flying counter on target creature" (CR 122.1b): the counters go on together, and nothing
/// happens if the target is no longer legal (CR 608.2b).
fn renew(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    g.add_counters_together(&[(creature, CounterKind::PlusOne, 2), (creature, CounterKind::Flying, 1)]);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    // Flying is a core keyword read from the card's JSON. "Whenever one or more cards leave your graveyard during your turn"
    // (CR 603.2c): cards that left together trigger it once. Renew is an ability word (CR 207.2c): an ordinary activated
    // ability of the card in the graveyard.
    abilities: &[
        triggered(
            when_asked(EventMask::ZONE_CHANGE, cards_leave_your_graveyard_during_your_turn),
            |g, item| tokens::create_treasure(g, item.controller),
            "Whenever one or more cards leave your graveyard during your turn, create a Treasure token.",
        )
        .once_per_batch(),
        ability(cost("{2}{B}{G}{U}").and(EXILE_THIS), renew, "Renew \u{2014} {2}{B}{G}{U}, Exile this card from your graveyard: Put two +1/+1 counters and a flying counter on target creature. Activate only as a sorcery.")
            .from(ZoneKind::Graveyard)
            .sorcery_speed()
            .targets(&[targets::CREATURE]),
    ],
    ..oracle::KHERU_GOLDKEEPER
};
