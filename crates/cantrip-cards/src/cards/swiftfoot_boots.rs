use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_mechanics::statics::{have_haste, have_hexproof};
use cantrip_oracle as oracle;

/// "Equipped creature has hexproof and haste." (CR 702.11, 702.10, layer 6).
fn have_hexproof_and_haste(q: &dyn LayerQuery, id: ObjectId, chars: &mut Chars) {
    have_hexproof(q, id, chars);
    have_haste(q, id, chars);
}

/// Swiftfoot Boots: "Equip {1}" (CR 702.6).
pub static CARD: CardDef = CardDef {
    abilities: &[equip(mana("{1}"), "Equip {1}")],
    statics: &[static_ability(
        Layer::Ability,
        attached_object,
        have_hexproof_and_haste,
        "Equipped creature has hexproof and haste.",
    )],
    ..oracle::SWIFTFOOT_BOOTS
};
