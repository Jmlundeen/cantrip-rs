//! Earthbend N (CR 701.66a): "Target land you control becomes a 0/0 creature with haste that's still a land. Put N
//! +1/+1 counters on it. When it dies or is put into exile, return it to the battlefield tapped." Badgermole Cub prints
//! it with 1.
//!
//! A card that earthbends lists `enters::<N>(text)` (or `TARGET` and `effect::<N>` for a spell or another ability) and
//! `RETURN` as its first `delayed` ability: the land is given that ability, standing for the delayed trigger that
//! watches it.

use crate::triggers::THIS_ENTERS;
use cantrip_api::*;

/// "Target land you control."
pub const TARGET: TargetSpec = crate::targets::LAND_YOU_CONTROL;

/// The land becomes a 0/0 creature with haste that's still a land, with no duration (CR 611.2a); it gets `N` +1/+1
/// counters; and it is given `RETURN`, the delayed ability 0 of the card whose ability this is.
pub fn effect<const N: i16>(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if !g.target_legal(item, 0) {
        return Ok(());
    }
    let Target::Object(land) = item.targets[0] else { return Ok(()) };
    let o = *g.obj(item.object);
    let card = o.copied_card().unwrap_or(o.card);
    g.add_effect(item.object, land, EffectKind::AddTypes(TypeSet::of(&[CardType::Creature])), Duration::Indefinite);
    crate::effects::gain_haste(g, item.object, land);
    g.add_effect(item.object, land, EffectKind::SetPt(0, 0), Duration::Indefinite);
    g.add_effect(item.object, land, EffectKind::AddAbility(AbilityRef::delayed(card, 0)), Duration::Indefinite);
    g.add_counters(land, CounterKind::PlusOne, N);
    Ok(())
}

/// "When this enters, earthbend N": `text` names the card.
pub const fn enters<const N: i16>(text: &'static str) -> AbilityDef {
    triggered(THIS_ENTERS, effect::<N>, text)
        .targets(&[TARGET])
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

pub const RETURN: AbilityDef = triggered(when(EventMask::ZONE_CHANGE, dies_or_is_exiled), return_tapped, "When this land dies or is put into exile, return it to the battlefield tapped.")
    .look_back();
