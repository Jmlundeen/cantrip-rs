use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::lands::{enters_tapped_unless_you_control_either, sub};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    enters: Some(enters_tapped_unless_you_control_either::<{ sub(Subtype::Island) }, { sub(Subtype::Swamp) }>),
    abilities: &[
        mana_ability(TAP, add(Mana::G), "{T}: Add {G}."),
        ability(cost("{3}{G}, {T}"), |g, item| effects::surveil(g, item, 2), "{3}{G}, {T}: Surveil 2."),
    ],
    ..oracle::KISHLA_VILLAGE
};
