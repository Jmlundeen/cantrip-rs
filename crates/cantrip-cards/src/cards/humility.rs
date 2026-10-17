use cantrip_api::*;
use cantrip_mechanics::statics::all_creatures;
use cantrip_oracle as oracle;

/// "All creatures lose all abilities and have base power and toughness 1/1." One ability, one effect in two layers
/// (CR 613.6): the abilities go in layer 6 (CR 613.1f), mana abilities and characteristic-defining ones among them, and
/// the base power and toughness is set in layer 7b (CR 613.4b), after any characteristic-defining ability of layer 7a
/// (Tarmogoyf is 1/1) and under the effects and counters that modify power and toughness (layer 7c). Its parts apply in
/// timestamp order with the other effects of their layers (CR 613.7), so a later "becomes 2/2" wins. It applies to
/// Humility itself once Humility is a creature: taking its own ability, it goes on applying (CR 613.6).
const TEXT: &str = "All creatures lose all abilities and have base power and toughness 1/1.";

/// "... have base power and toughness 1/1" (CR 613.4b, layer 7b).
fn base_one_one(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.power = Some(1);
    chars.toughness = Some(1);
}

pub static CARD: CardDef = CardDef {
    statics: &[loses_all_abilities(all_creatures, TEXT), static_ability(Layer::PtSet, all_creatures, base_one_one, TEXT).continues()],
    ..oracle::HUMILITY
};
