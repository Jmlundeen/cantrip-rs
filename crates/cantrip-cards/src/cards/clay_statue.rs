use cantrip_api::*;
use cantrip_mechanics::regenerate;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{2}"), regenerate::this_effect, "{2}: Regenerate this creature.")],
    ..oracle::CLAY_STATUE
};
