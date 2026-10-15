use cantrip_api::*;
use cantrip_mechanics::{mana, regenerate, targets};
use cantrip_oracle as oracle;

fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    regenerate::target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana::tap_for(mana::add_c, Mana::C, "{T}: Add {C}."),
        AbilityDef::Activated(ActivatedDef {
            text: "{G}, {T}: Regenerate target creature.",
            cost: Cost { mana: cantrip_api::mana(0, 0, 0, 0, 0, 1), tap: true, ..Cost::NONE },
            targets: &[targets::CREATURE],
            effect,
            ..ActivatedDef::BLANK
        }),
    ],
    ..oracle::YAVIMAYA_HOLLOW
};
