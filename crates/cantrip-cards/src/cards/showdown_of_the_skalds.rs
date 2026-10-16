use cantrip_api::*;
use cantrip_mechanics::{library::exile_top_may_play, targets};
use cantrip_oracle as oracle;

/// Chapter I: "Exile the top four cards of your library. Until the end of your next turn, you may play those cards."
/// The four leave together, face up (CR 101.4); each that reaches exile may be played, cast or as the land for the
/// turn (CR 305.1), until the cleanup step of its controller's next turn (CR 514.2). A library of fewer exiles what it has.
fn exile_four(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let grant = PlayGrant { instead: None, lands: true, expiry: Expiry::EndOfYourNextTurn, ..PlayGrant::CAST };
    exile_top_may_play(g, item.controller, 4, item.controller, grant);
    Ok(())
}

/// Chapters II and III: each makes its own delayed ability for the turn (CR 603.7b), so two chapters in one turn
/// trigger twice for each spell.
fn rally(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let this = g.card_id(&CARD);
    g.delayed_trigger(DelayedWhen::ThisTurn, item.object, item.controller, AbilityRef::delayed(this, 0), Target::None);
    Ok(())
}

/// "Whenever you cast a spell": a spell, not an activated ability, cast by this ability's controller.
fn you_cast_a_spell(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::SpellCast { player, .. } if *player == q.controller(source) => Fired::PLAIN,
        _ => None,
    }
}

fn counter_on_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        g.add_counters(creature, CounterKind::PlusOne, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        chapter(1, exile_four, "I \u{2014} Exile the top four cards of your library. Until the end of your next turn, you may play those cards."),
        chapter(2, rally, "II \u{2014} Whenever you cast a spell this turn, put a +1/+1 counter on target creature you control."),
        chapter(3, rally, "III \u{2014} Whenever you cast a spell this turn, put a +1/+1 counter on target creature you control."),
    ],
    delayed: &[triggered(when(EventMask::SPELL_CAST, you_cast_a_spell), counter_on_creature, "Whenever you cast a spell this turn, put a +1/+1 counter on target creature you control.")
        .targets(&[targets::CREATURE_YOU_CONTROL])],
    ..oracle::SHOWDOWN_OF_THE_SKALDS
};
