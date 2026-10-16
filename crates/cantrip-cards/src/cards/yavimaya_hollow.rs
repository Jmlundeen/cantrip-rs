use cantrip_api::*;
use cantrip_mechanics::{regenerate, targets};
use cantrip_oracle as oracle;

fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    regenerate::target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        ability(cost("{G}, {T}"), effect, "{G}, {T}: Regenerate target creature.")
            .targets(&[targets::CREATURE]),
    ],
    ..oracle::YAVIMAYA_HOLLOW
};
