use cantrip_api::*;
use cantrip_mechanics::targets::filters;
use cantrip_oracle as oracle;

/// "{T}, Sacrifice a creature: Add {B}{B}." A mana ability whose sacrifice comes with the activation.
pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        mana_ability(TAP.and(sacrifice(1, filters::creature)), add_n(Mana::B, 2), "{T}, Sacrifice a creature: Add {B}{B}."),
    ],
    ..oracle::PHYREXIAN_TOWER
};
