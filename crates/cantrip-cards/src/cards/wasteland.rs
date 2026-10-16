use cantrip_api::*;
use cantrip_mechanics::{destroy_target, targets};
use cantrip_oracle as oracle;

/// Destroy target nonbasic land.
fn destroy_land(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        ability(TAP.and(SACRIFICE_THIS), destroy_land, "{T}, Sacrifice Wasteland: Destroy target nonbasic land.")
            .targets(&[targets::NONBASIC_LAND]),
    ],
    ..oracle::WASTELAND
};
