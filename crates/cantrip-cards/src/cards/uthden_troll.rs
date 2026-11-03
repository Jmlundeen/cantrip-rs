use cantrip_api::*;
use cantrip_mechanics::regenerate;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{R}"), regenerate::this_effect, "{R}: Regenerate this creature.")],
    ..oracle::UTHDEN_TROLL
};
