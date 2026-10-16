use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Whenever a creature you control enters" (CR 603.6a): one trigger per creature, simultaneous entries included, and
/// this enchantment itself whenever it enters as a creature, since an enters ability looks at the permanent as it stands
/// on the battlefield (CR 603.10).
fn your_creature_enters(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { new, to, .. }
            if to.is(ZoneKind::Battlefield) && q.controller(*new) == q.controller(source) && q.chars(*new).is(CardType::Creature) =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

/// This enchantment deals 1 damage to each opponent (CR 120.3).
fn tremor(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let hits: Vec<(ObjectId, Target, u16)> = g.state().opponents(item.controller).map(|p| (item.object, Target::Player(p), 1)).collect();
    g.damage_together(&hits);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(when(EventMask::ZONE_CHANGE, your_creature_enters), tremor, "Whenever a creature you control enters, this enchantment deals 1 damage to each opponent.")],
    ..oracle::IMPACT_TREMORS
};
