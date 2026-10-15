use cantrip_api::*;
use cantrip_mechanics::mana;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[mana::tap_for(mana::add_g, Mana::G, "{T}: Add {G}.")],
    ..oracle::ELVISH_MYSTIC
};
