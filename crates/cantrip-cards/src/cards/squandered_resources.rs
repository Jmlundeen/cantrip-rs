use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[mana_ability(Cost::sacrifice(1, targets::filters::land), add_any_color(1), "Sacrifice a land: Add one mana of any type the sacrificed land could produce.")
        .colors_from_sacrifice()],
    ..oracle::SQUANDERED_RESOURCES
};
