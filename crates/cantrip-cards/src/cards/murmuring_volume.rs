use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add_any_color(1), "{T}: Add one mana of any color."),
        ability(cost("{2}, {T}").and(discard_cards(1)), |g, item| effects::draw(g, item, 1), "{2}, {T}, Discard a card: Draw a card."),
    ],
    ..oracle::MURMURING_VOLUME
};
