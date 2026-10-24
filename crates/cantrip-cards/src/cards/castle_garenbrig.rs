use cantrip_api::*;
use cantrip_mechanics::lands::{enters_tapped_unless_you_control, sub};
use cantrip_oracle as oracle;

/// "This land enters tapped unless you control a Forest." "{T}: Add {G}." "{2}{G}{G}, {T}: Add six {G}. Spend this mana only
/// to cast creature spells or activate abilities of creatures" (CR 106.6): "creatures" are creature permanents (CR 109.2;
/// its ruling: not a creature card's ability in another zone).
pub static CARD: CardDef = CardDef {
    enters: Some(enters_tapped_unless_you_control::<{ sub(Subtype::Forest) }>),
    abilities: &[
        mana_ability(TAP, add(Mana::G), "{T}: Add {G}."),
        mana_ability(
            cost("{2}{G}{G}, {T}"),
            add_n(Mana::G, 6).only_for(ManaRestriction::CREATURE_SPELLS_OR_ABILITIES_OF_CREATURES),
            "{2}{G}{G}, {T}: Add six {G}. Spend this mana only to cast creature spells or activate abilities of creatures.",
        ),
    ],
    ..oracle::CASTLE_GARENBRIG
};
