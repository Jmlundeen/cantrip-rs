use cantrip_api::*;
use cantrip_mechanics::effects::next_spell_cant_be_countered;
use cantrip_mechanics::lands::{enters_tapped_unless_you_control_either, sub};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    enters: Some(enters_tapped_unless_you_control_either::<{ sub(Subtype::Mountain) }, { sub(Subtype::Forest) }>),
    abilities: &[
        mana_ability(TAP, add(Mana::U), "{T}: Add {U}."),
        ability(cost("{U}, {T}"), next_spell_cant_be_countered, "{U}, {T}: The next spell you cast this turn can't be countered."),
    ],
    ..oracle::MISTRISE_VILLAGE
};
