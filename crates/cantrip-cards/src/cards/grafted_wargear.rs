use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_oracle as oracle;

fn bonus(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.power = chars.power.map(|v| v + 3);
    chars.toughness = chars.toughness.map(|v| v + 2);
}

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
        equip(mana(0, 0, 0, 0, 0, 0), "Equip {0}"),
    ],
    statics: &[StaticDef {
        text: "Equipped creature gets +3/+2.",
        layer: Layer::PtModify,
        affects: attached_object,
        apply: bonus,
        ..StaticDef::BLANK
    }],
    ..oracle::GRAFTED_WARGEAR
};
