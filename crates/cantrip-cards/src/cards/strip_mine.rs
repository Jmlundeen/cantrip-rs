use cantrip_api::*;
use cantrip_mechanics::{destroy_target, mana, targets};
use cantrip_oracle as oracle;

/// Destroy target land, if it is still a legal target.
fn destroy_land(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana::tap_for(mana::add_c, Mana::C, "{T}: Add {C}."),
        AbilityDef::Activated(ActivatedDef {
            text: "{T}, Sacrifice Strip Mine: Destroy target land.",
            cost: Cost { tap: true, sacrifice_self: true, ..Cost::NONE },
            targets: &[targets::LAND],
            effect: destroy_land,
            ..ActivatedDef::BLANK
        }),
    ],
    ..oracle::STRIP_MINE
};
