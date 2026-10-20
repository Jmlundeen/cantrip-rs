use cantrip_api::*;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    enters: Some(cantrip_mechanics::lands::enters_tapped_unless_you_control::<{ cantrip_mechanics::lands::sub(Subtype::Mountain) }>),
    abilities: &[
        mana_ability(TAP, add(Mana::R), "{T}: Add {R}."),
        mana_ability(cost("{R}, {T}").and(EXERT), add_n(Mana::R, 2).only_for(ManaRestriction::GainsHaste), "{R}, {T}, Exert this land: Add {R}{R}. If any of that mana is spent on a creature spell, it gains haste until end of turn. (An exerted permanent won't untap during your next untap step.)"),
    ],
    ..oracle::ARENA_OF_GLORY
};
