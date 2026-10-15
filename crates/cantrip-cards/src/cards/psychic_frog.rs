use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::targets::filters;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Whenever this creature deals combat damage to a player or planeswalker": one triggering for each player or
/// planeswalker it dealt damage to (CR 603.2c), judged as the damage is dealt (CR 510.2).
fn combat_damage_to_player_or_planeswalker(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Damage { source: dealer, target, amount, combat: true } if *dealer == source && *amount > 0 => match target {
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
        AbilityDef::Triggered(TriggeredDef {
            text: "Whenever this creature deals combat damage to a player or planeswalker, draw a card.",
            listens: EventMask::DAMAGE,
            matches: combat_damage_to_player_or_planeswalker,
            effect: effects::draw::<1>,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "Discard a card: Put a +1/+1 counter on this creature.",
            cost: Cost { discard: Some((1, filters::any)), ..Cost::NONE },
            effect: grow,
            ..ActivatedDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "Exile three cards from your graveyard: This creature gains flying until end of turn.",
            cost: Cost { exile_from_graveyard: 3, ..Cost::NONE },
            effect: gain_flying,
            ..ActivatedDef::BLANK
        }),
    ],
    ..oracle::PSYCHIC_FROG
};
