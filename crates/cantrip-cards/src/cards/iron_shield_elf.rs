use cantrip_api::*;
use cantrip_mechanics::effects::this_gains_indestructible_then_tap;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[ability(discard_cards(1), this_gains_indestructible_then_tap, "Discard a card: This creature gains indestructible until end of turn. Tap it.")],
    ..oracle::IRON_SHIELD_ELF
};
