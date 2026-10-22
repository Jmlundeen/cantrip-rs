use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "Whenever equipped creature dies" (CR 603.6c, 603.10a): the creature the Skullclamp was attached to, judged on the
/// board as it was before (CR 704.5n has unattached it from the corpse by the time anything downstream asks).
fn equipped_creature_dies(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { old, from, to, .. }
            if from.is(ZoneKind::Battlefield) && to.is(ZoneKind::Graveyard) && q.obj(source).attached_to == Some(*old) =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        equip(mana("{1}"), "Equip {1}"),
        triggered(when(EventMask::ZONE_CHANGE, equipped_creature_dies), |g, item| effects::draw(g, item, 2), "Whenever equipped creature dies, draw two cards."),
    ],
    statics: &[static_ability(Layer::PtModify, attached_object, cantrip_mechanics::statics::plus_pt::<1, -1>, "Equipped creature gets +1/-1.")],
    ..oracle::SKULLCLAMP
};
