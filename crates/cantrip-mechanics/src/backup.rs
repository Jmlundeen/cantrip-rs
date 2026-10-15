//! Backup N (CR 702.165a): "When this creature enters, put N +1/+1 counters on target creature. If that's another
//! creature, it gains the following abilities until end of turn." Death Greeter's Champion and Guardian Scalelord print
//! it.
//!
//! A card with backup lists `trigger(text, effect)` among its `abilities`, `effect` calling `resolve`. The abilities
//! granted are the printed ones, fixed when backup triggers (CR 702.165d): the keywords it names, and, if it lists the
//! creature's own ability as a delayed ability, that one.

use crate::targets;
use crate::triggers::this_enters;
use cantrip_api::*;

/// The backup trigger: it enters, targeting a creature; `effect` resolves it with `resolve`.
pub const fn trigger(text: &'static str, effect: fn(&mut dyn Game, &StackItem) -> Result<(), Suspend>) -> AbilityDef {
    AbilityDef::Triggered(TriggeredDef {
        text,
        listens: EventMask::ZONE_CHANGE,
        matches: this_enters,
        targets: &[targets::CREATURE],
        effect,
        ..TriggeredDef::BLANK
    })
}

/// Put `n` +1/+1 counters on the target creature; if that is another creature, it gains `keywords` and, if given, the
/// ability the card lists as its delayed ability `ability`, until end of turn.
pub fn resolve(g: &mut dyn Game, item: &StackItem, n: i16, keywords: &[Keyword], ability: Option<u8>) -> Result<(), Suspend> {
    if g.target_legal(item, 0)
        && let Target::Object(creature) = item.targets[0]
    {
        g.add_counters(creature, CounterKind::PlusOne, n);
        if creature != item.object {
            g.add_effect(item.object, creature, EffectKind::AddKeywords(Keywords::of(keywords)), Duration::EndOfTurn);
            if let Some(index) = ability {
                let this = g.obj(item.object).card;
                g.add_effect(item.object, creature, EffectKind::AddAbility(AbilityRef::delayed(this, index)), Duration::EndOfTurn);
            }
        }
    }
    Ok(())
}
