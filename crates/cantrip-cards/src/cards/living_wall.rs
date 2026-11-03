use cantrip_api::*;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(cost("{1}"), cantrip_mechanics::regenerate::this_effect, "{1}: Regenerate this creature."),
    ],
    ..oracle::LIVING_WALL
};
