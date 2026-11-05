use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{damage_target, permanents_with_subtype_you_control, targets};
use cantrip_oracle as oracle;

/// "When this creature enters, it deals damage equal to the number of Swamps you control to any target." (CR 120.3, 608.2b, 205.3i):
/// counted as the ability resolves, every permanent of the Swamp subtype its controller has, a basic Swamp or a land that is a
/// Swamp by its types alike. It is this creature that deals the damage, so its lifelink gains the life (CR 702.15b).
fn deals_damage_equal_to_your_swamps(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let swamp = g.words(item.object).subtype(Subtype::Swamp);
    let amount = permanents_with_subtype_you_control(g, item.controller, swamp) as u32;
    damage_target(g, item, 0, amount);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ENTERS,
        deals_damage_equal_to_your_swamps,
        "When this creature enters, it deals damage equal to the number of Swamps you control to any target.",
    )
    .targets(&[targets::ANY])],
    ..oracle::CAT_GATOR
};
