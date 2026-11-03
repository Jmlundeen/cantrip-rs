use cantrip_api::*;
use cantrip_mechanics::{lands::sub, statics::set_land_type};
use cantrip_oracle as oracle;

fn affects(_: &dyn LayerQuery, _: ObjectId, _: ObjectId, chars: &Chars) -> bool {
    chars.is(CardType::Land) && !chars.supertypes.basic
}

pub static CARD: CardDef = CardDef {
    statics: &[sets_land_subtypes(affects, set_land_type::<{ sub(Subtype::Island) }>, "Nonbasic lands are Islands.")],
    ..oracle::HARBINGER_OF_THE_SEAS
};
