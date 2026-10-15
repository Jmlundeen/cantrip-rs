use cantrip_api::*;
use cantrip_oracle as oracle;

fn affects(_: &dyn LayerQuery, _: ObjectId, _: ObjectId, so_far: &Chars) -> bool {
    so_far.is(CardType::Land)
}

fn apply(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.subtypes = chars.subtypes.with(Subtype::Forest);
}

/// Each land is a Forest in addition to its other land types (CR 305.7, 613.1d).
pub static CARD: CardDef = CardDef {
    statics: &[StaticDef { text: "Each land is a Forest in addition to its other land types.", layer: Layer::Type, affects, apply, ..StaticDef::BLANK }],
    ..oracle::YAVIMAYA_CRADLE_OF_GROWTH
};
