use cantrip_api::*;
use cantrip_mechanics::{counter_unless_pays, mana, targets};
use cantrip_oracle as oracle;

/// Counter target spell unless its controller pays {1} (CR 118.12).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    counter_unless_pays(g, item, 0, mana("{1}"))?;
    Ok(())
}

/// Jwari Ruins, the back face (CR 712.12): "This land enters tapped. {T}: Add {U}."
pub static BACK: CardDef = CardDef {
    abilities: &[mana_ability(TAP, add(Mana::U), "{T}: Add {U}.")],
    enters: Some(mana::enters_tapped),
    ..oracle::JWARI_RUINS
};

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::SPELL], effect),
    back: Some(&BACK),
    ..oracle::JWARI_DISRUPTION
};
