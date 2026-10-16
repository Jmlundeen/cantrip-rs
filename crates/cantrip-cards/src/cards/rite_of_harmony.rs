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
    cantrip_mechanics::delayed::this_turn(g, item, item.object);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], effect),
    ways: &[flashback::way(cost("{2}{W}{G}"), "Flashback {2}{G}{W}")],
    delayed: &[triggered(when(EventMask::ZONE_CHANGE, creature_or_enchantment_you_control_enters), |g, item| effects::draw(g, item, 1), "Whenever a creature or enchantment you control enters this turn, draw a card.")],
    ..oracle::RITE_OF_HARMONY
};
