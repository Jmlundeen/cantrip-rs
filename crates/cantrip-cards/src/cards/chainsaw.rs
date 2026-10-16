use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{damage_target, targets};
use cantrip_oracle as oracle;

/// When this Equipment enters, it deals 3 damage to up to one target creature.
fn three_damage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 3);
    Ok(())
}

/// "Whenever one or more creatures die" (CR 700.4, 603.2c): any creature put into a graveyard from the battlefield,
/// asked of each card of the batch, and one trigger for the creatures that die together. It was a creature as it
/// last existed on the battlefield.
fn creature_dies(q: &dyn Query, _: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { old, new, from, to, .. }
            if from.is(ZoneKind::Battlefield) && to.is(ZoneKind::Graveyard) && q.chars(*old).is(CardType::Creature) =>
        {
            Fired::about(Target::Object(*new))
        }
        _ => None,
    }
}

/// Put a rev counter on this Equipment.
fn rev_counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if is_live(g, item.object) {
        g.add_counters(item.object, CounterKind::Rev, 1);
    }
    Ok(())
}

/// "Equipped creature gets +X/+0, where X is the number of rev counters on this Equipment" (layer 7c): read afresh on
/// every pass, so a counter arriving later raises it at once (CR 611.3a).
fn plus_x_power(q: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let x = q.state().obj(source).counters(CounterKind::Rev);
    chars.power = chars.power.map(|v| v + x);
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, three_damage, "When this Equipment enters, it deals 3 damage to up to one target creature.")
            .targets(&[TargetSpec { min: 0, max: 1, ..targets::CREATURE }]),
        triggered(when(EventMask::ZONE_CHANGE, creature_dies), rev_counter, "Whenever one or more creatures die, put a rev counter on this Equipment.")
            .once_per_batch(),
        equip(mana(3, 0, 0, 0, 0, 0), "Equip {3}"),
    ],
    statics: &[StaticDef {
        text: "Equipped creature gets +X/+0, where X is the number of rev counters on this Equipment.",
        layer: Layer::PtModify,
        affects: attached_object,
        apply: plus_x_power,
        ..StaticDef::BLANK
    }],
    ..oracle::CHAINSAW
};
