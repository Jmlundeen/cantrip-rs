use cantrip_api::*;
use cantrip_mechanics::{lands, statics};
use cantrip_oracle as oracle;

/// "All Swamps are 1/1 black creatures that are still lands." (CR 604.1).
fn all_swamps_1_1_black_apply_1(
    q: &dyn LayerQuery,
    source: ObjectId,
    chars: &mut Chars,
) {
    let asker = &Asker::new(source, q.controller(source));
    chars.colors = ColorSet::NONE.with(q.words(asker.source).color(Color::Black));
}

pub static CARD: CardDef = CardDef {
    statics: &[
        static_ability(
            Layer::Type,
            statics::all_of_subtype::<{ lands::sub(Subtype::Swamp) }>,
            statics::creature_too,
            "All Swamps are 1/1 black creatures that are still lands.",
        ),
        static_ability(
                Layer::Color,
                statics::all_of_subtype::<{ lands::sub(Subtype::Swamp) }>,
                all_swamps_1_1_black_apply_1,
                "All Swamps are 1/1 black creatures that are still lands.",
            )
            .continues(),
        static_ability(
                Layer::PtSet,
                statics::all_of_subtype::<{ lands::sub(Subtype::Swamp) }>,
                statics::base_pt::<1, 1>,
                "All Swamps are 1/1 black creatures that are still lands.",
            )
            .continues(),
    ],
    ..oracle::KORMUS_BELL
};
