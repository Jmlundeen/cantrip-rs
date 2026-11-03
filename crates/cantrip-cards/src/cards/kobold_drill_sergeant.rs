use cantrip_api::*;
use cantrip_mechanics::{lands, statics};
use cantrip_oracle as oracle;

/// Other Kobold creatures you control (CR 604.1, 613.1f).
fn other_kobolds(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, chars: &Chars) -> bool {
    chars.is(CardType::Creature) && statics::other_of_subtype_you_control::<{ lands::sub(Subtype::Kobold) }>(q, source, affected, chars)
}

const TEXT: &str = "Other Kobold creatures you control get +0/+1 and have trample.";

pub static CARD: CardDef = CardDef {
    statics: &[
        static_ability(Layer::Ability, other_kobolds, statics::have_trample, TEXT),
        static_ability(Layer::PtModify, other_kobolds, statics::plus_pt::<0, 1>, TEXT).continues(),
    ],
    ..oracle::KOBOLD_DRILL_SERGEANT
};
