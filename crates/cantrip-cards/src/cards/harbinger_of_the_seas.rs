use cantrip_api::*;
use cantrip_oracle as oracle;

fn affects(_: &dyn LayerQuery, _: ObjectId, _: ObjectId, chars: &Chars) -> bool {
    chars.is(CardType::Land) && !chars.supertypes.basic
}

/// "Nonbasic lands are Islands." Setting the land type removes its other land types and rules-text abilities, while the
/// type layer's resulting Island supplies its intrinsic blue mana ability (CR 305.7, 613.1d).
fn apply(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    chars.set_land_subtypes(SubtypeSet::NONE.with(view.words(source).subtype(Subtype::Island)));
}

pub static CARD: CardDef = CardDef {
    statics: &[sets_land_subtypes(affects, apply, "Nonbasic lands are Islands.")],
    ..oracle::HARBINGER_OF_THE_SEAS
};
