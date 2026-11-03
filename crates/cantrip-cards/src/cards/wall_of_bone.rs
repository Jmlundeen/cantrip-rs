use cantrip_api::*;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(cost("{B}"), cantrip_mechanics::regenerate::this_effect, "{B}: Regenerate this creature."),
    ],
    ..oracle::WALL_OF_BONE
};
