use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Nonbasic lands ...": every land without the basic supertype, judged as layers 1 to 3 left it (CR 205.4a, 613.1d).
fn nonbasic_land(_: &dyn LayerQuery, _: ObjectId, _: ObjectId, so_far: &Chars) -> bool {
    so_far.is(CardType::Land) && !so_far.supertypes.basic
}

/// "... are Mountains" (CR 305.7, layer 4): the land's subtype is set to Mountain, the word of this card's text (CR 612).
/// It loses its other land types and the abilities of its rules text, and has "{T}: Add {R}" (CR 305.6); its name and
/// supertypes stay (the card's ruling).
fn mountains(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    chars.set_land_subtypes(SubtypeSet::NONE.with(view.words(source).subtype(Subtype::Mountain)));
}

pub static CARD: CardDef = CardDef {
    statics: &[sets_land_subtypes(nonbasic_land, mountains, "Nonbasic lands are Mountains.")],
    ..oracle::MAGUS_OF_THE_MOON
};
