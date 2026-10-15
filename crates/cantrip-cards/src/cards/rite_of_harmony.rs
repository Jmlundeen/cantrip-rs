use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::flashback;
use cantrip_oracle as oracle;

/// Whenever a creature or enchantment you control enters this turn, draw a card: the delayed ability this spell makes
/// (CR 603.7b). Each permanent entering triggers it once, judged as it exists on the battlefield (CR 603.6a, 603.10).
fn creature_or_enchantment_you_control_enters(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { new, to, .. }
            if to.is(ZoneKind::Battlefield) && q.controller(*new) == q.controller(source) && {
                let c = q.chars(*new);
                c.is(CardType::Creature) || c.is(CardType::Enchantment)
            } =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let this = g.card_id(&CARD);
    g.delayed_trigger(DelayedWhen::ThisTurn, item.object, item.controller, AbilityRef::delayed(this, 0), Target::None);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[], effect }),
    ways: &[flashback::way(mana(2, 1, 0, 0, 0, 1), "Flashback {2}{G}{W}")],
    delayed: &[AbilityDef::Triggered(TriggeredDef {
        text: "Whenever a creature or enchantment you control enters this turn, draw a card.",
        listens: EventMask::ZONE_CHANGE,
        matches: creature_or_enchantment_you_control_enters,
        effect: effects::draw::<1>,
        ..TriggeredDef::BLANK
    })],
    ..oracle::RITE_OF_HARMONY
};
