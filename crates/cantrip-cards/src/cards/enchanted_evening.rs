use cantrip_api::*;
use cantrip_oracle as oracle;

/// "All permanents": every permanent on the battlefield, whoever controls it, Enchanted Evening itself among them.
fn all_permanents(_: &dyn LayerQuery, _: ObjectId, _: ObjectId, _: &Chars) -> bool {
    true
}

/// "... are enchantments in addition to their other types" (CR 205.1b, 613.1d, layer 4). Being an enchantment gives it no
/// enchantment subtype (the ruling: it only lets them interact with what cares about enchantments).
fn enchantment_too(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.types = chars.types.with(CardType::Enchantment);
}

pub static CARD: CardDef = CardDef {
    statics: &[static_ability(Layer::Type, all_permanents, enchantment_too, "All permanents are enchantments in addition to their other types.")],
    ..oracle::ENCHANTED_EVENING
};
