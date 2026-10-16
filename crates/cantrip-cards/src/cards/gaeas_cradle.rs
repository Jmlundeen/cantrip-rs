use cantrip_api::*;
use cantrip_mechanics::mana;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[mana_ability(TAP, add_by(mana::add_green_for_each_creature), "{T}: Add {G} for each creature you control.")],
    ..oracle::GAEAS_CRADLE
};
