use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::targets::filters;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Whenever this creature deals combat damage to a player or planeswalker": one triggering for each player or
/// planeswalker it dealt damage to (CR 603.2c), judged as the damage is dealt (CR 510.2).
fn combat_damage_to_player_or_planeswalker(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Damage { source: dealer, target, amount, combat: true, .. } if *dealer == source && *amount > 0 => match target {
            Target::Player(_) => Fired::PLAIN,
            Target::Object(id) if q.chars(*id).is(CardType::Planeswalker) => Fired::PLAIN,
            _ => None,
        },
        _ => None,
    }
}

/// Put a +1/+1 counter on this creature.
fn grow(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if targets::permanent(g, item.object) {
        g.add_counters(item.object, CounterKind::PlusOne, 1);
    }
    Ok(())
}

/// This creature gains flying until end of turn.
fn gain_flying(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if targets::permanent(g, item.object) {
        g.add_effect(item.object, item.object, EffectKind::AddKeywords(Keywords::of(&[Keyword::Flying])), Duration::EndOfTurn);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(when(EventMask::DAMAGE, combat_damage_to_player_or_planeswalker), |g, item| effects::draw(g, item, 1), "Whenever this creature deals combat damage to a player or planeswalker, draw a card."),
        ability(discard(1, filters::any), grow, "Discard a card: Put a +1/+1 counter on this creature."),
        ability(exile_from_graveyard(3), gain_flying, "Exile three cards from your graveyard: This creature gains flying until end of turn."),
    ],
    ..oracle::PSYCHIC_FROG
};
