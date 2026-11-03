use cantrip_api::*;
use cantrip_mechanics::statics::enchantment_too;
use cantrip_oracle as oracle;

/// "All permanents": every permanent on the battlefield, whoever controls it, Enchanted Evening itself among them.
fn all_permanents(_: &dyn LayerQuery, _: ObjectId, _: ObjectId, _: &Chars) -> bool {
    true
}

pub static CARD: CardDef = CardDef {
    statics: &[static_ability(Layer::Type, all_permanents, enchantment_too, "All permanents are enchantments in addition to their other types.")],
    ..oracle::ENCHANTED_EVENING
};
