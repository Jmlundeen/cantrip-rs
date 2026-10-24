use cantrip_api::*;
use cantrip_mechanics::lands;
use cantrip_mechanics::statics::{other_of_subtype_you_control, plus_pt};
use cantrip_oracle as oracle;

/// "Other Angels you control get +1/+1 and have lifelink." One static ability in two layers (CR 613.6): lifelink in layer 6
/// (CR 613.1f, 702.15), +1/+1 in layer 7c (CR 613.4c), to the same objects: every other permanent its controller controls that is an
/// Angel as the layers have it so far, the type read through the words of this card's text (CR 612).
const TEXT: &str = "Other Angels you control get +1/+1 and have lifelink.";

fn apply(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = chars.keywords.with(Keyword::Lifelink);
}

pub static CARD: CardDef = CardDef {
    statics: &[
        static_ability(Layer::Ability, other_of_subtype_you_control::<{ lands::sub(Subtype::Angel) }>, apply, TEXT),
        static_ability(Layer::PtModify, other_of_subtype_you_control::<{ lands::sub(Subtype::Angel) }>, plus_pt::<1, 1>, TEXT).continues(),
    ],
    ..oracle::LYRA_DAWNBRINGER
};
