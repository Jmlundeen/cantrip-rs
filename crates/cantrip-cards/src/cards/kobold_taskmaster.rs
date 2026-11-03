use cantrip_api::*;
use cantrip_mechanics::{lands, statics};
use cantrip_oracle as oracle;

/// Other Kobold creatures you control, with Kobold read from this object's text (CR 612.2).
fn other_kobolds(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, chars: &Chars) -> bool {
    chars.is(CardType::Creature) && statics::other_of_subtype_you_control::<{ lands::sub(Subtype::Kobold) }>(q, source, affected, chars)
}

pub static CARD: CardDef = CardDef {
    statics: &[static_ability(Layer::PtModify, other_kobolds, statics::plus_pt::<1, 0>, "Other Kobold creatures you control get +1/+0.")],
    ..oracle::KOBOLD_TASKMASTER
};
