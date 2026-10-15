//! Earthbend N (CR 701.66a): "Target land you control becomes a 0/0 creature with haste that's still a land. Put N
//! +1/+1 counters on it. When it dies or is put into exile, return it to the battlefield tapped." Badgermole Cub prints
//! it with 1.
//!
//! A card that earthbends lists `enters::<N>(text)` (or `TARGET` and `effect::<N>` for a spell or another ability) and
//! `RETURN` as its first `delayed` ability: the land is given that ability, standing for the delayed trigger that
//! watches it.

use crate::triggers::this_enters;
use cantrip_api::*;

fn land_you_control(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if q.is_on_battlefield(id) && q.obj(id).became.is_none()
        && q.controller(id) == asker.controller && q.chars(id).is(CardType::Land))
}

/// "Target land you control."
pub const TARGET: TargetSpec = TargetSpec { min: 1, max: 1, filter: land_you_control };

/// The land becomes a 0/0 creature with haste that's still a land, with no duration (CR 611.2a); it gets `N` +1/+1
/// counters; and it is given `RETURN`, the delayed ability 0 of the card whose ability this is.
pub fn effect<const N: i16>(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if !g.target_legal(item, 0) {
        return Ok(());
    }
    let Target::Object(land) = item.targets[0] else { return Ok(()) };
    let o = *g.obj(item.object);
    let card = o.copy.map(|c| c.card).unwrap_or(o.card);
    g.add_effect(item.object, land, EffectKind::AddTypes(TypeSet::of(&[CardType::Creature])), Duration::Indefinite);
    g.add_effect(item.object, land, EffectKind::AddKeywords(Keywords::of(&[Keyword::Haste])), Duration::Indefinite);
    g.add_effect(item.object, land, EffectKind::SetPt(0, 0), Duration::Indefinite);
    g.add_effect(item.object, land, EffectKind::AddAbility(AbilityRef::delayed(card, 0)), Duration::Indefinite);
    g.add_counters(land, CounterKind::PlusOne, N);
    Ok(())
}

/// "When this enters, earthbend N": `text` names the card.
pub const fn enters<const N: i16>(text: &'static str) -> AbilityDef {
    AbilityDef::Triggered(TriggeredDef {
        text,
        listens: EventMask::ZONE_CHANGE,
        matches: this_enters,
        targets: &[TARGET],
        effect: effect::<N>,
        ..TriggeredDef::BLANK
    })
}

/// "When this dies or is put into exile" (CR 700.4): leaving the battlefield for a graveyard or exile, judged as it was.
fn dies_or_is_exiled(_: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { old, from, to, .. }
            if *old == source && from.is(ZoneKind::Battlefield) && (to.is(ZoneKind::Graveyard) || to.is(ZoneKind::Exile)) =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

/// Return it to the battlefield tapped, if it is still where it went.
fn return_tapped(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(card) = g.found_after_move(item.object) {
        g.put_onto_battlefield(card, item.controller, true)?;
    }
    Ok(())
}

pub const RETURN: AbilityDef = AbilityDef::Triggered(TriggeredDef {
    text: "When this land dies or is put into exile, return it to the battlefield tapped.",
    listens: EventMask::ZONE_CHANGE,
    matches: dies_or_is_exiled,
    effect: return_tapped,
    look_back: true,
    ..TriggeredDef::BLANK
});
