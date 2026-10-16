use cantrip_api::*;
use cantrip_mechanics::statics::players_cant_gain_life;
use cantrip_oracle as oracle;

/// "Whenever another creature enters" (CR 603.6a): a creature other than this one, under any player's control, the ones
/// entering together with it included (CR 603.6a, the rulings). The entering creature is the trigger's subject.
fn another_creature_enters(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { new, to, .. } if *new != source && to.is(ZoneKind::Battlefield) && q.chars(*new).is(CardType::Creature) => {
            Fired::about(Target::Object(*new))
        }
        _ => None,
    }
}

/// "This creature deals 1 damage to that creature's controller": its controller as the ability resolves, or as it last was
/// on the battlefield if it has left (CR 608.2h). The damage is dealt by this creature, as it last existed if it has left.
fn ping_its_controller(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let creature = item.subject.object().expect("the entering creature");
    let controller = g.controller(creature);
    g.damage(item.object, Target::Player(controller), 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    cant_gain_life: &[players_cant_gain_life],
    abilities: &[triggered(when(EventMask::ZONE_CHANGE, another_creature_enters), ping_its_controller, "Whenever another creature enters, this creature deals 1 damage to that creature's controller.")],
    ..oracle::RAMPAGING_FEROCIDON
};
