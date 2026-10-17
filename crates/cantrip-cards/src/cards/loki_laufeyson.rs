use cantrip_api::*;
use cantrip_mechanics::{delayed, power_up, targets, triggers};
use cantrip_oracle as oracle;

/// "Loki's power": the power Loki has now, or the power it last had on the battlefield once it has left (CR 608.2h,
/// 113.7a); it may be zero or less.
fn lokis_power(q: &dyn Query, loki: ObjectId) -> i16 {
    if targets::permanent(q, loki) {
        q.chars(loki).power.unwrap_or(0)
    } else {
        q.state().last_known(loki).and_then(|k| k.power).unwrap_or(0)
    }
}

/// "When you next cast an instant or sorcery spell with mana value less than or equal to Loki's power this turn": a spell
/// cast, not copied (CR 707.10a), by the ability's controller (CR 603.7d), whose mana value as it stands on the stack, X
/// included (CR 202.3e), is no more than Loki's power at the moment it is cast. The spell is the trigger's subject.
fn next_instant_or_sorcery_within_lokis_power(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    let fired = triggers::you_cast_instant_or_sorcery(q, me, event)?;
    let spell = fired.subject.object()?;
    (i64::from(q.mana_value(spell)) <= i64::from(lokis_power(q, me.source))).then_some(fired)
}

/// "{1}, {T}: When you next cast ... this turn, copy that spell": the ability sets up a delayed triggered ability that
/// triggers once, on the next such spell this turn, and ends with the turn if none is cast (CR 603.7b-c). Each activation
/// sets up its own, so two would both copy the same next spell.
fn watch(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    delayed::next_this_turn(g, item, 0, item.object);
    Ok(())
}

/// "Copy that spell. You may choose new targets for the copy" (CR 707.10, 707.10c): copied from its last-known information
/// if it has left the stack (CR 707.10, 608.2h).
fn copy_that_spell(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let spell = item.subject.object().expect("the cast spell");
    g.copy_spell(spell, item.controller, true)?;
    Ok(())
}

/// "Put two +1/+1 counters on Loki."
fn two_counters(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_counters(item.object, CounterKind::PlusOne, 2);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(
            cost("{1}").and(TAP),
            watch,
            "{1}, {T}: When you next cast an instant or sorcery spell with mana value less than or equal to Loki's power this turn, copy that spell. You may choose new targets for the copy.",
        ),
        power_up::ability(
            cost("{4}{R}"),
            two_counters,
            "Power-up \u{2014} {4}{R}: Put two +1/+1 counters on Loki. (Activate each power-up ability only once. Reduce the cost by his mana cost if he entered this turn.)",
        ),
    ],
    delayed: &[triggered(
        when_asked(EventMask::SPELL_CAST, next_instant_or_sorcery_within_lokis_power),
        copy_that_spell,
        "When you next cast an instant or sorcery spell with mana value less than or equal to Loki's power this turn, copy that spell. You may choose new targets for the copy.",
    )],
    ..oracle::LOKI_LAUFEYSON
};
