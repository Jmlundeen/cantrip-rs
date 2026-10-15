use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Whenever one or more other creatures you control with power 2 or less enter" (CR 603.2c): one trigger for the
/// batch that entered together, which the subjects of the matching events make up.
fn small_creatures_enter(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { new, to, .. }
            if *new != source
                && to.is(ZoneKind::Battlefield)
                && q.controller(*new) == q.controller(source)
                && q.chars(*new).is(CardType::Creature)
                && q.chars(*new).power.is_some_and(|p| p <= 2) =>
        {
            Fired::about(Target::Object(*new))
        }
        _ => None,
    }
}

/// "When this dies, if it was a creature" (CR 700.4, 603.4): judged on what it last was on the battlefield (CR 603.10a).
/// Once it comes back an enchantment, it is not a creature when it dies again and the ability does not trigger.
fn dies_as_a_creature(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { old, from, to, .. }
            if *old == source && from.is(ZoneKind::Battlefield) && to.is(ZoneKind::Graveyard) && q.chars(source).is(CardType::Creature) =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

/// "Return it to the battlefield under its owner's control. It's an enchantment. (It's not a creature.)": a layer-4
/// effect with no end (CR 611.2a, 613.1d), for as long as this is the object that returned.
fn return_as_enchantment(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(card) = g.found_after_move(item.object) else { return Ok(()) };
    let owner = g.obj(card).owner;
    let returned = g.put_onto_battlefield(card, owner, false)?;
    if g.is_on_battlefield(returned) {
        g.add_effect(item.object, returned, EffectKind::SetTypes(TypeSet::of(&[CardType::Enchantment])), Duration::Indefinite);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "Whenever one or more other creatures you control with power 2 or less enter, draw a card. This ability triggers only once each turn.",
            listens: EventMask::ZONE_CHANGE,
            matches: small_creatures_enter,
            effect: cantrip_mechanics::effects::draw::<1>,
            once_per_batch: true,
            triggers_each_turn: 1,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "When Enduring Innocence dies, if it was a creature, return it to the battlefield under its owner's control. It's an enchantment. (It's not a creature.)",
            listens: EventMask::ZONE_CHANGE,
            matches: dies_as_a_creature,
            effect: return_as_enchantment,
            look_back: true,
            ..TriggeredDef::BLANK
        }),
    ],
    ..oracle::ENDURING_INNOCENCE
};
