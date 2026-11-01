use cantrip_api::*;
use cantrip_mechanics::{effects, mana, targets};
use cantrip_oracle as oracle;

/// Jwari Ruins, the back face (CR 712.12): "This land enters tapped. {T}: Add {U}."
pub static BACK: CardDef = CardDef {
    abilities: &[mana_ability(TAP, add(Mana::U), "{T}: Add {U}.")],
    enters: Some(mana::enters_tapped),
    ..oracle::JWARI_RUINS
};

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::SPELL], effects::counter_first_target_unless_pays::<1>),
    back: Some(&BACK),
    ..oracle::JWARI_DISRUPTION
};
