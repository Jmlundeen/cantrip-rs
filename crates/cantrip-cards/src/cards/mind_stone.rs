use cantrip_api::*;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        ability(cost("{1}, {T}").and(SACRIFICE_THIS), cantrip_mechanics::effects::draw::<1>, "{1}, {T}, Sacrifice this artifact: Draw a card."),
    ],
    ..oracle::MIND_STONE
};
