use cantrip_api::*;
use cantrip_mechanics::{attach::attached_object, lands::count_you_control, targets};
use cantrip_oracle as oracle;

/// "Enchanted creature gets +X/+Y, where X is half the number of Forests you control, rounded down, and Y is half the number of
/// Forests you control, rounded up." (CR 613.4c, layer 7c): "you" is the Aura's controller (CR 109.5), wherever the enchanted
/// creature is. A Forest is a permanent with the Forest subtype as the earlier layers left it (CR 305.6), the word read through the
/// Aura's text (CR 612.2). The count is live (CR 611.3a), the Aura itself counted if something made it a Forest.
fn apply(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let forest = view.words(source).subtype(Subtype::Forest);
    let forests = count_you_control(view, source, &|c| c.has_subtype(forest));
    chars.power = chars.power.map(|v| v + forests / 2);
    chars.toughness = chars.toughness.map(|v| v + (forests + 1) / 2);
}

pub static CARD: CardDef = CardDef {
    enchant: Some(targets::CREATURE),
    statics: &[static_ability(
        Layer::PtModify,
        attached_object,
        apply,
        "Enchanted creature gets +X/+Y, where X is half the number of Forests you control, rounded down, and Y is half the number of Forests you control, rounded up.",
    )],
    ..oracle::ASPECT_OF_WOLF
};
