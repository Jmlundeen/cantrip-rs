use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// A non-Lemur creature spell with flying (CR 601.2, 702.9): as the spell's characteristics are, Lemur read through the
/// words of Momo's text (CR 612).
fn non_lemur_flying_creature_spell(q: &dyn Query, source: ObjectId, spell: ObjectId) -> bool {
    let c = q.chars(spell);
    c.is(CardType::Creature) && c.has(Keyword::Flying) && !c.subtypes.has(q.words(source).subtype(Subtype::Lemur))
}

/// "The first non-Lemur creature spell with flying you cast during each of your turns costs {1} less to cast"
/// (CR 601.2f). It is the first such spell whether or not the reduction found generic mana to take (a spell with no
/// generic mana in its cost uses the "first" up: the official ruling), so it is judged on the history of casts this turn,
/// and only on the active player's own turn.
fn first_flying_non_lemur_costs_less(q: &dyn Query, source: ObjectId, player: PlayerId, costed: Costed) -> CostDelta {
    let Costed::Spell(spell) = costed else { return CostDelta::NONE };
    if player != q.controller(source) || q.state().turn.active != player || !non_lemur_flying_creature_spell(q, source, spell) {
        return CostDelta::NONE;
    }
    let earlier = q.history().casts_by(player).any(|c| c.spell != spell && non_lemur_flying_creature_spell(q, source, c.spell));
    if earlier { CostDelta::NONE } else { CostDelta::less(1) }
}

/// "Whenever another creature you control with flying enters" (CR 603.6a): flying as it enters.
fn another_flyer_enters(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { new, to, .. }
            if *new != source
                && to.is(ZoneKind::Battlefield)
                && q.controller(*new) == q.controller(source)
                && q.chars(*new).is(CardType::Creature)
                && q.chars(*new).has(Keyword::Flying) =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

/// Momo gets +1/+1 until end of turn (CR 611.2a), if it is still on the battlefield.
fn grow(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if targets::permanent(g, item.object) {
        g.add_effect(item.object, item.object, EffectKind::Pump(1, 1), Duration::EndOfTurn);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(when(EventMask::ZONE_CHANGE, another_flyer_enters), grow, "Whenever another creature you control with flying enters, Momo gets +1/+1 until end of turn.")],
    cost_changes: &[first_flying_non_lemur_costs_less],
    ..oracle::MOMO_FRIENDLY_FLIER
};
