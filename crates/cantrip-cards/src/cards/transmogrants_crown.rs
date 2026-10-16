use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "Whenever equipped creature dies" (CR 603.6c, 603.10a): the creature the Crown was attached to, judged on the board
/// as it was before, so it triggers even though the Crown would be unattached from a corpse (CR 704.5n), and even when
/// the Crown leaves at the same time.
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
        triggered(when(EventMask::ZONE_CHANGE, equipped_creature_dies), |g, item| effects::draw(g, item, 1), "Whenever equipped creature dies, draw a card.")
            .look_back(),
        equip(mana("{2}"), "Equip {2}"),
        equip(mana("{B}"), "Equip {B}"),
    ],
    statics: &[static_ability(Layer::PtModify, attached_object, cantrip_mechanics::statics::plus_pt::<2, 0>, "Equipped creature gets +2/+0.")],
    ..oracle::TRANSMOGRANTS_CROWN
};
