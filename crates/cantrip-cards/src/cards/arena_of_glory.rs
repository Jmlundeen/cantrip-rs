use cantrip_api::*;
use cantrip_mechanics::mana;
use cantrip_oracle as oracle;

/// "Add {R}{R}. If that mana is spent on a creature spell, it gains haste until end of turn" (CR 106.6): the mana
/// carries the restriction; the pool and the spell's payment do the rest.
fn exert_for_red(g: &mut dyn Game, asker: &Asker, _: Option<Color>) -> Result<(), Suspend> {
    g.add_restricted_mana(asker.controller, Mana::R, 2, ManaRestriction::GainsHaste);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(cantrip_mechanics::lands::enters_tapped_unless_you_control::<{ cantrip_mechanics::lands::sub(Subtype::Mountain) }>),
    abilities: &[
        mana::tap_for(mana::add_r, Mana::R, "{T}: Add {R}."),
        AbilityDef::Activated(ActivatedDef {
            text: "{R}, {T}, Exert this land: Add {R}{R}. If that mana is spent on a creature spell, it gains haste until end of turn. (An exerted permanent won't untap during your next untap step.)",
            cost: Cost { mana: mana(0, 0, 0, 0, 1, 0), tap: true, exert: true, ..Cost::NONE },
            mana: Some(exert_for_red),
            adds: ManaPool([0, 0, 0, 2, 0, 0]),
            ..ActivatedDef::BLANK
        }),
    ],
    ..oracle::ARENA_OF_GLORY
};
