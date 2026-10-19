use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::targets::filters;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add_any_color(1), "{T}: Add one mana of any color."),
        ability(cost("{2}, {T}").and(discard(1, filters::any)), |g, item| effects::draw(g, item, 1), "{2}, {T}, Discard a card: Draw a card."),
    ],
    ..oracle::MURMURING_VOLUME
};
