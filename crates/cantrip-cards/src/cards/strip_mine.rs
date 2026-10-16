use cantrip_api::*;
use cantrip_mechanics::{destroy_target, targets};
use cantrip_oracle as oracle;

/// Destroy target land, if it is still a legal target.
fn destroy_land(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        ability(TAP.and(SACRIFICE_THIS), destroy_land, "{T}, Sacrifice Strip Mine: Destroy target land.")
            .targets(&[targets::LAND]),
    ],
    ..oracle::STRIP_MINE
};
