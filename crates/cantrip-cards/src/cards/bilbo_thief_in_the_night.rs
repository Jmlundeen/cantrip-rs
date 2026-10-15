use cantrip_api::*;
use cantrip_mechanics::{choose, triggers};
use cantrip_oracle as oracle;

/// Spells you cast from anywhere other than your hand cost {1} less to cast (CR 601.2f): the zone a spell is in as its
/// cost is worked out, or the one it was cast from once it is on the stack.
fn outside_hand_costs_one_less(q: &dyn Query, source: ObjectId, player: PlayerId, costed: Costed) -> i8 {
    let Costed::Spell(spell) = costed else { return 0 };
    if player != q.controller(source) {
        return 0;
    }
    let obj = q.obj(spell);
    let from = if obj.zone.is(ZoneKind::Stack) { obj.cast_from } else { Some(obj.zone.kind()) };
    if from.is_some_and(|z| z != ZoneKind::Hand) { -1 } else { 0 }
}

/// Whenever Bilbo attacks, you may cast an artifact, instant, or sorcery spell from your graveyard. If an instant or
/// sorcery spell cast this way would be put into your graveyard, exile it instead. Nothing is asked when no card there
/// can be cast.
fn recast(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let graveyard = g.state().zone(Zone::graveyard(you)).to_vec();
    let eligible: Vec<ObjectId> = graveyard
        .iter()
        .copied()
        .filter(|&card| {
            let c = g.chars(card);
            (c.is(CardType::Artifact) || c.is(CardType::Instant) || c.is(CardType::Sorcery)) && g.could_cast_during_resolution(card)
        })
        .collect();
    if let Some(&chosen) = choose::cards_from(g, you, item.object, Purpose::Cast, graveyard, eligible, 1, true)?.first() {
        g.cast_during_resolution_as(chosen, CastTerms { exile_after: true, ..CastTerms::default() })?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "Whenever Bilbo attacks, you may cast an artifact, instant, or sorcery spell from your graveyard. If an instant or sorcery spell cast this way would be put into your graveyard, exile it instead.",
        listens: EventMask::ATTACKS,
        matches: triggers::this_attacks,
        effect: recast,
        ..TriggeredDef::BLANK
    })],
    cost_changes: &[outside_hand_costs_one_less],
    ..oracle::BILBO_THIEF_IN_THE_NIGHT
};
