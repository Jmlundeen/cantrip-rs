use cantrip_api::*;
use cantrip_mechanics::{mana, targets};
use cantrip_oracle as oracle;

pub static BACK: CardDef = CardDef {
    abilities: &[mana_ability(TAP, add(Mana::G), "{T}: Add {G}.")],
    enters: Some(mana::enters_tapped),
    ..oracle::BALA_GED_SANCTUARY
};

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CARD_IN_YOUR_GRAVEYARD], cantrip_mechanics::effects::return_first_target_to_hand),
    back: Some(&BACK),
    ..oracle::BALA_GED_RECOVERY
};
