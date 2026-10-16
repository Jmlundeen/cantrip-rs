use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

fn plus_one_minus_one(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.power = chars.power.map(|v| v + 1);
    chars.toughness = chars.toughness.map(|v| v - 1);
}

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
        triggered(when(EventMask::ZONE_CHANGE, equipped_creature_dies), |g, item| effects::draw(g, item, 2), "Whenever equipped creature dies, draw two cards.")
            .look_back(),
    ],
    statics: &[static_ability(Layer::PtModify, attached_object, plus_one_minus_one, "Equipped creature gets +1/-1.")],
    ..oracle::SKULLCLAMP
};
