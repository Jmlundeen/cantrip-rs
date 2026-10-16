use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_oracle as oracle;

/// "Whenever this Equipment becomes unattached from a permanent" (CR 701.3d): any way it ends, judged looking back in
/// time (CR 603.10c), so leaving the battlefield triggers it. "That permanent" is the one it was attached to.
fn unattached(_: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Unattached { attachment, from } if *attachment == source => Fired::about(Target::Object(*from)),
        _ => None,
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(when(EventMask::UNATTACHED, unattached), cantrip_mechanics::delayed::sacrifice_subject, "Whenever this Equipment becomes unattached from a permanent, sacrifice that permanent.")
            .look_back(),
        equip(mana("{0}"), "Equip {0}"),
    ],
    statics: &[static_ability(Layer::PtModify, attached_object, cantrip_mechanics::statics::plus_pt::<3, 2>, "Equipped creature gets +3/+2.")],
    ..oracle::GRAFTED_WARGEAR
};
