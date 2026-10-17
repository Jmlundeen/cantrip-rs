use cantrip_api::*;
use cantrip_mechanics::{attach::attached_object, effects, targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "... is an Island" (CR 305.7, layer 4): the enchanted land's subtype is set to Island, the word of this card's text (CR
/// 612). It loses its other land types and the abilities of its rules text, and has "{T}: Add {U}" (CR 305.6); its name and
/// supertypes stay (the ruling).
fn island(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    chars.set_land_subtypes(SubtypeSet::NONE.with(view.words(source).subtype(Subtype::Island)));
}

pub static CARD: CardDef = CardDef {
    // "Enchant land" (CR 702.5a, 303.4a).
    enchant: Some(targets::LAND),
    // "When this Aura enters, draw a card." (CR 603.6a)
    abilities: &[triggered(THIS_ENTERS, |g, item| effects::draw(g, item, 1), "When this Aura enters, draw a card.")],
    // "Enchanted land is an Island." (CR 303.4m: the land it is attached to.)
    statics: &[sets_land_subtypes(attached_object, island, "Enchanted land is an Island.")],
    ..oracle::SPREADING_SEAS
};
