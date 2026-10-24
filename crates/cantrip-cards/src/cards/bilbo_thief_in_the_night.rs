use cantrip_api::*;
use cantrip_mechanics::{choose, triggers, zone_cast_from};
use cantrip_oracle as oracle;

/// Spells you cast from anywhere other than your hand cost {1} less to cast (CR 601.2f): the zone a spell is in as its
/// cost is worked out, or the one it was cast from once it is on the stack.
fn outside_hand_costs_one_less(q: &dyn Query, source: ObjectId, player: PlayerId, costed: Costed, _: &[Target]) -> CostDelta {
    let Costed::Spell(spell) = costed else { return CostDelta::NONE };
    if player != q.controller(source) {
        return CostDelta::NONE;
    }
    if zone_cast_from(q, spell).is_some_and(|z| z != ZoneKind::Hand) { CostDelta::less(1) } else { CostDelta::NONE }
}

/// Whenever Bilbo attacks, you may cast an artifact, instant, or sorcery spell from your graveyard. If an instant or
/// sorcery spell cast this way would be put into your graveyard, exile it instead. Nothing is asked when no card there
/// can be cast.
fn recast(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let graveyard = g.state().zone(Zone::graveyard(you)).to_vec();
    // "An artifact, instant, or sorcery spell": the face cast is the spell (CR 709.3b, 715.3b), so it is the face that is judged.
    let ways_of = |g: &dyn Game, card: ObjectId| -> Vec<CastWay> {
        g.cast_ways_during_resolution(card)
            .into_iter()
            .filter(|w| {
                let face = g.card(w.face);
                face.is(CardType::Artifact) || face.is(CardType::Instant) || face.is(CardType::Sorcery)
            })
            .collect()
    };
    let eligible: Vec<ObjectId> = graveyard.iter().copied().filter(|&card| !ways_of(g, card).is_empty()).collect();
    if let Some(&chosen) = choose::cards_from(g, you, item.object, Purpose::Cast, graveyard, eligible, 1, true)?.first() {
        let ways = ways_of(g, chosen);
        if let Some(way) = choose::cast_way(g, you, chosen, ways)? {
            g.cast_during_resolution_as(chosen, CastTerms { exile_instead: ExileInstead::InstantOrSorcery, way: Some(way.way), ..CastTerms::default() })?;
        }
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(triggers::THIS_ATTACKS, recast, "Whenever Bilbo attacks, you may cast an artifact, instant, or sorcery spell from your graveyard. If an instant or sorcery spell cast this way would be put into your graveyard, exile it instead.")],
    cost_changes: &[outside_hand_costs_one_less],
    ..oracle::BILBO_THIEF_IN_THE_NIGHT
};
