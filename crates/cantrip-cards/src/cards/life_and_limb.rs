use cantrip_api::*;
use cantrip_oracle as oracle;

/// "All Forests and all Saprolings are 1/1 green Saproling creatures and Forest lands in addition to their other types."
/// One ability, one effect in three layers (CR 613.6): the types are added in layer 4 (CR 205.1b: creature and land, the
/// creature type Saproling and the land type Forest, each with its own card type, the ruling), the color set green in
/// layer 5 and the base power and toughness set to 1/1 in layer 7b (CR 613.4b), the later parts to the permanents the
/// layer-4 part applied to. A Forest so made has "{T}: Add {G}" (CR 305.6). Its reach is judged once, on what the effects
/// before it made the permanent; making it a Forest Saproling changes nothing about whether it is one, so it applies once
/// (CR 613.8a: no effect depends on itself). An effect of layer 4 that takes the Forest type from a land (Blood Moon on a
/// nonbasic Forest) changes what this applies to, so it applies first (CR 613.8a-b). Forest, Saproling and green are words
/// of its text (CR 612.2).
const TEXT: &str =
    "All Forests and all Saprolings are 1/1 green Saproling creatures and Forest lands in addition to their other types.";

/// "All Forests and all Saprolings", as the layers before have them.
fn forests_and_saprolings(view: &dyn LayerQuery, source: ObjectId, _: ObjectId, so_far: &Chars) -> bool {
    let words = view.words(source);
    so_far.has_subtype(words.subtype(Subtype::Forest)) || so_far.has_subtype(words.subtype(Subtype::Saproling))
}

/// "... are ... Saproling creatures and Forest lands in addition to their other types" (CR 613.1d, layer 4).
fn saproling_forest(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let words = view.words(source);
    chars.types = chars.types.with(CardType::Creature).with(CardType::Land);
    chars.subtypes = chars.subtypes.with(words.subtype(Subtype::Saproling)).with(words.subtype(Subtype::Forest));
}

/// "... green ..." (CR 105.3, 613.1e, layer 5): it is green and no other color.
fn green(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    chars.colors = ColorSet::of(view.words(source).color(Color::Green));
}

/// "... 1/1 ..." (CR 613.4b, layer 7b).
fn one_one(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.power = Some(1);
    chars.toughness = Some(1);
}

pub static CARD: CardDef = CardDef {
    statics: &[
        static_ability(Layer::Type, forests_and_saprolings, saproling_forest, TEXT),
        static_ability(Layer::Color, forests_and_saprolings, green, TEXT).continues(),
        static_ability(Layer::PtSet, forests_and_saprolings, one_one, TEXT).continues(),
    ],
    ..oracle::LIFE_AND_LIMB
};
