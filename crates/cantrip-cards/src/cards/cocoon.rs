use cantrip_api::*;
use cantrip_mechanics::{attach, targets, triggers};
use cantrip_oracle as oracle;

/// Tap the enchanted creature and put three pupa counters on the Aura (CR 701.26a, 122.1).
fn enter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    attach::tap_attached(g, item);
    g.add_counters(item.object, CounterKind::Pupa, 3);
    Ok(())
}

/// The enchanted creature does not untap while this Aura has a pupa counter (CR 502.3).
fn no_untap(q: &dyn Query, asker: &Asker, id: ObjectId) -> bool {
    q.obj(asker.source).counters(CounterKind::Pupa) > 0 && attach::attached_to_it(q, asker, id)
}

/// Remove one pupa counter; if none can be removed, sacrifice the Aura and hatch the enchanted creature (CR 113.7a).
fn upkeep(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let removed = g.remove_counters(item.object, CounterKind::Pupa, 1);
    g.end_sentence();
    if removed == 0 {
        g.sacrifice(item.controller, &[item.object]);
        if let Some(creature) = g.obj(item.object).attached_to {
            g.add_counters(creature, CounterKind::PlusOne, 1);
            g.add_effect(item.object, creature, EffectKind::AddKeywords(Keywords::of(&[Keyword::Flying])), Duration::Indefinite);
        }
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::THIS_ENTERS, enter, "When this Aura enters, tap enchanted creature and put three pupa counters on this Aura."),
        triggered(triggers::YOUR_UPKEEP, upkeep, "At the beginning of your upkeep, remove a pupa counter from this Aura. If you can't, sacrifice it, put a +1/+1 counter on enchanted creature, and that creature gains flying."),
    ],
    untap_bans: &[no_untap],
    enchant: Some(targets::CREATURE_YOU_CONTROL),
    ..oracle::COCOON
};
