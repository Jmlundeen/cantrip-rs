use cantrip_api::*;
use cantrip_mechanics::mana;
use cantrip_oracle as oracle;

fn add_one_of_any_color(g: &mut dyn Game, asker: &Asker, color: Option<Color>) -> Result<(), Suspend> {
    // "Spend this mana only to cast a legendary spell, and that spell can't be countered" (CR 106.6).
    g.add_restricted_mana(asker.controller, color.unwrap_or(Color::White).mana(), 1, ManaRestriction::LegendaryUncounterable);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana::tap_for(mana::add_c, Mana::C, "{T}: Add {C}."),
        AbilityDef::Activated(ActivatedDef {
            text: "{T}: Add one mana of any color. Spend this mana only to cast a legendary spell, and that spell can't be countered.",
            cost: Cost { tap: true, ..Cost::NONE },
            mana: Some(add_one_of_any_color),
            any_color: true,
            adds_any: 1,
            restricted: Some(ManaRestriction::LegendaryUncounterable),
            ..ActivatedDef::BLANK
        }),
    ],
    ..oracle::DELIGHTED_HALFLING
};
