use cantrip_api::*;
use cantrip_mechanics::effects::this_gains_indestructible_then_tap;
use cantrip_mechanics::targets::filters;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[ability(discard(1, filters::any), this_gains_indestructible_then_tap, "Discard a card: This creature gains indestructible until end of turn. Tap it.")],
    ..oracle::IRON_SHIELD_ELF
};
