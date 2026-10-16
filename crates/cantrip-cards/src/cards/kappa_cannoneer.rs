use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers::{THIS_ENTERS, ward};
use cantrip_oracle as oracle;

/// "Whenever another artifact you control enters" (CR 603.6a): about something else entering under its controller.
fn another_artifact_enters(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { new, to, .. }
            if *new != source
                && to.is(ZoneKind::Battlefield)
                && q.controller(*new) == q.controller(source)
                && q.chars(*new).is(CardType::Artifact) =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

/// "Put a +1/+1 counter on this creature. It can't be blocked this turn."
fn grow(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if targets::permanent(g, item.object) {
        g.add_counters(item.object, CounterKind::PlusOne, 1);
        g.add_effect(item.object, item.object, EffectKind::AddKeywords(Keywords::of(&[Keyword::CantBeBlocked])), Duration::EndOfTurn);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ward::<4>("Ward {4} (Whenever this creature becomes the target of a spell or ability an opponent controls, counter it unless that player pays {4}.)"),
        triggered(THIS_ENTERS, grow, "Whenever Kappa Cannoneer enters, put a +1/+1 counter on it. It can't be blocked this turn."),
        triggered(when(EventMask::ZONE_CHANGE, another_artifact_enters), grow, "Whenever another artifact you control enters, put a +1/+1 counter on Kappa Cannoneer. It can't be blocked this turn."),
    ],
    ..oracle::KAPPA_CANNONEER
};
