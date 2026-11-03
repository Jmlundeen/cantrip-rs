use cantrip_api::*;
use cantrip_mechanics::{lands, statics::itself_while_you_control_a};
use cantrip_oracle as oracle;

/// "This creature has banding as long as you control a Plains." (CR 604.1).
fn creature_has_banding_as_long_apply_0(
    _: &dyn LayerQuery,
    _: ObjectId,
    chars: &mut Chars,
) {
    chars.keywords = chars.keywords.with(Keyword::Banding);
}

pub static CARD: CardDef = CardDef {
    statics: &[
        static_ability(
            Layer::Ability,
            itself_while_you_control_a::<{ lands::sub(Subtype::Plains) }>,
            creature_has_banding_as_long_apply_0,
            "This creature has banding as long as you control a Plains.",
        ),
    ],
    ..oracle::DIRE_WOLVES
};
