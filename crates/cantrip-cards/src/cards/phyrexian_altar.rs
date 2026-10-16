use cantrip_api::*;
use cantrip_mechanics::targets::filters;
use cantrip_oracle as oracle;

/// "Sacrifice a creature: Add one mana of any color." A mana ability whose sacrifice comes with the activation.
pub static CARD: CardDef = CardDef {
    abilities: &[mana_ability(Cost::sacrifice(1, filters::creature), add_any_color(1), "Sacrifice a creature: Add one mana of any color.")],
    ..oracle::PHYREXIAN_ALTAR
};
