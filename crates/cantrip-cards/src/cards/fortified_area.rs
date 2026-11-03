use cantrip_api::*;
use cantrip_oracle as oracle;
use cantrip_mechanics::{lands, statics};

/// "Wall creatures you control get +1/+0 and have banding." (CR 604.1).
fn wall_creatures_control_get_1_affects(
    q: &dyn LayerQuery,
    source: ObjectId,
    affected: ObjectId,
    so_far: &Chars,
) -> bool {
    so_far.is(CardType::Creature)
        && statics::your_of_subtype::<{ lands::sub(Subtype::Wall) }>(q, source, affected, so_far)
}

/// "Wall creatures you control get +1/+0 and have banding." (CR 604.1).
fn wall_creatures_control_get_1_apply_1(
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
            wall_creatures_control_get_1_affects,
            wall_creatures_control_get_1_apply_1,
            "Wall creatures you control get +1/+0 and have banding.",
        ),
        static_ability(
                Layer::PtModify,
                wall_creatures_control_get_1_affects,
                statics::plus_pt::<1, 0>,
                "Wall creatures you control get +1/+0 and have banding.",
            )
            .continues(),
    ],
    ..oracle::FORTIFIED_AREA
};
