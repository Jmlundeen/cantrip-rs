use cantrip_api::*;
use cantrip_mechanics::mana;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[mana_ability(TAP, add_by(mana::add_blue_for_each_artifact), "{T}: Add {U} for each artifact you control.")],
    ..oracle::TOLARIAN_ACADEMY
};
