use cantrip_api::*;
use cantrip_mechanics::mana as mana_abilities;
use cantrip_oracle as oracle;

fn add_ccc(g: &mut dyn Game, asker: &Asker, _: Option<Color>) -> Result<(), Suspend> {
    g.add_mana(asker.controller, Mana::C, 3);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    // "Grim Monolith doesn't untap during your untap step."
    keywords: Keywords::of(&[Keyword::DoesNotUntap]),
    abilities: &[
        mana_abilities::tap_for_n(add_ccc, Mana::C, 3, "{T}: Add {C}{C}{C}."),
        AbilityDef::Activated(ActivatedDef {
            text: "{4}: Untap Grim Monolith.",
            cost: Cost::mana(mana(4, 0, 0, 0, 0, 0)),
            effect: cantrip_mechanics::effects::untap_self,
            ..ActivatedDef::BLANK
        }),
    ],
    ..oracle::GRIM_MONOLITH
};
